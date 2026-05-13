# Prompt à transmettre à Claude Code — C09 frontend Sobr.ia

> **Mode d'emploi** : copie tout ce qui suit le séparateur `--- PROMPT ---`
> ci-dessous et colle-le dans une session Claude Code à la racine du repo
> `defis-lia-generatif`. Claude Code lit déjà `CLAUDE.md` à chaque session.

---

## Pourquoi ce prompt existe

- Cowork (moi) a livré le **backend Rust + IPC Tauri** dans
  `crates/sobria-app/` (commandes, DTO, `AppState`, capabilities,
  `tauri.conf.json`).
- Claude Code se charge maintenant du **frontend SvelteKit** en intégrant
  le design produit par Claude Design :
  `https://api.anthropic.com/v1/design/h/pzqXRiIjWHO1dkAt2b91cw`.
- Référentiel : `briefs/chantiers/C09-tauri-integration.md` (§3 = contrats
  IPC précis).

---

## --- PROMPT ---

```
Tu es Claude Code, en charge de la réalisation du frontend de Sobr.ia
(chantier C09 — voir briefs/chantiers/C09-tauri-integration.md).

OBJECTIF
========
Créer le dossier `web/` à la racine du repo avec une application
SvelteKit 2 + TypeScript strict qui :

1. Fetch le design Claude Design ci-dessous, lit son README, et implémente
   les écrans pertinents en respectant ses tokens visuels :

       Fetch this design file, read its readme, and implement the
       relevant aspects of the design.
       https://api.anthropic.com/v1/design/h/pzqXRiIjWHO1dkAt2b91cw
       Implement: the designs in this project

2. Câble les commandes Tauri 2 exposées par `crates/sobria-app/` via
   `@tauri-apps/api/core` (fonction `invoke`).

3. Démontre la chaîne complète : formulaire → IPC → affichage des
   indicateurs P5-P50-P95 + journalisation visible dans l'écran *Audit*.

CONTEXTE OBLIGATOIRE À LIRE EN PREMIER
======================================
- CLAUDE.md (racine) — conventions du projet, anti-patterns.
- briefs/chantiers/C09-tauri-integration.md — contrats IPC (§3 = DTO).
- docs/CAHIER-DES-CHARGES-v1.0.md §6 (UX) et §7 (architecture).
- docs/ux/MAQUETTE-UI-TEXTUELLE.md — écrans cibles côté contenu.
- docs/adr/ADR-0001-rust-tauri.md, ADR-0008 (frontend SvelteKit).
- crates/sobria-app/src/dto.rs — types Rust à mirrorer côté TypeScript.
- crates/sobria-app/tauri.conf.json — devUrl, frontendDist, productName.
- crates/sobria-app/capabilities/default.json — capabilities Tauri.

CONTRATS IPC (RAPPEL — voir C09 §3 pour les DTO complets)
=========================================================
Commandes exposées par sobria-app, invocables côté front via
`invoke('<name>', args)` :

- meta_info() -> MetaInfo
- list_models() -> ModelPresetDto[]
- estimate_prompt({ req }) -> EstimationResultDto
- verify_audit() -> IntegrityReportDto
- list_audit_entries({ limit, offset }) -> AuditEntrySummaryDto[]
- export_audit_ndjson({ path }) -> number (lignes écrites)

Erreurs : { code: string, message: string, details?: any }.
Codes connus : unknown_model, invalid_request, estimator_error,
audit_error, io_error, internal.

LIVRABLES ATTENDUS
==================

A) `web/` — SvelteKit 2 + TS strict (à initialiser via
   `npm create svelte@latest web -- --template skeleton --types typescript`,
   puis adapter pour cible statique `@sveltejs/adapter-static` avec
   `fallback: 'index.html'` pour Tauri).

B) `web/src/lib/api.ts` — wrapper typé autour de `invoke` :
   - Types TS qui mirrorent **à l'identique** les DTO de `dto.rs`.
   - Une fonction par commande, retour `Promise<T>`, erreurs typées.
   - **Pas de mock, pas de fallback, pas de données factices.** Tous les
     appels passent par `invoke()` réel. Si le contexte Tauri n'est pas
     disponible (ex: ouverture de `npm run dev` dans un navigateur seul),
     l'UI doit afficher un message d'erreur clair (« L'application doit
     être lancée via `cargo run -p sobria-app` ») plutôt que d'afficher
     du faux contenu.
   - Le développement se fait **toujours** via `cargo tauri dev` (ou
     équivalent) — pas en SvelteKit pur isolé.

C) `web/src/routes/+layout.svelte` + `+page.svelte` — coque + écran
   *Estimer* avec :
   - <Select> liste de modèles (depuis `list_models`).
   - Inputs `tokens_in`, `tokens_out_estimated` (number, ≥ 0).
   - Bouton « Estimer » → `estimate_prompt` → carte résultat.
   - Carte résultat : 3 indicateurs (CO₂eq, énergie, eau) avec P5-P50-P95,
     équivalents parlants, drawer « hypothèses » (key/value/source).
   - Badge calibration du modèle (validated/indicative/extrapolated)
     avec tooltip explicatif.

D) `web/src/routes/audit/+page.svelte` — écran *Audit* :
   - Bouton « Vérifier la chaîne » → `verify_audit`, affiche
     valid/invalid + total + message.
   - Tableau paginé via `list_audit_entries` (limit=50).
   - Bouton « Exporter NDJSON » → ouvre un dialog save via
     `@tauri-apps/plugin-dialog`, puis `export_audit_ndjson`.

E) `web/src/routes/methodo/+page.svelte` — écran *Méthodologie* :
   liens vers `docs/methodology/*` (rendus en Markdown côté front avec
   `marked`), badges de licences (Etalab, MIT, CC-BY).

F) Tests Playwright `web/tests/estimate.spec.ts` :
   - Charge l'app (mock Tauri activé).
   - Sélectionne « gpt-4o-mini ».
   - Saisit 100 / 500 tokens.
   - Clique « Estimer ».
   - Vérifie l'apparition de 3 indicateurs avec valeurs numériques.

G) Documentation `web/README.md` + mise à jour `crates/sobria-app/README.md`
   pointant vers `web/`.

CONTRAINTES NON-NÉGOCIABLES
===========================
1. TypeScript strict (`"strict": true`, `"noUncheckedIndexedAccess": true`).
2. ESLint + Prettier (config racine).
3. Pas d'import de framework UI lourd (pas de Bootstrap, Material, etc.) —
   le design Claude Design fournit les tokens, on construit les composants.
4. **Aucune** dépendance > 100 ko gzip sans justification écrite dans une
   note de commit ou un ADR léger.
5. i18n FR par défaut, structure prête pour EN (clés en kebab-case via
   `@inlang/paraglide` OU un store `t()` léger custom — au choix
   documenté).
6. A11y : tous les inputs labellés, tabindex cohérent, contrastes WCAG AA.
7. Pas d'appel réseau externe (CSP `default-src 'self'` — voir
   `tauri.conf.json`).
8. Branding : `productName: "Sobr.ia"`, titre fenêtre déjà fixé dans
   `tauri.conf.json`. Ne pas modifier sans demander.

INSTALLATION DES PLUGINS TAURI ATTENDUS
========================================
Ajoute dans `crates/sobria-app/Cargo.toml` :
- `tauri-plugin-dialog = "2"` (pour le save dialog NDJSON).

Et côté front :
- `@tauri-apps/api` (>= 2.0.0)
- `@tauri-apps/plugin-dialog` (>= 2.0.0)

Enregistre le plugin dans `main.rs` :
```rust
tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .setup(...)
```

Et mets-à-jour `capabilities/default.json` permissions :
- `dialog:default`, `dialog:allow-save`, `dialog:allow-open`.

WORKFLOW SUGGÉRÉ
================
1. Lis CLAUDE.md, le brief C09, le dto.rs.
2. Fetch le design Claude Design + son README, note les tokens
   (couleurs, type scale, spacing).
3. Scaffold `web/` (SvelteKit 2 + adapter-static).
4. Écris `web/src/lib/api.ts` + mocks.
5. Écris la coque `+layout.svelte` + les 3 routes minimales.
6. Lance `npm run dev` → vérifie l'écran *Estimer* avec mocks.
7. Lance `cargo run -p sobria-app` (dev mode, devUrl = SvelteKit dev) →
   vérifie l'aller-retour IPC réel.
8. Ajoute les tests Playwright.
9. Vérifie : `npm run lint && npm run check && npm run test`.
10. Vérifie : `cargo clippy -p sobria-app -- -D warnings`,
    `cargo test -p sobria-app`.
11. Mets à jour CHANGELOG.md + briefs/chantiers/C09-RETROSPECTIVE.md.

PIÈGES CONNUS
=============
- Tauri 2 a renommé `invoke` import : c'est `@tauri-apps/api/core`,
  pas `@tauri-apps/api/tauri`.
- `tauri::generate_context!()` lit `tauri.conf.json` relatif au manifest
  Cargo de `sobria-app` — ne déplace pas le fichier.
- Sur Linux, le build Tauri requiert `libwebkit2gtk-4.1-dev`,
  `libsoup-3.0-dev`, etc. — voir le bootstrap script si Claude Code
  installe.
- N'invente pas de modèle : la liste exacte vient de `list_models()`
  (≥ 8 presets définis dans crates/sobria-estimator/src/model_presets.rs).

DÉFINITION DE TERMINÉ (cf C09 §7)
==================================
- [ ] `cargo run -p sobria-app` ouvre la fenêtre avec l'UI.
- [ ] L'écran *Estimer* fait un aller-retour IPC réel en < 200 ms.
- [ ] L'écran *Audit* affiche les entrées créées.
- [ ] Tests Playwright verts.
- [ ] `npm run lint && npm run check` verts.
- [ ] `cargo clippy -p sobria-app -- -D warnings` propre.
- [ ] `briefs/chantiers/C09-RETROSPECTIVE.md` rédigé.
- [ ] Tag candidat `v0.2.0-app`.

Si quelque chose dans ce prompt entre en conflit avec CLAUDE.md ou le
brief C09, **demande confirmation à Thibault avant de trancher**.
```

---

## Notes complémentaires (hors-prompt — pour toi, Thibault)

- Le prompt ci-dessus est volontairement long mais **autonome** : Claude
  Code n'a pas besoin de cette conversation, il a le CLAUDE.md + le
  brief C09 + le code Rust déjà en place.
- Si tu veux tester en bourse l'IPC sans frontend : `cargo test -p sobria-app`
  exerce déjà toutes les commandes via la couche `logic::*` (les commandes
  Tauri sont des délégations 1-pour-1).
- Quand Claude Code aura livré, je rédigerai la **rétrospective C09** et
  taggerai `v0.2.0-app`.
