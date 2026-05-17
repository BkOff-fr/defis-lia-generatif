# Changelog — site Sobr.ia

Toutes les modifications notables du site `sobria.brilliantstudio.co`. Versionné
indépendamment du produit (tags `site-vX.Y.Z`).

Format : [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/).

## [Unreleased]

### Added — C33.4 Doc interactive + Pagefind (2026-05-17)

- Astro 5 Content Layer : 2 collections `docs` (24 entries) + `adrs` (15 entries),
  glob loader sur `src/content/{docs,adrs}/`, schemas Zod tous fields optionnels.
- `scripts/sync-docs.mjs` : sync cross-platform Node (Windows + Linux),
  exécuté en `prebuild` + `predev`. Injecte sourcePath front-matter sans
  casser les MD existants (aucun n'avait de front-matter — fallback first H1).
- DocLayout : sidebar tree groupée par dossier, breadcrumb, footer
  « Éditer sur GitHub » avec URL `https://github.com/<repo>/edit/main/<path>`.
- Routes générées : `/docs/`, `/docs/<slug>`, `/adrs/`, `/adrs/<slug>`
  (42 pages totales avec landing).
- CSS `prose-doc` global : h2 souligné, code blocks ink-3, blockquote
  bordure lime, tables, listes, alignés design system v2.
- Pagefind : `postbuild` indexe `dist/`, 7491 mots français.
- SearchBar Svelte 5 : lazy-load Pagefind UI via script tag dynamique
  (évite résolution Vite au build), raccourci clavier `/` ou `⌘K`,
  traductions FR complètes, palette lime/ink intégrée via CSS variables.
- Topbar : SearchBar injectée, GitHub button masqué sur mobile pour laisser
  la place. URL GitHub corrigée (BkOff-fr).

### Added — C33.2 Hero 3D + sections statiques (2026-05-17)

- HeroGlobe.svelte : globe Three.js procédural (wireframe lime/ink + 28
  datacenters pulsants + 1000 particules ambre) chargé en dynamic import
  (lazy `client:visible`), avec auto-orbit + drag pointer.
- WebGL detection + prefers-reduced-motion respect : fallback SVG poster
  inline (GlobeFallback.astro) servi côté serveur, zéro JS si pas de WebGL.
- Hero.astro : headline « L'empreinte de vos prompts IA, mesurée. »,
  sous-headline « En local. Sans inscription. Sans envoyer un mot à nos
  serveurs. », 2 CTAs (Télécharger #download / Voir le code GitHub),
  proof line AFNOR + EcoLogits + data.gouv.fr.
- PourQui.astro : 5 PersonaCard avec icônes Lucide (GraduationCap, Code2,
  Building2, Landmark, Microscope), taglines fidèles au README post-C32.
- CommentCaMarche.astro : 3 étapes avec icônes Lucide (Download, Type,
  Activity), fade-in séquentiel via IntersectionObserver natif.
- Footer enrichi : badge candidature data.gouv.fr lime + rangée ADRs
  cliquables (0009 médaillon, 0012 multi-méthodo, 0013 pairing équipe,
  0014 dual-track).
- Util libs : `src/lib/three-utils.ts` (latLonToVec3, hasWebGL,
  prefersReducedMotion) + `src/lib/viewport.ts` (observeOnce).
- Données : 28 datacenters extraits de
  `crates/sobria-geoloc/data/datacenters_demo.json` vers
  `site/src/data/datacenters.json` (id, name, country, lat, lon).
- Bundles vérifiés : initial JS = 18 KB gzip (≤ 100 KB budget),
  bundle 3D lazy = 181 KB gzip (≤ 300 KB budget).

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
