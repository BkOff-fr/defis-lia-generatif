# Changelog — site Sobr.ia

Toutes les modifications notables du site `sobria.brilliantstudio.co`. Versionné
indépendamment du produit (tags `site-vX.Y.Z`).

Format : [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/).

## [Unreleased]

## [0.1.0] — 2026-05-17

Premier site internet Sobr.ia (chantier C33), déployé sur
`https://sobria.brilliantstudio.co/` via rsync SSH self-hosted nginx +
Let's Encrypt. Astro 5 static + Svelte 5 islands + Tailwind 4 +
Three.js + Pagefind. 8 sections landing + doc interactive (45 pages
indexées Pagefind) + 3 pages secondaires (cloud, télécharger,
candidature). Zéro tracking, fonts auto-hébergées, prefers-reduced-motion
respecté, WebGL fallback.

### Added — C33.7 Polish + tag site-v0.1.0 (2026-05-17)

- OG image `/og-image.png` 1200×630 (96 KB, generated via sharp from
  inline SVG), Twitter card `summary_large_image`, locale `fr_FR`,
  og:site_name.
- BaseLayout enrichi : og:image, og:image:width/height, og:locale,
  og:site_name, twitter:card/title/description/image.
- Astro `@astrojs/sitemap` activé → `/sitemap-index.xml` généré au build,
  référencé depuis `public/robots.txt`.
- Script `scripts/generate-og.mjs` pour régénérer l'OG image (à lancer
  manuellement si le brand ou la headline change).
- Smoke test cross-browser : prefers-reduced-motion + WebGL absent →
  fallback SVG poster du globe; security headers `https://sobria.brilliantstudio.co/`
  vérifiés (HSTS, X-Frame-Options DENY, CSP, etc.).
- Tag `site-v0.1.0` poussé.

### Added — C33.6 CI/CD self-hosted nginx (2026-05-17)

- Serveur Thibault (80.11.20.55, Ubuntu 22.04, nginx 1.18) provisionné :
  - User `deployer` créé (UID 1007, group www-data).
  - Sudoers `/etc/sudoers.d/deployer` limité à `systemctl reload/restart/status
nginx` + `nginx -t` (paths `/bin/systemctl` ET `/usr/bin/systemctl` pour
    couvrir les deux binaires distincts sur Ubuntu 22).
  - Site dir `/var/www/sobria-site` perms `deployer:www-data 750`.
  - Clé SSH ed25519 dédiée `sobria_deploy` (générée localement, pub poussée
    dans `/home/deployer/.ssh/authorized_keys`).
  - Nginx server block `sobria.brilliantstudio.co` avec HTTP→HTTPS 301,
    HTTP/2, security headers serveur-level (HSTS, X-Frame-Options DENY,
    Referrer-Policy, Permissions-Policy, CSP self-only), gzip, cache
    immutable `/_astro/` et fonts (1 an), images 30j, html 5min.
  - Certbot Let's Encrypt OK (cert expire 2026-08-15), email
    `thibault@brilliantstudio.co`, auto-renewal via `certbot.timer` (daily).
- Workflow `.github/workflows/site-deploy.yml` : push main paths
  `site/**` + `docs/**` + script sync → build Astro + sync docs + rsync SSH
  vers `deployer@80.11.20.55:/var/www/sobria-site/` + smoke test curl HTTPS.
- Premier deploy manuel via `scp -r site/dist/*` validé : site live sur
  https://sobria.brilliantstudio.co/ avec headers sécurité, /docs/, /pagefind/,
  /adrs/, /telecharger/, /cloud/, /candidature/ tous opérationnels.

5 GitHub Secrets à configurer (UI repo → Settings → Secrets → Actions) avant
que le workflow puisse rsync depuis CI :

- `SOBRIA_DEPLOY_SSH_KEY` : contenu de `~/.ssh/sobria_deploy` (private ed25519)
- `SOBRIA_DEPLOY_HOST` : `80.11.20.55`
- `SOBRIA_DEPLOY_USER` : `deployer`
- `SOBRIA_DEPLOY_PATH` : `/var/www/sobria-site/`
- `SOBRIA_DEPLOY_KNOWN_HOSTS` : sortie `ssh-keyscan -t ed25519 80.11.20.55`

### Added — C33.5 Workflows release + #download + pages secondaires (2026-05-17)

- `.github/workflows/app-release.yml` : trigger sur tag `v*.*.*` + workflow_dispatch.
  Matrix 4 jobs (Windows / macOS ARM / macOS Intel macos-13 / Linux Ubuntu 22.04),
  installe Tauri CLI 2.x, build web frontend puis `cargo tauri build`, collecte
  bundles (.msi/.exe/.dmg/.deb/.AppImage), calcule SHA-256, upload sur GitHub
  Release via softprops/action-gh-release@v2. Cache cargo via Swatinem/rust-cache.
- `.github/workflows/extension-release.yml` : trigger identique. Build Chrome
  zip et Firefox xpi via scripts existants extension/, upload + SHA-256 sur la release.
- DownloadSection.svelte : section `#download` avec auto-détection OS
  (`navigator.userAgent` + `userAgentData.platform`), 7 cards (Win / macOS ARM /
  macOS Intel / Linux / Chrome / Firefox / Android+iOS "Bientôt"), highlight
  carte recommandée + badge lime.
- `/telecharger/` : page détaillée avec section vérification SHA-256 (commandes
  Win/macOS/Linux), avertissements OS (Gatekeeper / SmartScreen / chmod), code
  source + reproductibilité.
- `/cloud/` : narratif ADR-0014 dual-track. Mode Équipe self-hosted (Disponible)
  vs Cloud managé v1.3 (Bientôt). Bouton mailto:contact@sobria.brilliantstudio.co
  pour opt-in (zéro service tiers, RGPD-friendly par absence).
- `/candidature/` : statut candidature data.gouv.fr (v0.8.0 → v1.0 cible 2026-Q4),
  4 ADRs clés (0009/0012/0013/0014) avec résumés, liens audit sources +
  catalogue datacenters + repo GitHub + cahier des charges.
- index.astro : placeholder #download remplacé par DownloadSection ; 8/8 sections
  désormais wirées.
- Build : 45 pages (+3), Pagefind 7578 mots fr.

Note : le trigger workflow_dispatch sur tag v0.7.1 reste à faire côté
Thibault depuis l'UI GitHub Actions (ou `gh workflow run app-release.yml
--ref v0.7.1`) pour rétro-publier les binaires sur la release v0.7.1
existante. Les URLs `releases/latest/download/...` deviendront alors
opérationnelles.

### Added — C33.3 4 sections 3D animées (2026-05-17)

- MonteCarloViz.svelte : Three.js InstancedMesh 10 000 sphères, distribution
  gaussienne autour d'axe X avec convergence animée au scroll-entry (IO),
  toggle AFNOR/Sobr.ia vs EcoLogits (lerp positions cibles via `$effect`),
  P5/P50/P95 stats live + bandes plan ambre P5/P95 dans la scène.
  Reuses Three.js chunk lazy-loaded depuis HeroGlobe (~12 KB de code propre).
- VendorDisclosure.astro : table 5 vendors (Mistral×ADEME 🇫🇷 / Google Gemini /
  Meta Llama 3.x / Anthropic / OpenAI) avec chiffres clés (1,14 g · 0,03 g ·
  11 390 t), badges ✓/~/✕ par disclosure prompt-level + training, révélation
  séquentielle au scroll via IntersectionObserver.
- TerritoireFR.astro : carte FR SVG outline stylisé + datacenters FR filtrés
  depuis datacenters.json (projection lat/lon → viewBox), CSS perspective +
  rotateX 28° pour effet 2.5D, hover → unfold partiel, prefers-reduced-motion
  désactive le tilt.
- Entreprises.astro : 2 cards (Mode Équipe self-hosted dispo / Cloud managé
  v1.3 bientôt) avec SVG inline schéma serveur + 4 avatars employés
  stylisés, CTAs vers /docs/operations/team-aggregator/ et /cloud/.
- index.astro : sections 5/6/7/8 wirées ; seul `#download` (4) reste
  placeholder pour C33.5.

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
