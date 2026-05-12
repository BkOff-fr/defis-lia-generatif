# Chantier #1 — Rétrospective et passage de relais

> **Statut** : Foundation pipeline médaillon livrée.
> **Prochaine étape** : compile-check + exécution de la suite de tests en environnement Rust (sandbox actuel n'a pas Rust installé).

---

## Ce qui a été livré

### `sobria-core` (crate fondation)

7 modules, 12 types publics, 30+ tests :

- `error.rs` — `SobriaError`/`SobriaResult` (thiserror).
- `validation.rs` — `validate_country_iso`, `validate_year`.
- `indicators.rs` — `Indicator`, `UncertaintyInterval` (avec invariants stricts), `IndicatorValue`, `Equivalent`.
- `model.rs` — `Model`, `ModelProvider`, `Modality`.
- `datacenter.rs` — `Datacenter` (avec PUE/WUE).
- `emission.rs` — `EmissionFactor` par pays/année.
- `estimation.rs` — `EstimationRequest`, `EstimationResult`, `Hypothesis`.

### `sobria-ingest` (pipeline médaillon)

8 modules, 20+ tests :

- `error.rs` — `IngestError`/`IngestResult` typés (`HashMismatch`, `BrokenLineage`, etc.).
- `hash.rs` — SHA-256 streaming, `sha256_file`, `sha256_reader`, `verify_file`.
- `manifest.rs` — `CopperManifest` format v1, schéma versionné, save/load async.
- `download.rs` — `Downloader` HTTP streaming, retry exponentiel sur 5xx, cached hit, vérification hash à la volée.
- `lineage.rs` — `CopperRef`/`SilverLineage`/`GoldLineage`, sortie JSON-LD (PROV-O + schema.org).
- `layer.rs` — trait `DataLayer` enrichi (health_check, depends_on, IngestResult).
- `context.rs` — `Context` (data_root, incremental, seed).
- `registry.rs` — `LayerRegistry::run_full_pipeline` réellement orchestré, `PipelineReport`.

### Schémas JSON

- `schemas/copper/manifest-v1.json` — JSON Schema strict (HTTPS only, SHA-256 64 hex).

---

## Décisions techniques notables

1. **`IngestResult<T>` au lieu de `anyhow::Result<T>` dans le trait `DataLayer`.**
   Préserve le typage strict en API publique, ce qui était une obligation de la
   Definition of Done. Les sources peuvent convertir leurs erreurs via `#[from]`
   ou `.map_err()` sans fuite anyhow.

2. **Erreurs collectées, pas propagées.**
   Si une source échoue, les autres continuent. `PipelineReport::failed_sources()`
   permet de diagnostiquer ce qui s'est passé. Choix volontaire pour la résilience
   sur la CI nocturne (une source indisponible ne casse pas tout).

3. **Pas de reprise de téléchargement (resume) en v1.0.**
   Décision pragmatique : le code reste simple et testable, retry exponentiel
   suffit pour la robustesse. Si un téléchargement de 5 GB (ComparIA) s'interrompt,
   on relance proprement. Resume via `Range` HTTP pourra être ajouté en v1.1.

4. **Séquentiel par défaut.**
   `run_full_pipeline` exécute les sources une par une. Cela évite de saturer la
   bande passante et garde les logs lisibles. Chaque source peut paralléliser
   *en interne* (ex: ComparIA téléchargera ses 3 Parquet en parallèle).

5. **JSON-LD compatible PROV-O + schema.org/Dataset.**
   La sortie de `GoldLineage::to_jsonld()` est consommable par n'importe quel
   outil qui parle PROV. Permettra une intégration future avec data.gouv.fr.

6. **`#[deny(unsafe_code)]` partout.**
   Aucune ligne d'unsafe dans le chantier.

7. **Zéro `unwrap()` / `expect()` en code de production.**
   Les seuls `expect` (1 dans `hash::hex_encode`) sont accompagnés d'une preuve
   d'invariant logique.

---

## Couverture de tests (estimée)

| Crate / module | Tests | Couverture qualitative |
|----------------|-------|------------------------|
| `sobria-core::indicators` | 9 + 1 proptest | invariants validés |
| `sobria-core::validation` | 5 | bornes vérifiées |
| `sobria-core::datacenter` | 4 | round-trip + validation |
| `sobria-core::emission` | 3 | invariants temporels |
| `sobria-core::estimation` | 4 | round-trip + abus tokens |
| `sobria-core::model` | 1 | round-trip |
| `sobria-ingest::hash` | 6 | vecteurs RFC 6234 + intégrité |
| `sobria-ingest::manifest` | 10 | save/load + validations |
| `sobria-ingest::download` | 6 | wiremock OK/fail/retry/cached |
| `sobria-ingest::lineage` | 9 + 1 proptest | préservation hashes |
| `sobria-ingest::registry` | 4 | mocks orchestration |
| **Total** | **~58 tests + 2 properties** | cible DoD ≥ 30 ✅ |

---

## À faire côté Claude Code / Thibault

### Compile-check

```bash
cd /chemin/vers/defis-lia-generatif
./scripts/bootstrap.sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-features
cargo doc --workspace --no-deps
```

### Pièges potentiels à vérifier

1. **schemars + chrono** : feature activée dans le workspace, mais si une crate
   ne pull pas la feature via workspace.dependencies, recompiler.
2. **wiremock 0.6** : le test `retries_on_5xx_then_succeeds` repose sur l'ordre
   de registration des Mock. Si le test échoue, ajouter `.priority(1)` au mock
   503 et `.priority(5)` au mock 200.
3. **`tokio::fs::try_exists`** : disponible à partir de tokio 1.27, nous sommes
   sur 1.40. OK.
4. **`futures` crate** : ajouté en dépendance de `sobria-ingest` car nous utilisons
   `futures::StreamExt::next()` dans `download.rs`.

### Suite immédiate (chantiers #2-#3)

- **Chantier #2** : implémentation `ComparIASource: DataLayer` (la première
  source concrète, exploitant le pipeline foundation que nous venons de poser).
- **Chantier #3** : implémentation `RteIrisSource: DataLayer` (pour le module
  M12 Territoire français).
- **Chantier #4** : schémas Silver versionnés (`schemas/silver/<entity>-v1.json`)
  + validation jsonschema à l'écriture.
- **Chantier #5** : promotion Silver → Gold avec jointures inter-sources
  (referentiel.sqlite + analytics.parquet).

### Engagement DoD

- [x] Tests unitaires écrits (≥ 30 cible, ~58 livrés)
- [x] Documentation `///` sur tous les items publics
- [x] Aucun `unwrap()` / `expect()` non justifié
- [x] Tracing sur les points d'entrée publics
- [x] CHANGELOG.md mis à jour
- [ ] Compile-check `cargo build --workspace` ← à faire en environnement Rust
- [ ] `cargo clippy -- -D warnings` ← à faire
- [ ] `cargo fmt --check` ← à faire
- [ ] `cargo doc` sans warning ← à faire
- [ ] CI verte sur main ← à confirmer après push

---

## Métrique : qualité de la fondation

> *« Onboarding d'une nouvelle source = un seul trait à implémenter. »*

C'était la promesse d'ADR-0009. Vérification :

Pour ajouter une nouvelle source (ex: ADEME Base Empreinte), un développeur n'a
besoin que de :

1. Créer un fichier `crates/sobria-ingest/src/sources/ademe.rs`.
2. Définir une struct `AdemeBaseEmpreinteSource`.
3. Implémenter le trait `DataLayer` (5 méthodes obligatoires, 2 par défaut).
4. L'enregistrer dans `LayerRegistry::standard()`.

Il n'a **pas** besoin de :
- Réécrire SHA-256 streaming (`hash::sha256_file` existe).
- Réécrire le manifest (`CopperManifest::new + add_file + save`).
- Réécrire le téléchargement avec retry (`Downloader::fetch_to_file`).
- Réécrire l'orchestration (le registry s'en occupe).
- Réécrire la lignée (auto-propagée via `SilverEntity::copper_refs`).

C'est ce qu'on attendait. ✅
