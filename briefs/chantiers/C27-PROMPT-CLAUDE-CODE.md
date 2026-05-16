# C27 — Prompt Claude Code (v0.6.0 — Extension navigateur + pairing perso)

> **Mode d'emploi** : copier-coller le contenu ci-dessous dans une nouvelle
> session Claude Code (CLI) à la racine du repo. Le prompt est auto-suffisant
> et démarre par `/using-superpower` pour mobiliser toutes les capacités.

---

```
/using-superpower

# Mission : C27 — Extension navigateur Sobr.ia + pairing perso (v0.6.0)

Tu vas implémenter de bout en bout l'extension navigateur WebExtension MV3
de Sobr.ia, avec son pairing par code à 6 chiffres entre l'extension et
l'app Tauri, et son ingestion native dans le Journal/Dashboard.

## Contexte projet à charger AVANT toute action

Lis ces fichiers dans l'ordre, sans en sauter :

1. `CLAUDE.md` — règles projet, anti-patterns, privacy by design (§7),
   ce que tu ne dois jamais faire (§13).
2. `docs/adr/ADR-0013-extension-pairing-team-mode.md` — décision
   architecturale globale (Phase 1 = perso v0.6.0, Phase 2 = équipe v0.7.0).
3. `briefs/chantiers/C27-extension-navigateur.md` — brief complet C27,
   périmètre, découpage en C27.1 → C27.6, DoD.
4. `docs/adr/ADR-0012-multi-methodology-engine.md` — multi-méthodologie
   (l'extension doit honorer le catalogue AFNOR + EcoLogits).
5. `crates/sobria-estimator/src/` (lib.rs, engine_trait.rs, ecologits.rs,
   monte_carlo.rs) — moteurs Rust à porter en JS avec parité ≤ 2 %.
6. `crates/sobria-app/src/logic.rs` — patterns IPC + tables SQLite
   existantes (Journal, Dashboard, audit ledger).
7. `web/src/lib/styles/` ou équivalent — design system Sobr.ia (palette,
   typo, tokens) à réutiliser dans la popup extension.
8. `CHANGELOG.md` entrée `[0.5.0]` — ce qui a été shippé juste avant.

Le périmètre C28 / mode Équipe self-hosted est explicitement EXCLU de
v0.6.0. Voir ADR-0013 §"Implémentation phasée".

## Stratégie données + sécurité

- L'extension est **standalone** pour la mesure : moteur AFNOR/EcoLogits
  porté en JS, presets modèles embarqués (subset Gold), zéro dépendance
  réseau.
- Le pairing avec l'app Tauri est **opt-in et facile** : 1 install bridge
  + 1 code à 6 chiffres à saisir.
- Aucun envoi vers un serveur externe. Aucune télémétrie. Aucun tracking.
- Native messaging WebExtensions (sécurité OS) + secret partagé Argon2id
  (révocation).
- Permissions minimales : `activeTab`, `storage`. `nativeMessaging` séparée
  et opt-in lors du pairing.
- CSP stricte : pas de `unsafe-eval`, pas de `unsafe-inline`.

## Plan d'exécution

### C27.1 — Bootstrap projet + manifest (0.5 jour)

Livrables :

- `extension/package.json` :
  - Dépendances : `webextension-polyfill@^0.10`, `vite@^5`,
    `typescript@^5`, `@types/chrome`, `@types/firefox-webext-browser`,
    `@types/webextension-polyfill`, `vitest@^1`, `@playwright/test@^1`,
    `eslint@^8`, `@typescript-eslint/*`, `prettier@^3`.
  - Scripts : `dev`, `build`, `build:firefox`, `lint`, `check`,
    `test`, `e2e`, `package` (zip Chrome + xpi Firefox).
- `extension/tsconfig.json` strict, target ES2022, module ESNext.
- `extension/manifest.json` (MV3) :
  - `manifest_version: 3`, `name`, `version: "0.6.0"`, `description`.
  - Permissions : `["activeTab", "storage"]`. (`nativeMessaging` ajoutée
    dynamiquement au pairing — voir C27.5.)
  - `host_permissions` : `["https://chat.openai.com/*", "https://claude.ai/*",
    "https://chat.mistral.ai/*"]`.
  - `content_scripts` : 3 entrées (une par site, match exact, `run_at:
    document_idle`).
  - `background.service_worker: "src/background/service-worker.ts"`.
  - `action.default_popup: "src/popup/index.html"`,
    `action.default_icon` (16/48/128).
  - `options_page: "src/options/index.html"`.
  - CSP : `"extension_pages": "script-src 'self'; object-src 'self'"`.
- `extension/manifest.firefox.json` variant avec
  `browser_specific_settings.gecko.id` (`sobria@sobr.ia`) et
  `background.scripts: ["src/background/service-worker.ts"]` (Firefox
  préfère encore les scripts background dans certaines versions).
- `extension/vite.config.ts` :
  - Multi-entry : `popup`, `options`, `service-worker`,
    `content-chatgpt`, `content-claude`, `content-le-chat`.
  - Plugin pour copier manifest + icons + assets dans `dist/`.
  - Mode `--firefox` produit `dist-firefox/` avec manifest variant.
- Icônes placeholder lime (`extension/src/assets/icon-{16,48,128}.png`)
  + `leaf.svg`. Tu peux générer des PNG simples programmatiquement
  (carré lime `#a0e060` avec une feuille blanche, rien de fancy v1).
- `extension/.eslintrc.cjs` + `.prettierrc` partagés avec `web/`.
- `extension/README.md` quickstart (install dev mode Chrome/Firefox,
  scripts npm, structure).

DoD C27.1 : `cd extension && npm install && npm run check` propre,
extension installable en dev mode Chrome (popup s'ouvre, "Hello Sobr.ia").

### C27.2 — Port JS du moteur (1 jour)

Livrables :

- `extension/src/lib/empreinte/afnor.ts` :
  Port direct des formules AFNOR/Sobr.ia depuis
  `crates/sobria-estimator/src/monte_carlo.rs`. Mode **point-estimate**
  (P50 uniquement, pas de Monte-Carlo — l'extension doit rester légère).
  Constantes `K_DECODE_MJ_PER_TOKEN_PER_B = 25.0` etc. citées avec leur
  source.
- `extension/src/lib/empreinte/ecologits.ts` :
  Port direct depuis `crates/sobria-estimator/src/ecologits.rs`.
  Citer doi:10.21105/joss.07471. Point-estimate P50.
- `extension/src/lib/empreinte/index.ts` :
  Facade `EmpreinteEngine` :
  ```ts
  type Method = "afnor_sobria" | "ecologits";
  type Estimate = {
    method: Method;
    gco2eq: number;
    waterMl: number;
    energyWh: number;
    notes: string[];
  };
  export function estimate(input: {
    method: Method;
    modelId: string;
    tokensIn: number;
    tokensOut: number;
    region?: string;  // FR par défaut
  }): Estimate;
  ```
- `extension/src/lib/presets.ts` :
  8 presets modèles en JSON statique : GPT-4o, GPT-4o-mini, Claude 3.5
  Sonnet, Claude 3 Opus, Llama 3.1 70B, Llama 3.1 405B, Mistral Large 2,
  Mistral Small 3. Champs : `id`, `name`, `vendor`, `paramsBillion`,
  `defaultRegion`, `defaultPue`, `architectureFamily`.
- `extension/src/lib/types.ts` : types DTO partagés.
- `extension/src/lib/i18n.ts` : FR + EN minimaliste (un Record<key, {fr,
  en}>), démarrer en FR.
- Tests `tests/unit/empreinte.spec.ts` :
  3 ReproductionCase identiques à
  `crates/sobria-estimator/tests/reproduction.rs` (Llama 3.1 70B FR/USVA,
  Mistral Large 2). Assertion : `Math.abs(jsP50 - rustP50) / rustP50 <
  0.02` (≤ 2 % parité). Les valeurs Rust attendues sont à recopier en
  dur dans le test JS (constantes commentées avec le commit hash Rust).

DoD C27.2 : `npm run test` vert, parité ≤ 2 % validée sur les 3 cas
contre les valeurs Rust connues.

### C27.3 — Détection prompts par site (1 jour)

Livrables :

- `extension/src/content/shared/prompt-detector.ts` :
  Helper `observePromptSubmission(config: {
    selectorTextarea: string;
    selectorSendButton: string;
    extractModelId: () => string | null;
    onSubmit: (data: { prompt: string; modelId: string | null }) => void;
  })`. Utilise MutationObserver + delegation `click` + `keydown` (Enter
  sans Shift). Throttle 200 ms pour éviter les doubles déclenchements.
- `extension/src/content/chatgpt.ts` :
  - `selectorTextarea: "#prompt-textarea"`.
  - `selectorSendButton: "[data-testid='send-button']"`.
  - `extractModelId` lit la modale model picker ou l'URL hash.
  - Mapping nom DOM → presetId (ex: "GPT-4o" → "gpt-4o").
- `extension/src/content/claude.ts` :
  - `selectorTextarea: "div[contenteditable='true']"`.
  - `selectorSendButton: "button[aria-label='Send Message']"`.
  - `extractModelId` lit le sélecteur en haut de page.
- `extension/src/content/le-chat.ts` :
  - Selectors capturés depuis fixtures statiques (à committer en
    `tests/e2e/fixtures/le-chat-2026-05.html`).
- `extension/src/content/shared/badge-injector.ts` :
  - Injecte après chaque message envoyé une bulle compacte :
    ```html
    <div class="sobria-badge sobria-badge--lime">
      🌱 0,42 gCO₂eq · 1,8 mL · 0,12 Wh
    </div>
    ```
  - Style inline (CSS isolé via shadow DOM) pour pas être cassé par les
    CSS des sites.
  - Couleur :
    - `< 1 gCO₂eq` → lime
    - `1-5 gCO₂eq` → ambre
    - `> 5 gCO₂eq` → coral
  - Click sur le badge → ouvre la popup (`browser.action.openPopup()`).
- Chaque content script :
  - Détecte le prompt à la soumission.
  - Calcule via `lib/empreinte` avec la méthode courante (lue depuis
    `chrome.storage.local`, défaut `afnor_sobria`).
  - Injecte le badge.
  - Envoie un message au service worker pour persistance + forward bridge.

Tests :
- `tests/unit/prompt-detector.spec.ts` avec fixtures HTML statiques
  capturées (anonymisées) en `tests/fixtures/{chatgpt,claude,le-chat}-2026-05.html`.
- `tests/e2e/badge-display.spec.ts` Playwright sur fixtures.

DoD C27.3 : sur les 3 sites cibles, un prompt envoyé affiche un badge
`gCO₂eq` dans la page en < 1 s. Confirmé manuellement par capture.

### C27.4 — Popup + Options (1 jour)

Livrables :

- `extension/src/popup/index.html` + `src/popup/main.ts` :
  - Header : logo Sobr.ia (24px) + version.
  - Carte "Dernier prompt" : modèle, tokens in/out, gCO₂eq (P50 +
    intervalle si Monte-Carlo dispo via app, sinon P50 seul), eau, énergie.
  - Carte "Aujourd'hui" : compteur prompts, total gCO₂eq, total eau,
    total énergie. Tronqué à minuit local.
  - Toggle "Méthodologie" : AFNOR ⇄ EcoLogits (persisté `chrome.storage`).
  - Bouton "Voir dans Sobr.ia" → deeplink `sobria://journal` si bridge
    natif détecté (sinon disabled + tooltip).
  - Bouton "Réglages" → ouvre `options_page`.
- `extension/src/popup/popup.css` : design system Sobr.ia (palette lime
  `#a0e060`, ambre `#e0c060`, coral `#e07060`, fond clair, typo system).
  Mobile-friendly (popup MV3 fait 380×600 max).
- `extension/src/popup/components/` :
  - `result-card.ts` (compact, pas de framework).
  - `daily-total.ts`.
  - `method-toggle.ts`.
- `extension/src/options/index.html` + `main.ts` :
  - Section "Pairing app Sobr.ia" :
    - Statut bridge : "Détecté ✓" ou "Non détecté".
    - Si non pairée : champ "Code à 6 chiffres" + bouton "Connecter".
    - Si pairée : affichage du fingerprint + "Dépaire cette extension".
  - Section "Sites surveillés" : toggles ChatGPT/Claude/Le Chat
    (persisté). Désactiver supprime le content script via
    `chrome.scripting.unregisterContentScripts` à la volée.
  - Section "Confidentialité" :
    - Bouton "Purger toutes les données locales" (vide `chrome.storage`
      + envoie un revoke au bridge si pairée).
    - Bouton "Exporter mes données (JSON)" → download `sobria-export-
      YYYY-MM-DD.json`.
    - Toggle "Afficher le badge sur les pages" (défaut on).
  - Section "Méthodologie" : explication courte AFNOR vs EcoLogits + lien
    vers `/methodologies` de l'app Tauri (si bridge OK) ou page docs.
  - Section "À propos" : version, licence, lien repo GitHub.

DoD C27.4 : popup affiche le total journalier persistant après
rafraîchissement page navigateur, options enregistre les préférences,
toggles sites coupent bien les content scripts.

### C27.5 — Bridge natif + pairing 6 chiffres + ingestion app (1.5 jour)

**C27.5.a — Crate `sobria-bridge`** :

- `crates/sobria-bridge/Cargo.toml` : binaire standalone, deps minimes
  (`serde`, `serde_json`, `tokio` rt-multi-thread, `anyhow`,
  `bytes`).
- `crates/sobria-bridge/src/main.rs` :
  - Lit stdin length-prefixed (uint32 LE + JSON bytes).
  - Écrit stdout pareil.
  - Protocole de messages : `Pair{ code }`, `Estimate{ secret, payload }`,
    `Revoke{ secret }`, `Ping`.
  - Forwarde vers IPC Tauri via socket Unix (`/tmp/sobria-bridge.sock`
    Unix, named pipe `\\.\pipe\sobria-bridge` Windows). Si socket
    inaccessible (app pas lancée) → écrit dans
    `~/.sobria/spool/incoming.jsonl` (append-only, JSON Lines), max
    10 MB rotaté.
  - Tests : `tests/protocol.rs` roundtrip stdin/stdout JSON length-prefixed.

**C27.5.b — Auto-install par l'app Tauri** :

- `crates/sobria-app/src/bridge_install.rs` (nouveau module) :
  ```rust
  pub enum BrowserKind { Chrome, Firefox, Edge, Brave }
  pub fn detect_browsers() -> Vec<BrowserKind>;
  pub fn install_native_manifest(browser: BrowserKind, bridge_path: &Path) -> Result<()>;
  pub fn uninstall_native_manifest(browser: BrowserKind) -> Result<()>;
  pub fn manifest_path(browser: BrowserKind) -> PathBuf;
  ```
  - Sur macOS : écrit `~/Library/Application Support/<Browser>/NativeMessagingHosts/com.sobria.bridge.json`.
  - Sur Linux : `~/.config/<browser>/NativeMessagingHosts/com.sobria.bridge.json`.
  - Sur Windows : clé registre `HKEY_CURRENT_USER\Software\<Browser>\NativeMessagingHosts\com.sobria.bridge`.
  - Le manifest contient : `name`, `description`, `path` (vers
    `sobria-bridge` binaire), `type: "stdio"`, `allowed_origins: [<id
    extension>]`.
- IPC `install_extension_bridge`, `uninstall_extension_bridge`,
  `bridge_status` (retourne `{installed: bool, browsers: [...], bridge_path}`).
- Au premier démarrage v0.6.0, dialog Svelte : "Activer la synchronisation
  avec l'extension navigateur ? Aucune donnée n'est envoyée ailleurs.
  Vous pouvez désactiver à tout moment dans les paramètres." → 3
  boutons : "Oui, activer", "Plus tard", "Ne plus demander".

**C27.5.c — Pairing 6 chiffres** :

- Nouvelle table SQLite `device_pairings` :
  ```sql
  CREATE TABLE device_pairings (
      id TEXT PRIMARY KEY,            -- uuid v4
      fingerprint TEXT NOT NULL,      -- ex: "chrome-mac-abc123"
      secret_hash TEXT NOT NULL,      -- Argon2id du secret 32 bytes
      created_at TEXT NOT NULL,
      last_seen_at TEXT,
      revoked_at TEXT,
      UNIQUE(fingerprint)
  );
  ```
- Module `crates/sobria-app/src/pairing.rs` :
  ```rust
  pub struct PairingCode { pub code: String, pub expires_at: DateTime<Utc> }
  pub fn generate_pairing_code(state: &mut AppState) -> Result<PairingCode>;
  pub fn verify_pairing_code(state: &mut AppState, code: &str, fingerprint: &str) -> Result<PairingSecret>;
  pub fn list_pairings(state: &AppState) -> Result<Vec<PairingDto>>;
  pub fn revoke_pairing(state: &mut AppState, id: &str) -> Result<()>;
  pub fn verify_secret(state: &AppState, secret: &str, fingerprint: &str) -> Result<PairingId>;
  ```
  - Codes : 6 chiffres OS RNG, TTL 5 min, stockés en mémoire AppState
    (pas en SQLite — éphémères).
  - Secrets : 32 bytes random, hash Argon2id (params standards :
    `params = Params::new(15000, 2, 1, None)`), stocké en `device_pairings.secret_hash`.
  - Constant-time compare pour code (subtle ou implem maison).
- IPC `regenerate_pairing_code`, `list_pairings`, `revoke_pairing`.
- UI Svelte dans `/parametres → Extension navigateur` :
  - Si bridge non installé : bouton "Activer la synchronisation".
  - Si bridge installé, aucune extension pairée :
    - Code affiché grand format (police monospace 32px).
    - Compte-à-rebours TTL.
    - Bouton "Régénérer le code".
    - Instructions : "Saisir ce code dans la popup de l'extension Sobr.ia".
  - Si extensions pairées : liste avec fingerprint navigateur + dernière
    activité + bouton "Dépaire".

**C27.5.d — Ingestion côté app** :

- Nouvelle table SQLite :
  ```sql
  CREATE TABLE extension_events (
      id TEXT PRIMARY KEY,
      pairing_id TEXT NOT NULL REFERENCES device_pairings(id),
      ts TEXT NOT NULL,
      method TEXT NOT NULL,
      model_id TEXT NOT NULL,
      tokens_in INTEGER NOT NULL,
      tokens_out INTEGER NOT NULL,
      gco2eq_p50 REAL NOT NULL,
      gco2eq_p5 REAL,
      gco2eq_p95 REAL,
      water_ml REAL NOT NULL,
      energy_wh REAL NOT NULL,
      raw_payload_json TEXT NOT NULL,
      ingested_at TEXT NOT NULL
  );
  CREATE INDEX idx_extension_events_ts ON extension_events(ts);
  CREATE INDEX idx_extension_events_pairing ON extension_events(pairing_id);
  ```
- IPC `drain_extension_spool` : lit
  `~/.sobria/spool/incoming.jsonl`, vérifie secret Argon2id pour chaque
  ligne, insère dans `extension_events`, tronque le spool atomiquement.
  Émet event Tauri `extension_event_ingested` (Svelte écoute pour
  refresh UI).
- Timer Tokio dans `setup` : `drain_extension_spool` toutes les 5 s.
- Module `crates/sobria-app/src/journal.rs` : ajoute paramètre `source`
  optional pour `list_journal_entries` + UNION query qui agrège
  `audit_entries` + `extension_events`.
- Le Journal frontend ajoute filtre "Toutes / App / Extension" + badge
  "Extension" sur les entrées concernées.
- Dashboard M15 ajoute breakdown "App vs Extension" dans la card "Origine".

**Côté extension (TypeScript)** :

- `extension/src/background/native-messaging.ts` :
  ```ts
  export class BridgeClient {
    private port?: chrome.runtime.Port;
    async connect(): Promise<boolean>;
    async pair(code: string): Promise<{ secret: string }>;
    async sendEstimate(payload: EstimatePayload): Promise<void>;
    async revoke(): Promise<void>;
  }
  ```
  - `connect` tente `chrome.runtime.connectNative('com.sobria.bridge')`.
    Retourne false si pas dispo (catch erreur), bandeau "Installer
    Sobr.ia desktop".
- `extension/src/background/service-worker.ts` :
  - Écoute messages des content scripts.
  - Pour chaque estimation : si pairée → forward au bridge ; toujours →
    persiste local pour popup.
- `extension/src/options/main.ts` :
  - Champ code 6 chiffres → appelle `bridgeClient.pair(code)` → si OK,
    stocke secret + fingerprint dans `chrome.storage.local`. Affiche
    "Pairing OK ✓".

**Tests** :

- `crates/sobria-bridge/tests/protocol.rs` : roundtrip JSON length-prefixed.
- `crates/sobria-app/tests/pairing.rs` :
  - Code valide → pairing OK + secret retourné.
  - Code expiré (avance Tokio time) → reject.
  - Code invalide (mauvais chiffres) → reject.
  - Code déjà utilisé → reject (single-use).
  - Secret révoqué → estimations rejetées.
- `extension/tests/unit/native-messaging.spec.ts` : mock `chrome.runtime.connectNative`.
- `extension/tests/e2e/pairing.spec.ts` Playwright : install simulé +
  saisie code + estimation ingérée dans spool fichier.

DoD C27.5 : flux complet (bridge install → code → pairing → estimation →
Journal app) fonctionne sur au moins Chrome macOS. À documenter pour
Linux + Windows.

### C27.6 — Build, packaging, doc (0.5 jour)

Livrables :

- `scripts/build-extension.sh` (Unix) + `scripts/build-extension.ps1`
  (Windows) :
  - `cd extension && npm ci && npm run build && npm run build:firefox`.
  - Zippe `dist/` → `dist/sobria-extension-chrome-v0.6.0.zip`.
  - Zippe `dist-firefox/` → `dist/sobria-extension-firefox-v0.6.0.xpi`.
  - Imprime SHA-256 des deux fichiers.
- `.github/workflows/extension-release.yml` :
  - Trigger : push tag `v0.6.0` ou dispatch manuel.
  - Job : checkout, setup Node 20, install Rust pour build bridge,
    `npm ci && npm run lint && npm run check && npm run test &&
    npm run build && npm run build:firefox`.
  - Build `sobria-bridge` release pour 3 OS (matrix linux/macos/windows).
  - Upload assets en GitHub Release : zip Chrome, xpi Firefox, 3
    binaires `sobria-bridge-<os>`.
- `extension/README.md` :
  - Section "Install dev mode" (Chrome `chrome://extensions`, Firefox
    `about:debugging`).
  - Section "Pairing avec l'app Sobr.ia" (étapes 1-2-3 avec captures).
  - Section "Troubleshooting" (bridge non détecté, code expiré, badge
    n'apparaît pas).
  - Section "Privacy" (qu'est-ce qui est stocké où).
- `docs/extension/architecture.md` :
  - Diagramme flux (extension → bridge → spool → app → SQLite → UI).
  - Audit permissions (chaque permission justifiée).
  - Security model (pairing 6 chiffres, secret Argon2id, native messaging
    OS-level).
- `docs/extension/privacy-policy.md` :
  - Quelles données sont traitées, où, par quoi.
  - Pré-requis pour soumission stores v0.6.1.
- README racine :
  - Section "Extension navigateur" ajoutée avec capture popup + lien
    install.
- Dossier candidature data.gouv.fr (`docs/candidature/`) :
  - Ajout démo extension (capture badge sur ChatGPT, capture popup,
    capture pairing en cours).

DoD C27.6 : `scripts/build-extension.sh` produit les 2 archives sans
warning, taille ≤ 500 KB chacune, SHA-256 imprimés.

## Definition of Done globale v0.6.0

- [ ] `cd extension && npm run check && npm run lint && npm run test` propres.
- [ ] `cd extension && npm run e2e` propre.
- [ ] `cargo test --workspace` 100 % vert (incluant `sobria-bridge`
      + `sobria-app::pairing`).
- [ ] `cargo clippy --workspace -- -D warnings` propre.
- [ ] `cargo fmt --check` propre.
- [ ] `cd web && npm run check && npm run lint` propre.
- [ ] Extension installable sur Chrome + Firefox sans warning console.
- [ ] Flux pairing fonctionne sur au moins macOS (Chrome) ou Linux Chrome.
- [ ] Bundle Chrome ≤ 500 KB, Firefox ≤ 500 KB.
- [ ] `npm audit --audit-level=moderate` propre.
- [ ] CHANGELOG entrée `[0.6.0] — YYYY-MM-DD` complète.
- [ ] ADR-0013 mis à jour : statut "Accepted → Implemented (Phase 1)".
- [ ] Bump versions :
  - `Cargo.toml` workspace.package : `0.5.0 → 0.6.0`
  - `crates/sobria-app/tauri.conf.json` : `0.5.0 → 0.6.0`
  - `web/package.json` : `0.5.0 → 0.6.0`
  - `extension/package.json` : nouveau, `0.6.0`
  - `extension/manifest.json` : version `0.6.0`
- [ ] Commits Conventional Commits + tag `v0.6.0`.

## Convention de commit

```
feat(ext): C27.1 bootstrap extension + manifest MV3
feat(ext): C27.2 port JS moteur AFNOR + EcoLogits (parité ≤ 2 %)
feat(ext): C27.3 détection prompts ChatGPT + Claude + Le Chat + badge en page
feat(ext): C27.4 popup + options
feat(bridge): C27.5.a binaire sobria-bridge stdin/stdout JSON length-prefixed
feat(app): C27.5.b auto-install native manifests + bridge_install IPC
feat(app): C27.5.c pairing 6 chiffres + table device_pairings + UI /parametres
feat(app): C27.5.d ingestion extension_events + filtre Journal + breakdown Dashboard
build(ext): C27.6 scripts build + workflow release + doc
chore(release): bump v0.6.0
```

Tag final :

```bash
git tag -a v0.6.0 -m "v0.6.0 — Extension navigateur Sobr.ia + pairing perso (C27)

Extension WebExtension MV3 mesurant l'empreinte des prompts en direct
sur ChatGPT, Claude et Le Chat (Mistral). Moteur AFNOR + EcoLogits porté
en JS (parité ≤ 2 % vs Rust). Pairing local par code à 6 chiffres
avec l'app Sobr.ia desktop via native messaging — les estimations
remontent dans Journal/Dashboard avec tag source = 'extension'.

100 % local, aucun cloud, aucun compte. ADR-0013 Phase 1.

Build assets : sobria-extension-chrome-v0.6.0.zip + sobria-extension-firefox-v0.6.0.xpi."
```

## Garde-fous

- **JAMAIS** d'envoi vers un serveur externe (CLAUDE.md §7).
- **JAMAIS** de tracking sans opt-in.
- **JAMAIS** de `unsafe-eval` ou `unsafe-inline` dans la CSP.
- **JAMAIS** de permission supplémentaire au-delà de `activeTab`,
  `storage`, `nativeMessaging` (opt-in).
- **TOUJOURS** citer la source des constantes scientifiques en commentaire JS.
- **TOUJOURS** demander si une ambiguïté apparaît dans CDC, brief, ou ADR-0013.
- **TOUJOURS** comparer la parité JS vs Rust avant de toucher une formule.
- Le code 6 chiffres est éphémère TTL 5 min, single-use, constant-time compare.
- Le mode Équipe (sobria-team-aggregator) est HORS PÉRIMÈTRE — c'est C28/v0.7.0.

Bonne mission. Commence par C27.1.
```

---

## Notes pour Thibault

- Avant de tag v0.6.0, smoke test manuel sur les 3 sites (envoie un
  prompt, vérifie le badge, vérifie la popup).
- Si le pairing ne marche pas sur Firefox, ne bloque pas v0.6.0 — on
  documente "Chrome only en v0.6.0" et on patch en v0.6.1.
- Le bridge binaire doit être signé en release (codesign macOS, signtool
  Windows) — sinon les OS le bloquent. Si pas de cert dispo encore, on
  documente "désactiver Gatekeeper / SmartScreen pour install" et on
  signe en v0.6.1.
- Garder en tête : le mode Équipe self-hosted (C28) reprendra une partie
  du protocole. Donc le format JSON `EstimatePayload` doit être versionné
  dès v0.6.0 (`{ "v": 1, ... }`) pour éviter des breaks en v0.7.0.
