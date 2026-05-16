# C32 — Prompt Claude Code (v0.8.0 — Clarté produit)

> **Mode d'emploi** : copier-coller le bloc ci-dessous dans une nouvelle session Claude Code (CLI) à la racine du repo. Le prompt démarre par `/using-superpower`.

---

```
/using-superpower

# Mission : C32 — Clarté produit (v0.8.0)

Tu vas livrer une release intermédiaire v0.8.0 « Clarté produit » qui
comble le gap UX identifié par l'audit produit C32.0 avant la candidature
data.gouv.fr v1.0. Sprint ~4-5 jours, focus copy + UX + intégration
vendors disclosure.

## Contexte à charger AVANT toute action

Lis ces fichiers dans l'ordre, sans en sauter :

1. `CLAUDE.md` — règles, anti-patterns, DoD.
2. **`docs/product/AUDIT-PRODUIT-2026-Q3.md`** — l'audit produit qui
   motive ce chantier (source de vérité pour les findings et le ton).
3. `briefs/chantiers/C32-clarte-produit.md` — le brief C32 détaillé
   (livrables par sous-chantier, DoD, anti-périmètre).
4. `docs/sources/AUDIT-2026-Q3.md` — l'audit datasets (notamment §D
   pour les vendors disclosure à intégrer).
5. `docs/adr/ADR-0014-dual-track-local-cloud.md` — vision long terme.
6. `CHANGELOG.md` entrée [0.7.1] — ce qui vient d'être shippé.
7. `web/src/routes/onboarding/+page.svelte` + `web/src/lib/preferences.ts`
   — onboarding et catalogue modules actuels.
8. `crates/sobria-core/src/preferences.rs` — bundles personas (à
   modifier en C32.1 pour retirer M14).
9. `crates/sobria-estimator/src/presets.rs` — presets modèles (à
   enrichir en C32.4 avec vendors disclosure).
10. `README.md` — à refondre en C32.1.

## Stratégie + garde-fous

- **C32 est principalement du COPY / UX / docs**, pas du code métier.
- **Aucun nouveau module**, pas de refactor moteur.
- **Tone unifié** : on garde la rigueur scientifique en doc technique
  approfondie (méthodologies, ADRs), on simplifie radicalement le pitch
  utilisateur en surface (README en tête, onboarding, labels).
- **Sourcer chaque facteur d'équivalence** humaine (douches, km, etc.)
  avec ADEME Base Empreinte ou Shift Project.
- **JAMAIS** retirer la section "Méthodologies disponibles" du README —
  on l'enrichit avec un préambule, on ne la remplace pas.
- **TOUJOURS** garder M14 « À propos » accessible (lien menu / footer),
  juste pas dans les bundles personas.
- **DEMANDER** si tu hésites sur un wording (notamment value
  proposition, taglines persona, encadrés vendor disclosure).

## Plan d'exécution

### C32.1 — Messaging + labels + nettoyage (1 j)

Voir brief §C32.1. Résumé :

- **Refondre `README.md`** :
  - Bloc tête **"Sobr.ia, c'est quoi ?"** en langage simple (3 phrases max).
  - Section **"Pour qui ?"** : 5 cartes persona avec value prop + lien
    doc spécifique (à créer dans `docs/personas/{student,pro-tech,
    enterprise,public-sector,researcher}.md` — ~1 page chacun).
  - **Value prop 1 phrase** en exergue (cf. brief). À placer juste sous
    le titre, avant tout le reste.
  - Le contenu technique actuel (Différenciateurs, Méthodologies,
    Architecture) **descend** après "Pour qui ?".
- **Renaming labels modules** dans `web/src/lib/preferences.ts` :
  - M9 « Référentiel modèles » → **« Bibliothèque de modèles »**
  - M17 « Empreinte projet » → **« Datasheet scientifique »**
  - M22 « Rapport CSRD/AGEC » → **« Rapport réglementaire (CSRD/AGEC) »**
- **Cleanup IDs UI** : retirer "M1", "M3"... du rail nav + breadcrumbs
  + titres de page. Garder URLs (`/m3`, `/m13`) inchangées.
- **Retirer M14** des bundles personas dans
  `crates/sobria-core/src/preferences.rs` + adapter les tests
  `default_modules`. M14 reste accessible via menu footer.

DoD C32.1 :
- README v2 vendable lisible par non-tech ET tech (smoke read).
- Aucun "Mx" visible dans les labels UI utilisateur.
- 5 pages persona dans `docs/personas/` (1 page chacune avec : qui, que
  résout-il, top 3 use cases, modules pertinents, commande quickstart).
- Tests `cargo test -p sobria-core` verts.

### C32.2 — Onboarding pédagogique + fil narratif (1 j)

Voir brief §C32.2. Résumé :

- **Nouvelle étape 1.5 "Sobr.ia en 30 secondes"** dans
  `web/src/routes/onboarding/+page.svelte` entre splash et persona :
  - 4 phrases max + composant SVG `<EquivalenceSchema />` (1 prompt
    typique = X gCO₂eq = Y douches).
  - Bouton "Continuer" + bouton "Passer cette étape" (persiste
    `welcome_skipped` dans préférences).
- **Bannière "Et après ?"** dans `web/src/routes/+page.svelte` (M1
  Atelier), affichée seulement après 1er prompt soumis :
  - 3 cartes suggestions contextuelles : Comparer / Voir usage / Fixer
    budget.
  - Stockage local dismiss possible.
- **Tooltips "Pourquoi ces modules ?"** dans onboarding étape 3 +
  `/parametres` section Modules : au survol, explication pourquoi ce
  module est dans le bundle de ce persona.

DoD C32.2 :
- Smoke test : nouveau user → splash 3s → "Sobr.ia en 30 sec" → persona
  picker. L'étape est visuellement légère, pas overwhelming.
- Bannière "Et après ?" apparaît après le 1er prompt, peut être
  dismissed, ne réapparaît pas.

### C32.3 — Équivalences humaines + Mode Équipe guidé (1.5 j)

Voir brief §C32.3. Résumé :

- **Composant `<EquivalenceCarbon />`** dans
  `web/src/lib/components/EquivalenceCarbon.svelte` :
  - Props : `{ gco2eq: number, waterMl?: number, energyWh?: number }`.
  - Table d'équivalences (sourcées ADEME / Shift Project) :
    - 1 g CO₂eq ≈ 5 m en voiture thermique
    - 1 g CO₂eq ≈ 4 min de streaming vidéo SD
    - 1 L eau ≈ 1/9 douche éco (8 L pour douche)
    - 1 Wh ≈ 1 min LED 60W
  - Format adaptatif : pour 0.4 g → "≈ 2 m voiture · 2 min TV LED",
    pour 4 g → "≈ 2 douches · 4 min streaming".
  - Disclaimer "ordre de grandeur" + lien source en hover.
- Intégrer dans M1 Atelier (sous résultat), M15 Dashboard (cards
  totaux), M25 Eco-budget (jauge atteinte).
- **Panneau "Activer Mode Équipe"** dans `/parametres` (section dédiée
  si pas configuré) :
  - Bouton ouvre dialog `aria-modal` 3 étapes :
    1. Télécharger `sobria-team-aggregator` (lien GitHub release).
    2. Initialiser le serveur (commande copy-paste).
    3. Distribuer codes aux employés (lien dashboard admin).
  - Lien "Voir le guide complet 5 minutes" → `docs/operations/team-aggregator.md`.
- **Quickstart "5 minutes" en TÊTE de
  `docs/operations/team-aggregator.md`** :
  - 5 étapes numérotées avec commandes copy-paste.
  - Section "Pour les non-IT" + "Pour les DSI" séparées.

DoD C32.3 :
- Composant EquivalenceCarbon affiché sur M1, M15, M25 avec valeurs
  correctes.
- Dialog Activer Mode Équipe ouvre proprement, lien GitHub OK.
- Quickstart doc lisible par un DSI non développeur.

### C32.4 — Vendors disclosure (C31.1 ramené pré-v1.0) (1.5 j)

Voir brief §C32.4. Résumé :

- **Migration SQLite v3** : nouvelle table
  ```sql
  CREATE TABLE vendor_disclosures (
      id TEXT PRIMARY KEY,
      model_id TEXT NOT NULL,
      vendor TEXT NOT NULL,
      scope TEXT NOT NULL CHECK (scope IN ('training', 'inference_per_prompt')),
      value REAL NOT NULL,
      unit TEXT NOT NULL,
      source_url TEXT NOT NULL,
      published_at TEXT NOT NULL,
      methodology_note TEXT
  );
  ```
- **Seed** : 3 vendors disclosures (Mistral × ADEME Large 2, Google
  Gemini, Meta Llama 3.1 + 3.3).
- **Presets enrichis** dans `crates/sobria-estimator/src/presets.rs` :
  - Champ optionnel `vendor_disclosure: Option<VendorDisclosure>`.
  - Quand présent, utiliser en première priorité dans `EmpreinteEngine`.
- **Encadrés vendor disclosure dans M9 fiche modèle** :
  - Mistral Large 2 : "Données ACV vendor (vérifiées ADEME)" — 1.14
    gCO₂eq par requête 400 tokens, 20.4 ktCO₂eq training.
  - Google Gemini : "Données vendor (méthodo Google)" — 0.03 gCO₂eq
    prompt médian, 0.24 Wh.
  - Meta Llama 3.x : "Données training Meta (location + market-based)"
    — 11 390 tCO₂eq location-based, 0 tCO₂eq market-based, encadré
    pédagogique sur la différence.
- **Table comparaison vendor disclosure** dans M9 (page principale ou
  modal) :
  - 5 lignes : Mistral / Google / Meta / Anthropic / OpenAI.
  - Colonnes : Prompt-level ? / Training ? / Source.
  - ❌ visible sur Anthropic + OpenAI ("Pas de disclosure officielle").

DoD C32.4 :
- `cargo test -p sobria-estimator` vert avec nouveaux presets.
- Migration SQLite v3 idempotente.
- M9 fiche Mistral Large 2 affiche bien l'encadré ACV vendor + valeurs.
- Table comparaison affichée sur M9 page principale.

### C32.5 — Polish + DOI + ship v0.8.0 (0.5 j)

Voir brief §C32.5. Résumé :

- **DOI Zenodo** : créer release Zenodo via intégration GitHub (le
  workflow `.github/workflows/zenodo.yml` à créer si pas déjà fait).
  Ajouter badge DOI en haut du README + section "Citation" BibTeX.
- **CHANGELOG entrée `[0.8.0] — YYYY-MM-DD — Clarté produit (C32)`** :
  - 5 sections (C32.1 à C32.5) avec récap.
- **Bump versions** : Cargo workspace + tauri.conf + web/package +
  extension/package + extension/manifest + web-team/package : tous
  passent à **0.8.0**.
- **Smoke test E2E manuel des 5 personas onboarding** :
  - Pour chaque persona, faire le walkthrough (réset preferences,
    ouvrir onboarding, choisir persona, finir, faire 1 prompt, vérifier
    bannière "Et après ?").
  - Documenter dans `docs/qa/smoke-test-v0.8.0-2026-05.md`.
- **Tag `v0.8.0`** :

```bash
git tag -a v0.8.0 -m "v0.8.0 — Clarté produit (C32)

Release intermédiaire centrée UX et messaging avant la candidature
data.gouv.fr v1.0. Aucun nouveau module métier, mais le produit raconte
enfin son histoire pour les 5 personas (student / pro_tech /
enterprise / public_sector / researcher).

Changements visibles :
- README refondu avec value prop en exergue + section 'Pour qui ?'.
- Onboarding pédagogique : étape 'Sobr.ia en 30 secondes' avant le
  choix de persona.
- Labels modules clarifiés (Bibliothèque modèles, Datasheet
  scientifique, Rapport réglementaire).
- Cleanup IDs internes (M1, M3...) hors UI utilisateur.
- Équivalences humaines (douches, km voiture, kWh frigo) dans M1, M15
  et M25, sourcées ADEME et Shift Project.
- Bannière 'Et après ?' contextuelle post-prompt.
- Panneau 'Activer Mode Équipe' guidé dans /parametres + quickstart 5
  minutes pour DSI.
- 3 vendors disclosure dans le produit : Mistral × ADEME (1.14 g/400
  tk), Google Gemini (0.03 g/médian), Meta Llama 3.x (11390 tCO₂eq
  training location-based, 0 market-based).
- Table comparaison vendor disclosure dans M9 (5 vendors, transparence
  sur Anthropic + OpenAI sans disclosure).
- DOI Zenodo publié pour citation académique.

ADR-0014 reste cohérent (local-first + cloud opt-in v1.3+). Phase
candidature data.gouv.fr v1.0 sera la prochaine étape."
```

## DoD globale

- [ ] `cargo test --workspace` 100 % vert.
- [ ] `cargo clippy --workspace -- -D warnings` propre.
- [ ] `cargo fmt --check` propre.
- [ ] `cd web && npm run check && npm run lint && npm run test` propre.
- [ ] `cd web-team && npm run check && npm run lint && npm run test` propre.
- [ ] `cd extension && npm run check && npm run lint && npm run test` propre.
- [ ] Smoke test 5 personas documenté dans `docs/qa/smoke-test-v0.8.0-2026-05.md`.
- [ ] README v2 lisible non-tech + section "Pour qui ?".
- [ ] 5 pages persona dans `docs/personas/`.
- [ ] Aucun "Mx" visible dans labels UI.
- [ ] Composant EquivalenceCarbon intégré M1+M15+M25.
- [ ] 3 vendors disclosures intégrés + encadrés M9 + table comparaison.
- [ ] DOI Zenodo + badge README.
- [ ] CHANGELOG [0.8.0] complète.
- [ ] Bump versions cohérent.
- [ ] Tag v0.8.0 créé.

## Convention de commit

```
refactor(ui): C32.1 README refondu + value prop en exergue + section Pour qui
refactor(core,ui): C32.1 labels modules clarifiés + retrait IDs UI utilisateur
chore(core): C32.1 retirer M14 des bundles personas
feat(ui): C32.2 onboarding étape "Sobr.ia en 30 secondes" + équivalence
feat(ui): C32.2 bannière "Et après ?" post-prompt + tooltips bundles
feat(ui): C32.3 composant EquivalenceCarbon + intégration M1/M15/M25
feat(ui,docs): C32.3 panneau Activer Mode Équipe + quickstart 5 minutes
feat(estimator,core): C32.4 vendors disclosure (Mistral × ADEME, Google Gemini, Meta Llama)
feat(ui): C32.4 encadrés vendor disclosure + table comparaison M9
docs(release): C32.5 DOI Zenodo + badge README
chore(release): bump v0.8.0
```

## Garde-fous

- **JAMAIS** retirer la section "Méthodologies disponibles" du README
  actuel — l'enrichir, pas la remplacer.
- **JAMAIS** déstabiliser les URL routes existantes (`/m3`, `/m13`
  restent valides).
- **JAMAIS** retirer M14 "À propos" complètement — garder accessible
  via menu footer + page directe.
- **TOUJOURS** citer la source ADEME / Shift Project pour chaque facteur
  d'équivalence carbone.
- **TOUJOURS** garder le tone scientifique en doc technique
  (méthodologies, ADRs).
- **DEMANDER** si tu hésites sur les wordings critiques (value
  proposition, taglines persona, encadrés vendor disclosure).

Bonne mission. Commence par C32.1 (le plus visible et structurant),
puis C32.4 vendors (le scoop pitch), puis C32.2 + C32.3 + C32.5 dans
l'ordre.
```

---

## Notes pour Thibault

- Sprint ~4-5 jours. Surtout du copy, peu de code lourd.
- Au retour : `git log --oneline -15` + on review avant tag v0.8.0.
- Smoke test des 5 personas est critique : si tu peux trouver 1-2
  testeurs externes (étudiant + DSI par ex.) pour valider l'onboarding
  refondu, c'est l'idéal.
- Après v0.8.0 → **v1.0 candidature data.gouv.fr** avec un produit qui
  se vend lui-même.
