# Plan de commits — addendum C43 → C45 (session Cowork 2026-06-12)

> Complète `COMMIT-PLAN-C37-C42.md` (à dérouler AVANT celui-ci).
> Listes établies à la main (environnement git indisponible en fin de
> session) : vérifie avec `git status` que rien ne manque, et lance les
> **vérifications C44 §5 / C45 §5 avant de commiter** ces lots.

## 9. `feat(ext): popup 3 niveaux + indicateurs apaisés (C43)`

```bash
git add extension/src/popup/index.html extension/src/popup/main.ts extension/src/popup/popup.css
git add extension/src/content/shared/badge-injector.ts extension/src/content/shared/composer-indicator.ts
git add extension/tests/unit/popup-helpers.spec.ts extension/tests/unit/badge-injector.spec.ts
```

## 10. `feat(site): refonte accueil + structure produit/equipe/methode (C43+C44)`

```bash
git add site/src/pages/index.astro site/src/pages/produit.astro site/src/pages/equipe.astro
git add site/src/pages/methode.astro site/src/pages/telecharger.astro site/src/pages/cloud.astro
git add site/src/components/Topbar.astro site/src/components/Footer.astro
git add site/src/components/sections/DownloadSection.svelte site/src/components/sections/Entreprises.astro
```

## 11. `feat(team-aggregator): politique de visibilité + dimension projet (C44, ADR-0016)`

```bash
git add crates/sobria-team-aggregator/src/policy.rs crates/sobria-team-aggregator/src/lib.rs
git add crates/sobria-team-aggregator/src/storage/schema.rs crates/sobria-team-aggregator/src/storage/estimations.rs
git add crates/sobria-team-aggregator/src/storage/analytics.rs crates/sobria-team-aggregator/src/storage/users.rs
git add crates/sobria-team-aggregator/src/server/api/estimations.rs crates/sobria-team-aggregator/src/server/api/me.rs
git add crates/sobria-team-aggregator/src/server/api/admin/analytics.rs crates/sobria-team-aggregator/src/server/api/admin/users.rs
git add crates/sobria-team-aggregator/src/server/api/admin/user_detail.rs crates/sobria-team-aggregator/src/server/api/admin/mod.rs
git add crates/sobria-team-aggregator/src/commands/config.rs crates/sobria-team-aggregator/src/cli.rs
git add docs/operations/team-aggregator.md
```

## 12. `feat(web-team): badge politique, projets, détail employé (C44)`

```bash
git add web-team/src/routes/admin/dashboard/+page.svelte
git add "web-team/src/routes/admin/users/[id]/+page.svelte" "web-team/src/routes/admin/users/[id]/+page.ts"
git add web-team/src/routes/admin/users/+page.svelte web-team/src/routes/user/dashboard/+page.svelte
```

## 13. `feat(ext): projet par conversation (C44)`

```bash
git add extension/src/content/shared/projects.ts extension/src/lib/team-client.ts
git add extension/src/background/service-worker.ts
git add extension/src/popup/index.html extension/src/popup/main.ts extension/src/popup/popup.css
git add extension/tests/unit/projects.spec.ts
```
> NB : les fichiers popup portent C43 **et** C44 — si tu veux des
> commits purs, fais le commit 9 d'abord puis celui-ci capture le delta.

## 14. `feat(site): manifeste immersif « Le poids invisible » (C45)`

```bash
git add site/src/pages/manifeste.astro site/src/pages/index.astro site/src/components/Footer.astro
```

## 15. `docs(adr): ADR-0016 politique de visibilité + ADR-0017 contrat démo`

```bash
git add docs/adr/ADR-0016-politique-visibilite-deploiement.md docs/adr/ADR-0017-contrat-demo-web.md
```

## 16. `docs: briefs C43-C45 + plans de commits + CHANGELOG (hunks session)`

```bash
git add briefs/chantiers/C43-vitrine-extension-site.md briefs/chantiers/C44-politique-projets-site.md
git add briefs/chantiers/C45-manifeste-immersif.md briefs/COMMIT-PLAN-C43-C45.md
git add -p CHANGELOG.md   # hunks C43/C44/C45 uniquement (WIP préexistant mêlé)
```

## ⚠ Rappels

1. **Vérifications AVANT commit des lots 11-14** (jamais lancées, env
   mort) : `cargo test -p sobria-team-aggregator` + clippy ·
   `web-team: npm run check && build` · `extension: npm test && build` ·
   `site: npx astro build` (+ `/manifeste` à l'œil).
2. `extension/src/content/shared/projects.ts` a reçu une retouche
   locale (réécriture immutable de `setProjectForThread`) — conserve-la.
3. Le site embarque `site/src/content/` généré (gitignoré normalement —
   ne pas committer s'il apparaît).
