# ADR-0009 — Architecture médaillon (Copper / Silver / Gold) pour le traitement de la donnée

- **Statut** : Implemented (v0.5.0, 2026-05-15)
- **Date** : 2026-05-12
- **Date d'implémentation** : 2026-05-15 (chantier C26)
- **Décideurs** : Thibault, Cowork
- **Contexte** : Sprint S2 (avant tout travail d'ingestion)

## Contexte et énoncé du problème

Sobr.ia agrège des données hétérogènes (ADEME, RTE, Hugging Face, EcoLogits, CodeCarbon, ML.Energy, papers PDF…) de qualité, fréquence, format et licence variables. Sans discipline, on tombe dans le piège classique du *spaghetti ETL* : transformations *ad hoc*, recalculs non reproductibles, perte de traçabilité scientifique.

Or notre projet a deux exigences contradictoires :
1. **Souveraineté scientifique** — pouvoir remonter de chaque chiffre publié jusqu'à la donnée brute et justifier chaque transformation.
2. **Performance applicative** — l'app et la doc doivent lire vite, sans rejouer les transformations.

L'architecture médaillon (popularisée par Databricks, généralisée dans le monde data engineering) répond aux deux : on conserve le brut, on raffine en étapes nommées, on ne lit que la couche utile.

## Décision

Adopter une **architecture médaillon à 3 couches**, implémentée **automatiquement** via un trait Rust unique appliqué à chaque source. Pipeline orchestré par DVC, exécutable d'une seule commande.

### Couches

#### 🟫 Copper Layer — *raw, immutable, source-of-truth*

- Données brutes telles que reçues de la source.
- Format d'origine préservé (JSON, CSV, PDF, XML, HTML scrap).
- **Append-only** : on n'écrase jamais, on ajoute un snapshot daté.
- Métadonnées obligatoires : URL, timestamp UTC, hash SHA-256, headers HTTP, signature mTLS si dispo, licence détectée.
- Aucun nettoyage, aucune validation autre que l'intégrité du transfert.

```
data/copper/
├── ademe-base-empreinte/
│   ├── 2026-05-12/
│   │   ├── factors-electricity.csv
│   │   └── manifest.json   ← URL, hash, ts, license
│   └── 2026-06-15/
│       └── ...
├── rte-eco2mix/
├── hf-energy-score/
├── ecologits-models/
├── codecarbon-runs/
├── ml-energy-leaderboard/
├── papers/
│   ├── luccioni-2023-bloom.pdf
│   └── manifest.json
└── geolite2/
```

#### 🥈 Silver Layer — *cleaned, conformed, validated*

- Format unifié : **Parquet** (colonnaire, compressé Zstd, fast scan).
- Une entité (table) par source : `models.parquet`, `datacenters.parquet`, `emission_factors.parquet`…
- Schémas figés et versionnés (`schemas/silver/<entity>-v<n>.json`).
- Validation à l'écriture (Rust : `arrow-schema` + JSON Schema via `schemars`).
- Déduplication, normalisation des unités SI, harmonisation des codes pays (ISO 3166), conversion des dates en UTC.
- **Lignée (lineage)** conservée : chaque ligne Silver pointe vers son hash Copper d'origine.
- Pas de jointure inter-source à ce stade : chaque source est cantonnée à son propre Parquet.

```
data/silver/
├── ademe/
│   ├── electricity_factors.parquet
│   └── hardware_factors.parquet
├── rte/
│   └── mix_hourly.parquet
├── hf/
│   └── model_energy_score.parquet
├── ecologits/
│   └── models.parquet
├── codecarbon/
│   └── training_runs.parquet
├── ml_energy/
│   └── inference_benchmarks.parquet
├── papers/
│   └── extracted_measures.parquet
└── geolite2/
    └── ip_to_zone.parquet
```

#### 🥇 Gold Layer — *business-ready, consumption-optimized*

- **Deux artefacts cible** :
  - `gold/referentiel.sqlite` → lu par l'app Tauri (ACID, indexé, FTS5 pour recherche).
  - `gold/analytics.parquet` → lu par DuckDB pour scénarios macro et notebook Quarto.
- Jointures, déduplications inter-sources, résolution des conflits (règles documentées par champ).
- Vues matérialisées orientées usage : `model_overview`, `scenario_inputs`, `time_series_mix`, `comparison_matrix`.
- Datasheet (Gebru et al. 2018) embarquée comme JSON-LD `datasheet.jsonld`.
- Signature d'intégrité : `MANIFEST.sha256` + signature GPG du mainteneur.

```
data/gold/
├── referentiel.sqlite              ← lu par l'app
├── analytics.parquet               ← lu par DuckDB
├── datasheet.jsonld                ← métadonnées scientifiques
└── MANIFEST.sha256                 ← intégrité (signé GPG)
```

## Implémentation automatique

### Trait unique par source

Chaque source du référentiel implémente le trait `DataLayer` exposé par `sobria-ingest` :

```rust
// crate sobria-ingest, src/layer.rs

#[async_trait]
pub trait DataLayer: Send + Sync {
    /// Identifiant stable et unique de la source.
    fn id(&self) -> &'static str;

    /// Métadonnées (licence, URL, fréquence de mise à jour…).
    fn meta(&self) -> SourceMeta;

    /// Étape 1 — récupération brute → Copper layer.
    /// Doit être idempotente : un même snapshot doit produire le même hash.
    async fn ingest_copper(&self, ctx: &Context) -> Result<CopperSnapshot>;

    /// Étape 2 — promotion vers Silver.
    /// Reçoit le snapshot Copper, écrit un (ou plusieurs) Parquet validé.
    async fn promote_silver(
        &self,
        snapshot: &CopperSnapshot,
        ctx: &Context,
    ) -> Result<Vec<SilverEntity>>;

    /// Étape 3 — contribution à Gold.
    /// Retourne les fragments SQL/SQL-Arrow à intégrer au merge Gold.
    async fn contribute_gold(
        &self,
        silver: &[SilverEntity],
        ctx: &Context,
    ) -> Result<GoldContribution>;
}
```

### Registry et orchestrateur

```rust
// crate sobria-ingest, src/registry.rs

pub struct LayerRegistry {
    sources: Vec<Box<dyn DataLayer>>,
}

impl LayerRegistry {
    pub fn default() -> Self {
        Self {
            sources: vec![
                Box::new(AdemeBaseEmpreinteSource::new()),
                Box::new(RteEco2MixSource::new()),
                Box::new(ElectricityMapsSource::new()),
                Box::new(HuggingFaceEnergyScoreSource::new()),
                Box::new(EcoLogitsSource::new()),
                Box::new(CodeCarbonSource::new()),
                Box::new(MlEnergyLeaderboardSource::new()),
                Box::new(PapersSource::new()),
                Box::new(GeoLite2Source::new()),
            ],
        }
    }

    pub async fn run_full_pipeline(&self, ctx: &Context) -> Result<PipelineReport> { … }
}
```

### Commande unique

```bash
# Récupère, valide, normalise, agrège, produit referentiel.sqlite + analytics.parquet
cargo run -p sobria-ingest -- pipeline run

# Ou en mode incrémental (ne ré-ingère que ce qui a changé)
cargo run -p sobria-ingest -- pipeline run --incremental

# Source unique (debug / dev)
cargo run -p sobria-ingest -- pipeline run --source rte-eco2mix
```

### Orchestration DVC

`dvc.yaml` à la racine déclare chaque transition comme une *stage* :

```yaml
stages:
  copper:
    cmd: cargo run -p sobria-ingest -- copper --all
    deps:
      - crates/sobria-ingest/src/sources
    outs:
      - data/copper:
          cache: true
          push: true

  silver:
    cmd: cargo run -p sobria-ingest -- silver --all
    deps:
      - data/copper
      - schemas/silver
    outs:
      - data/silver

  gold:
    cmd: cargo run -p sobria-ingest -- gold
    deps:
      - data/silver
      - schemas/gold
    outs:
      - data/gold/referentiel.sqlite
      - data/gold/analytics.parquet
      - data/gold/datasheet.jsonld
      - data/gold/MANIFEST.sha256
```

`dvc repro` rejoue uniquement les étages dont les inputs ont changé. La CI nocturne exécute `dvc repro && dvc push` automatiquement.

## Garanties produites par l'architecture

| Garantie | Mécanisme |
|----------|-----------|
| Traçabilité scientifique de bout en bout | Lineage (`copper_hash` propagé jusqu'au Gold) |
| Reproductibilité | DVC + seeds + builds déterministes |
| Idempotence | `ingest_copper` doit produire le même hash pour un même snapshot |
| Validation des schémas | JSON Schema + arrow-schema à chaque écriture Silver |
| Pas de régression silencieuse | Tests `proptest` + golden files sur chaque transformation |
| Lecture rapide en prod | Gold = SQLite indexé + Parquet colonnaire |
| Versionnage temporel | Snapshots Copper datés permettent le *time travel* |

## Conséquences

**Positives** :
- Discipline ETL professionnelle, défendable devant n'importe quel jury data.
- Chaque chiffre Sobr.ia est traçable jusqu'à la donnée brute originale.
- Le code d'ingestion devient mécaniquement uniforme entre sources.
- Onboarding d'une nouvelle source = implémenter un seul trait.
- Le notebook Quarto peut citer les hashes Copper comme références bibliographiques.

**Négatives** :
- Coût de stockage triple (mais Copper est compressible et purgable au-delà de N snapshots).
- Discipline d'ingénierie plus forte requise.
- Plus de cérémonie au démarrage de chaque nouvelle source.

**Neutres** :
- L'app finale ne « voit » que la couche Gold — l'utilisateur final n'est pas exposé à la complexité.

## Politique de rétention Copper

- Snapshots des **30 derniers jours** : conservation complète.
- Au-delà : conservation **mensuelle** (premier jour du mois) pendant 2 ans.
- Au-delà : conservation **annuelle** indéfiniment.
- DVC garbage collection automatisé en CI hebdo.

## Évolutions futures (hors v1.0)

- Couche **Platinum** pour données dérivées scientifiquement (résultats Monte-Carlo agrégés, projections).
- Materialized views incrémentales (DuckDB 1.x supportera mieux).
- Plugin Quarto pour citer automatiquement les sources Copper dans le notebook.

## Conséquences observées (v0.5.0, post-implémentation C26)

Mise à jour à l'issue du chantier C26 (sous-chantiers C26.1 → C26.5).

### Ce qui a tenu ses promesses

- **Trait `DataLayer` unique** : ComparIA et RTE IRIS implémentent le
  même trait (`ingest_copper`, `promote_silver`, `contribute_gold`).
  Onboarder une nouvelle source Tier 2/3 = écrire 1 fichier `sources/<id>.rs`,
  l'enregistrer dans `LayerRegistry::standard()` — pas de modification
  des étages aval.
- **Lineage de bout en bout** : chaque entité Silver écrit deux colonnes
  systématiques (`_copper_sha256`, `_ingested_at`) ; le module
  `sobria_ingest::lineage::GoldLineage::copper_hashes()` permet de
  remonter du Gold vers les hashes Copper d'origine en O(N). La datasheet
  JSON-LD au format Gebru et al. 2018 expose cette traçabilité.
- **Validation à l'écriture** : `silver_validate::validate_silver` rejette
  toute écriture Silver qui ne respecterait pas son schéma JSON Schema
  2020-12 versionné — interception immédiate des régressions de schéma
  amont.
- **Reproductibilité bit-à-bit** : `dvc repro` produit des hashes
  `referentiel.sqlite` stables sous `SOBRIA_SEED=42`. Vérifié sur
  fixtures wiremock (cf. `tests/gold_pipeline.rs`).
- **Découplage app** : `sobria-app` ne connaît plus que `sobria-referentiel`
  qui ne lit que `data/gold/referentiel.sqlite` ; aucun couplage direct
  entre l'app Tauri et les sources amont.

### Ce qui a coûté plus cher que prévu

- **`assemble_gold` est devenu copieux** (~280 lignes pour la fonction
  + ses helpers). 5 nouvelles tables matérialisées (model_overview avec
  FTS5, scenario_inputs, time_series_mix, comparison_matrix,
  datacenter_iris_link) + datasheet validée + signature GPG optionnelle.
  Annoté `#[allow(clippy::too_many_lines)]` — un découpage en sub-fns
  fragmenterait la lecture séquentielle SQL sans bénéfice.
- **Polars en spawn_blocking** partout : `validate_silver`, `populate_model_overview`,
  `build_analytics_parquet` doivent toutes encapsuler le code synchrone
  Polars. Ajoute un peu de cérémonie mais reste lisible.
- **Couplage `sobria-ingest` → `sobria-geoloc`** : nécessaire pour
  `datacenter_iris_link`. Pas idéal architecturalement (Gold devrait
  être agnostique des consommateurs) mais pragmatique pour v1. À
  réévaluer en v2 si on veut un Gold publiable indépendamment.

### Ce qui n'a pas été implémenté (différé)

- **Stage `dvc gc` automatisé** dans le workflow nocturne (rétention
  Copper 30j/2ans/∞ encore manuelle). Reporté en v0.6.
- **Materialized views incrémentales** : aujourd'hui `assemble_gold`
  recalcule toujours toutes les tables. Acceptable sur volumes Tier 1
  (4 entités Silver, ~5 GB Copper). À réévaluer si Tier 2/3 explose
  les volumes.
- **Plugin Quarto pour citation automatique** des hashes Copper :
  reporté v1.x.
- **Couche Platinum** : pas commencée. Les résultats Monte-Carlo
  restent calculés à la volée côté app.

### Métriques de l'implémentation

| Métrique | Valeur |
|----------|--------|
| Sources Tier 1 fonctionnelles | 2 (ComparIA, RTE IRIS) |
| Entités Silver versionnées | 4 (3 ComparIA + 1 RTE IRIS) |
| Schémas JSON Schema 2020-12 | 5 (4 Silver + 1 Gold datasheet + 1 Copper manifest) |
| Tables Gold | 8 (3 catalogue + 5 vues matérialisées) |
| Index FTS5 | 1 (model_overview_fts) |
| Tests automatisés | 508 (workspace) |
| LoC ajoutées (chantier C26) | ~6000 |
| Stages DVC | 4 (copper, silver, gold, validate) |
| Workflow CI nocturne | 1 (`.github/workflows/dvc-nightly.yml`) |

## Liens

- ADR-0003 (SQLite + DuckDB).
- ADR-0007 (DVC).
- CDC §5.1 (Module M1 Référentiel), §8 (Sources).
- Référence : Bronze/Silver/Gold pattern (Databricks, 2020+). Notre nomenclature « Copper » remplace « Bronze » par préférence projet.
