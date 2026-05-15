# Changelog Sobr.ia

Toutes les modifications notables sont documentées ici, conformément à [Keep a Changelog 1.1.0](https://keepachangelog.com/fr/1.1.0/) et [SemVer](https://semver.org/).

Format : `[X.Y.Z] - YYYY-MM-DD`
Types : `Added`, `Changed`, `Deprecated`, `Removed`, `Fixed`, `Security`.

## [0.5.0] — 2026-05-15 — Activation du pipeline médaillon (C26)

### Added — C26.1 Câblage CLI sobria-ingest

- **`LayerRegistry::standard()`** instancie désormais `ComparIASource` +
  `RteIrisSource` (Tier 1 du défi data.gouv.fr). Avant 0.5.0 le registre
  standard était vide (`TODO(sobria-003)`).
- **Binaire `sobria-ingest`** complètement câblé : `pipeline run`, `copper`,
  `silver`, `gold`, `validate` appellent les vraies méthodes du registre.
  Plus de stubs `tracing::info!("... (stub)")`.
- **Module `sobria-ingest::cli`** (testable) avec :
  - `build_context(incremental)` honore `SOBRIA_DATA_ROOT` + `SOBRIA_SEED`.
  - `build_context_with(data_root, seed, incremental)` variante injectable
    pour les tests parallèles.
  - `filter_registry(Option<&str>)` filtre le registre standard sur une
    source (`--source <id>`) avec erreur claire si l'id est inconnu.
  - `standard_source_ids()` introspection des sources Tier 1.
- **`CopperManifest::verify_files(&self, dir)`** : recalcule le SHA-256 de
  chaque fichier du snapshot et compare au hash enregistré. Utilisé par la
  sous-commande `validate` pour détecter la corruption.
- **Sous-commande `validate`** parcourt `data/copper/<source>/<date>/`,
  charge chaque `manifest.json`, recalcule les hashes, et reporte OK/KO.
  Code de sortie ≠ 0 si au moins un manifest est corrompu.

### Added — C26.2 Schémas Silver versionnés + validation à l'écriture

- **Module `sobria_ingest::silver_validate`** : valide chaque entité Silver
  contre son schéma JSON Schema 2020-12 versionné (`schemas/silver/<entity>-v<n>.json`)
  avant retour à l'orchestrateur. Lit le schéma Arrow du Parquet via
  `polars::LazyFrame::scan_parquet`, vérifie la présence des colonnes
  `required` et la compatibilité des types (`string` ↔ `Utf8`, `integer` ↔
  `Int*`, etc.).
- **Schémas Silver** : 4 schémas embarqués via `include_str!`
  (`comparia_conversations`, `comparia_votes`, `comparia_reactions`,
  `rte_iris_consommation`). Chaque schéma exige les colonnes lineage
  systématiques `_copper_sha256` (regex hex 64) et `_ingested_at` (RFC 3339).
  Le schéma RTE IRIS exige en plus la maille `code_iris` — clé de jointure
  unique avec le référentiel INSEE et le futur `datacenter_iris_link` Gold.
- **`CopperSnapshot::from_manifest(snapshot_dir)`** (sur `crates/sobria-ingest/src/layer.rs`) :
  reconstruit un snapshot Copper à partir d'un dossier persistant
  (`data/copper/<source>/<YYYY-MM-DD>/`) en chargeant `manifest.json` et
  en vérifiant l'intégrité de chaque fichier (`verify_files`). Permet à la
  sous-commande `silver` de repartir d'un Copper figé sans re-télécharger.
- **`cli::latest_copper_snapshot(ctx, source_id)`** + **`cli::rehydrate_copper(ctx, registry)`**
  exposés publiquement et testables. Le second produit un
  `Vec<StepResult<CopperSnapshot>>` directement consommable par
  `LayerRegistry::run_silver` — équivalent fonctionnel d'un `run_copper`
  mais lu depuis disque.
- **Tests** : `crates/sobria-ingest/tests/silver_validation.rs` (proptest +
  golden snapshots insta sur les 4 schémas) et `tests/copper_rehydrate.rs`
  (round-trip ingest → from_manifest → promote_silver, détection de
  corruption, message d'erreur explicite quand aucun snapshot n'existe).

### Added — C26.3 Gold complet (jointures + datasheet Gebru)

- **Tables matérialisées dans `referentiel.sqlite`** :
  - `model_overview(id, name, family, vendor, n_conversations)` — un modèle
    par ligne, peuplé depuis `comparia_conversations` Silver (extraction
    distincte sur `model_id`/`model`/`model_name` + heuristique
    famille/vendor).
  - `scenario_inputs(model_id, country_iso, pue, if_g_per_kwh, wue_l_per_kwh)`
    — table dénormalisée prête pour le simulateur M13.
  - `time_series_mix(region_iso, hour_utc, production_mw)` — placeholder
    v1, peuplé en v2 quand RTE eco2mix sera ingéré.
  - `comparison_matrix(model_id, method, co2_g_per_request, computed_at)` —
    vide à l'init, remplie au runtime par l'app.
  - `datacenter_iris_link(datacenter_id, code_iris, distance_km, …)` —
    join géographique datacenter européen ↔ maille IRIS la plus proche
    (haversine sur centroïdes IRIS extraits du GeoJSON Copper RTE).
- **Index FTS5** : table virtuelle `model_overview_fts(name, family,
  vendor)` pour la recherche full-text M9.
- **Module `sobria_ingest::iris_link`** : parser GeoJSON IRIS, calcul de
  centroïdes, distance haversine WGS84, jointure nearest-neighbor
  datacenter ↔ IRIS. Tolérant : si le snapshot Copper RTE est absent ou
  vide, la table reste vide sans casser le pipeline.
- **Module `sobria_ingest::datasheet`** : Datasheet for Datasets (Gebru
  et al. 2018, doi:10.48550/arXiv.1803.09010) avec les 7 sections
  obligatoires (Motivation, Composition, Collection, Preprocessing, Uses,
  Distribution, Maintenance) + JSON-LD multi-vocabulaire (schema.org +
  DCAT 3 + PROV-O + vocabulaire Sobr.ia). Validée à l'écriture contre
  `schemas/gold/datasheet-v1.json` — toute datasheet incomplète fait
  échouer l'assemblage Gold.
- **Schéma Gold `schemas/gold/datasheet-v1.json`** : JSON Schema 2020-12
  qui formalise le format de la datasheet (sections requises, types des
  champs, regex SHA-256 sur les hashes Copper et artefacts).
- **Signature GPG optionnelle** : si la variable d'environnement
  `SOBRIA_GPG_KEY_ID` est définie, `MANIFEST.sha256` est signé en
  détaché ASCII (`MANIFEST.sha256.asc`) via `gpg --detach-sign`.
  Skippable silencieusement si la variable n'est pas définie ou si gpg
  est absent du PATH (compatible CI sans clé).
- **Dépendance `sobria-ingest` → `sobria-geoloc`** : ajout pour accéder à
  `all_datacenters()` lors de l'assemblage Gold.
- **Tests** : extension de `tests/gold_pipeline.rs` (vérifie les 5
  nouvelles tables + FTS5 fonctionnel + inference vendor + datasheet
  Gebru complète) + nouveau `tests/datasheet_jsonld.rs` (8 cas couvrant
  validation contre schéma, présence des 7 sections, lineage Copper
  intact, rejet d'une section manquante).

### Changed

- `LayerRegistry::standard()` n'est plus une simple alias de `new()` : elle
  retourne le registre Tier 1 par défaut.
- `print_pipeline_report` affiche désormais les chemins des 4 artefacts
  Gold + le résumé chronométré du pipeline.
- **`promote_silver` ComparIA + RTE IRIS** appelle systématiquement
  `silver_validate::validate_silver` avant d'ajouter une entité au résultat.
  Une entité dont le Parquet ne respecte pas son schéma versionné fait
  échouer toute la promotion Silver de la source concernée.
- **Sous-commande `silver`** : ne ré-ingère plus la couche Copper en amont.
  Elle réhydrate les snapshots Copper persistants via `rehydrate_copper`
  et échoue avec un message explicite si aucun snapshot n'est disponible
  pour une source du registre filtré.
- **`assemble_gold`** : pré-calcule la jointure `datacenter_iris_link`
  avant d'ouvrir la transaction SQLite, puis enchaîne SQLite (avec toutes
  les vues matérialisées + FTS5), Parquet catalogue, datasheet validée,
  manifest hashé, signature GPG optionnelle.
- **`GoldArtifacts`** expose un nouveau champ `manifest_signature:
  Option<PathBuf>` qui pointe vers `MANIFEST.sha256.asc` quand la
  signature GPG a réussi.

### Added — C26.4 Orchestration DVC + CI nocturne

- **`.dvc/config`** : remote local par défaut (`.dvc-cache/`) +
  template pour basculer vers S3 (`dvc remote default s3-prod` après
  `dvc remote modify`).
- **`.dvc/.gitignore`** : exclut `cache/`, `tmp/`, `plots/` du repo Git.
- **`.dvcignore`** étendu : ignore `target/`, `node_modules/`, `dist/`,
  `build/`, `.svelte-kit/`, `.vite/`, fixtures de tests, notebooks
  rendus, etc.
- **`.gitignore`** : ajoute `.dvc-cache/` (le remote local DVC ne doit
  jamais être poussé sur GitHub).
- **`docs/operations/dvc.md`** : guide opérateur (~150 lignes) avec
  quick start, table des stages, politique de rétention référencée à
  ADR-0009, instructions pour basculer vers un remote S3/HTTP, et FAQ
  (différence avec Git LFS, rôle de `dvc.lock`, vérification de
  reproductibilité bit-à-bit, dépannage `dvc: command not found`).
- **Workflow `.github/workflows/dvc-nightly.yml`** : job cron quotidien
  03:00 UTC + déclenchement manuel (`workflow_dispatch` avec input
  `force`). Étapes : checkout, install Rust + Python + DVC + dvc-s3,
  configuration conditionnelle remote S3 et clé GPG (skip si secrets
  absents), `dvc pull`, `cargo build --release`, `dvc repro`,
  `validate`, `dvc push`, upload des artefacts Gold (rétention 30 j),
  summary Markdown avec hashes SHA-256.

### Changed

- `dvc.yaml` : annotations `desc` enrichies pour les 3 stages, garantie
  de reproductibilité documentée. (Stages `copper`, `silver`, `gold`,
  `validate` déjà présents depuis C01-C04, juste annotations améliorées.)

### Added — C26.5 Reconnexion app au Gold

- **Crate `sobria-referentiel`** désormais fonctionnelle (auparavant
  squelette vide) :
  - `Referentiel::open(&Path)` ouvre le SQLite Gold en lecture seule
    (mode WAL `SQLITE_OPEN_READ_ONLY`).
  - `load()` honore `SOBRIA_REFERENTIEL_PATH` (défaut
    `data/gold/referentiel.sqlite`).
  - `Referentiel::status()` renvoie un `ReferentielStatus { version,
    snapshot_date, sha256, source_count, model_count, path }` —
    SHA-256 calculé en streaming pour les gros fichiers.
  - Tolérant aux Gold legacy : si `model_overview` n'existe pas (Gold
    pré-C26.3), `model_count` retourne 0 sans erreur.
  - 5 tests unitaires + 1 doctest.
- **IPC Tauri** (`get_referentiel_status`, `reload_referentiel`) :
  - `get_referentiel_status` ne lance jamais d'erreur — encapsule
    l'absence du fichier dans `available=false` + `message`, pour que
    l'UI puisse proposer une action plutôt que crasher.
  - `reload_referentiel` invoque `dvc pull` via `std::process::Command`,
    capture stdout/stderr (tronqués à 4 ko), retourne le statut résultant.
    Skip silencieux si DVC est absent du PATH (message d'aide explicite).
- **`crates/sobria-app/src/dto.rs`** : `ReferentielStatusDto` +
  `ReferentielReloadResultDto` (mirroir TS dans `web/src/lib/api.ts`).
- **Web `web/src/lib/api.ts`** : types + fonctions
  `getReferentielStatus()` / `reloadReferentiel()`.
- **Page Paramètres `/parametres`** : nouvelle section "Référentiel"
  (au-dessus de "Runtime") avec :
  - Statut (available/unavailable badge + message).
  - Version, snapshot (formaté FR), SHA-256 tronqué (12 chars + tooltip
    full hash), nombre de sources, nombre de modèles, chemin du SQLite.
  - Bouton "Recharger le référentiel" qui appelle `reload_referentiel`,
    affiche le résultat (succès / erreur DVC).
  - Callout warning explicite quand le Gold est absent.

### Changed — C26.5

- **`crates/sobria-geoloc/data/datacenters.json` → `datacenters_demo.json`**
  pour distinguer la donnée **statique embarquée** (fallback hors-ligne
  M9/M12) du **référentiel Gold dynamique** produit par le pipeline
  médaillon. La doc-comment du module l'explicite. À terme, une source
  Tier 2 (Cloud Carbon Footprint, Climatiq Datacenters…) alimentera
  dynamiquement cette table dans le Gold ; le fichier embarqué restera
  comme fallback.

### Fixed — workspace clippy

- Plusieurs lints `clippy::pedantic` / `clippy::cast_lossless` /
  `clippy::manual_string_new` / `clippy::format_push_string` /
  `clippy::same_item_push` corrigés dans `sobria-app` et `sobria-export`
  pour permettre `cargo clippy --workspace --all-targets -- -D warnings`
  propre.

---

## [0.4.0] — 2026-05-14 — Catalogue multi-méthodologie (C24 + polish A-H)

### Added — Catalogue multi-méthodologie (chantier C24)

- **Trait `EmpreinteEngine`** dans `sobria-estimator` : interface commune
  à toutes les méthodologies d'empreinte LLM embarquées.
- **Type `EmpreinteMethod`** (sobria-core) : enum stable `afnor_sobria` /
  `ecologits` partagé par tous les crates.
- **`EcoLogitsEngine`** : port direct des formules officielles EcoLogits
  2026-01 (Rincé & Banse 2025, doi:10.21105/joss.07471, CC BY-SA 4.0).
  Reproduction validée à ≤ 1 % vs Python notebook.
- **Page `/methodologies`** : catalogue UI permettant à l'utilisateur de
  choisir sa méthodologie par défaut + d'activer d'autres méthodos en
  référence (panneau « Voir aussi » dans M1 Atelier).
- **IPC `list_methodologies` + `estimate_for_comparison`** : exposent le
  catalogue et permettent les calculs comparatifs éphémères (non
  journalisés).
- **Migration audit ledger v1 → v2** : nouvelle colonne `method` sur
  `audit_entries`, idempotente. Les ledgers historiques conservent leur
  intégrité SHA-256 ; les entrées pré-C24 sont étiquetées rétroactivement
  `afnor_sobria` (seul moteur historique).
- **ADR-0012** : décision multi-méthodologie complète (contexte, 4
  alternatives rejetées, conséquences).
- **Notebook `notebook/validation.qmd`** : reproduction Python des 3
  ReproductionCase EcoLogits, exécutable de bout en bout.

### Fixed — Audit B (mai 2026)

- **`K_DECODE_MJ_PER_TOKEN_PER_B`** recalibré de `0.025` à `25.0` (factor
  1000 manquant). Toutes les estimations Sobr.ia produites avant 0.4
  étaient sous-évaluées d'un facteur ~1000.
- **Bug RTE eco2mix** : `FACTOR = 0.25/1e6` → `0.5/1e6` (le pas réalisé
  est 30 min, pas 15 min). Production totale FR 2023 passe de 243 TWh
  (faux) à 487 TWh (≈ 2 % du Bilan RTE 2023 publié).
- **`REPRODUCTION_CASES` vide** : remplacé par 3 cas réels reproduits à
  ≤ 1 % contre EcoLogits Python (Llama 3.1 70B FR/USVA + Mistral Large 2).

### Changed — Cohérence multi-méthodologie (Polish A → H)

- **A** Hygiène ledger : panneau « Voir aussi » via `estimate_for_comparison`
  (non journalisé).
- **B** Discoverability : cross-links `/methodo` ↔ `/methodologies`,
  rail labels désambigus.
- **C** Badges méthodo visibles : M1 ResultBlock + colonne Journal.
- **D** M3 Comparer modèles honore `default_method`.
- **E** Dashboard breakdown par méthodologie + warning multi-méthodo.
- **F** PDF CSRD + sidecar PROV-O tracent les méthodos réellement utilisées.
- **G** M9 fiche modèle, M12 datacenter, M13 simulateur, M16 forecaster,
  M17 datasheet Gebru → tous routés via `engine_for(default_method)`.
  `simulate()` et `forecast_yearly()` prennent désormais
  `&dyn EmpreinteEngine`.
- **H** Bump version 0.2.0 → 0.4.0, `/parametres` expose le choix
  méthodologie, onboarding mentionne le catalogue, toggle FR/EN désactivé
  (i18n v1.1), cleanup routes zombies (`/workbench`, `/importer`,
  `/exporter`).

### Removed

- Faux toggle FR/EN dans `/parametres` désactivé (i18n non implémentée,
  sera v1.1).
- Routes orphelines `/workbench` (doublon `/m9`), `/importer` et
  `/exporter` (ADR-0011 différé v1.1+).

### Fixed — Bugs UAT I1 → I5 (post-polish)

- **I1 — Dashboard M15 axe X illisible** (`web/src/routes/m15/+page.svelte`) :
  labels « ma18 ma19 ma10 » qui se chevauchaient. `shouldShowXLabel` passe
  d'un seuil binaire à un stride adaptatif `ceil(n / 10)`, garantissant
  au plus ~10 labels visibles quelle que soit la fenêtre.
- **I2 — M25 Eco-budget bouton « Enregistrer » muet**
  (`web/src/routes/m25/+page.svelte`) : Svelte 5 + `<input type="number">`
  + `bind:value` coerce silencieusement en `number`, le `.trim()` ultérieur
  faisait planter le handler. Passage en `type="text" inputmode="decimal"`
  + parsing explicite `parseFloat(value.replace(',', '.'))`.
- **I3 — M20 Territoire FR carte vide malgré données présentes**
  (`crates/sobria-geoloc/src/{sankey_fr,territoire_fr}.rs` + UI) :
  fichiers présents dans le repo mais absents du `data_root` au runtime.
  Fix : `const DEFAULT_*_JSON: &str = include_str!(...)` avec fallback
  embarqué dans `load_*()`. Ajout d'un bouton « Recharger » (icône
  `RefreshCw`) côté UI pour invalidation manuelle.
- **I4 — M12 Datacenters carte invisible**
  (`web/package.json` + `web/src/lib/components/m12/DatacenterMap.svelte`
  + `web/src/routes/datacenters/+page.svelte`) : root cause = `leaflet`
  et `@types/leaflet` absents de `package.json`. Le dynamic import
  échouait silencieusement. Fix dépendances + défensifs CSS :
  `.map-wrapper { height: 560px }` explicite, `requestAnimationFrame`
  avant `map.invalidateSize()`, `ResizeObserver` sur le conteneur,
  `.col-c { display: block }` au lieu de flex. M20 bénéficie du même fix
  (même pattern Leaflet).
- **I5 — Modules différés invisibles dans `/parametres`**
  (`web/src/routes/parametres/+page.svelte`) : section unique « Modules
  disponibles » splittée en deux — _Modules disponibles_ (activables tout
  de suite) et _À venir en v1.1+_ (badge `v1.1` lime, contrôles désactivés
  pour les modules ADR-0011 différés).

---

## [Unreleased]

### Added — Chantier C10 : Onboarding personas + module gating (S6 / S7) — `v0.3.0-onboarding`

> ADR-0010 « Personas et gating modulaire par préférences utilisateur ». Brief `briefs/chantiers/C10-onboarding-personas.md`. Bundle Tauri + frontend.

#### Backend Rust (par Cowork — déjà mergé en `feat(app): C09 Estimer screen + C10 personas/gating IPC`)

- `sobria-core::preferences` : enums `Persona` (5 valeurs : `student`, `pro_tech`, `enterprise`, `public_sector`, `researcher`) et `ModuleId` (24 valeurs `m1..m25` sans `m4` réservé) sérialisables JSON. `Persona::default_modules` mirror des bundles ADR-0010. 12 tests garantissent : M1 dans tous les bundles, pas de doublons, M4 absent, serde round-trip, bundles ⊆ `ModuleId::all`.
- `sobria-app::dto::AppPreferencesDto` (persona + enabled_modules + onboarded + lang) + `defaults()` = bundle `ProTech` (cf. ADR-0010 §"Onboarding non-bloquant").
- Table `app_preferences` créée dans `referentiel.sqlite` (4 clés : `persona`, `enabled_modules`, `onboarded`, `lang`).
- 2 commandes IPC : `get_app_preferences` (renvoie defaults si vide) et `set_app_preferences` (validation persona/modules/lang + UPSERT transactionnel).

#### Frontend SvelteKit (chantier C10.2)

- `web/src/lib/api.ts` : ajout `Persona`, `ModuleId`, `AppPreferencesDto` (types snake_case mirroir Rust) + `getAppPreferences()` / `setAppPreferences(prefs)`.
- `web/src/lib/preferences.ts` : store typé strict `writable<PreferencesState>` avec flag `loaded`, `loadPreferences()` au boot, `savePreferences()` optimistic + rollback IPC, helpers `defaultModulesFor`, `moduleLabel`, `moduleDescription`, `moduleCategory`, `personaLabel`, `personaTagline`, `personaEmoji`. Catalogue des 24 modules + 5 personas, sans persistance LocalStorage (CLAUDE.md §13).
- **Route `/onboarding` (4 étapes, Svelte 5 runes)** :
  1. _Splash_ — logo + tagline italique + mission ; auto-advance 3 s ou clic « Continuer ».
  2. _Persona picker_ — 5 cartes (`student`, `pro_tech`, `enterprise`, `public_sector`, `researcher`) + lien « choisir à la carte ».
  3. _Bundle_ — checkboxes pré-cochées du persona (8-11 mod.) + section collapsable « + Plus de modules disponibles » (24 - bundle), compteur live.
  4. _Premier prompt_ — aperçu illustré de l'atelier M1 avec tooltip lime animé sur le sélecteur de modèle, bouton « Terminer » (set `onboarded=true` + `window.location.replace('/')`) + lien discret « Passer cette étape ».
- **Garde de layout** dans `+layout.svelte` : `onMount` → `loadPreferences()` ; si `!onboarded && pathname !== '/onboarding'` → `window.location.replace('/onboarding')`. Hors Tauri (IPC indisponible), pas de redirection — le rail reste affiché avec tous les modules (mode coque).
- **Rail filtré par `enabled_modules`** : chaque entrée du rail porte un `data-module-id` (m1..m25). `visible()` masque les entrées non activées une fois les préférences chargées. Bouton « + Ajouter des modules » lime persistant en bas du rail → `/parametres`.
- **Route `/parametres` réécrite** : 5 sections — _Persona courant_ (avec sélecteur 5 boutons + dialog de confirmation `aria-modal` avant remplacement du bundle), _Modules activés_ groupés par catégorie (Estimation / Visualisation / Reporting / Pédagogie), _Modules disponibles_ non cochés, _Réinitialiser & langue_ (bouton « Refaire l'onboarding » + radio FR/EN pour préparer C12), _Runtime_ (lecture seule via `meta_info`). Toutes les écritures passent par `savePreferences` (optimistic + rollback).
- **Gardes de route**  posées sur M1 (`/`) et M13 (`/simuler`) : `$effect` réactif au store ; si `preferences.loaded && !enabled_modules.includes(MODULE_ID)` → `window.location.replace('/?disabled=mXX')`.
- **Bandeau « module désactivé »** sur `/` : `?disabled=mXX` → coral dashed avec lien « → Activer dans Paramètres ».
- **Correctif `/simuler`** : `moduleId` mis à jour de `M4` (réservé) vers `M13` (Simulateur « Et si...? »), libellé chantier passé à C11.

#### Vérifications

- `npm run check` : 0 erreurs (1 warning préexistant sur types `node`).
- `npm run lint` (Prettier + ESLint) : propre.
- 4 tests Playwright `tests/onboarding.spec.ts` (contrat no-mock, contexte navigateur) : splash → persona picker → bundle Étudiant pré-coché à 8 modules, dépliage « + Plus de modules » sur bundle Enterprise (M22/CSRD coché), tentative « Terminer » hors Tauri affiche la bannière d'erreur, garde de route `/?disabled=m13` affiche le bandeau coral.
- `tests/parametres.spec.ts` mis à jour pour la nouvelle UI (persona/modules/runtime) + stub `/simuler` désormais M13 au lieu de M4.

#### Non-objectifs (différés)

- Bundle « partager mon bundle avec un collègue » → C11+.
- Traduction EN complète → C12.
- Tutoriel interactif au-delà du tooltip étape 4 → backlog v1.1.
- Mode multi-utilisateurs → backlog v1.1.

---

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
