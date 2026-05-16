# Chantier C27 — Extension navigateur Sobr.ia (WebExtension MV3) + pairing perso

> **Version cible** : v0.6.0
> **Sprint** : S13 (post-v0.5.0 ship)
> **Approche** : extension WebExtension Manifest V3, TypeScript strict, vanilla DOM + utilitaires légers
> **Cible** : Chrome 120+ et Firefox 120+
> **Pré-requis** : v0.5.0 (pipeline médaillon actif, référentiel Gold disponible) + CDC §M6 + **ADR-0013 (pairing + mode équipe self-hosted)**
> **Distribution v0.6.0** : GitHub Releases (.zip Chrome + .xpi Firefox). Soumission Chrome Web Store + AMO différée à v0.6.1 après UAT.
>
> **Périmètre C27 (v0.6.0)** : extension + **pairing perso par code 6 chiffres** (cas particulier). Le **mode Équipe self-hosted** (cas entreprise) est différé à C28 / v0.7.0 — voir ADR-0013 §"Implémentation phasée".

---

## 0. Pourquoi maintenant ?

L'extension navigateur est la brique encore vide du CDC. Elle complète l'écosystème Sobr.ia avec un cas d'usage **en mobilité** : mesurer l'empreinte des prompts directement où l'utilisateur les écrit (ChatGPT, Claude, Le Chat), sans ouvrir l'app Tauri. Côté défi data.gouv.fr c'est un fort argument de démonstration (« regardez, je tape un prompt et je vois `1,2 gCO₂eq` en direct »).

Trois sites cibles v1 :
- **ChatGPT** (`chat.openai.com`) — le plus utilisé, prio 1
- **Claude** (`claude.ai`) — DOM stable, prio 2
- **Le Chat / Mistral** (`chat.mistral.ai`) — souveraineté FR, bon argument data.gouv.fr

Gemini reporté en v0.7 (DOM mouvant + Google iframes complexes).

---

## 1. Périmètre

### En périmètre

- WebExtension MV3 : `manifest.json`, service worker, content scripts, popup.
- Détection des prompts soumis sur les 3 sites cibles (DOM observer + heuristique URL).
- Estimation locale via port JS du moteur Sobr.ia (AFNOR/Sobr.ia + EcoLogits) — pas d'IPC avec l'app Tauri à l'init, l'extension est **standalone par défaut**.
- Pont optionnel vers l'app native via **native messaging** (Chrome/Firefox standard, sécurité OS) :
  - Si l'app Tauri tourne avec le manifest natif installé → les estimations remontent dans le Journal/Dashboard.
  - Sinon, l'extension stocke en local (`chrome.storage.local`).
- Popup compacte : dernier résultat, dernier modèle détecté, total journalier (gCO₂eq + eau + énergie).
- Badge sur icône extension : nombre de prompts mesurés aujourd'hui.
- Settings : choix méthodologie (AFNOR/EcoLogits), opt-in pont natif, opt-in remontée anonymisée vers app, opt-out par site.
- i18n FR/EN basique (clés JSON), démarrer en FR par défaut.
- Build pipeline : Vite + TypeScript + cross-browser webextension-polyfill.
- Audit sécurité : pas de remote code, pas de tracking, CSP stricte, permissions minimales (`activeTab` + `storage` + `nativeMessaging` opt-in).

### Hors périmètre v0.6.0

- Soumission Chrome Web Store + AMO (différée v0.6.1 après UAT).
- Gemini, Perplexity, Copilot, Poe, etc.
- Popup riche multi-onglets (équivalent app Tauri) — différé v0.7+.
- Comparaison de modèles en direct dans la page.
- Plugin Quarto (cf. ADR-0009 §"Évolutions futures").
- **Mode Équipe self-hosted** (sobria-team-aggregator) — différé à C28 / v0.7.0 (cf. ADR-0013 Phase 2).
- SSO entreprise (SAML/OIDC), multi-device, RBAC — différés v0.8+ (ADR-0013 Phase 3).

---

## 2. Architecture

```
extension/
├── manifest.json                       # MV3, FR + EN
├── package.json                        # vite + ts + webextension-polyfill
├── tsconfig.json
├── vite.config.ts                      # build vers extension/dist/
├── src/
│   ├── background/
│   │   ├── service-worker.ts           # event-driven, pas de persistance state
│   │   ├── native-messaging.ts         # bridge Tauri app (opt-in)
│   │   └── storage.ts                  # chrome.storage.local typé
│   ├── content/
│   │   ├── chatgpt.ts                  # observer DOM ChatGPT
│   │   ├── claude.ts                   # observer DOM Claude
│   │   ├── le-chat.ts                  # observer DOM Le Chat
│   │   └── shared/
│   │       ├── prompt-detector.ts      # heuristique générique
│   │       ├── badge-injector.ts       # bulle « gCO₂eq » dans la page
│   │       └── model-id.ts             # mapping URL/header → modelId
│   ├── popup/
│   │   ├── index.html
│   │   ├── main.ts                     # vanilla, sans framework
│   │   ├── popup.css                   # design system Sobr.ia compact
│   │   └── components/
│   │       ├── result-card.ts          # dernier résultat
│   │       ├── daily-total.ts          # somme journalière
│   │       └── method-toggle.ts        # AFNOR ⇄ EcoLogits
│   ├── options/
│   │   ├── index.html                  # page de réglages (Settings)
│   │   └── main.ts
│   ├── lib/
│   │   ├── empreinte/
│   │   │   ├── afnor.ts                # port JS du moteur AFNOR
│   │   │   ├── ecologits.ts            # port JS du moteur EcoLogits
│   │   │   └── index.ts                # facade EmpreinteEngine
│   │   ├── presets.ts                  # presets modèles embarqués (subset Gold)
│   │   ├── i18n.ts                     # FR + EN minimaliste
│   │   └── types.ts                    # types partagés DTO
│   └── assets/
│       ├── icon-16.png
│       ├── icon-48.png
│       ├── icon-128.png
│       └── leaf.svg
├── tests/
│   ├── unit/
│   │   ├── empreinte.spec.ts           # parité ≤ 2 % vs Rust EcoLogits
│   │   └── prompt-detector.spec.ts
│   └── e2e/
│       └── chatgpt.spec.ts             # Playwright + fixtures HTML statiques
└── README.md
```

### Pont app native + pairing code 6 chiffres (ADR-0013 Phase 1)

**Décision Thibault** : la synchro extension ↔ app doit être **facile** (1 install + 1 code à saisir) ET sécurisée (anti-spoofing, révocable). Les estimations doivent apparaître dans les suivis (Dashboard, Journal, Forecaster) de l'app comme des entrées normales, tagguées `source = 'extension'`.

Mécanisme : **native messaging WebExtensions** (sécurité OS, pas de port ouvert) + **pairing par code à 6 chiffres** (anti-spoofing) + **secret partagé chiffré** (révocation).

**Flux complet (séquence)** :

1. **Install bridge natif par l'app** : au premier démarrage post-installation v0.6.0, l'app détecte les navigateurs présents (Chrome, Firefox, Edge si installés) et propose dans un dialog : « Activer la synchronisation avec l'extension navigateur ? Sobr.ia va écrire un petit fichier dans la config de chaque navigateur. Aucune donnée n'est envoyée ailleurs. » Si consentement : l'app écrit les fichiers manifest natifs aux bons emplacements OS :
   - Chrome (macOS) : `~/Library/Application Support/Google/Chrome/NativeMessagingHosts/com.sobria.bridge.json`
   - Chrome (Linux) : `~/.config/google-chrome/NativeMessagingHosts/com.sobria.bridge.json`
   - Chrome (Windows) : clé registre `HKEY_CURRENT_USER\Software\Google\Chrome\NativeMessagingHosts\com.sobria.bridge`
   - Firefox équivalents.

2. **Génération code éphémère** : l'app génère un code à 6 chiffres (entropie 20 bits, OS RNG), TTL **5 minutes** (régénération automatique au timeout). Affiché dans `/parametres → Extension navigateur`. Bouton "Régénérer".

3. **Install extension côté navigateur** : l'utilisateur télécharge et installe l'extension (`chrome://extensions` dev mode ou store).

4. **Saisie du code dans l'extension** : à l'ouverture de la popup, si pas encore pairée :
   - L'extension tente `chrome.runtime.connectNative('com.sobria.bridge')`.
   - Si KO → bandeau "Sobr.ia desktop pas détecté. [Installer →]" (lien vers la page de release).
   - Si OK → champ "Code de pairing à 6 chiffres" + bouton "Connecter".
   - L'utilisateur saisit le code affiché dans son app.

5. **Validation pairing** : le bridge transmet `{ action: "pair", code: "123456" }` à l'app. L'app vérifie le code (constant-time compare), génère un **secret partagé** 32 bytes random (`pairing_secret`), le stocke dans la table SQLite `device_pairings` (hash Argon2id), et retourne le secret en clair à l'extension. L'extension le stocke dans `chrome.storage.local` (sandbox navigateur). Le code 6 chiffres est invalidé.

6. **Estimations** : à chaque prompt détecté, l'extension calcule localement, puis envoie `{ secret, payload: { method, modelId, tokens_in, tokens_out, ts, gco2eq, ... } }` au bridge. L'app vérifie le secret (Argon2id verify), insère dans `extension_events`, et le Journal/Dashboard s'actualise.

7. **Révocation** : bouton "Dépaire cette extension" dans `/parametres → Extension navigateur`. Supprime la ligne `device_pairings`, l'extension perd l'accès à la prochaine estimation, bandeau "Pairing révoqué".

**Ingestion côté app** :
- Nouvelle table `device_pairings(id, fingerprint, secret_hash, created_at, last_seen_at, revoked_at)`.
- Nouvelle table `extension_events(id, pairing_id, ts, method, model_id, tokens_in, tokens_out, gco2eq_p50, ... )`.
- IPC `drain_extension_spool` + `revoke_pairing(id)` + `regenerate_pairing_code()`.
- Le Dashboard M15 affiche un filtre "Toutes / App / Extension".
- Le Journal affiche un badge "Extension" sur les entrées concernées.

**Important** : le bridge n'est PAS un démon. Instancié par le navigateur uniquement à chaque message. Pas de service permanent, pas de surveillance silencieuse. Spool fichier rotaté à 10 MB.

---

## 3. Découpage en sous-chantiers

### C27.1 — Bootstrap projet + manifest (0.5 jour)

- `extension/package.json` (Vite 5, TypeScript 5, `@types/chrome`, `@types/firefox-webext-browser`, `webextension-polyfill`, `vitest`, `@playwright/test`).
- `extension/manifest.json` MV3 :
  - `manifest_version: 3`
  - Permissions : `activeTab`, `storage` (NPS `nativeMessaging` séparée, opt-in).
  - `host_permissions` minimales : `https://chat.openai.com/*`, `https://claude.ai/*`, `https://chat.mistral.ai/*`.
  - `content_scripts` 3 (un par site).
  - `background.service_worker`.
  - `action.default_popup` + icônes.
  - CSP stricte sans `unsafe-eval` ni `unsafe-inline`.
- `extension/vite.config.ts` produit `extension/dist/` (Chrome) et `extension/dist-firefox/` (avec `manifest.json` adapté key `browser_specific_settings`).
- `extension/README.md` install dev mode + structure.

### C27.2 — Port JS du moteur (1 jour)

- `src/lib/empreinte/afnor.ts` : port direct des formules AFNOR/Sobr.ia (sans Monte-Carlo — usage point-estimate suffit côté extension, MC v0.7+).
- `src/lib/empreinte/ecologits.ts` : port direct des formules EcoLogits 2026-01 (parité ≤ 2 % vs Rust validée par 3 reproduction cases).
- `src/lib/presets.ts` : 8 presets modèles embarqués (Llama 3.1 70B, GPT-4o, Claude 3.5, Mistral Large 2, etc.) — JSON statique sans dépendance réseau.
- `src/lib/empreinte/index.ts` : facade `estimate({ method, modelId, tokens_in, tokens_out, region })`.
- Tests `tests/unit/empreinte.spec.ts` : 3 cases identiques aux ReproductionCase Rust → assertion P50 ± 2 %.

### C27.3 — Détection prompts par site (1 jour)

- `src/content/shared/prompt-detector.ts` : MutationObserver générique avec callback typé.
- `src/content/chatgpt.ts` :
  - Observer `[data-testid="send-button"]` et `textarea[id="prompt-textarea"]`.
  - Extraction prompt complet à la soumission.
  - Détection modèle via URL (`?model=gpt-4o`) ou header en page.
- `src/content/claude.ts` :
  - Observer le textarea `[contenteditable="true"]` + clic « Send ».
  - Modèle depuis le sélecteur de modèle visible en page.
- `src/content/le-chat.ts` :
  - Pattern similaire, modèle depuis `data-attr` ou URL.
- `src/content/shared/badge-injector.ts` : insère après chaque message envoyé une bulle compacte `🌱 0,42 gCO₂eq · 1,8 mL · 0,12 Wh` (couleurs lime/coral selon seuil).
- Tests unitaires : `tests/unit/prompt-detector.spec.ts` avec fixtures HTML statiques (capturées depuis prod, anonymisées).

### C27.4 — Popup + Options (1 jour)

- `src/popup/index.html` + `main.ts` : vanilla DOM, design system Sobr.ia.
  - Carte résultat du dernier prompt (modèle, tokens, gCO₂eq P50 + intervalle P5-P95).
  - Total journalier (4 chiffres : prompts, gCO₂eq, eau, énergie).
  - Toggle méthodologie AFNOR ⇄ EcoLogits (synchronisé avec `chrome.storage`).
  - Lien « Ouvrir l'app Sobr.ia » (`sobria://` deeplink si app installée).
- `src/options/index.html` + `main.ts` :
  - Section "Pont app native" (opt-in, instructions install bridge).
  - Section "Sites" (toggle par site, désactivable individuellement).
  - Section "Confidentialité" (purge données locales, export JSON, opt-out badge en page).
  - Section "Méthodologie" (lien vers `/methodologies` de l'app Tauri).
- `src/popup/popup.css` : palette + tokens du design system Sobr.ia (réutilisés de `web/src/lib/styles/`).

### C27.5 — Bridge natif + pairing 6 chiffres + ingestion app (1.5 jour)

**Le morceau central et le plus délicat.** Découpé en 4 sous-étapes :

**C27.5.a — Crate `sobria-bridge`** :
- `crates/sobria-bridge/Cargo.toml` + `src/main.rs` :
  - Binaire ~500 KB. Parse stdin/stdout length-prefixed JSON (protocole standard MV3).
  - Forwarde vers IPC Tauri via socket Unix (`/tmp/sobria.sock` Unix, named pipe `\\.\pipe\sobria` Windows).
  - Si app pas lancée → forwarde vers spool fichier (`~/.sobria/spool/incoming.jsonl`).
- Manifest natif (template) `sobria-bridge.manifest.json.tmpl` qui sera rendu à l'install par l'app Tauri (substitution `{{BRIDGE_PATH}}` + `{{ALLOWED_ORIGINS}}`).

**C27.5.b — Auto-install par l'app Tauri** :
- Module `crates/sobria-app/src/bridge_install.rs` :
  - `detect_browsers() -> Vec<BrowserKind>` (présence des dossiers config Chrome/Firefox/Edge).
  - `install_native_manifest(browser, bridge_path)` écrit le manifest aux bons emplacements OS.
  - `uninstall_native_manifest(browser)` supprime.
- IPC `install_extension_bridge()` + `uninstall_extension_bridge()` + `bridge_status()`.
- Dialog Svelte au premier démarrage : "Activer la synchro extension navigateur ?" → Oui / Non / Plus tard.
- Section dédiée `/parametres → Extension navigateur` : statut bridge, navigateurs détectés, install/uninstall.

**C27.5.c — Pairing code 6 chiffres** :
- Table SQLite `device_pairings(id, fingerprint, secret_hash, code_expires_at, created_at, last_seen_at, revoked_at)`.
- Module `crates/sobria-app/src/pairing.rs` :
  - `generate_pairing_code() -> String` (6 chiffres, OS RNG, TTL 5 min).
  - `verify_pairing_code(code) -> Result<PairingSecret>` (constant-time compare, génère secret 32 bytes, hash Argon2id).
  - `revoke_pairing(id) -> Result<()>`.
- IPC `regenerate_pairing_code`, `list_pairings`, `revoke_pairing`.
- UI Svelte dans `/parametres → Extension navigateur` :
  - Code à 6 chiffres affiché grand format avec compte-à-rebours (TTL).
  - Bouton "Régénérer".
  - Liste des extensions pairées (fingerprint navigateur + dernière activité) avec bouton "Dépaire".

**C27.5.d — Ingestion côté app** :
- Table `extension_events(id, pairing_id, ts, method, model_id, tokens_in, tokens_out, gco2eq_p50, gco2eq_p5, gco2eq_p95, water_ml, energy_wh, raw_payload_json)`.
- IPC `drain_extension_spool()` qui (a) lit le spool, (b) vérifie le secret Argon2id, (c) insère dans `extension_events`, (d) émet event Tauri `extension_event_ingested`.
- Polling 5 s côté app (timer Tokio).
- Le Journal frontend ajoute filtre "Toutes / App / Extension" + badge "Extension".
- Dashboard M15 ajoute breakdown "App vs Extension".

**Tests** :
- `crates/sobria-bridge/tests/protocol.rs` : roundtrip stdin/stdout JSON length-prefixed.
- `crates/sobria-app/tests/pairing.rs` : code valide / expiré / invalide / révoqué.
- `extension/tests/unit/native-messaging.spec.ts` : mock du port natif.
- E2E `extension/tests/e2e/pairing.spec.ts` : install simulé + pairing + estimation ingérée.

### C27.6 — Build, packaging, doc (0.5 jour)

- `scripts/build-extension.sh` :
  - `cd extension && npm ci && npm run build` produit `dist/` et `dist-firefox/`.
  - Zippe `dist/` en `sobria-extension-chrome-v0.6.0.zip`.
  - Zippe `dist-firefox/` en `sobria-extension-firefox-v0.6.0.xpi` (Firefox accepte le `.zip` renommé).
- `.github/workflows/extension-release.yml` : déclenché sur tag `v0.6.0`, build + upload des 2 fichiers en GitHub Release Asset.
- `extension/README.md` :
  - Install dev mode (Chrome `chrome://extensions`, Firefox `about:debugging`).
  - Install pont natif optionnel.
  - Troubleshooting (extension désactivée, console DevTools, etc.).
- `docs/extension/architecture.md` : diagramme flux, sécurité, audit permissions.
- README racine + dossier candidature data.gouv.fr : section "Extension navigateur" ajoutée.

---

## 4. Definition of Done v0.6.0

- [ ] `cd extension && npm run check` → 0 errors TS strict.
- [ ] `npm run lint` (ESLint + Prettier) propre.
- [ ] `npm run test` (Vitest) ≥ 90 % couverture sur `src/lib/empreinte/`.
- [ ] `npm run e2e` (Playwright) passe sur fixtures HTML statiques des 3 sites.
- [ ] L'extension s'installe en mode dev sur Chrome + Firefox sans warning console.
- [ ] Sur ChatGPT/Claude/Le Chat, un prompt envoyé affiche un badge `gCO₂eq` dans la page < 1 s.
- [ ] La popup affiche le total journalier persistant après rafraîchissement.
- [ ] Si bridge natif installé, l'estimation apparaît dans le Journal de l'app Tauri.
- [ ] Permissions auditées : `activeTab`, `storage`, `nativeMessaging` (opt-in). Aucune permission supplémentaire.
- [ ] CSP stricte : pas de `unsafe-eval` ni `unsafe-inline`.
- [ ] Audit `npm audit --audit-level=moderate` propre.
- [ ] Bundle Chrome `.zip` ≤ 500 KB. Bundle Firefox `.xpi` ≤ 500 KB.
- [ ] CHANGELOG entrée `[0.6.0] — YYYY-MM-DD` complète.
- [ ] ADR-0013 « WebExtension MV3 + native messaging bridge » rédigé.
- [ ] Tag `v0.6.0` poussé + release notes auto-générées + assets uploadés.

---

## 5. Anti-périmètre (différé v0.7+)

- Soumission Chrome Web Store + AMO (v0.6.1 ou v0.7.0).
- Gemini, Perplexity, Copilot, Poe, You.com.
- Comparateur de modèles direct dans la page (équivalent M3).
- Popup riche multi-onglets (Dashboard, Forecaster, etc.).
- Mode "groupe de travail" / synchronisation multi-poste.
- Bouton "améliorer mon prompt" (LLM optimization).

---

## 6. Risques + mitigations

| Risque | Probabilité | Mitigation |
|--------|-------------|------------|
| DOM ChatGPT change toutes les 2 semaines | Haute | Tests E2E avec fixtures HTML capturées datées + GitHub Actions cron qui ouvre une issue si selectors cassent |
| Native messaging bloqué par OS/AV | Moyenne | Fallback gracieux vers storage local + message clair "App non détectée" |
| Bundle MV3 trop gros (> 500 KB) | Faible | Tree-shaking Vite + presets modèles minimaux + pas de bundler runtime |
| Refus Chrome Web Store pour permission `nativeMessaging` | Moyenne | Argumentaire en page de soumission + opt-in explicite + privacy policy publique. Soumission différée v0.6.1 de toute façon. |
| Parité moteur JS vs Rust > 2 % | Moyenne | 3 ReproductionCase identiques côté JS, snapshot golden values commitées |

---

## 7. Livrables annexes

- ADR-0013 « WebExtension MV3 + native messaging bridge ».
- README racine : section "Extension navigateur" avec capture popup + lien install.
- Dossier candidature data.gouv.fr : ajout démo extension (capture badge dans ChatGPT).
- Politique de confidentialité publique (`docs/extension/privacy-policy.md`) — pré-requis pour soumission stores v0.6.1.

---

## 8. Découpage temporel

| Jour | Sous-chantier | Livrable |
|------|---------------|----------|
| J1 | C27.1 + C27.2 | Bootstrap + moteur JS testé contre Rust |
| J2 | C27.3 | Détection ChatGPT + Claude + Le Chat + badge en page |
| J3 | C27.4 | Popup + Options fonctionnels |
| J4-J5 | C27.5 a/b/c/d | Bridge + auto-install + pairing 6 chiffres + ingestion app (1.5 jour) |
| J6 | C27.6 + ship | Build, packaging, doc, tag v0.6.0 |

Total estimé : **5-6 jours** selon densité (le pairing ajoute ~1.5 j vs le brief initial).
