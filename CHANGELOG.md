# Changelog Sobr.ia

Toutes les modifications notables sont documentées ici, conformément à [Keep a Changelog 1.1.0](https://keepachangelog.com/fr/1.1.0/) et [SemVer](https://semver.org/).

Format : `[X.Y.Z] - YYYY-MM-DD`
Types : `Added`, `Changed`, `Deprecated`, `Removed`, `Fixed`, `Security`.

## [Unreleased]

### Added — Cadrage et bootstrap (S0-S1)
- Pack de cadrage initial : CDC v1.2, 9 ADR, roadmap 12 semaines, brief S0, catalogue sources, maquette UI textuelle.
- Architecture médaillon Copper / Silver / Gold (ADR-0009).
- Module M12 — Territoire français (cartographie IRIS, croisement ComparIA × RTE IRIS).
- Bootstrap technique : workspace Cargo, CI GitHub Actions, DVC pipeline, `scripts/bootstrap.sh`.

### Added — Chantier #1 : Foundation pipeline médaillon
- `sobria-core` : types fondamentaux (`Datacenter`, `EmissionFactor`, `EstimationRequest`, `EstimationResult`, `Hypothesis`) + validation stricte (`UncertaintyInterval::new` garantit p5 ≤ p50 ≤ p95).
- `sobria-ingest::hash` : SHA-256 streaming, vérification de fichier, vecteurs RFC 6234 testés.
- `sobria-ingest::manifest` : `CopperManifest` format v1, schéma JSON Schema versionné, save/load async.
- `sobria-ingest::download` : `Downloader` HTTP streaming avec retry exponentiel 5xx, vérification hash à la volée, cached hit, mocks wiremock.
- `sobria-ingest::lineage` : `CopperRef`/`SilverLineage`/`GoldLineage`, sortie JSON-LD compatible PROV-O + schema.org/Dataset.
- `sobria-ingest::layer` : trait `DataLayer` enrichi avec `health_check` et `depends_on` (impls par défaut), basculé sur `IngestResult<T>`.
- `sobria-ingest::registry` : `LayerRegistry::run_full_pipeline` réellement orchestré, gestion d'erreurs par source (pas de fail-fast), `PipelineReport` avec `failed_sources()`.
- Schéma JSON `schemas/copper/manifest-v1.json` strict (HTTPS only, SHA-256 64 hex).
- Tests : ≥ 50 tests unitaires + 2 propriétés `proptest` (invariants `UncertaintyInterval`, garantie de préservation des hashes Copper dans `GoldLineage`).

### Added — Chantier #2 : Source ComparIA
- `ComparIASource` : implémentation complète du trait `DataLayer` pour Compar:IA (Beta.gouv / Ministère de la Culture).
- Téléchargement des 3 Parquet ComparIA (conversations 682 MB, votes 733 MB, réactions 3,4 GB) depuis data.gouv.fr.
- `promote_silver` via polars 0.46 (en `spawn_blocking`) avec enrichissement systématique `_copper_sha256` + `_ingested_at`.
- 3 schémas Silver versionnés permissifs : `comparia_{conversations,votes,reactions}-v1.json`.
- 3 tests d'intégration end-to-end (wiremock + Parquet synthétique).
- Loopback HTTP autorisé dans le manifest pour les tests (HTTPS strict en production).

### Added — Chantier #3 : Source RTE / NaTran / Teréga IRIS
- `RteIrisSource` : deuxième source officielle du défi (territorial).
- Téléchargement CSV de consommation (90 MB) + GeoJSON des géométries IRIS (91 MB) depuis ODRÉ.
- CSV promu en Silver via `polars::LazyCsvReader`, GeoJSON conservé en Copper uniquement (consommation directe par le module M12 plus tard).
- Schéma Silver `rte_iris_consommation-v1.json` permissif.
- 3 tests d'intégration (registry, end-to-end, GeoJSON préservé).

### Added — Chantier #4 : Assemblage Gold
- `sobria-ingest::gold` : `assemble_gold` orchestre 4 fonctions d'écriture.
- `referentiel.sqlite` (mode WAL) avec 3 tables : `sources`, `silver_entities`, `lineage` (FK + index).
- `analytics.parquet` (catalogue tabulaire des entités Silver, lisible DuckDB).
- `datasheet.jsonld` (PROV-O + schema.org/Dataset, pretty-printed).
- `MANIFEST.sha256` au format `sha256sum` standard.
- `LayerRegistry::run_full_pipeline` appelle `assemble_gold` en fin de chaîne ; `PipelineReport.gold_artifacts: Option<GoldArtifacts>` expose les chemins.
- Tests : 4 unitaires (assemble, sqlite, parquet, empty case) + 1 d'intégration end-to-end ComparIA + RTE IRIS → Gold final, vérifie 2 sources, 4 entités Silver, 4 lignes lineage, 3 lignes manifest.

### Changed
- Pivot stratégique sur les datasets officiels du défi data.gouv.fr (ComparIA + RTE IRIS).
- 0 clé API bloquante en v1.0 (RTE eco2mix reste optionnel pour le live FR).
- `schemars` activé avec la feature `chrono` (sérialisation de `DateTime<Utc>`).
- `polars` bumpé 0.43 → 0.46 (compatibilité hashbrown 0.15+).
- `rustfmt.toml` allégé en options stable-only.
- Allows clippy ciblés (`missing_errors_doc`, `missing_panics_doc`, `doc_markdown`, `duration_suboptimal_units`, `needless_pass_by_value`, `float_cmp`) — discipline pédantique sans bruit cosmétique.

### Removed
- Sources Electricity Maps et MaxMind GeoLite2 (paywalls / licences virales).

### Métriques finales du jalon
- **3 264 lignes** de code Rust (sobria-ingest) + 759 lignes (sobria-core) + 635 lignes de tests d'intégration.
- **~85 tests** automatiques (unit + intégration + property-based).
- **2 sources Tier 1 du défi** opérationnelles (ComparIA + RTE IRIS).
- **Pipeline Copper → Silver → Gold complet** sur des données synthétiques en CI.

---

## [0.1.0] - À venir

Première release publique : cadrage + S0 (revue biblio) + S1-S5 (foundation + 2 sources + Gold) terminés.
