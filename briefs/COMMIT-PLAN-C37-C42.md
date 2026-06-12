# Plan de commits — session Cowork 2026-06-12 (C37 → C42)

> Généré depuis `git status --porcelain` réel. **Avant tout : relire les
> diffs.** Le montage de session a connu des corruptions (réparées et
> vérifiées par checksums — cf. C39 §4) ; si un diff te semble étrange,
> stoppe et signale-le.

## Ordre proposé (Conventional Commits)

### 1. `docs(adr): ADR-0015 périmètre privacy du Mode Équipe`

```bash
git add "docs/adr/ADR-0015-privacy-mode-equipe.md"
git commit -m "docs(adr): ADR-0015 périmètre privacy du Mode Équipe"
```

### 2. `feat(team-aggregator): k-anonymat, partage opt-in, rétention, CLI config (C38)`

```bash
git add "crates/sobria-team-aggregator/src/commands/config.rs"
git add "crates/sobria-team-aggregator/src/cli.rs"
git add "crates/sobria-team-aggregator/src/commands/mod.rs"
git add "crates/sobria-team-aggregator/src/commands/serve.rs"
git add "crates/sobria-team-aggregator/src/server/api/admin/analytics.rs"
git add "crates/sobria-team-aggregator/src/server/api/me.rs"
git add "crates/sobria-team-aggregator/src/server/api/mod.rs"
git add "crates/sobria-team-aggregator/src/storage/analytics.rs"
git add "crates/sobria-team-aggregator/src/storage/estimations.rs"
git add "crates/sobria-team-aggregator/src/storage/schema.rs"
git add "crates/sobria-team-aggregator/src/storage/users.rs"
git add "crates/sobria-team-aggregator/tests/integration_admin.rs"
git add "docs/operations/team-aggregator.md"
git commit -m "feat(team-aggregator): k-anonymat, partage opt-in, rétention, CLI config (C38)"
```

### 3. `feat(web-team): dashboard privacy-first — participants opt-in, garde k, toggle salarié (C38)`

```bash
git add "web-team/src/lib/api.ts"
git add "web-team/src/routes/admin/dashboard/+page.svelte"
git add "web-team/src/routes/admin/users/+page.svelte"
git add "web-team/src/routes/login/+page.svelte"
git add "web-team/src/routes/user/dashboard/+page.svelte"
git commit -m "feat(web-team): dashboard privacy-first — participants opt-in, garde k, toggle salarié (C38)"
```

### 4. `feat(web): mode démo, rail simplifié, première heure, boucle Réduire, lexique, slugs (C37/C39/C40/C41/C42)`

```bash
git add "web/src/app.d.ts"
git add "web/src/lib/components/DemoBanner.svelte"
git add "web/src/lib/components/ReduceSuggestions.svelte"
git add "web/src/lib/components/Term.svelte"
git add "web/src/lib/demo/"
git add "web/src/lib/lexique.ts"
git add "web/src/routes/datasheets/"
git add "web/src/routes/eco-budget/"
git add "web/src/routes/modeles/"
git add "web/src/routes/suivi/"
git add "web/tests/datasheets.spec.ts"
git add "web/tests/eco-budget.spec.ts"
git add "web/tests/modeles.spec.ts"
git add "web/tests/suivi.spec.ts"
git add "web/.prettierignore"
git add "web/eslint.config.js"
git add "web/package-lock.json"
git add "web/package.json"
git add "web/playwright.config.ts"
git add "web/src/app.css"
git add "web/src/lib/api.ts"
git add "web/src/lib/components/ComingSoon.svelte"
git add "web/src/lib/components/Composer.svelte"
git add "web/src/lib/components/DatacenterPicker.svelte"
git add "web/src/lib/components/EquivalenceCarbon.svelte"
git add "web/src/lib/components/HypothesisBlock.svelte"
git add "web/src/lib/components/ModalitiesPanel.svelte"
git add "web/src/lib/components/ResultBlock.svelte"
git add "web/src/lib/components/m12/CountryDrillDown.svelte"
git add "web/src/lib/components/m12/DatacenterDrillDown.svelte"
git add "web/src/lib/components/m12/DatacenterFilters.svelte"
git add "web/src/lib/components/m12/DatacenterMap.svelte"
git add "web/src/lib/components/m13/DominantLever.svelte"
git add "web/src/lib/components/m13/Forecast.svelte"
git add "web/src/lib/components/m13/LeverPanel.svelte"
git add "web/src/lib/components/m13/Verdict.svelte"
git add "web/src/lib/components/m13/Waterfall.svelte"
git add "web/src/lib/components/m20/RegionDrillDown.svelte"
git add "web/src/lib/components/m20/SankeyChart.svelte"
git add "web/src/lib/components/m20/SiteDrillDown.svelte"
git add "web/src/lib/components/m20/TerritoireFilters.svelte"
git add "web/src/lib/components/m20/TerritoireMap.svelte"
git add "web/src/lib/components/m9/ModelCard.svelte"
git add "web/src/lib/components/m9/ModelDetailDrawer.svelte"
git add "web/src/lib/components/m9/ModelFilters.svelte"
git add "web/src/lib/preferences.ts"
git add "web/src/lib/team-store.ts"
git add "web/src/routes/+layout.svelte"
git add "web/src/routes/+page.svelte"
git add "web/src/routes/a-propos/+page.svelte"
git add "web/src/routes/comparer/+page.svelte"
git add "web/src/routes/datacenters/+page.svelte"
git add "web/src/routes/journal/+page.svelte"
git add "web/src/routes/m15/+page.svelte"
git add "web/src/routes/m17/+page.svelte"
git add "web/src/routes/m25/+page.svelte"
git add "web/src/routes/m9/+page.svelte"
git add "web/src/routes/methodo/+page.svelte"
git add "web/src/routes/methodologies/+page.svelte"
git add "web/src/routes/onboarding/+page.svelte"
git add "web/src/routes/parametres/+page.svelte"
git add "web/src/routes/rapport-csrd/+page.svelte"
git add "web/src/routes/simuler/+page.svelte"
git add "web/src/routes/territoire/+page.svelte"
git add "web/tests/a-propos.spec.ts"
git add "web/tests/comparer.spec.ts"
git add "web/tests/datacenter-picker.spec.ts"
git add "web/tests/datacenters-immersive.spec.ts"
git add "web/tests/datacenters.spec.ts"
git add "web/tests/estimate.spec.ts"
git add "web/tests/journal.spec.ts"
git add "web/tests/m15.spec.ts"
git add "web/tests/m17.spec.ts"
git add "web/tests/m25.spec.ts"
git add "web/tests/m9.spec.ts"
git add "web/tests/onboarding.spec.ts"
git add "web/tests/parametres-mode-equipe.spec.ts"
git add "web/tests/parametres.spec.ts"
git add "web/tests/rapport-csrd.spec.ts"
git add "web/tests/simuler.spec.ts"
git add "web/tests/territoire.spec.ts"
git add "web/vite.config.ts"
git commit -m "feat(web): mode démo, rail simplifié, première heure, boucle Réduire, lexique, slugs (C37/C39/C40/C41/C42)"
```

### 5. `build(deploy): kit Docker Mode Équipe + kits UAT/communication (C40)`

```bash
git add "deploy/"
git add "docs/operations/deploiement-equipe.md"
git add "docs/operations/modeles-communication.md"
git add "docs/qa/uat/"
git commit -m "build(deploy): kit Docker Mode Équipe + kits UAT/communication (C40)"
```

### 6. `ci: build CI de l'image équipe + smoke /health (C42)`

```bash
git add ".github/workflows/team-docker.yml"
git commit -m "ci: build CI de l'image équipe + smoke /health (C42)"
```

### 7. `chore(tools): générateur de fixtures démo depuis le moteur (C37)`

```bash
git add "tools/"
git commit -m "chore(tools): générateur de fixtures démo depuis le moteur (C37)"
```

### 8. `docs: briefs C37→C42`

```bash
git add "briefs/chantiers/C37-mode-demo-web.md"
git add "briefs/chantiers/C38-dashboard-equipe-privacy.md"
git add "briefs/chantiers/C39-mode-simple-rail.md"
git add "briefs/chantiers/C40-sprint-utilisabilite.md"
git add "briefs/chantiers/C41-finitions-utilisabilite.md"
git commit -m "docs: briefs C37→C42"
```

### 9. Suppressions à finaliser (le montage ne pouvait pas)

```bash
git rm web/tests/m9.spec.ts web/tests/m15.spec.ts web/tests/m17.spec.ts web/tests/m25.spec.ts
git commit -m "test(web): retire les specs des routes renommées (C42)"
```

## ⚠ Fichiers MIXTES (mes changements + ton WIP préexistant)

- `CHANGELOG.md` — j'y ai ajouté les sections C37→C42 sous `[Non publié]`,
  mais il était déjà modifié avant la session. **Relis hunk par hunk**
  (`git add -p CHANGELOG.md`) et range mes sections avec le commit docs.
- `web/package.json` + `web/package-lock.json` — ajout `leaflet` (C37) ;
  inclus dans le commit web.

## Préexistant — ton WIP, à trier par toi (224 fichiers)

Aucun de ces fichiers n'a été modifié pendant la session (sauf mention
ci-dessus). Échantillon par zone :

- `crates/` : 93 fichier(s)
- `sobr-ia-design-system/` : 43 fichier(s)
- `extension/` : 29 fichier(s)
- `web-team/` : 21 fichier(s)
- `briefs/` : 12 fichier(s)
- `docs/` : 10 fichier(s)
- `site/` : 9 fichier(s)
- `.github/` : 4 fichier(s)
- `CHANGELOG.md/` : 1 fichier(s)
- `Cargo.toml/` : 1 fichier(s)
- `scripts/` : 1 fichier(s)

Liste complète : `git status --porcelain` en filtrant les chemins des
commits 1-9 ci-dessus.
