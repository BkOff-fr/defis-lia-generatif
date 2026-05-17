# Sobr.ia — site internet (sobria.brilliantstudio.co)

Site Astro 5 + Svelte 5 + Tailwind 4 + Three.js + Motion. Statique, déployé
via rsync SSH sur nginx self-hosted (cf. [`../docs/operations/site-deploy.md`](../docs/operations/site-deploy.md)).

## Prérequis

- Node ≥ 22
- npm ≥ 10

## Setup local

```bash
cd site
npm install
npm run dev          # http://localhost:4321
```

## Scripts

| Commande            | Action                                      |
| ------------------- | ------------------------------------------- |
| `npm run dev`       | Serveur de développement (port 4321)        |
| `npm run build`     | Build statique → `site/dist/`               |
| `npm run preview`   | Sert le build statique localement           |
| `npm run check`     | TypeScript check via `astro check`          |
| `npm run lint`      | Prettier + ESLint                           |
| `npm run format`    | Prettier --write                            |
| `npm run test:a11y` | Playwright + axe-core                       |
| `npm run test:lh`   | Smoke-test homepage (Lighthouse réel en CI) |

## Structure

Cf. [`../briefs/chantiers/C33-site-internet.md`](../briefs/chantiers/C33-site-internet.md) §2 pour l'architecture complète.

```
site/
├── src/
│   ├── pages/             # Routes Astro
│   ├── components/        # Astro + Svelte islands
│   ├── layouts/           # BaseLayout
│   ├── styles/global.css  # Tokens v2 + reset + reduced-motion
│   └── lib/               # (à venir : motion.ts, three-utils.ts, ...)
├── public/
│   ├── fonts/             # Geist / Instrument Serif / JetBrains Mono
│   ├── favicon.svg
│   └── robots.txt
└── tests/                 # Playwright a11y + lighthouse
```

## Déploiement

Push sur `main` → workflow `.github/workflows/site-deploy.yml` (wiré en C33.6)
build Astro et rsync sur `sobria.brilliantstudio.co`. Voir
[`../docs/operations/site-deploy.md`](../docs/operations/site-deploy.md) pour
le provisioning serveur et les secrets GitHub Actions.

## Conventions

- Privacy by design : aucun tracking, aucune Google Font, aucun analytics tiers.
- Frugalité : bundle initial ≤ 100 KB gzip, bundle 3D ≤ 300 KB gzip lazy-loaded.
- Accessibilité : `prefers-reduced-motion` respecté, WebGL fallback, axe-core en CI.
- Versionnage indépendant : `site/package.json` + tag `site-vX.Y.Z` (pas de bump app).

## Brief & doc

- Brief chantier : [`../briefs/chantiers/C33-site-internet.md`](../briefs/chantiers/C33-site-internet.md)
- Deploy ops : [`../docs/operations/site-deploy.md`](../docs/operations/site-deploy.md)
- ADR architecture : [`../docs/adr/ADR-0014-dual-track-local-cloud.md`](../docs/adr/ADR-0014-dual-track-local-cloud.md)
- Plan d'implémentation C33.1 : [`../docs/superpowers/plans/2026-05-17-c33.1-site-bootstrap.md`](../docs/superpowers/plans/2026-05-17-c33.1-site-bootstrap.md)
