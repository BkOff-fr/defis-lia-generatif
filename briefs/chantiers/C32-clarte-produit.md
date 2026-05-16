# Chantier C32 — Clarté produit (v0.8.0)

> **Version cible** : v0.8.0 — release intermédiaire « Clarté produit »
> **Sprint** : 4-5 jours focalisés UX, copy, narratif
> **Pré-requis** : v0.7.1 shippée, **audit produit C32.0 livré** (`docs/product/AUDIT-PRODUIT-2026-Q3.md`)
> **Objectif** : combler le gap de clarté UX identifié par l'audit avant la candidature data.gouv.fr v1.0. Score clarté moyen actuel 6/10 → cible **8/10 minimum** sur les 5 personas.

---

## 0. Pourquoi ce chantier maintenant

L'audit produit C32.0 a révélé que Sobr.ia est **techniquement excellent** mais **pédagogiquement confus** pour 3 personas sur 5 (Student 4/10, Enterprise 5/10, Public Sector 6/10). Le jury data.gouv.fr ouvrira l'app comme un Student curieux, pas comme un dev senior. **Sans C32, la candidature est faisable mais perd ses meilleurs arguments.**

C32 ne touche **presque pas au code métier** — il s'agit principalement de :
- Réécriture de textes (README, taglines, labels modules)
- Refonte de l'onboarding (1 étape ajoutée)
- Cleanup des artefacts internes qui fuient en UI (M1, M3...)
- Ajout d'équivalences humaines (gCO₂eq → douches, km voiture)
- Panneau guidé Mode Équipe
- Intégration vendors disclosure (C31.1 ramené pré-v1.0)
- DOI Zenodo

---

## 1. Périmètre par sous-chantier

### C32.1 — Messaging + labels + nettoyage (1 jour)

**Livrables** :

- **README racine refondu** :
  - Bloc en tête **"Sobr.ia, c'est quoi ?"** en langage simple (≤ 3 phrases, sans jargon).
  - Section **"Pour qui ?"** avec 5 cartes persona — chaque carte = 1 phrase value prop + 1 lien doc spécifique.
  - **Value proposition 1 phrase** en exergue, placardée en haut : *« Sobr.ia mesure l'empreinte de vos prompts IA en local, agrège les chiffres officiels des fabricants (Mistral × ADEME, Google, Meta) et vous donne un journal scientifique reproductible — pour particulier, équipe ou administration, sans cloud Sobr.ia. »*
  - Le contenu technique actuel (Monte-Carlo, AFNOR SPEC 2314, etc.) **redescend** plus bas (après la section "Pour qui ?").
- **Renaming labels modules** dans `web/src/lib/preferences.ts` (`MODULES` const) :
  - M9 « Référentiel modèles » → **« Bibliothèque de modèles »**
  - M17 « Empreinte projet » → **« Datasheet scientifique »**
  - Tous les autres labels : revue rapide, garder ce qui parle.
- **Cleanup IDs dans l'UI** :
  - Retirer "M1", "M3"... des labels visibles dans le rail nav, breadcrumbs, titres de page.
  - Garder les IDs en URL (`/m3`, `/m13`) et dans le code (`ModuleId::M3`).
  - Le rail devrait afficher juste "Estimer", "Comparer", "Journal", etc.
- **Retirer M14 "À propos" des bundles personas** dans `crates/sobria-core/src/preferences.rs` :
  - M14 doit rester accessible (lien footer ou menu kebab) mais **pas** apparaître dans les bundles par défaut.
  - Adapter les tests `default_modules` qui vérifient M14 dans les bundles.
- **Renommer M22 « Rapport CSRD/AGEC »** → **« Rapport réglementaire (CSRD/AGEC) »** (le mot "réglementaire" parle plus large que le sigle).

**Commits** :
- `refactor(ui): C32.1 README refondu + value prop en exergue + section Pour qui`
- `refactor(core,ui): C32.1 labels modules clarifiés (Bibliothèque modèles, Datasheet scientifique, etc.)`
- `chore(core): C32.1 retirer M14 des bundles personas (toujours accessible via menu)`

### C32.2 — Onboarding pédagogique + fil narratif (1 jour)

**Livrables** :

- **Nouvelle étape "Sobr.ia en 30 secondes"** entre splash et persona picker, dans `web/src/routes/onboarding/+page.svelte` :
  - 4 phrases max (un peu comme le pitch refondu).
  - 1 schéma SVG simple "1 prompt = X gCO₂eq = Y douches" (composant `<EquivalenceSchema />`).
  - Bouton "Continuer" pour passer au persona picker.
  - Bouton "Passer cette étape" pour les pressés.
- **Bannière "Et après ?"** sous le résultat M1 (Atelier) — `web/src/routes/+page.svelte` :
  - 3 cartes suggestions contextuelles, ex :
    - "Comparer ce modèle à d'autres → /comparer"
    - "Voir votre usage cumulé → /m15"
    - "Fixer un budget mensuel → /m25"
  - Affichée seulement après le 1er prompt soumis (pas avant), discrète mais visible.
- **Tooltip "Pourquoi ces modules ?"** dans onboarding étape 3 + `/parametres` section Modules :
  - Au survol d'un module pré-coché, tooltip qui explique pourquoi il est dans le bundle de ce persona.
  - Ex (Student + M13 Simulateur) : *« Comprendre l'impact de tes choix : changer de modèle, raccourcir tes prompts, mode batch… »*

**Commits** :
- `feat(ui): C32.2 onboarding étape "Sobr.ia en 30 secondes" + schéma équivalence`
- `feat(ui): C32.2 bannière "Et après ?" post-prompt + tooltip pourquoi modules`

### C32.3 — Équivalences humaines + Mode Équipe guidé (1.5 jour)

**Livrables** :

- **Composant `<EquivalenceCarbon />`** réutilisable dans `web/src/lib/components/` :
  - Props : `{ gco2eq: number }` (et optionnel `waterMl: number`, `energyWh: number`).
  - Affichage : `= 0,4 km voiture · 0,3 douche · 2 minutes de TV LED` (équivalences variables selon les unités).
  - Table d'équivalences ergonomique :
    - 1 g CO₂eq ≈ 5 m en voiture thermique
    - 1 g CO₂eq ≈ 4 minutes de streaming vidéo SD
    - 1 mL eau = goutte d'eau
    - 1 L eau ≈ 1/9 d'une douche (8 L pour douche éco)
    - 1 Wh ≈ 1 minute LED 60W
  - Sources commentées dans le code (ADEME Base Empreinte + études Shift Project).
- **Intégration dans M1 Atelier** (sous le résultat) + **M15 Dashboard** (cards totaux) + **M25 Eco-budget** (jauge atteinte).
- **Panneau "Activer Mode Équipe"** dans `/parametres` (section dédiée si Mode Équipe non encore configuré) :
  - Bouton "Activer Mode Équipe (mon entreprise)" — affiche dialog `aria-modal`.
  - Dialog explique en 3 étapes simples : (1) Télécharger le serveur, (2) Initialiser, (3) Distribuer codes aux employés.
  - Lien direct "Télécharger sobria-team-aggregator" pointant vers la dernière release GitHub.
  - Lien "Voir le guide complet (5 minutes)" → `docs/operations/team-aggregator.md`.
- **Quickstart "5 minutes — déployer en entreprise"** ajouté en TÊTE de `docs/operations/team-aggregator.md` :
  - 5 étapes numérotées avec commandes copy-paste.
  - Captures écran du dashboard admin.
  - Section "Pour les non-IT" + section "Pour les DSI" séparées.

**Commits** :
- `feat(ui): C32.3 composant EquivalenceCarbon réutilisable + intégration M1/M15/M25`
- `feat(ui,docs): C32.3 panneau Activer Mode Équipe + quickstart 5 minutes`

### C32.4 — Vendors disclosure (C31.1 ramené pré-v1.0) (1.5 jour)

**Livrables** : reprendre le contenu de C31.1 du brief `C31-integration-tier2-datasets.md` :

- **Mistral × ADEME Large 2** :
  - Mise à jour preset `mistral-large-2` dans `crates/sobria-estimator/src/presets.rs` avec valeurs Mistral/ADEME (training : 20.4 ktCO₂eq + 281 000 m³ eau ; inference : 1.14 gCO₂eq par requête de 400 tokens).
  - Citation source dans commentaire : `// Source: Mistral AI × ADEME × Carbone 4 (2025-08), https://mistral.ai/news/our-contribution-to-a-global-environmental-standard-for-ai`.
  - Encadré "Données ACV vendor (vérifiées ADEME)" dans M9 fiche modèle (`web/src/routes/m9/[id]/+page.svelte` ou équivalent).
- **Google Gemini** :
  - Preset Gemini enrichi : 0.24 Wh/prompt médian, 0.03 gCO₂eq, 0.26 mL eau.
  - Citation Google paper août 2025.
  - Encadré M9 + avertissement méthodologie ("médian text prompt, méthodo Google").
- **Meta Llama 3.x** :
  - Preset Llama 3.1 / 3.3 enrichi (training : 39.3M GPU h H100, 11 390 tCO₂eq location-based / 0 tCO₂eq market-based).
  - **Encadré pédagogique** "location-based vs market-based" dans M9 fiche Llama : *« Meta affiche 0 tCO₂eq en market-based car ils achètent des REC qui matchent leur conso annuelle. Mais l'élec localement consommée par les datacenters au moment du training est bien 11 390 tCO₂eq. Sobr.ia présente les deux pour transparence totale. »*
- **Nouvelle table SQLite Gold** `vendor_disclosures(model_id, vendor, scope, value, unit, source_url, published_at, methodology_note)` — migration v3.
- **Nouvelle vue M9** : "Données vendor disclosure" — table comparaison `Mistral / Google / Meta / Anthropic / OpenAI` montrant ce qui est publié vs ce qui ne l'est pas (avec "❌ Pas de disclosure" pour Anthropic + OpenAI).

**Commits** :
- `feat(estimator,core): C32.4 vendors disclosure (Mistral × ADEME, Google Gemini, Meta Llama)`
- `feat(ui): C32.4 encadrés vendor disclosure + table comparaison M9`

### C32.5 — Polish + DOI + ship v0.8.0 (0.5 jour)

**Livrables** :

- **DOI Zenodo** :
  - Créer release Zenodo via intégration GitHub (workflow standard).
  - Ajouter badge DOI en haut du README.
  - Ajouter section "Citation" dans README avec citation BibTeX format Zenodo.
- **CHANGELOG entrée `[0.8.0] — YYYY-MM-DD — Clarté produit (C32)`** avec récap C32.1 → C32.4.
- **Bump versions** :
  - `Cargo.toml` workspace : `0.7.1 → 0.8.0`
  - `crates/sobria-app/tauri.conf.json` : `0.7.1 → 0.8.0`
  - `web/package.json` : `0.7.1 → 0.8.0`
  - `extension/package.json` + `manifest.json` : `0.7.1 → 0.8.0`
  - `web-team/package.json` : `0.7.1 → 0.8.0`
- **Smoke test E2E des 5 onboardings personas** :
  - Pour chaque persona, faire le walkthrough complet (onboarding → premier prompt → suggestion "Et après ?").
  - Documenter dans `docs/qa/smoke-test-2026-05.md` les findings.
- **Tag `v0.8.0`** + push.

---

## 2. Definition of Done v0.8.0

- [ ] `cargo test --workspace` 100 % vert.
- [ ] `cargo clippy --workspace -- -D warnings` propre.
- [ ] `cargo fmt --check` propre.
- [ ] `cd web && npm run check && npm run lint && npm run test` propre.
- [ ] `cd web-team && npm run check && npm run lint && npm run test` propre.
- [ ] `cd extension && npm run check && npm run lint && npm run test` propre.
- [ ] **Smoke test manuel des 5 personas onboarding** : chaque persona arrive sur la page d'accueil de l'app avec un fil narratif clair (étape "Sobr.ia en 30s" + bundle expliqué + bannière "Et après ?").
- [ ] **README refondu** : section "Pour qui ?" en tête, value proposition 1 phrase, jargon technique repoussé après.
- [ ] **Labels modules** : plus aucun "M1", "M3"... visible côté UI utilisateur.
- [ ] **Bundles personas** : M14 retiré.
- [ ] **Équivalences humaines** présentes dans M1 + M15 + M25.
- [ ] **3 vendors disclosure** dans presets + encadrés M9.
- [ ] **Table comparaison vendor disclosure** dans M9.
- [ ] **Panneau "Activer Mode Équipe"** dans `/parametres` avec dialog 3 étapes.
- [ ] **Quickstart 5 minutes** dans `docs/operations/team-aggregator.md`.
- [ ] **DOI Zenodo** publié + badge dans README.
- [ ] CHANGELOG `[0.8.0]` complète.
- [ ] Bump versions cohérent partout.
- [ ] Tag `v0.8.0`.

---

## 3. Anti-périmètre

- Pas de refactor du moteur Monte-Carlo (changements vendor disclosure = juste presets + nouvelles tables).
- Pas de SSO / RBAC / multi-device (différé v0.9 / v1.2).
- Pas de cloud beta managed (différé v1.3+).
- Pas de nouvelles méthodologies (ML.ENERGY, HF AI Energy Score, ADEME Base Empreinte API → C31 v1.1 post-candidature).
- Pas de nouveaux modules (focus polish des 13 existants).

---

## 4. Découpage temporel suggéré

| Jour | Sous-chantier | Livrable |
|---|---|---|
| J1 | C32.1 messaging + labels | README + labels + bundles |
| J2 | C32.2 onboarding + fil narratif | Étape 30s + bannière "Et après ?" |
| J3 | C32.3 équivalences + Mode Équipe guidé (1/2) | Composant Équivalence + intégration M1/M15/M25 |
| J3.5 | C32.3 Mode Équipe guidé (2/2) | Panneau + quickstart doc |
| J4 | C32.4 vendors disclosure (1/2) | Presets + table SQLite |
| J4.5 | C32.4 vendors disclosure (2/2) | Encadrés M9 + table comparaison |
| J5 | C32.5 polish + DOI + ship | DOI + smoke test + tag v0.8.0 |

Total : **4,5-5 jours**.

---

## 5. Risques + mitigations

| Risque | Mitigation |
|---|---|
| Réécriture README trahit le ton scientifique attendu par chercheurs | Garder section "Méthodologies disponibles" actuelle intacte, ajouter "Pour qui ?" en TÊTE, pas en remplacement |
| Équivalences carbone "douches/voiture" jugées imprécises par jury sci | Citer ADEME Base Empreinte comme source de chaque facteur d'équivalence, afficher avec disclaimer "ordre de grandeur" |
| Onboarding étape supplémentaire perçue comme friction | Bouton "Passer cette étape" toujours visible, persisté en préférence `welcome_skipped` |
| Cleanup IDs casse les URL routes | Garder `/m3`, `/m13` etc. en URL, retirer juste des labels affichés |
| Vendor disclosure schema migration v3 | Migration idempotente avec PRAGMA user_version |

---

## 6. Livrables annexes

- `docs/qa/smoke-test-2026-05.md` — résultats du walkthrough 5 personas.
- README v2 vendable lisible par non-tech ET tech.
- `docs/operations/team-aggregator.md` enrichi avec quickstart 5 min.
- DOI Zenodo permanent pour citation académique.

---

## 7. Et après v0.8.0 ?

- **v1.0.0 candidature data.gouv.fr** (sprint final, ~1 semaine) : dossier candidature + vidéo démo + binaires signés + UAT 5 testeurs.
- **v1.1.0 intégration Tier 2 datasets** (C31 complet) : ADEME Base Empreinte API, ML.ENERGY, EpochAI, HF AI Energy Score, IEA, Shift Project, ODRE complémentaire, ARCEP, CCF.
- **v1.2.0 admin avancée** : SSO, RBAC, multi-device.
- **v1.3.0 cloud beta managed** (ADR-0014 Phase 5).
