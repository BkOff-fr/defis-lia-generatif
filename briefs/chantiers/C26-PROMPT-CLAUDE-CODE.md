# C26 — Prompt Claude Code (v0.5.0 — Pipeline médaillon)

> **Mode d'emploi** : copier-coller le contenu ci-dessous dans une nouvelle
> session Claude Code (CLI) à la racine du repo. Le prompt est auto-suffisant
> et démarre par `/using-superpower` pour mobiliser toutes les capacités.
>
> **Pré-requis état repo** : C26.1 est déjà mergé (voir `[Unreleased]` dans
> CHANGELOG.md). Le brief complet est `briefs/chantiers/C26-pipeline-medaillon-activation.md`.

---

```
/using-superpower

# Mission : finaliser le chantier C26 (v0.5.0) — Activation du pipeline médaillon

Tu vas exécuter les sous-chantiers C26.2 → C26.5 du brief
`briefs/chantiers/C26-pipeline-medaillon-activation.md`. C26.1 (câblage CLI +
`LayerRegistry::standard()`) est déjà fait — voir l'entrée `[Unreleased]` du
CHANGELOG.md, et les fichiers `crates/sobria-ingest/src/{cli.rs, main.rs}` +
`crates/sobria-ingest/src/manifest.rs` (méthode `verify_files` ajoutée).

## Contexte projet à charger AVANT toute action

Lis ces fichiers dans l'ordre, sans en sauter :

1. `CLAUDE.md` — règles projet (frugalité, conventions Rust/TS, DoD, ADR-0009 = règle stricte)
2. `docs/adr/ADR-0009-medallion-architecture.md` — architecture cible
3. `briefs/chantiers/C26-pipeline-medaillon-activation.md` — découpage des 5 sous-chantiers
4. `crates/sobria-ingest/src/{lib.rs, cli.rs, layer.rs, registry.rs, gold.rs}` — état actuel du code
5. `crates/sobria-ingest/src/sources/{comparia.rs, rte_iris.rs}` — sources Tier 1
6. `crates/sobria-ingest/tests/{comparia.rs, rte_iris.rs, gold_pipeline.rs}` — patterns de test
7. `CHANGELOG.md` entrée `[Unreleased]` — ce qui est déjà fait

## Stratégie données = DVC + cache local 5 GB

Décision Thibault : on **NE PAS** committer de fixtures Parquet 30 MB
dans le repo. À la place :

- Les vrais snapshots ComparIA + RTE IRIS sont stockés via **DVC remote**
  (configurable, par défaut local `./.dvc-cache/`).
- `data/copper`, `data/silver`, `data/gold` sont ajoutés à `.gitignore`
  et déclarés `outs:` dans `dvc.yaml`.
- Les tests d'intégration continuent d'utiliser **wiremock** pour simuler
  data.gouv.fr (déjà en place dans `tests/comparia.rs` et `tests/gold_pipeline.rs`).
  Ne les remplace pas.
- Le dev sans connexion : `dvc pull` télécharge les snapshots depuis le
  remote, puis `cargo run -p sobria-ingest -- silver --all` (qui ré-utilise
  Copper existant — voir C26.2 ci-dessous).

## Plan d'exécution

### C26.2 — Schémas Silver versionnés + validation (2 jours)

**Objectif** : chaque écriture Silver est validée par JSON Schema + arrow-schema.

Livrables :

1. `schemas/silver/` à la racine (nouveau dossier) contenant :
   - `comparia_conversations-v1.json`
   - `comparia_votes-v1.json`
   - `comparia_reactions-v1.json`
   - `rte_iris_consommation-v1.json`

   Chaque schéma JSON Schema 2020-12 décrit :
   - Le shape Parquet attendu (colonnes + types Arrow)
   - Les colonnes systématiques `_copper_sha256` (string 64 hex) + `_ingested_at` (string ISO 8601)
   - Les contraintes métier non-nulles (ex: `code_iris` requis pour RTE)

2. `crates/sobria-ingest/src/silver_validate.rs` (nouveau module) :
   ```rust
   pub fn validate_silver(
       entity: &SilverEntity,
       schema_path: &Path,
   ) -> IngestResult<()>;
   ```
   Lit le Parquet via `polars::LazyFrame::scan_parquet`, extrait le schéma
   Arrow, valide structurellement contre le JSON Schema embarqué via
   `include_str!`. Erreur claire si colonne manquante / type incompatible.

3. Refactorer Silver pour appeler `validate_silver` :
   - `comparia.rs::promote_silver` : valide chaque entité produite contre `comparia_<entity>-v1.json`
   - `rte_iris.rs::promote_silver` : valide contre `rte_iris_consommation-v1.json`

4. Refactorer `silver` CLI pour ré-utiliser un Copper existant :
   - Si `data/copper/<source>/<latest>/manifest.json` existe et `verify_files` OK → on
     reconstruit le `CopperSnapshot` depuis le manifest plutôt que de re-télécharger.
   - Sinon → erreur explicite « lancez d'abord `copper --source <id>` ou `pipeline run` ».
   - Méthode utilitaire `CopperSnapshot::from_manifest(snapshot_dir)` à ajouter sur `layer.rs`.

5. Tests :
   - `crates/sobria-ingest/tests/silver_validation.rs` — proptest qui génère des Parquet
     avec/sans colonnes lineage, vérifie que validate_silver rejette les invalides.
   - `crates/sobria-ingest/tests/snapshots/` — golden files `insta` pour
     les schémas Silver (un par entité, format JSON formaté).
   - Tests existants `tests/comparia.rs`, `tests/rte_iris.rs` doivent passer
     sans modification (les Parquet synthétiques générés satisfont déjà le schéma v1
     passthrough).

DoD C26.2 : `cargo test -p sobria-ingest` 100 % vert, `cargo clippy --workspace
-- -D warnings` propre, golden files committés.

---

### C26.3 — Gold complet (jointures + datasheet Gebru) (2 jours)

**Objectif** : `referentiel.sqlite` + `analytics.parquet` + `datasheet.jsonld`
+ `MANIFEST.sha256` réellement consommables par l'app.

Livrables :

1. `crates/sobria-ingest/src/gold.rs` enrichi :
   - **Vues matérialisées SQLite** :
     - `model_overview` : un modèle = une ligne, depuis ComparIA
       (extraction `model_id` distinct dans `comparia_conversations` + métadonnées).
     - `scenario_inputs` : table dénormalisée prête pour M13 simulateur.
     - `time_series_mix` : mix horaire RTE par région NUTS-2.
     - `comparison_matrix` : tableau croisé modèles × méthodologies (vide à l'init,
       remplie au runtime par l'app).
   - **Index FTS5** sur `model_overview(name, family, vendor)` (recherche M9).
   - **Jointures inter-sources** : table `datacenter_iris_link` mappant chaque
     datacenter Europe (depuis `sobria-geoloc::datacenters`) à sa maille IRIS
     la plus proche (haversine), produite à l'assemblage Gold.

2. `datasheet.jsonld` (format Gebru et al. 2018 + schema.org/Dataset + DCAT) :
   - Contexte JSON-LD `@context` complet (schema.org + DCAT + PROV-O).
   - Section "Motivation", "Composition", "Collection Process",
     "Preprocessing", "Uses", "Distribution", "Maintenance".
   - Pour chaque source : licence, URL canonique, hash Copper, date snapshot.
   - Référence DOI / catalogue data.gouv.fr.
   - Validation : `schemars` + jsonschema sur un schéma `schemas/gold/datasheet-v1.json`.

3. `MANIFEST.sha256` signé GPG (optionnel) :
   - Format `sha256sum` standard (`<hash>  <filename>`).
   - Si `SOBRIA_GPG_KEY_ID` est défini, signe et produit aussi `MANIFEST.sha256.asc`.
   - Le test peut skipper la signature (var env absente).

4. Tests :
   - `tests/gold_pipeline.rs` (existant) étendu : vérifie présence des vues
     matérialisées, FTS5 fonctionne, `datacenter_iris_link` non vide.
   - `tests/datasheet_jsonld.rs` (nouveau) : valide le JSON-LD contre son
     schéma + vérifie présence de tous les champs Gebru.

DoD C26.3 : `cargo test -p sobria-ingest` 100 % vert. Capture d'un snippet
du datasheet en exemple dans le README section "Pipeline de données".

---

### C26.4 — Orchestration DVC (1 jour)

**Objectif** : `dvc repro` rejoue à l'identique, `dvc push` publie sur remote.

Livrables :

1. `dvc.yaml` à la racine (3 stages copper / silver / gold) :
   ```yaml
   stages:
     copper:
       cmd: cargo run -p sobria-ingest -- copper --all
       deps:
         - crates/sobria-ingest/src
         - crates/sobria-core/src
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
         - data/silver:
             cache: true
             push: true
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

2. `.dvc/config` configuré pour un remote local par défaut :
   ```ini
   [core]
       remote = local
   ['remote "local"']
       url = .dvc-cache
   ```
   Doc : si Thibault veut publier ailleurs (S3, HTTP), commande
   `dvc remote modify local url <new_url>`.

3. `.dvcignore` excluant `target/`, `node_modules/`, tests fixtures, etc.

4. `.gitignore` ajoute `data/copper/`, `data/silver/`, `data/gold/`, `.dvc-cache/`.

5. `docs/operations/dvc.md` (nouveau, ~150 lignes) :
   - Quick start (`dvc pull`, `dvc repro`, `dvc push`).
   - Politique de rétention (cf. ADR-0009 §"Politique de rétention Copper").
   - Comment basculer le remote sur S3 / HTTP.
   - FAQ : "Pourquoi DVC plutôt que Git LFS ?", "Que fait `dvc repro` ?", etc.

6. `.github/workflows/dvc-nightly.yml` (nouveau) :
   - Trigger : cron quotidien 03:00 UTC + dispatch manuel.
   - Steps : checkout, install Rust + DVC, `dvc pull --remote local`,
     `cargo build -p sobria-ingest --release`, `dvc repro`, `dvc push`.
   - Secrets requis : `DVC_REMOTE_URL`, `DVC_REMOTE_TOKEN` (si remote distant).
   - Notification Slack/email si KO (optionnel, skippable).

DoD C26.4 : `dvc repro` fonctionne en local. Hash du `referentiel.sqlite`
reproductible entre deux runs (seed déterministe = `SOBRIA_SEED=42`).

---

### C26.5 — Reconnexion app au Gold (1 jour)

**Objectif** : l'app Tauri lit `data/gold/referentiel.sqlite` au lieu du
référentiel embarqué.

Livrables :

1. `crates/sobria-referentiel/src/lib.rs` :
   - `pub fn load() -> Result<Referentiel>` lit depuis
     `SOBRIA_REFERENTIEL_PATH` (défaut `data/gold/referentiel.sqlite`).
   - `pub struct ReferentielStatus { version, snapshot_date, sha256, source_count, model_count }`
   - `pub fn status() -> ReferentielStatus` (lit les méta dans `sources` table Gold).

2. `crates/sobria-app/src/logic.rs` :
   - Nouvelle IPC `get_referentiel_status` exposée à Svelte.
   - Bootstrap utilisateur : à la première ouverture, si Gold absent :
     - Tente `dvc pull` (via `std::process::Command`).
     - Si DVC indisponible → message clair + lien vers
       `docs/operations/dvc.md`.

3. `web/src/lib/api.ts` :
   - Type `ReferentielStatus`.
   - Fonction `getReferentielStatus()`.

4. `web/src/routes/parametres/+page.svelte` :
   - Section "Référentiel" affichant version, date, hash (tronqué à 12 chars),
     nb sources, nb modèles.
   - Bouton "Recharger le référentiel" (appel `dvc pull` côté Rust).

5. Suppression du référentiel embedded :
   - Garder les `include_str!` JSON pour les démos hors-ligne (M20, M12)
     mais les renommer `*_demo.json` pour distinguer du vrai référentiel Gold.
   - Documenter dans README.

DoD C26.5 : l'app Tauri démarre, l'onglet Paramètres affiche le statut Gold,
smoke test M1/M9/M12/M15/M20 passe sans régression visible.

---

## Definition of Done globale v0.5.0

- [ ] `cargo test --workspace` 100 % vert
- [ ] `cargo clippy --workspace -- -D warnings` propre
- [ ] `cargo fmt --check` propre
- [ ] `cd web && npm run check && npm run lint` propre
- [ ] `cargo run -p sobria-ingest -- pipeline run` produit Gold non vide
- [ ] `dvc repro` reproductible (hash Gold stable)
- [ ] App Tauri démarre + smoke test M1/M9/M12/M15/M20 OK
- [ ] CHANGELOG entrée `[0.5.0] — YYYY-MM-DD` complète
- [ ] `docs/adr/ADR-0009-medallion-architecture.md` mis à jour : statut
      `Accepted → Implemented`, section "Conséquences observées" ajoutée
- [ ] Bump versions :
  - `Cargo.toml` workspace.package : `0.4.0 → 0.5.0`
  - `crates/sobria-app/tauri.conf.json` : `0.4.0 → 0.5.0`
  - `web/package.json` : `0.4.0 → 0.5.0`
- [ ] Commit Conventional Commits + tag `v0.5.0`

## Convention de commit

Fais des commits intermédiaires propres :

- `feat(ingest): C26.2 schémas Silver versionnés + validation`
- `feat(ingest): C26.3 Gold jointures + datasheet Gebru`
- `feat(ingest): C26.4 orchestration DVC (dvc.yaml + CI nocturne)`
- `feat(app): C26.5 reconnexion référentiel au Gold`
- `chore(release): bump v0.5.0`

Tag final :

```bash
git tag -a v0.5.0 -m "v0.5.0 — Activation du pipeline médaillon (C26)

Premier référentiel généré de bout en bout depuis ComparIA + RTE IRIS
(Tier 1 du défi data.gouv.fr).

- Schémas Silver versionnés + validation (4 entités).
- Gold complet : referentiel.sqlite avec FTS5 + jointures IRIS,
  datasheet.jsonld au format Gebru et al. 2018, MANIFEST.sha256.
- Orchestration DVC reproductible (dvc repro).
- App Tauri reconnectée au Gold (fin du référentiel embedded).

ADR-0009 passé en statut Implemented."
```

## Garde-fous

- **JAMAIS** de fixtures Parquet committées dans le repo (`.gitignore`).
- **JAMAIS** de transformation Silver/Gold en dehors du trait `DataLayer`.
- **JAMAIS** de secrets en clair (DVC remote token via GitHub Encrypted Secrets).
- **TOUJOURS** citer la source de toute formule / valeur numérique en commentaire.
- **TOUJOURS** demander si une ambiguïté apparaît dans le CDC, le brief, ou ADR-0009.
- Respect CLAUDE.md §13 (anti-patterns) à la lettre.

Bonne mission. Commence par C26.2.
```

---

## Notes pour Thibault

- Si Claude Code bloque sur un point (ex: dépendance qui refuse de
  compiler, ambiguïté de spec ComparIA), il te demandera — réponds
  rapidement pour ne pas bloquer.
- Si le téléchargement réel ComparIA (5 GB) est nécessaire pour
  valider C26.3 jointures, lance-le en background : `nohup cargo run
  -p sobria-ingest -- copper --source comparia &`. Sinon les tests
  wiremock suffisent.
- Le DVC remote par défaut est local (`./.dvc-cache`). Pour publier
  ailleurs (S3 OVH, HTTP serveur Sobr.ia), configure-le APRÈS C26.4 :
  `dvc remote add -d ovh s3://sobria-data/...`.
- Avant de tag v0.5.0, fais une review rapide de tout le diff : `git
  diff main..HEAD --stat` puis `git diff main..HEAD -- '*.rs'` pour
  les changements Rust critiques.
