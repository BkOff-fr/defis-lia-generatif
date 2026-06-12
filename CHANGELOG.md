# Changelog Sobr.ia

Toutes les modifications notables sont documentées ici, conformément à [Keep a Changelog 1.1.0](https://keepachangelog.com/fr/1.1.0/) et [SemVer](https://semver.org/).

Format : `[X.Y.Z] - YYYY-MM-DD`
Types : `Added`, `Changed`, `Deprecated`, `Removed`, `Fixed`, `Security`.

## [Non publié]

### Added — C46 CI réparée + gate a11y réel

- **Suite a11y réelle** (`web/tests/a11y/`) : audit axe-core (WCAG 2A/AA)
  des 5 pages clés en mode démo, gate sur les violations
  critical/serious — le job CI « a11y » pointait vers un répertoire
  inexistant depuis sa création.
- **`<html lang="fr">`** : le placeholder `%lang%` n'était jamais
  substitué et partait tel quel en prod (défaut a11y détecté par l'audit).
- **Port e2e dédié** (`SOBRIA_E2E_PORT`, `--strictPort`) :
  `reuseExistingServer` pouvait silencieusement tester le dev server d'un
  AUTRE projet ouvert sur 5173.
- **CI** : tests unitaires extension ajoutés au job (90 tests, jamais
  exécutés en CI auparavant) ; couverture tarpaulin passée en informatif
  avec artefact publié — le gate `--fail-under 80` n'avait jamais produit
  une seule mesure en 41 runs.

### Fixed — C46 Violations a11y réelles (premier passage du gate axe)

- **Contraste AA** : `--ivory-3`/`--ivory-4` recalés (4,63→5,5:1 et
  2,9→4,9:1 sur le panneau le plus clair) — l'ancien ivory-4 était
  documenté « décoratif seulement » mais portait du texte utile dans
  ~30 composants (jusqu'à 33 nœuds non conformes sur /comparer).
- **ARIA** : nom accessible sur le combobox datacenter, `role="img"` sur
  les icônes Oui/Non du tableau vendors, `role="listitem"` sur les
  lignes de méthodologies (le `role="list"` parent était orphelin —
  violation critical).

### Fixed — C46 Stabilisation pré-commit (audit 2026-06-12)

- **Route détail employé inopérante** : `/admin/users/{id}/analytics`
  utilisait la syntaxe de paramètre d'axum 0.8 dans un projet en axum
  0.7 (`{id}` y est un littéral) → 404 systématique sur l'endpoint
  vedette de C44. Corrigé en `:id` + test d'intégration HTTP couvrant
  les 3 politiques ADR-0016 (la route n'en avait aucun).
- **`cargo test -p sobria-team-aggregator` compile à nouveau** : helper
  de test des alertes non mis à jour après le DDL v4 (champ `project`
  manquant).
- **CORS pour l'extension** : le service worker MV3 appelle l'API
  équipe depuis une origine `chrome-extension://…` — sans en-têtes CORS
  Chrome bloquait la réponse et l'envoi best-effort échouait en silence.
  `CorsLayer` limité aux schémas d'extensions navigateur (jamais
  d'origine web), preflight couvert par test d'intégration.
- **Manifests extension resynchronisés** : Firefox gelé à 0.6.0 avec une
  description antérieure au Mode Équipe → aligné sur 0.9.0 ; typo
  « sou self-hosted » corrigée côté Chrome ; artefacts de packaging
  locaux (`Sobria.zip`, copies de `dist-firefox`) ignorés par git.

### Added — Docs de clôture (post-C45)

- **ADR-0017** : le « contrat démo web » (C37) enfin formalisé —
  fixtures du moteur réel, jamais dans l'app de bureau, transparence
  permanente, couverture partielle assumée.
- **`briefs/COMMIT-PLAN-C43-C45.md`** : addendum au plan de commits
  (lots 9-16) avec rappel des vérifications avant commit.
- `/equipe` : lien ADR-0016 câblé (était en texte nu).

### Added — C45 Manifeste immersif « Le poids invisible »

- **`/manifeste`** : expérience en 7 scènes-mots (Invisible · Mesurer ·
  Douter · Choisir · Ensemble · Ouvrir · Commencer) — déclarations
  géantes, reveals au scroll, marquees, compteur et pliage k-anonyme
  animés, chiffres réels du moteur (seed 42). Direction inspirée des
  références validées (manifeste en scènes + jeu typographique).
  Amélioration progressive stricte : page complète sans JS,
  `prefers-reduced-motion` intégral, zéro dépendance.
- Entrées : lien hero de l'accueil + footer. DA + storyboard :
  `briefs/chantiers/C45-manifeste-immersif.md`.

> ⚠ C45 : v1 écrite SANS rendu écran (environnement indisponible) —
> itération visuelle obligatoire avant mise en avant (brief §5).

### Added — C44 Politique de visibilité + dimension projet (ADR-0016)

- **`visibility_policy`** (anonymous | opt_in défaut | identified) :
  choisie au déploiement, modifiable par CLI ; le mode nominatif exige
  une **attestation CSE/salariés** (`config set visibility_policy
  identified --attest "…"`), stockée et tracée. Matrice appliquée côté
  serveur sur analytics, liste employés et détail individuel.
- **Détail par employé** : `GET /admin/users/:id/analytics` + page
  web-team `/admin/users/[id]` (série quotidienne, modèles, méthodos) —
  403 expliqué quand la politique l'interdit.
- **Dimension projet** : `estimations.project` (DDL v4), étiquette par
  CONVERSATION choisie dans le popup extension (clé URL d'onglet,
  résolue au dispatch via sender), agrégats « Par projet » au dashboard
  avec repli k-anonyme (« autres projets ») hors mode nominatif.
- `/me/sharing` renvoie la politique : chaque salarié sait sous quel
  régime il travaille (ligne dédiée dans son espace).
- **Site** : nouvelles pages `/produit`, `/equipe` (3 politiques,
  tableau comparatif), `/methode` ; `/telecharger` restructuré ;
  `/cloud` → `/equipe` ; topbar/footer réécrits (51 pages, build vert).

> ⚠ C44 : vérification partielle (panne de l'environnement en cours de
> chantier) — voir `briefs/chantiers/C44-politique-projets-site.md` §5
> pour la frontière vérifié/non-vérifié et les commandes à lancer.

### Changed — C43 Vitrine : extension + site de présentation

- **Popup extension refondu** en 3 niveaux (chiffre du jour, état
  d'association, actions) avec détails repliés, états vides soignés,
  vouvoiement, contrastes AA, ≥ 12px ; indicateurs in-page apaisés
  (« fourchette x – y g (confiance 90 %) », fin du pulse infini).
  +19 tests (84/84 verts).
- **Site (accueil scrollytelling)** : chapitre Équipe aligné sur
  l'ADR-0015 (participants opt-in + agrégat anonyme + seuil k — fin du
  classement nominatif fictif), nouveau chapitre « Mesurer, puis
  réduire » (deltas réels du moteur), vouvoiement intégral, chiffres
  défendables (28 datacenters documentés), v0.9.0, CTA final 3 boutons
  (extension · app · Mode Équipe self-hosted).


### Changed — C42 Slugs parlants + cohérence hors-Tauri

- **URLs humaines** : `/m9 /m15 /m17 /m25` → `/modeles /suivi /datasheets
  /eco-budget`, redirections client depuis les anciennes adresses
  (query/hash préservés). Derniers codes internes retirés de la façade.
- **Plus d'actions qui mentent hors Tauri** : bannière au chargement de
  rapport-csrd (+ Générer désactivé), boutons ledger du journal,
  « Nouveau projet » (datasheets) et submit éco-budget désactivés avec
  explication au survol.

### Added — C42

- **CI `team-docker`** : build de l'image Mode Équipe (kit C40) + smoke
  test `/health` en conteneur — valide ce qui ne pouvait pas l'être en
  session.
- **`briefs/COMMIT-PLAN-C37-C42.md`** : 8 commits proposés, fichiers
  exacts, WIP préexistant isolé (224 fichiers hors périmètre).


### Changed — C41 Finitions utilisabilité

- **Modèle par défaut du Composer** : `gpt-4o-mini` (deprecated) →
  `claude-haiku-4-5` — la boucle « Réduire » montre de vraies
  alternatives dès la première estimation.
- **« Comparer en détail » pré-remplit le comparateur**
  (`?models=…&tin=…&tout=…`, ids validés, ≥ 2 requis).
- **Lexique inline** : 10 définitions (P50, P5–P95, prefill, PUE…) en
  tooltip accessible reliées au glossaire M8 — posées sur ResultBlock
  et Composer (`lib/lexique.ts`, `Term.svelte`).

### Fixed — C41

- **Paramètres** : bootstrap scindé — Runtime/Référentiel/Méthodologies
  s'affichent en démo même quand le pairing (desktop-only) rejette
  (bug C37 §1) ; dernier message `cargo run` remplacé.
- **ESLint** : global `__APP_VERSION__` déclaré (no-undef passé inaperçu,
  l'étape eslint étant tronquée par les timeouts sandbox).


### Added — C40 Kit de déploiement Mode Équipe (Docker + communication)

### Added — C40 Parcours première heure + boucle « Réduire »

- **Onboarding** : carte « Mesure automatique » à l'étape finale —
  génération du code de pairing extension sans passer par Paramètres.
- **Boucle « Réduire »** : sous chaque résultat, jusqu'à 3 alternatives
  plus sobres réestimées par le moteur (`estimate_for_comparison`),
  deltas %, liens Comparer / Éco-budget. Jamais d'heuristique client.
- **M15** : état vide accueillant avec CTA (premier prompt / extension).
- **Kit UAT** (docs/qa/uat/) : protocole, script SUS, 27 tâches sur
  5 personas, grilles — prêt à dérouler (C36).
- e2e : rail C39 asserté (essentiels + toggle « Plus ») et boucle
  Réduire couverte dans estimate.spec.

- **`deploy/team/`** : Dockerfile multi-stage (build SvelteKit `web-team`
  → build Rust avec dashboard embarqué rust-embed → runtime
  `debian:stable-slim` non-root, volume `/data`, healthcheck `/health`),
  `docker-compose.yml` (service unique, volume nommé) et `entrypoint.sh`
  (`init` automatique au premier démarrage via
  `SOBRIA_TEAM_ADMIN_PASSWORD`, puis `serve`).
- **`docs/operations/deploiement-equipe.md`** : guide PME ~30 min —
  démarrage Compose, Option A cert auto-signé / Option B reverse proxy
  Caddy + Let's Encrypt, codes d'enrôlement, réglages privacy ADR-0015,
  sauvegarde SQLite/WAL, mise à jour, dépannage. ⚠️ Images non testées en
  CI à ce jour.
- **`docs/operations/modeles-communication.md`** : modèles d'emails
  information-consultation CSE + annonce salariés (garanties ADR-0015,
  disclaimer « à faire valider par votre juriste »).

### Changed — C39 Rail simplifié + jargon retiré

- **Rail de navigation repensé** : 5 essentiels avec labels (Estimer,
  Comparer, Suivi, Modèles, Datacenters), 9 modules derrière « Plus »
  (état persisté, auto-dépliage si page active) — fini les 13 icônes
  mystères. Largeur 76 → 96px, snippet unique au lieu de 3 duplications.
- **Codes modules retirés de l'UI** : eyebrows « Module M9 · … »
  humanisés sur 12 pages (codes conservés dans la doc).

### Added — C38.x finitions aggregator

- **Rétention** : purge automatique des estimations > `retention_days`
  (défaut 730 j) au démarrage puis quotidienne.
- **CLI `config list|get|set`** (k_anonymity_min, retention_days) avec
  planchers validés ; doc opérateur étendue (privacy, CSE, RGPD).

### Fixed

- **Formatage web-team** : dérive Prettier préexistante résorbée sur
  l'ensemble du package.


### Added — C38 Dashboard équipe privacy-first (ADR-0015)

- **ADR-0015** : périmètre privacy du Mode Équipe — k-anonymat des
  agrégats admin (seuil `config.k_anonymity_min`, défaut 5, plancher 3),
  identification **opt-in contrôlée par le salarié**, self-service
  intégral, masquage appliqué côté serveur.
- **`users.share_identified`** (DDL v3) + **`GET|PUT /api/v1/me/sharing`**
  (route user-only, admin → 403) + toggle dans l'espace salarié web-team.
- **Garde k-anonymat** sur `/api/v1/admin/analytics` : sections vides +
  bloc `k_anonymity{required, active_users, blocked}` si activité
  insuffisante ; carte explicative côté dashboard.

### Changed — C38

- **`top_users` (nominatif) → `top_users_shared`** : participants opt-in
  nommés + agrégat anonyme `N participants` ; suppression du checkbox
  « Anonymiser » client-side (pseudo-protection).
- **`/api/v1/admin/users`** : `totals: null` sans consentement — la page
  Employés devient une vue de gestion des enrôlements.
- Tests : +5 unitaires (92 au total sur l'aggregator), intégration
  admin réécrite sur le contrat ADR-0015 (blocage k, opt-in, 403 admin).


### Added — C37 Mode démo web (fixtures du moteur réel)

- **Mode démo hors Tauri** : la démo web déployée sert désormais des
  fixtures précalculées par `sobria-estimator` (seed 42, N = 10⁴) au lieu
  d'écrans vides — 204 résultats Monte-Carlo (34 modèles × 2 méthodologies
  × 3 tailles), catalogue M9, datacenters M12, dashboard M15 étiqueté
  « (démo) ». Chargement paresseux : le bundle desktop n'embarque rien.
  Bannière « Mode démo » + hypothèse `mode_demo` sur chaque résultat.
  Générateur reproductible : `tools/fixturegen/`. Voir
  `briefs/chantiers/C37-mode-demo-web.md`.
- **`tools/fixturegen/`** : crate hors-workspace générant les fixtures
  depuis le vrai moteur (models/methodologies/estimates JSON).

### Changed — C37 a11y, typographie, voix

- **Contrastes WCAG AA** : `--ivory-3` 3,94:1 → 5,0:1 (#83817a) ;
  `--ivory-4` 2,01:1 → 3,1:1 (#62605a, réservé décoratif).
- **Échelle typographique en rem** (respect du réglage navigateur),
  corps 14 → 15px, plancher 12px (308 tailles 8-11px relevées dans
  41 fichiers, annotations SVG ≥ 10px).
- **Version dynamique** dans le rail (`__APP_VERSION__` via Vite define,
  était « v0.3.0 » hardcodé) + suffixe LOCAL/DÉMO.
- **Vouvoiement unifié** et messages d'erreur orientés utilisateur final
  (plus aucun `cargo run -p sobria-app` visible ; libellé « Application
  de bureau requise »).
- **Eyebrow home** : « Module M2 » → « Module M1 » (M2 hors périmètre v1.0).
- **Suite e2e Playwright** migrée du « contrat no-mock » au « contrat
  démo » (30 passed, 2 skipped documentés) ; en-tête de
  `playwright.config.ts` réécrit.

### Fixed — C37

- **`leaflet` manquant de `web/package.json`** (utilisé par M12/M20,
  installé localement sans `--save`) : `npm ci` aurait échoué en CI.
  Ajouté avec `@types/leaflet`, lock régénéré.
- **Hydratation morte en dev** : l'import runtime de `package.json` dans
  le layout violait `server.fs.allow` (403) → version injectée
  compile-time par `define`.


### Added — Claude Opus 4.8 + sync catalogue extension

- **Claude Opus 4.8** ajouté au registre Rust `MODEL_REGISTRY`
  (`crates/sobria-estimator/src/model_presets.rs`) — paramètres extrapolés
  même classe taille qu'Opus 4.7 (2000B, ε_decode P50 = 50 000 mJ/tok,
  embodied P50 = 0.5 gCO₂eq/req), `release_date: "2026-05-15"`.
- **Extension `MODEL_PRESETS`** synchronisée 8 → **34 presets** :
  catalogue C34 complet (Claude Opus/Sonnet/Haiku 4.x + 3.7 Sonnet,
  GPT-5.5 / 5.5 Thinking / 5.5 Pro / o3, Gemini 3.5 Flash / 3.1 Pro /
  2.5 Pro, Llama 4 Scout/Maverick + 3.3 70B, Mistral Medium 3.5 / Small 4
  / Large 3, DeepSeek V4 Pro / R1, Grok 4, Qwen 3.6 Plus, Phi-4
  Reasoning + Vision) + 8 modèles 2024 conservés `deprecated` pour
  reproductibilité audit ledger et parité tests historiques.
- **Mappings content scripts** étendus pour reconnaître les nouveaux
  modèles sur les UI vendor (claude.ai : Opus/Sonnet/Haiku 4.x ;
  chatgpt.com : GPT-5.5 / o3 ; chat.mistral.ai : Medium 3.5 / Large 3 /
  Small 4). Mode `specific-first` enforced pour éviter le matching
  greedy de `String.includes()`.

### Fixed

- **Bug latent matching modèle ChatGPT** : `text.includes('gpt-4o')`
  matchait avant `gpt-4o mini` (ordre d'insertion `Object.entries`).
  Réordonné specific-first dans les 3 content scripts. Sans correction,
  un user sur ChatGPT « GPT-4o mini » se voyait calculer comme GPT-4o.

### Fixed — Polish visuel UI (tokens fantômes + redondance)

- **DatacenterPicker illisible** : remplacement des tokens de design
  inexistants (`--accent` → `--lime`, `--ink-mute` → `--ivory-3`) et de
  `color: var(--ink)` (texte quasi-noir sur fond sombre) → `--ivory`. Le
  panneau déroulant passe d'un fond translucide (`--surface`) à un fond
  opaque (`--ink-2` + `--border-hi`). La valeur sélectionnée et les options
  sont désormais lisibles.
- **Filtres / drill-down M12** (`CountryDrillDown`, `DatacenterDrillDown`,
  `DatacenterFilters`) : `--ink-mute` invalide dans les bordures `color-mix`
  → `--ivory-3`.
- **Sankey M20** : sous-labels TWh `#72706a` → `#b8b4ac` (contraste) + taille
  8 → 9 px.

### Changed

- **M1 — équivalences** : suppression de la ligne texte `EquivalenceCarbon`
  (« ≈ cm voiture · s streaming · s LED ») qui doublonnait les tuiles
  visuelles « Pour mettre cela en perspective ». Le composant reste utilisé
  en M15/M25.
- **A11y** : affordance focus (`:focus-within` lime) sur les lignes de
  filtres M9/M12/M20 ; états hover/focus sur la recherche du DatacenterPicker ;
  espacement `.kpi` du tiroir détail modèle (M9) 2 → 5 px.

## [0.9.0] — 2026-05-20 — Catalogue 2026 + modalités + overhead (C34)

> Crédibilité scientifique du moteur. Le catalogue rattrape 2 ans de
> releases LLM (Claude 4.x, GPT-5.5, Gemini 3.x, Llama 4, DeepSeek V4,
> Mistral Large 3, Grok 4, Qwen 3.6, Phi-4 reasoning). Le moteur
> modélise enfin les modalités non-texte (vision / document / audio)
> et l'overhead système (system prompt + tools + memory + thinking
> tokens des reasoning models). Cette release précède C33 site internet :
> on ne peut pas marketer un moteur avec un catalogue 2024.

### Added — C34.2 Catalogue 25 presets

- **17 nouveaux presets 2025-2026** dans `model_presets.rs` :
  Claude Opus 4.7 (2026-04-16), Claude Sonnet 4.6 (2026-02-17), Claude
  Haiku 4.5, Claude Opus 4, Claude Sonnet 4, Claude 3.7 Sonnet, GPT-5.5
  (2026-04-23) + GPT-5.5 Thinking + GPT-5.5 Pro, OpenAI o3, Gemini 3.5
  Flash (2026-05), Gemini 3.1 Pro, Gemini 2.5 Pro, Llama 4 Scout (MoE
  109B/17B), Llama 4 Maverick (MoE 400B/17B), Llama 3.3 70B, Mistral
  Large 3 (MoE 675B/41B), Mistral Small 4, Mistral Medium 3.5, DeepSeek
  V4 Pro (MoE 1600B/49B), DeepSeek R1 (MoE 671B/37B), Grok 4, Qwen
  3.6-Plus, Phi-4 Reasoning, Phi-4 Reasoning Vision 15B.
- **3 nouveaux enums** : `ModelFamily` (anthropic / open_ai / google_deep_mind
  / meta_ai / mistral_ai / deep_seek / xai / alibaba / microsoft / other),
  `ArchitectureKind` (dense_transformer / moe { experts, active_experts }
  / mamba / hybrid), `VisionPricing` (open_ai_tiles / anthropic_area /
  gemini_native / llama_patches).
- **`VisionPricing::tokens_for(count, width, height, high_detail)`** —
  formules sourcées docs officielles vendor (OpenAI Vision pricing,
  Anthropic vision, Gemini API vision, Llama 4 multimodal blog).
- **`available_models_filtered(include_deprecated: bool)`** — nouveau
  helper pour filtrer les deprecated dans l'UI.
- **35 nouveaux tests** sur `model_presets.rs` (vision_pricing,
  release_date validation, source_url HTTPS, MoE active < total, etc.).

### Added — C34.3 Modalités d'input + overhead système

- **`sobria_core::InputModality`** tagged union : `text` | `vision_low
  { image_count }` | `vision_high { image_count, avg_width, avg_height }`
  | `document { page_count }` | `audio_input { duration_seconds }`. Avec
  `default_token_count()` agnostique du preset (formule générique fallback).
- **`sobria_core::ContextOverhead`** struct : `system_prompt_tokens`,
  `tools_definition_tokens`, `memory_tokens`, `thinking_tokens_p50`.
  Sépare physiquement `total_input()` (prefill) et `total_output()`
  (decode/thinking).
- **`EstimationRequest`** étendu avec `modalities: Vec<InputModality>`
  et `overhead: ContextOverhead`, `#[serde(default)]` pour compat audit
  ledger v0.8.x.
- **Module `sobria_estimator::effective_tokens`** — bridge entre
  `InputModality` (sobria-core) et `VisionPricing` (sobria-estimator).
  Calcule `effective_in = tokens_in + overhead.total_input() + Σ modalities`
  et `effective_out = tokens_out + overhead.total_output() + auto_thinking`.
- **Thinking tokens automatiques** pour reasoning models : si l'user
  n'a pas fourni `overhead.thinking_tokens_p50`, l'engine ajoute
  `output × geometric_mean(P5, P95)` du `thinking_token_multiplier`
  du preset (sources : system cards o3, DeepSeek R1 arXiv 2501.12948,
  Anthropic extended thinking doc, Gemini 2.5+ thinking doc).
- **Engines AFNOR Monte-Carlo + EcoLogits** intégrés à
  `effective_tokens()` — les modalités contribuent au pipeline énergétique.

### Added — C34.4 UI M1 modalités + détails techniques

- **`ModalitiesPanel.svelte`** (nouveau composant sous le Composer) :
  - Toggles Image / Document PDF / Audio (désactivés si le preset ne
    supporte pas la capability — badge N/A).
  - Sous-formulaires : image_count + radio basse/haute résolution +
    avg_width × avg_height pour Vision High ; page_count pour Document ;
    duration_seconds pour Audio.
  - Mode Simple/Expert (persisté localStorage). Simple cache les
    détails techniques par défaut.
  - **Badge bleu reasoning** affiché automatiquement si le modèle a
    reasoning_capable, montrant "~N tokens thinking (P50, ratio P5×-P95×
    P5-P95)".
  - Détails techniques (`<details>`) : system_prompt_tokens (pré-rempli
    depuis preset.default_context_overhead_tokens à chaque changement
    de modèle), tools_definition_tokens, memory_tokens, et thinking
    override pour reasoning models. Footer affiche total overhead.
  - **Disclaimer ± 50 %** en jaune sur la summary, conformément à la
    décision Thibault (cf. brief C34 §5).
- **`+page.svelte` M1** : intègre `ModalitiesPanel`, state
  `modalities: InputModality[]` et `overhead: ContextOverhead`, payload
  d'estimation enrichi seulement si non-vide (compat audit ledger).

### Added — C34.5 UI M9 capacités + viz vision + viz reasoning

- **Section "Capacités"** dans `ModelDetailDrawer.svelte` :
  - Header avec release_date.
  - Badges Vision (bleu) / Audio (violet) / Reasoning (lime) /
    MoE+Dense (ambre/gris) / Deprecated (corail).
  - Cap row "Paramètres actifs" pour MoE (active drives énergie).
  - Cap row "System prompt typique" avec disclaimer ± 50 %.
  - **Viz Vision** (si vision_capable) : 3 cas calculés via mirror JS
    de `VisionPricing::tokens_for` — 512×512 low, 1024×1024 high,
    2048×2048 high + lien vers doc vendor.
  - **Viz Reasoning** (si reasoning_capable) : tableau P5 / P50
    (geomean P5×P95) / P95 du multiplier + exemple "500 tokens output
    → ~N tokens thinking (P50)".
  - Bouton "Model card officielle" linkant source_url.
- **`ModelPresetDto` + `ModelDetailDto`** étendus avec capabilities,
  vision_pricing JSON tagged, et meta MoE (experts / active_experts).
- **Mirror TS `VisionPricing`** typed tagged union dans `api.ts`.

### Added — C34.6 ReproductionCase modalités

- **5 nouveaux `ReproductionCase`** dans `validation/cases.rs` :
  - `c34-vision-gpt4o-2-high-images-fr` : GPT-4o + 2 images 1024×1024
    haute rés. (1530 tokens vision via OpenAI tiles formula).
  - `c34-document-gpt4o-5-pages-fr` : PDF 5 pages → 5500 tokens
    génériques (1100 tokens/page).
  - `c34-audio-gpt4o-30s-fr` : 30 secondes audio → 300 tokens (Whisper
    rate 10 tokens/s).
  - `c34-reasoning-o3-complex-fr` : o3 reasoning, auto-thinking ~12k
    tokens (geomean √(5×30) ≈ 12.25 × 1000 output).
  - `c34-overhead-claude-3-7-fr` : Claude 3.7 Sonnet + overhead claude.ai
    2000 tokens system + auto thinking (geomean √(2×50) = 10 × 500 output).
- **`ReproductionCase` étendu** avec `modalities: &'static [InputModality]`
  et `overhead: ContextOverhead` (zéros par défaut, cas legacy).

### Changed — C34 Anciens presets 2024 deprecated

- Les 8 presets 2024 (`gpt-4o`, `gpt-4o-mini`, `claude-3-5-sonnet`,
  `mistral-large-2`, `mistral-medium-3`, `llama-3-1-70b`, `llama-3-1-8b`,
  `gemini-2-0-flash`) sont marqués `deprecated: true`. Conservés pour
  reproductibilité historique du ledger audit (estimations passées
  restent valides). Les vendor_disclosures C32.4 (Mistral × ADEME,
  Meta location/market-based, Google Gemini) sont préservées intactes.

### Changed — C34.3 Rename `Modality` → `ModelDomain` (breaking)

- **Breaking** : `sobria_core::Modality` renommé en `ModelDomain`.
  Sémantique inchangée : qualifie le **modèle entier** (LLM/Image/
  Audio/Video). Le rename résout le conflit avec le nouveau
  `InputModality` (type d'INPUT d'un prompt).
- Le champ `Model.modality: ModelDomain` reste accessible avec le
  même nom de champ. Seule la déclaration du type est renommée.

### Methodology

- **Disclaimer overhead système** : « Estimation overhead système ± 50 %
  — basée sur leaks publics et reverse-engineering interfaces vendor
  (Claude.ai, ChatGPT app, Gemini app). À surcharger en mode Expert si
  vous connaissez votre valeur exacte. » Visible dans tooltips M1 et
  doc M9.
- **Sources vision tokens** :
  - OpenAI : <https://platform.openai.com/docs/guides/vision/calculating-costs>
  - Anthropic : <https://docs.anthropic.com/en/docs/build-with-claude/vision>
  - Google Gemini : <https://ai.google.dev/gemini-api/docs/vision>
  - Meta Llama : <https://ai.meta.com/blog/llama-4-multimodal-intelligence/>
- **Sources thinking tokens** :
  - OpenAI o3 / GPT-5.5 Thinking : system cards (P5=5, P95=30)
  - DeepSeek R1 / V4 : <https://arxiv.org/abs/2501.12948> (P5=8, P95=25)
  - Claude extended thinking : Anthropic doc (P5=2, P95=50, configurable)
  - Gemini 2.5+ thinking : Google doc (P5=3, P95=25)
  - Phi-4 reasoning : Microsoft Research technical report (P5=5, P95=15)
- **Cutoff** : shortlist validée par 10 WebSearch officiels au
  2026-05-20. Zéro preset fantôme (modèles non sortis exclus).
  Documentation : `briefs/chantiers/C34-shortlist-models-validated.md`.

## [0.8.0] — 2026-05-17 — Clarté produit (C32)

> Release intermédiaire centrée UX et messaging avant la candidature
> data.gouv.fr v1.0. Aucun nouveau module métier — mais le produit
> raconte enfin son histoire pour les 5 personas (student / pro_tech /
> enterprise / public_sector / researcher) identifiées dans
> `docs/product/AUDIT-PRODUIT-2026-Q3.md`. Score moyen clarté UX
> 6/10 → cible 8/10 minimum sur les 5 personas.

### Added — C32.1 Messaging + labels + nettoyage

- **README v2 refondu** : value proposition en exergue en haut, bloc
  « Sobr.ia, c'est quoi ? » en langage simple sans jargon, section
  « Pour qui ? » avec 5 cartes persona + liens docs spécifiques. Le
  contenu technique (méthodologies, architecture) redescend après
  les cartes. La section « Méthodologies disponibles » est préservée
  intacte (garde-fou audit C32.0).
- **5 guides persona** dans `docs/personas/` (`student.md`,
  `pro-tech.md`, `enterprise.md`, `public-sector.md`, `researcher.md`)
  — ~1 page chacun avec pitch, top use cases, modules pertinents et
  quickstart copy-paste.
- **Audit produit C32.0** versionné dans
  `docs/product/AUDIT-PRODUIT-2026-Q3.md`.

### Changed — C32.1 Labels modules clarifiés

- **M9** « Référentiel modèles » → **« Bibliothèque de modèles »**.
- **M17** « Empreinte projet » → **« Datasheet scientifique »**.
- **M22** « Rapport CSRD / AGEC » → **« Rapport réglementaire (CSRD/AGEC) »**.
- Rail nav (`+layout.svelte`), titres `<title>` + breadcrumbs des
  pages M9 / M17 / Rapport CSRD alignés sur les nouveaux labels.
- Specs Playwright `m9.spec.ts`, `m17.spec.ts`, `rapport-csrd.spec.ts`
  mises à jour pour les nouveaux titres.

### Removed — C32.1 Cleanup IDs UI utilisateur

- Retrait des `<span class="module-id">{m.toUpperCase()}</span>` dans
  l'onboarding (étape Bundle × 2) et `/parametres` (3 sections
  modules). Les IDs `M1`, `M3`, etc. ne fuient plus dans l'UI
  utilisateur mais restent **inchangés** en URL (`/m3`, `/m13`) et
  en code Rust/TS (`ModuleId::M3`).
- **M14 « À propos » retiré des 5 bundles persona par défaut**
  (`sobria_core::Persona::default_modules`). M14 reste accessible via
  l'entrée rail « À propos » (`alwaysVisible: true`). Nouveau test
  invariant `no_persona_bundle_contains_m14` qui scelle la décision.
- **TS bundles alignés sur Rust** : drift pré-existant (M11, M24, M5,
  M21, M2, M6, M10, M16, M19, M23 dans le TS mais pas le Rust) résorbé
  en faveur du canon Rust.

### Added — C32.2 Onboarding pédagogique + fil narratif

- **Nouvelle étape 2 « Sobr.ia en 30 secondes »** insérée entre Splash
  et Persona Picker : 4 phrases en langage simple + schéma SVG
  d'équivalence (1 prompt typique Mistral Large 2 = 1,14 g CO₂eq
  ≈ 5 m voiture / 5 min TV LED). Bouton « Continuer » + lien discret
  « Passer cette étape » avec persistance `localStorage.sobria_welcome_skipped=true`.
- **Renumérotation onboarding** : 4 → 5 étapes (Splash 1, Intro 2,
  Persona 3, Bundle 4, Ready 5). Progression dots, eyebrows, IDs
  sections, focus management : tout cohérent.
- **Bannière « Et après ? »** sous le résultat M1 Atelier après le
  1er prompt soumis. 3 cartes contextuelles : Comparer ce modèle →
  /comparer · Voir votre usage cumulé → /m15 · Fixer un budget mensuel
  → /m25. Dismiss définitif via `localStorage.sobria_narrative_banner_dismissed=true`.
- **Tooltips « Pourquoi ces modules ? »** dans l'onboarding (étape
  Bundle) et `/parametres` (Section 2 « Vos modules actifs »).
  Lookup persona-spécifique (~30 entrées) via la nouvelle fonction
  `moduleReason(persona, id): string | undefined`. Attribut `title`
  natif — fonctionne au survol ET au focus clavier.

### Added — C32.3 Équivalences humaines + Mode Équipe guidé

- **Composant `<EquivalenceCarbon />`** réutilisable dans
  `web/src/lib/components/`. Props
  `{ gco2eq, waterMl?, energyWh?, compact? }`. 4 facteurs canoniques
  exportés en constantes : `CAR_GCO2EQ_PER_M=0.2` (ADEME Base Empreinte
  2025), `STREAMING_GCO2EQ_PER_MIN=0.25` (Shift Project Lean ICT 2019),
  `SHOWER_L=8` (ADEME douche éco), `LED_WH_PER_MIN=1`. Format adaptatif
  par magnitude : cm/m/km voiture, s/min/h streaming et LED,
  mL/%/douches. Tooltip source par équivalent + disclaimer « ordre de
  grandeur ».
- **Intégration M1 Atelier** : sous `ResultBlock`, ligne droite.
- **Intégration M15 Dashboard** : sous les 3 stat-cards CO₂/énergie/eau,
  encadré bleuté « Cette période, vous avez consommé l'équivalent
  de : … ».
- **Intégration M25 Eco-budget** : sous chaque progress bar de budget,
  mode `compact`. Affiche l'équivalence selon l'indicateur du budget.
- **Panneau « Activer Mode Équipe »** dans `/parametres` (visible
  uniquement si `!$teamStore.url`). Bouton primary lime ouvre un
  **dialog `aria-modal` à 3 étapes** : (1) Télécharger le binaire →
  Releases GitHub, (2) Initialiser le serveur → 3 commandes
  copy-paste, (3) Distribuer les codes aux employés → flow admin.
- **`docs/operations/team-aggregator.md` enrichi** : nouveau bloc
  **TL;DR** en tête (TPE/PME/freelance vs DSI), renommage
  `## Pour les TPE/PME` → `## Pour les non-IT (TPE/PME, freelances)`
  aligné audit C32.0 finding #6, avertissement « si vous n'avez pas
  d'équipe IT, le binaire fait tout tout seul ».

### Added — C32.4 Vendors disclosure

- **Nouveau struct Rust `VendorDisclosure`** dans
  `crates/sobria-estimator/src/model_presets.rs` avec enums
  `VendorScope` (`Training` | `InferencePerPrompt`) et `VendorUnit`
  (`TCo2Eq` | `GCo2Eq` | `Wh` | `MlWater` | `M3Water`). Champ
  `vendor_disclosures: &[VendorDisclosure]` ajouté à `ModelPreset`.
- **3 vendors disclosures intégrées dans `MODEL_REGISTRY`** :
  - **Mistral Large 2** × ADEME × Carbone 4 (2025-08) : training
    20 400 tCO₂eq + 281 000 m³ eau + inference 1,14 gCO₂eq pour 400
    tokens. Première ACV complète publiée par un vendor mondial.
  - **Gemini 2.0 Flash** (Google paper 2025-08) : 0,03 gCO₂eq + 0,24
    Wh + 0,26 mL eau pour un prompt texte médian. Note méthodologique
    « médian Google, sous-estime les requêtes complexes ».
  - **Llama 3.1 70B** (Meta model card 2024-07) : training
    location-based 11 390 tCO₂eq vs market-based 0 tCO₂eq, avec note
    pédagogique anti-greenwashing sur la distinction REC.
- **Migration SQLite Gold v3 idempotente** dans
  `crates/sobria-ingest/src/gold.rs` : `CREATE TABLE IF NOT EXISTS
  vendor_disclosures (id, model_id, vendor, scope CHECK IN (training |
  inference_per_prompt), value, unit, source_url, published_at,
  methodology_note)` + 2 index (model_id, vendor).
- **DTO + IPC** : `VendorDisclosureDto` et `VendorComparisonRowDto`
  dans `crates/sobria-app/src/dto.rs`. Nouveau IPC handler
  `list_vendor_comparison()` qui agrège par fabricant pour la table
  M9 (5 lignes : Mistral / Google / Meta / Anthropic / OpenAI).
- **Encadré « Données vendor disclosure »** dans
  `ModelDetailDrawer.svelte` (fiche M9), visible uniquement si le
  modèle a au moins une disclosure. Cartes différenciées training
  (violet) / inference (lime), notes méthodologiques, liens sources
  cliquables.
- **Table comparaison vendor** sur la page principale M9 sous le hero
  (4 colonnes : Fabricant / Prompt-level / Training / Source ; ✅ ou
  ❌ explicite, « Pas de disclosure officielle » pour Anthropic +
  OpenAI).

### Added — C32.5 DOI Zenodo + smoke-test 5 personas

- **DOI Zenodo** : workflow GitHub Actions
  `.github/workflows/zenodo.yml` qui déclenche l'archivage Zenodo sur
  chaque release. Badge DOI ajouté en haut du README. Section
  « Citation » BibTeX prête pour reproductibilité académique. Le DOI
  effectif est généré par Zenodo à la première release v0.8.0
  publiée sur GitHub.
- **Smoke-test 5 personas** : runbook structuré dans
  `docs/qa/smoke-test-v0.8.0-2026-05.md` qui couvre le walkthrough
  complet par persona (Splash → Intro 30 sec → Persona → Bundle →
  Ready → 1er prompt → bannière « Et après ? »). Sert de check-list
  manuelle avant publication GitHub.

### Tests

- **`cargo test --workspace`** : 100 % vert sur les crates touchées —
  `sobria-core` 49/49 (dont nouvel invariant
  `no_persona_bundle_contains_m14`), `sobria-app` 247/247 (+3 tests
  vendor disclosure), `sobria-estimator` 96/96 (+5 tests vendor
  disclosure).
- **`cargo clippy --workspace -- -D warnings`** clean sur les crates
  touchées.
- **`cargo fmt --check`** clean.
- **`cd web && npm run check && npm run lint`** clean sur les
  fichiers touchés (warnings prettier pré-existants C29 hors scope).

### Sécurité + privacy

- **Aucun nouveau flux réseau** introduit par v0.8.0. Mode local-first
  préservé (cf. ADR-0014). Vendor disclosures stockées en données
  statiques compilées dans le binaire, pas de fetch runtime.
- **Bannière « Et après ? »** : dismiss persisté en localStorage
  uniquement, jamais envoyé au serveur. Cohérent CLAUDE.md §7.
- **Tooltips persona** : pure UI client, aucune télémétrie.

### Bump versions

- `Cargo.toml` workspace : `0.7.1` → `0.8.0` (propage aux 11 crates
  via `version.workspace = true`).
- `crates/sobria-app/tauri.conf.json` : `0.7.1` → `0.8.0`.
- `web/package.json` : `0.7.1` → `0.8.0`.
- `web-team/package.json` : `0.7.1` → `0.8.0`.
- `extension/package.json` + `extension/manifest.json` : `0.7.1` →
  `0.8.0`.

## [0.7.1] — 2026-05-16 — Polish Mode Équipe (C29)

> Patch release qui ferme les 4 manques honnêtement notés en C28 :
> UI Mode Équipe côté app Tauri (les 8 IPC `team_*` étaient prêts mais
> non câblés au front), reset-password admin, rotation TLS, et alertes
> seuils CSRD. Voir `briefs/chantiers/C29-v0.7.1-polish-mode-equipe.md`
> et `docs/operations/team-aggregator.md` § « Opérations courantes ».

### Added — C29.1 UI Mode Équipe (app Tauri)

- **Backend Rust non-breaking** : `team_settings::TeamStatus` étendu avec
  `last_seen_at: Option<String>` et `estimations_sent: u32`. Les IPC
  `team_ping` et `team_push_estimation` mettent à jour ces champs sur
  succès. 5 nouveaux unit tests + tests étendus (244 tests `sobria-app`).
- **TypeScript** : 7 wrappers IPC + types dans `web/src/lib/api.ts`
  (`TeamMode`, `TeamStatusDto`, `TeamHealthResponseDto`,
  `TeamEnrollResponseDto`).
- **Store** `web/src/lib/team-store.ts` : `writable<TeamState>` typé +
  `loadTeam()` + `saveTeamField()` optimistic + rollback.
- **Section UI** « Mode Équipe self-hosted » dans `/parametres` entre
  Extension navigateur et Runtime — 4 sous-blocs : Statut (pill
  lime/ambre/coral), Configuration (URL + toggle cert auto-signé),
  Enrôlement (code 12 chiffres + password 8+ + display name), Dispatcher
  (radios Local/Équipe/Les deux). 2 tests Playwright no-mock verts.
- **Fingerprint éphémère** par session : `crypto.randomUUID()` côté UI,
  persistance côté Rust après `/enroll`.

### Added — C29.2 CLI admin reset-password + list

- Nouvelle sous-commande `admin {list, reset-password}` dans
  `sobria-team-aggregator`.
- **`reset-password`** : prompt double via `rpassword`, hash Argon2id PHC,
  `last_login_at = NULL`, révocation de **TOUS** les tokens admin actifs.
  Affiche « Mot de passe de X réinitialisé. N token(s) révoqué(s). ».
- **`list`** : table `id | username | created_at | last_login_at`.
- Storage helpers : `admins::list_all`, `admins::set_password_hash`,
  `tokens::revoke_all_for_admin`. 4 unit tests + 2 integration tests
  (`tests/cli_admin.rs`).

### Added — C29.3 Rotation TLS (`serve --regen-cert`)

- Nouveau flag `serve --regen-cert` : avant de bind, sauvegarde
  `cert.pem` + `key.pem` en `*.pem.bak.<unix_ts>` puis régénère via
  `rcgen` (mêmes SANs `localhost`/`127.0.0.1`/`::1` + hostname OS,
  validité 10 ans). Affiche l'empreinte SHA-256 du nouveau cert.
- Helpers `crypto::tls::regen_self_signed` + `cert_fingerprint_sha256`.
- 4 unit tests + 2 integration tests (`tests/regen_cert.rs`).
- **`docs/operations/team-aggregator.md`** enrichi : nouvelle section
  « Opérations courantes » avec « Rotation TLS » détaillée + reset
  password admin + configuration des alertes (webhook + SMTP).

### Added — C29.4 Alertes seuils (CSRD)

- **Migration SQLite v2** dans `storage/schema.rs` (idempotente,
  pilotée par `PRAGMA user_version`) :
  - `alert_thresholds` (id ULID, scope user|team, target_id?, period
    daily|weekly|monthly, gco2eq_max, notify_kind webhook|email|log_only,
    notify_target?, created_by_admin_id, created_at, disabled_at) avec
    CHECK XOR sur (scope, target_id).
  - `alert_triggers` (id, threshold_id, period_start, period_end,
    observed_gco2eq, triggered_at, notified_at, notify_error) avec
    UNIQUE sur `(threshold_id, period_start)` — garantit **1 trigger
    par période** même sous concurrence.
- **Module `src/alerts/`** :
  - `periods.rs` — bornes UTC pour daily / weekly ISO / monthly,
    leap year safe. 7 unit tests.
  - `store.rs` — CRUD typé sur les 2 tables, validations métier
    (scope/target cohérent, notify_kind/notify_target cohérent,
    gco2eq_max > 0). 5 unit tests.
  - `checker.rs` — `check_thresholds_for_user(conn, user_id, now)` :
    liste seuils actifs (user + team), calcule l'observé, insère
    trigger si dépassement (UNIQUE protège contre re-déclenchement).
    4 unit tests.
  - `notify.rs` — `notify(event, smtp_cfg).await` :
    - **webhook** : POST JSON `reqwest` 5s timeout, payload documenté.
    - **email** : `lettre = "0.11"` SMTP (smtp:// + smtps://), creds
      `user:pass@` optionnels. Config lue depuis KV `config`
      (`smtp_url` + `smtp_from`).
    - **fallback log_only** si SMTP non configuré, webhook sans URL,
      ou email sans destinataire (graceful degrade exigé par le brief).
    - 4 unit tests.
- **Routes admin REST** `/api/v1/admin/alerts*` :
  - `POST   /alerts`          : créer un seuil.
  - `GET    /alerts`          : lister (actifs + désactivés).
  - `DELETE /alerts/:id`      : soft delete (`disabled_at`).
  - `GET    /alerts/triggers` : historique (limit 1..500, defaut 50).
- **Wiring** dans `server::api::estimations::handle` : après
  `estimations::insert`, appelle `alerts::checker::check_thresholds_for_user`
  sous le lock SQLite, drop le lock, puis `tokio::spawn` la
  notification (non-bloquante pour l'ack du client). Persiste
  `notified_at`/`notify_error` après envoi.
- **UI `/admin/alerts`** (web-team) : form de création (scope radio,
  user picker conditionnel, period radio, gCO₂eq max, notify_kind
  + target conditionnel), table des seuils avec bouton « Désactiver »,
  historique des 50 derniers triggers (timestamp, threshold, observé,
  badge notification OK/erreur/en cours). Nouvelle entrée dans la
  nav admin.
- **Tests d'intégration** `tests/integration_alerts.rs` :
  - CRUD end-to-end (POST scope=user sans target_id → 400, POST
    team OK, DELETE soft + idempotent).
  - **Webhook réel via wiremock** : 10 estimations dépassant un seuil
    team daily de 5g → 1 trigger inséré, webhook POST reçu par le
    mock (`expect(1..)` + `verify().await`).

### Changed

- `team_settings::TeamSettingsStore::clear_session` purge maintenant
  aussi `last_seen_at` + `estimations_sent` (rotation propre au logout).
- `team_settings::TeamMode` est exporté côté TS comme `'local' | 'team' | 'both'`.
- `IpcErrorCode` (`web/src/lib/api.ts`) étend la liste des codes
  team (`no_url`, `bad_request`, `unauthorized`, `http_error`,
  `transport`, `storage`).
- Migration SQLite v1 → v2 idempotente : les bases C28 (v1) ne
  ré-installent que le DDL v2 (alert tables), pas le DDL v1.

### Tests

- `sobria-team-aggregator` : **106 tests verts** (88 unit + 18 intégration,
  vs 81 + 12 en v0.7.0).
- `sobria-app` : +5 tests `team_settings` → 244 tests verts (+0 régression).
- `web` (Playwright) : +2 tests `parametres-mode-equipe` no-mock verts.
- `cargo clippy --workspace -D warnings` clean, `cargo fmt --check` clean.
- `cd web && npm run check && npm run lint` clean (sur les fichiers
  touchés ; les autres warnings sont pré-existants).
- `cd web-team && npm run check && npm run lint` clean.

### Sécurité

- Argon2id PHC partout pour les hashs (admin reset, refresh tokens).
- Pas d'OpenSSL : rustls + ring + rcgen.
- Notifications email via `lettre` avec backend rustls (pas de native-tls).
- Webhook payload ne contient **pas** de prompts utilisateurs (uniquement
  les agrégats de seuil). Conforme CLAUDE.md §7.
- Fallback `log_only` quand SMTP non configuré → pas de crash, log clair
  avec `notify_error`.

## [0.7.0] — 2026-05-16 — Mode Équipe self-hosted (C28)

> Binaire Rust standalone `sobria-team-aggregator` déployable par une
> entreprise sur son infrastructure (poste admin, NAS, VPS interne).
> **Aucun cloud Sobr.ia n'est impliqué.** Voir
> `briefs/chantiers/C28-mode-equipe-self-hosted.md`, ADR-0013 Phase 2, et
> `docs/operations/team-aggregator.md`.

### Added — C28.1 Bootstrap + TLS

- **Nouvelle crate** `crates/sobria-team-aggregator/` (binaire HTTPS ~15 MB,
  rustls + ring, pas d'OpenSSL). CLI `init` + `serve` :
  - `init` : crée `team.sqlite` (schéma v1 complet, WAL), génère cert TLS
    auto-signé via rcgen (SANs `localhost`/`127.0.0.1`/`::1`/hostname OS,
    validité 10 ans), pose JWT signing key 32 octets, crée admin initial
    Argon2id PHC.
  - `serve` : axum 0.7 + axum-server + tokio-rustls, route `/health`,
    tracing middleware. `serve --regen-cert` planifié v0.7.1.
- **Schéma SQLite v1** : `admins`, `enrollment_codes`, `users`, `tokens`,
  `estimations`, `config`. Tables STRICT, `CHECK ((user_id IS NULL) <>
  (admin_id IS NULL))` sur `tokens` pour XOR rôle. Migration idempotente
  via `PRAGMA user_version`.
- **Argon2id PHC** partout (passwords admin/user, hash des codes, hash des
  refresh tokens). Aligné sur le pattern C27 v0.6.0.

### Added — C28.2 Auth + API REST core

- **JWT HS256 24h** + **refresh tokens 7j** au format `<ulid>.<uuid_v4>`
  (selector+verifier — lookup O(1) par PRIMARY KEY puis Argon2id verify).
  Rotation à chaque `/refresh` (ancien révoqué avant émission du nouveau).
- **Routes `/api/v1`** : `POST /enroll` (code 12 chiffres + password + fingerprint),
  `POST /login` (admin OU user), `POST /refresh` (rotation), `POST /estimations`
  (auth user, payload compat extension v0.6.0 camelCase), `GET /me/usage`
  (totaux agrégés du user).
- **ApiError → IntoResponse** : mapping HTTP propre (401/403/409/400/500)
  + JSON body `{ error, code }` + logs structurés.
- **CLI `code`** : `create N --ttl-days 7`, `list`, `revoke <id>`. Codes
  12 chiffres OS RNG, hashés Argon2id, affichés en clair UNE seule fois.

### Added — C28.3 API admin + analytics SQL

- **Routes admin** sous `/api/v1/admin/*` (RequireAdmin → 401 sans Bearer,
  403 si role=user) : `GET /users` (liste + totaux LEFT JOIN), `POST /codes`
  (clamp count ∈ [1, 500]), `DELETE /codes/:id` (soft delete idempotent),
  `GET /analytics?from&to&group_by=day|week|month` (4 sections agrégées).
- **`storage/analytics.rs`** : 4 fonctions SQL pures (`time_buckets`,
  `top_models`, `top_users`, `method_breakdown`) avec `strftime` pour le
  bucketing temporel.

### Added — C28.4 Dashboard Svelte embedded

- **Nouveau projet `web-team/`** (SvelteKit 2 + adapter-static + Svelte 5
  runes + TypeScript strict). 5 pages : `/login` (3 onglets admin / employé
  / s'enrôler), `/admin/dashboard` (4 cards + 3 charts SVG + breakdown),
  `/admin/codes`, `/admin/users`, `/user/dashboard`.
- **Charts SVG manuels** (`LineChart`, `BarChart`, `DonutChart`) — pas de
  Plot/D3, ~3 KB chacun gzip. Économie ~200 KB. System fonts (pas de woff2
  embarqué) → -300 KB supplémentaires. Bundle final 201 KB.
- **Auth client** : access token JWT en mémoire (pas localStorage — XSS),
  refresh en sessionStorage, retry auto sur 401, rotation honorée.
- **`rust-embed` + SPA fallback** : `web-team/build/` embarqué via
  `#[derive(RustEmbed)]` (features `include-exclude` + `debug-embed`).
  Cache `_app/immutable/*` 1 an, fallback `index.html` SPA. Page « Bundle
  non buildé » inline si compile sans frontend.

### Added — C28.5 Exports CSRD + PROV-O + CSV agrégés équipe

- **3 routes admin** `POST /api/v1/admin/exports/{csrd|prov-o|csv}` →
  `application/pdf`, `application/ld+json`, `text/csv` avec
  `Content-Disposition: attachment`.
- **CSRD** : réutilise `sobria-export::generate_report` (PDF visuellement
  identique au rapport desktop). Convertit chaque row team `estimations`
  en `AuditEntry` shim ; synthèse `[P50×0.85, P50×1.15]` quand P5/P95
  manquent.
- **PROV-O** : variant team-spécifique (différent du sidecar audit-ledger).
  `@graph` : `prov:Bundle` + `prov:SoftwareAgent` + N `prov:Agent` (users,
  anonymisables → `fingerprint = null`, `displayName` → `Employé #N`) +
  M `prov:Activity` (`prov:wasAssociatedWith` → user).
- **CSV** : RFC 4180 UTF-8, 13 colonnes. Aliases `Employé #N` stables par
  `user_id`.

### Added — C28.6 Mode Équipe dans extension + app Tauri

- **Extension** :
  - `src/content/shared/team-storage.ts` (chrome.storage wrapper :
    URL serveur, mode `local|team|both`, tokens, user_id, fingerprint).
  - `src/lib/team-client.ts` (REST + Bearer JWT + rotation refresh auto
    sur 401, `TeamApiError` typée).
  - **Options page** : section « Mode Équipe self-hosted » (URL + ping +
    enroll + dispatch radio + logout). Warning visible sur cert auto-signé
    (utilisateur doit accepter le cert dans un autre onglet — limitation
    `chrome.fetch`).
  - **`service-worker.ts` dual-dispatch** sur `estimation_submitted` :
    bridge natif (mode=local|both) + serveur équipe (mode=team|both),
    best-effort sur chaque destination.
- **App Tauri** :
  - `crates/sobria-app/src/team_settings.rs` (SQLite KV dans
    `referentiel.sqlite`).
  - `crates/sobria-app/src/team_client.rs` (reqwest + rustls + opt-in
    `accept_invalid_certs`). Pattern `ClientConfig` snapshot pour ne pas
    garder `MutexGuard` à travers `.await`.
  - **8 IPC commands** : `team_status`, `team_set_url`, `team_set_mode`,
    `team_set_accept_invalid_certs`, `team_ping`, `team_enroll`,
    `team_logout`, `team_push_estimation`.
  - Activation auto `mode=both` au premier enrollment.

### Added — C28.7 Doc + packaging

- **`docs/operations/team-aggregator.md`** (~300 lignes) : quickstart 5 min,
  sections TPE/PME (systemd) + DSI (reverse proxy Caddy/nginx + Let's
  Encrypt + firewall + ACLs Windows), sauvegardes SQLite `.backup`,
  upgrade, troubleshooting (cert refusé, code rejeté, refresh expiré,
  bundle non buildé, performance).
- **`crates/sobria-team-aggregator/Dockerfile`** (bonus) : multi-stage
  build (node 22 web → rust 1.79 builder → debian:bookworm-slim runtime),
  volume `/data`, expose 8443, healthcheck `/health`, user non-root.
- **`.github/workflows/team-aggregator-release.yml`** : trigger tag
  `v*.*.*`, matrix Linux x86_64 / macOS arm64 / Windows x86_64, build
  frontend Svelte avant cargo build, upload assets + sha256 sidecars.
- **README racine** : section « Mode Équipe self-hosted (v0.7.0) ».

### Tests

- **`sobria-team-aggregator`** : 68 tests verts (56 unit + 1 admin
  intégration + 3 user API intégration + 1 health + 1 embedded UI +
  6 exports intégration).
- **`sobria-app`** : +13 tests `team_*` (5 settings + 8 client).
- **Extension** : `npm run check` propre.
- `cargo clippy --workspace -D warnings` clean. fmt clean partout.

### Différé v0.7.1+

- Page `/parametres → Mode Équipe` côté `web/` (frontend Svelte de l'app
  Tauri) — les 8 IPC sont prêts à être consommés via `invoke`. **→ Livré
  en v0.7.1 (C29.1).**
- Alertes seuils (table `alert_thresholds`). **→ Livré en v0.7.1 (C29.4).**
- Commande `admin reset-password`. **→ Livré en v0.7.1 (C29.2).**
- Commande `serve --regen-cert` / `--regen-key`. **→ Livré en v0.7.1 (C29.3).**

### Sécurité

- Aucun trafic vers Sobr.ia — le serveur appartient à l'entreprise.
- Pas d'OpenSSL : rustls + ring partout (axum, axum-server, tokio-rustls,
  reqwest, rcgen).
- Argon2id PHC pour tous les hashs (passwords, codes, refresh tokens).
- TLS 1.2/1.3 acceptés ; le cert auto-signé `init` est valable 10 ans.
- Le data dir doit être protégé par les ACLs OS (cf. doc déploiement).

## [0.6.0] — 2026-05-16 — Extension navigateur + pairing perso (C27)

### Added — C27.1/2/3/4 Extension WebExtension MV3

- **Nouvelle crate workspace logique** : `extension/` (TypeScript strict,
  Vite multi-pass, vanilla DOM). Manifest V3 Chrome + Firefox, deux artefacts
  packagés : `sobria-extension-chrome-v0.6.0.zip` (~207 KB) et
  `sobria-extension-firefox-v0.6.0.xpi` (~207 KB) — bien sous les 500 KB
  visés par la DoD.
- **Port JS du moteur Sobr.ia** dans `src/lib/empreinte/` : AFNOR/Sobr.ia +
  EcoLogits 2026-01, presets modèles minimaux, parité < 2 % vs Rust
  (golden snapshots dans `tests/unit/empreinte.spec.ts`).
- **Détection de prompts** sur ChatGPT, Claude (claude.ai) et Le Chat
  (chat.mistral.ai) via content scripts isolés (`content-chatgpt.js`,
  `content-claude.js`, `content-le-chat.js`). Heuristique DOM partagée
  (`prompt-detector.ts`) + mapping URL → `modelId`.
- **Indicateur composer** : badge circulaire 36 px avec progress ring,
  injecté à droite du prompt input. Affiche le score Sobr.ia (A-F) au
  passage de la souris, l'estimation gCO₂eq + Wh + mL après l'envoi.
  Pas de bannière en bas de chat (retour utilisateur — *less is more*).
- **Modèles non supportés** : badge dédié sans estimation factice (pas de
  fallback silencieux sur un preset par défaut, conformément à CLAUDE.md
  §13 « Ne JAMAIS implémenter un calcul scientifique sans source »).
- **Popup compacte** : dernier résultat, modèle détecté, total journalier
  (gCO₂eq + eau + énergie), avec persistance via `chrome.storage.local`.
- **Page Options** : choix méthodologie (AFNOR/EcoLogits), opt-out par
  site (toggles), opt-in pont natif vers app desktop, section pairing
  (cf. C27.5).
- **Build pipeline** : `scripts/build.js` Vite multi-pass (ES modules pour
  background + service worker, IIFE par content script pour isolation
  cross-frame). `scripts/package.js` génère .zip Chrome + .xpi Firefox
  avec SHA-256 de chaque archive.
- **Tests Vitest** : 55 tests verts (happy-dom), couvrant `empreinte`
  (24), `prompt-detector` (9) et `badge-injector` (22).
- **CSP stricte** : pas de `unsafe-eval` ni `unsafe-inline`. Aucune
  dépendance runtime hors `webextension-polyfill`.

### Added — C27.5 Bridge natif + pairing 6 chiffres + ingestion app

- **Nouveau binaire `sobria-bridge`** (`crates/sobria-bridge/`) : pont
  Native Messaging WebExtensions. Lit `stdin` (uint32 LE + JSON UTF-8) du
  navigateur, écrit `stdout` au même format. Pas de port réseau, pas de
  service permanent — sécurité OS standard.
  - `Ping` → `{ pong: true }` (heartbeat extension).
  - `Estimate{ secret, payload }` → spool fichier append-only
    `~/.sobria/spool/incoming.jsonl` (rotation 10 MB).
  - `Pair{ code }` / `Revoke{ secret }` forwardés en temps réel à l'app
    via **socket forward** (Unix domain socket `/tmp/sobria-bridge.sock`
    sur macOS/Linux, named pipe `\\.\pipe\sobria-bridge` sur Windows),
    timeout 2 s avec fallback spool fichier si l'app n'est pas lancée.
  - Manifest template `manifest/com.sobria.bridge.json.tmpl` + README
    d'installation manuelle (macOS / Linux / Windows × Chrome / Firefox).
  - 8 tests d'intégration (`tests/protocol.rs`) couvrant lecture / EOF /
    oversize / write length-prefix / handle ping/pair/estimate / rotation
    spool.
- **Module `crates/sobria-app::pairing`** : logique pure du pairing par
  code 6 chiffres, TTL 5 min, single-use, comparaison constant-time.
  - `PendingCode::new()` — 6 chiffres random, padding zéro (ex. `042039`).
  - `PairingSecret::new()` — 32 octets random (OS RNG), hash **Argon2id**
    (PHC string, params standards), stocké tel quel dans `secret_hash`
    (le PHC inclut le sel — plus de colonne `salt_hex` séparée).
    Migration SQLite v3 : les pairings v2 (SHA-256+salt) sont
    automatiquement révoqués au boot, l'utilisateur re-saisit son code.
  - `verify_code(pending, submitted, now)` — constant-time + expire-aware.
  - 14 tests unitaires.
- **Module `crates/sobria-app::extension_store`** : persistance SQLite
  dans `referentiel.sqlite`.
  - Tables `device_pairings(id, fingerprint, secret_hash, salt_hex,
    created_at, last_seen_at, revoked_at)` (UNIQUE sur fingerprint avec
    REPLACE pour ré-appariement après dépair) et `extension_events(id,
    pairing_id, ts, method, model_id, tokens_in, tokens_out, gco2eq_p50,
    water_ml, energy_wh, raw_payload_json, ingested_at)` (FK pairing_id).
  - **ULID** comme identifiant (26 chars Crockford Base32, time-sortable —
    ordre lexicographique = ordre chronologique, monotone pour B-tree).
  - `drain_spool(store, spool_path)` : lit le spool fichier, valide les
    secrets, insère dans `extension_events`. Atomique (rename → read →
    remove pour éviter les pertes pendant le drain).
  - 14 tests unitaires.
- **Wiring `AppState`** : `pending_code: Mutex<Option<PendingCode>>` +
  `extension_store: Mutex<ExtensionStore>` ouverts depuis
  `referentiel.sqlite` au boot.
- **7 nouvelles commandes IPC Tauri** dans `crates/sobria-app/src/main.rs`
  + mirrors TypeScript dans `web/src/lib/api.ts` :
  - `regenerate_pairing_code()` → `PairingCodeDto`
  - `get_pairing_code_status()` → `Option<PairingCodeDto>`
  - `verify_pairing_code(code, fingerprint)` → `PairingSecretDto`
  - `list_pairings()` → `Vec<PairingDto>`
  - `revoke_pairing(id)` → `()`
  - `list_extension_events(limit, offset)` → `Vec<ExtensionEventDto>`
  - `drain_extension_spool()` → `usize`
- **Nouvelle section UI** dans `/parametres` (entre Référentiel et
  Runtime) : grille 2 colonnes — code 6 chiffres affichage grand format
  (chaque chiffre dans un cadre lime sur fond ink, font mono) + compte-
  à-rebours TTL en `M:SS`, bouton « Générer / Régénérer un code » ; et
  liste des appariements actifs avec fingerprint, dates création / vu /
  révocation, bouton X par ligne pour révoquer.

### Added — Patches finaux v0.6.0 (avant tag)

- **Auto-install des manifests natifs par l'app Tauri** (nouveau module
  `crates/sobria-app/src/bridge_install.rs`) : détection des navigateurs
  installés (Chrome, Firefox, Edge, Brave, Chromium) + écriture
  programmatique des manifests `com.sobria.bridge.json` aux bons
  emplacements OS (macOS / Linux / clé registre Windows). Dialog Svelte
  `aria-modal` avec consentement explicite, toast non bloquant au premier
  démarrage post-update. IPC `bridge_status`, `install_extension_bridge`,
  `uninstall_extension_bridge`. Les scripts `crates/sobria-bridge/scripts/`
  restent en fallback pour les setups custom.
- **Socket forward temps réel bridge ↔ app** (`crates/sobria-app/src/bridge_server.rs`
  + extension de `crates/sobria-bridge/src/lib.rs`) : Unix domain socket
  sur macOS/Linux, named pipe sur Windows. Pair/Revoke répondent en ≤ 2 s
  quand l'app tourne. Fallback spool fichier conservé pour le mode offline.

### Added — Documents

- **ADR-0013** « WebExtension MV3 + native messaging bridge + pairing
  perso/équipe » documente la séparation Phase 1 (pairing perso v0.6.0,
  code 6 chiffres + spool fichier) vs Phase 2 (mode Équipe self-hosted
  différé à C28/v0.7.0) vs Phase 3 (SSO entreprise, multi-device, RBAC —
  v0.8+). Statut : **Phase 1 Implemented (v0.6.0, 2026-05-16)**.
- **`crates/sobria-bridge/README.md`** : guide d'installation manuelle
  du manifest natif sur macOS / Linux / Windows pour Chrome / Firefox /
  Chromium / Brave / Edge.
- **`briefs/chantiers/C27-extension-navigateur.md`** + **`C27-PROMPT-CLAUDE-CODE.md`** :
  brief chantier + prompt structuré utilisé pour piloter Claude Code.

### Quality gates v0.6.0

- ✅ `cargo test -p sobria-app -p sobria-bridge` : 198 + 8 = 206 tests verts.
- ✅ `cargo clippy -p sobria-app -p sobria-bridge -- -D warnings` clean
  (pedantic activé).
- ✅ `cargo fmt --check` clean.
- ✅ `cd web && npm run check` : 0 erreur, 1 warning préexistant (tsconfig
  node types).
- ✅ `cd extension && npm run check && npm run lint && npm run test` :
  TypeScript strict + Prettier + 55 Vitest tests verts.
- ✅ Bundles : Chrome 206.7 KB, Firefox 206.8 KB (< 500 KB DoD).
- ⚠️ `npm audit` extension : 6 vulnérabilités moderate dans des deps
  **devDependencies** uniquement (vite, vitest, @vitest/mocker) — aucun
  code livré au navigateur. Acceptées pour v0.6.0, à revoir au prochain
  bump majeur de Vite (différé hors C27).

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
- Tests : 4 unitaires (assemble, sqlite, parquet, empty case) + 1 d'intégration end-to-end 