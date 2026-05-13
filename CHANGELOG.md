# Changelog Sobr.ia

Toutes les modifications notables sont documentées ici, conformément à [Keep a Changelog 1.1.0](https://keepachangelog.com/fr/1.1.0/) et [SemVer](https://semver.org/).

Format : `[X.Y.Z] - YYYY-MM-DD`
Types : `Added`, `Changed`, `Deprecated`, `Removed`, `Fixed`, `Security`.

## [Unreleased]

### Added — Chantier C09 : Intégration Tauri + UI desktop v0.2 (S6) — `v0.2.0-estimer`

#### Wrapper IPC + design system v2

- `web/src/lib/api.ts` : 6 fonctions typées mirrorant `crates/sobria-app/src/dto.rs` (`metaInfo`, `listModels`, `estimatePrompt`, `verifyAudit`, `listAuditEntries`, `exportAuditNdjson`). `SobriaIpcError extends Error` + `isTauriContext()` strict — aucun mock, aucun fallback (CLAUDE.md §13).
- Adoption du design system v2 (`sobr-ia-design-system/` Claude Design) : palette `ink #0a0d0b` / `lime #c5f04a` / `ivory #f0ece3`, **Instrument Serif italic** (display) + **Geist** (UI) + **JetBrains Mono** (chiffres), grille 4 px, animations 150-300 ms ease-out.
- 8 WOFF2 self-host dans `web/static/fonts/` (latin + latin-ext seulement, ~155 ko total), CSP `default-src 'self'` respectée.
- Licences fontes documentées dans `docs/LICENSES-FONTS.md` (SIL OFL 1.1 + SHA-256 par fichier).
- Tokens design dans `web/src/app.css`, composants atomiques (Composer, ResultBlock, HypothesisBlock, ComingSoon).

#### Écrans livrés

- **Estimer (M2)** route `/` : hero éditorial, composer (sélecteur modèle custom + textarea prompt + tokens auto + datacenter + mix), résultat avec hero metric CO₂eq P50 + **distribution Monte-Carlo SVG depuis `bins`** (option A1, fallback gaussien), 3 indicateurs side (énergie + eau + métaux), équivalents parlants, drawer hypothèses, signature ledger cliquable, CTA cross-screen → Comparer.
- **Journal d'audit (M7)** route `/journal` : table paginée (limit 50), drawer détail entrée, verdict d'intégrité, export NDJSON via `@tauri-apps/plugin-dialog`, query param `?focus=N` depuis Signature.
- **Workbench (M3)** route `/workbench` : filtres multi-select (provider / calibration / ouverture) + recherche full-text client + tri 5 colonnes (`aria-sort`), drawer fiche modèle avec sources cliquables, CTA « Estimer avec ce modèle » (`?model=<id>`).
- **Comparer (M5)** route `/comparer` : sélection 2-8 modèles, fan-out `estimatePrompt` parallèle (Promise.allSettled), **verdict éditorial style M15** (« X est Y× plus sobre que Z » + delta `−N %`), cards par modèle avec barres normalisées CO₂eq / énergie / eau + intervalle P5-P95, drawer hypothèses + sources + lien ledger. Accepte `?prompt=...&tokensOut=...&model=...` depuis Estimer.
- **Méthodologie (M8)** route `/methodo` : formule de référence, validation croisée (3 cartes), glossaire FR/EN 15 termes, références normatives (AFNOR SPEC 2314, ISO 21031, ITU-T L.1410, GHG Protocol, ADEME), bibliographie sélective. TOC sticky desktop / horizontal mobile.
- **Paramètres** route `/parametres` : runtime via `meta_info` IPC (version app, seed, N, audit path, data root) + boutons copie clipboard, section « Préférences à venir » (Thème, Langue, Seed perso, Télémétrie) avec leur chantier.

#### Stubs documentés (4 routes)

- `/simuler` (M4 · C10), `/importer` (M10 · C11), `/territoire` (M9+M12 · C12), `/exporter` (M6 · C10).
- Composant partagé `<ComingSoon>` : eyebrow amber « Module Mx · en chantier » + carte status dashed avec liste explicite des **IPC Rust attendus** + EF couvertes du CDC + chantier prévu.

#### Méthodologie

- Distribution Monte-Carlo journalisée : **option A1 actée** — `IndicatorValue.bins: Option<DistributionBins>` dans `sobria-core`, 50 bins équi-width sur les 10⁴ tirages, présents dans le payload audit (~600 B / entrée). Front rend la queue droite log-normale réelle ; fallback gaussien si bins absentes.
- Tokenizer FR : `prompt.length / 3.3` (médiane FR 3,0-3,5) avec tooltip pointant vers le tokenizer BPE de v0.3.
- Auto-rescale d'unité : `pickScale(p50, baseUnit)` choisit kg/g/mg/µg/ng selon le P50 — cohérence inter-percentile garantie, plus de `0,00 g` pour les petits modèles.
- Format numérique partout en `maximumSignificantDigits: 3` (Intl FR).
- Verdict comparateur basé sur le ratio worst/best CO₂eq P50 (pas de score composite à poids, mathématiquement faux tant que les indicateurs sont colinéaires en v0.2 du moteur).

#### Tooling

- ESLint : `typescript-eslint` méta-package (remplace les splits `@typescript-eslint/eslint-plugin` + `parser`).
- Prettier : `.prettierignore` + `.svelte-kit` / `build` / `node_modules` / `test-results` / `playwright-report` ignorés.
- `npm run clean` : nettoie `.vite`, `.svelte-kit`, `build`, `test-results`, `playwright-report` (rimraf cross-OS) — palliatif au cache Vite sale après changement de deps (cf. migration lucide).
- `npm run clean:full` : idem + supprime `node_modules` + relance `npm install`.
- `lucide-svelte` (Svelte 3/4 legacy) → `@lucide/svelte` (officiel Svelte 5 runes).
- Playwright : 12 tests « no-mock contract », `workers: 1` pour éviter les courses sur le dev Vite partagé.

#### Tauri (côté Rust, par Cowork)

- Runtime Tauri activé (`tauri::Builder` + `generate_context!()` + icônes via `npx tauri icon` + `tauri-plugin-dialog`).
- 6 commandes IPC enregistrées (`meta_info`, `list_models`, `estimate_prompt`, `verify_audit`, `list_audit_entries`, `export_audit_ndjson`).
- `IndicatorValue.bins` (option A1) + `bin_samples()` helper avec garde `N >= 10` & `min < max`.
- Capabilities `dialog:default`, `dialog:allow-save`, `dialog:allow-open` dans `crates/sobria-app/capabilities/default.json`.

#### Vérifications

- `npm run check` : 0 erreurs / 3 784 fichiers.
- `npm run lint` : OK (prettier + eslint).
- `npx playwright test` : **12 passed (5 s)** — contrat « no-mock » verrouillé sur les 6 écrans fonctionnels + 4 smoke tests sur les stubs.

Voir `briefs/chantiers/C09-RETROSPECTIVE.md` pour le détail des décisions méthodologiques et la feuille de route C10+.

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
