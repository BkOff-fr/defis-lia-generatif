# Changelog — site Sobr.ia

Toutes les modifications notables du site `sobria.brilliantstudio.co`. Versionné
indépendamment du produit (tags `site-vX.Y.Z`).

Format : [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/).

## [Unreleased]

### Added — C33.1 Bootstrap (2026-05-17)

- Structure Astro 5 + Svelte 5 (runes) + Tailwind 4 (`@tailwindcss/vite`).
- Output static (`output: 'static'`) pour déploiement rsync nginx self-hosted.
- Design tokens Sobr.ia v2 (ink/ivory/lime + Instrument Serif/Geist/JetBrains Mono).
- Fonts auto-hébergées (pas de Google Fonts, privacy by design CLAUDE.md §7).
- Composants UI primitifs : Button, Card, Badge (Svelte 5 runes).
- Topbar sticky + Footer 4 colonnes.
- Homepage avec 8 sections numérotées (placeholders pour C33.2/C33.3/C33.5).
- Squelettes tests Playwright a11y + lighthouse.
- README + ce CHANGELOG indépendants du root.

[Unreleased]: https://github.com/anthropics/defis-lia-generatif/compare/...HEAD
