# C42 — Slugs parlants, cohérence hors-Tauri, CI Docker, plan de commits

> **Statut** : exécuté le 2026-06-12 (session Cowork), à relire + commiter.

## 1. Slugs parlants (dernier jargon des URLs)

`/m9` → **/modeles** · `/m15` → **/suivi** · `/m17` → **/datasheets** ·
`/m25` → **/eco-budget**. Anciennes routes conservées en pages de
redirection client (`location.replace`, query+hash préservés — SPA
adapter-static, pas de redirect serveur). Rail, liens croisés (home,
suivi→modeles, ReduceSuggestions, preferences.ts) migrés. Specs renommés
(`modeles/suivi/datasheets/eco-budget.spec.ts`) + test de redirection.
Les IDs modules (mX) restent internes (gating personas inchangé).

## 2. Cohérence hors-Tauri : plus d'actions qui mentent

- **rapport-csrd** : la bannière « Application de bureau requise »
  (branche morte depuis C37) réveillée sur `isTauriContext()`, message
  humanisé, bouton Générer désactivé — fini l'échec tardif code `internal`.
- **journal** : « Vérifier la chaîne » + « Exporter NDJSON » désactivés
  hors Tauri avec explication au survol.
- **datasheets** : « Nouveau projet » idem. **eco-budget** : submit idem.
- Specs alignés (rapport-csrd, datasheets ; eco-budget déjà compatible).

## 3. CI : `.github/workflows/team-docker.yml`

Build du Dockerfile C40 (jamais buildé localement) sur push/PR touchant
deploy/team, l'aggregator ou web-team + **smoke test `/health`** dans le
conteneur. Pas de push registry (viendra après preuve de build vert).

## 4. Plan de commits — `briefs/COMMIT-PLAN-C37-C42.md`

8 commits Conventional Commits avec listes de fichiers exactes générées
depuis le porcelain, fichiers MIXTES signalés (CHANGELOG hunk par hunk),
et 224 fichiers de WIP préexistant explicitement hors périmètre.
⚠ Les 4 anciens specs `web/tests/mX.spec.ts` sont neutralisés (le montage
refusait l'unlink) — `git rm` à faire (commande fournie).

## 5. Vérifications

svelte-check 0 erreur · eslint complet ✓ · prettier ✓ · build ✓ ·
specs : modeles(+redirection)/suivi/datasheets ✓ (4), eco-budget/
rapport-csrd/journal ✓ (3), estimate/comparer/parametres verts en C41 ·
sync checksums 0 divergence.

## 6. Restes

UAT à dérouler · push registry ghcr après build CI vert · gating section
équipe (parametres) à l'occasion · tooltips lexique autres écrans ·
i18n EN post-candidature.
