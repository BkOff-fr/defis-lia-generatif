# Chantier C33 — Site internet (sobria.brilliantstudio.co)

> **Version cible** : `site-v0.1.0` (release web séparée, pas de bump app)
> **Sprint** : 5-7 jours focalisés design + 3D + animations + doc interactive
> **Pré-requis** : v0.8.0 shippée (C32 Clarté produit), wordings consolidés
> **Output** : site déployé sur serveur Thibault (Ubuntu + nginx self-hosted), accessible HTTPS sur `https://sobria.brilliantstudio.co`
> **Lien** : ADR-0014 (dual-track local + cloud opt-in v1.3+) + `docs/operations/site-deploy.md` (nouveau)

---

## 0. Pourquoi ce chantier maintenant

L'audit produit C32.0 a montré que le pitch externe manque autant que le pitch interne. La candidature data.gouv.fr v1.0 sera servie infiniment mieux par un **site dédié haute qualité visuelle** plutôt que par un repo GitHub austère. Le jury va chercher "Sobr.ia" sur Google — il doit tomber sur un truc qui le frappe en 5 secondes.

Couplé avec C32 (qui aligne le produit lui-même), le site est l'**autre face de la médaille** : la même clarté produit, projetée à l'extérieur, avec un wow factor visuel.

Et au-delà de la candidature, le site est aussi le **point d'entrée canonique** pour :
- Particuliers curieux qui découvrent Sobr.ia.
- Entreprises qui cherchent à comprendre le Mode Équipe et la future offre managed.
- Chercheurs qui veulent citer Sobr.ia (DOI Zenodo, BibTeX).
- Contributeurs potentiels (GitHub, ADRs).

---

## 1. Périmètre

### En périmètre — site `site-v0.1.0`

- Site Astro 5 dans `/site/` du monorepo (single source of truth `docs/`).
- Build statique (`adapter-static`), zéro runtime serveur, déployé via rsync SSH sur nginx existant.
- **Hero animé 3D Three.js** : globe Terre avec datacenters illuminés + particules carbone montant.
- **7 sections** scrollables avec animations Motion One :
  1. Hero 3D + value proposition + 2 CTAs (Télécharger → ancre `#download` / Voir la démo).
  2. "Pour qui ?" — 5 cartes persona (réutilise wordings C32).
  3. "Comment ça marche ?" — 3 étapes animées (install → prompt → journal).
  4. **"Télécharger Sobr.ia" (`#download`)** — bloc dédié avec auto-détection OS + 7 boutons (Windows / macOS / Linux / Chrome / Firefox / Android (bientôt) / iOS (bientôt)).
  5. "Méthodologies" — viz Monte-Carlo 3D 10⁴ tirages.
  6. "Vendors disclosure" — table 5 vendors animée (Mistral × ADEME, Google, Meta, Anthropic ❌, OpenAI ❌).
  7. "Territoire FR" — carte IRIS 3D ou 2.5D (différenciateur défi data.gouv).
  8. "Pour les entreprises" — Mode Équipe self-hosted + Cloud managé "Coming soon" + CTA mailing list.
  9. Footer (ADRs, licences, candidature, contributeurs).
- **Topbar** sticky : Logo · Pour qui · Doc · Télécharger · GitHub · ⭐ (call to star).
- **Doc interactive `/docs/*`** : Astro Content Collections qui lit `docs/*.md` du repo principal, full-text search via Pagefind.
- **Page `/cloud/` placeholder** : "Mode équipe self-hosted dispo aujourd'hui · Cloud managé bientôt" + form mailing list (Buttondown ou Resend, opt-in RGPD).
- **Page `/telecharger/`** : liens vers binaires Win/macOS/Linux (GitHub Releases) + extension Chrome/Firefox + bridge natif.
- **Page `/candidature/`** : statut candidature data.gouv.fr + sources audit.
- **CI/CD** : `.github/workflows/site-deploy.yml` — push sur main → build Astro → rsync sur serveur Thibault → reload nginx (si config change).
- **Domaine** : `sobria.brilliantstudio.co` (sous-domaine sur domaine studio existant). DNS géré côté brilliantstudio.co (record A vers `80.11.20.55`).
- **Performance** :
  - Lighthouse ≥ 90 sur les 4 catégories.
  - Bundle initial ≤ 100 KB gzip (sans 3D).
  - Bundle 3D lazy-loaded ≤ 300 KB gzip, chargé en Intersection Observer.
  - Pas de tracking / pas de Google Fonts / pas d'analytics tiers (cohérent CLAUDE.md §7).
- **Accessibilité** :
  - Respect `prefers-reduced-motion` : 3D désactivée, animations remplacées par fade-in léger.
  - WebGL fallback : si pas de WebGL, image statique du globe en fallback.
  - Contraste AA WCAG 2.2, navigation clavier, alt sur tout.
  - Tested with axe-core en CI.

### Hors périmètre

- App in-browser de l'app Tauri (juste un lien Télécharger).
- Marketing automation / CRM (juste un email opt-in basique).
- Analytics tiers (Google Analytics, Plausible…) — pas en v0.1.0, peut-être en v1.0 avec **Plausible self-hosted** uniquement.
- Multi-langue EN/FR : FR-only en v0.1.0, EN en v0.2.0 site.
- Privacy policy détaillée RGPD légale : page placeholder, à compléter avant cloud beta (v1.3+).

---

## 2. Architecture

```
site/                                          # nouveau dossier monorepo
├── package.json                               # version "0.1.0"
├── astro.config.mjs                          # adapter cloudflare ou static
├── tsconfig.json
├── tailwind.config.mjs                       # palette Sobr.ia (lime/ambre/coral)
├── public/
│   ├── icons/
│   ├── og-image.png                          # OpenGraph card 1200x630
│   └── robots.txt
├── src/
│   ├── pages/
│   │   ├── index.astro                       # Landing principale
│   │   ├── cloud.astro                       # Cloud "coming soon"
│   │   ├── telecharger.astro                 # Liens binaires
│   │   ├── candidature.astro                 # Statut candidature
│   │   └── docs/
│   │       ├── [...slug].astro               # Génère pages docs depuis md
│   │       └── index.astro                   # Sommaire doc
│   ├── content/
│   │   ├── config.ts                         # Content Collections schema
│   │   ├── docs/                             # symlink ou copie de ../docs/
│   │   └── adrs/                             # symlink ou copie de ../docs/adr/
│   ├── components/
│   │   ├── Topbar.astro                      # Header sticky
│   │   ├── Hero.svelte                       # 3D globe + value prop
│   │   ├── three/
│   │   │   ├── HeroGlobe.svelte              # Three.js globe + datacenters
│   │   │   ├── MonteCarloViz.svelte          # 10⁴ tirages animés
│   │   │   ├── IrisMap3D.svelte              # Carte IRIS 2.5D
│   │   │   └── VendorCards3D.svelte          # 5 cartes 3D vendeurs
│   │   ├── sections/
│   │   │   ├── PourQui.svelte                # 5 personas cards
│   │   │   ├── CommentCaMarche.svelte        # 3 étapes animées
│   │   │   ├── Methodologies.svelte          # viz + table
│   │   │   ├── VendorDisclosure.svelte       # table 5 vendors
│   │   │   ├── TerritoireFR.svelte           # carte IRIS
│   │   │   ├── Entreprises.svelte            # Mode Équipe + Cloud teaser
│   │   │   └── Footer.astro                  # ADRs + licences + candidature
│   │   └── ui/
│   │       ├── Button.svelte
│   │       ├── PersonaCard.svelte
│   │       ├── CodeBlock.svelte
│   │       └── EquivalenceCarbon.svelte      # réutilisé de C32.3
│   ├── lib/
│   │   ├── motion.ts                         # Helpers GSAP / Motion One
│   │   ├── three-utils.ts                    # Helpers Three.js (lazy load, RAF)
│   │   └── reduced-motion.ts                 # Respect prefers-reduced-motion
│   └── styles/
│       └── global.css                        # design system palette
└── tests/
    ├── lighthouse.spec.ts                    # CI Lighthouse ≥ 90
    └── a11y.spec.ts                          # axe-core
```

### Choix techniques détaillés

| Aspect | Choix | Rationale |
|---|---|---|
| Framework | Astro 5 | SSG fastest, support Svelte islands, MDX, Content Collections natif |
| UI islands | Svelte 5 runes | Cohérence stack Sobr.ia (déjà utilisé dans web/ + extension + web-team) |
| Styling | Tailwind 4 | Pas de SCSS, palette Sobr.ia portée |
| 3D | Three.js raw r170+ | Pas de wrapper (Threlte non nécessaire), contrôle fin perf |
| Anim scroll | **Motion One** (verrouillé) | Lightweight (~3 KB gzip), ScrollTrigger éq., API simple, perf top |
| Doc full-text search | Pagefind | Static, 0 dépendance serveur, multilingue ready |
| Hébergement | **Self-hosted nginx** sur serveur Thibault (Ubuntu 22.04+, 80.11.20.55) | Souverain, gratuit, cohérent pitch CLAUDE.md §7 + ADR-0014 (pas de cloud Sobr.ia central) |
| Déploiement | **rsync SSH** depuis GitHub Action vers user dédié `deployer` | Simple, sécurisé, idempotent |
| Domaine | `sobria.brilliantstudio.co` | Sous-domaine domaine studio existant. Record A → 80.11.20.55 |
| HTTPS | **Let's Encrypt** via certbot (renouvellement auto) | Gratuit, standard, automatique |
| Email opt-in | Form mailto simple en v0.1.0 | Pas de service tiers, RGPD friendly by absence |

### Section Télécharger — auto-détection OS + 7 plateformes

**Composant** : `<DownloadSection />` Svelte 5, placé sur la landing à l'ancre `#download`.

**Logique de détection** :

- Au mount, lecture `navigator.userAgent` + (si dispo) `navigator.userAgentData.platform`.
- Détection OS et highlight automatique du bon bouton :

```ts
function detectOs(): 'windows' | 'macos' | 'linux' | 'android' | 'ios' | 'unknown' {
  const ua = navigator.userAgent.toLowerCase();
  const platform = (navigator as any).userAgentData?.platform?.toLowerCase() ?? '';
  if (platform.includes('android') || ua.includes('android')) return 'android';
  if (/iphone|ipad|ipod/.test(ua)) return 'ios';
  if (platform === 'windows' || /win(dows)?/.test(ua)) return 'windows';
  if (platform === 'macos' || /mac os|macintosh/.test(ua)) return 'macos';
  if (platform === 'linux' || /linux|x11/.test(ua)) return 'linux';
  return 'unknown';
}
```

- L'OS détecté → carte mise en avant (bordure lime, badge "Recommandé pour vous").
- Tous les autres téléchargements restent visibles, juste dépriorisés visuellement.
- Si JS désactivé : tous les boutons identiques, ordering OS standard (Windows / macOS / Linux / Chrome / Firefox / mobiles).

**Sources des binaires** : GitHub Releases `latest`. Pattern URL stable :

```
https://github.com/<owner>/defis-lia-generatif/releases/latest/download/<filename>
```

GitHub redirige automatiquement vers la dernière release tagguée, donc 0 maintenance manuelle. Filenames attendus (à confirmer côté CI Tauri build qui doit déjà exister) :

| Plateforme | Filename probable | Source |
|---|---|---|
| **Windows** | `Sobr.ia_<version>_x64-setup.exe` (installer) ou `_x64_en-US.msi` | Tauri bundle `nsis` ou `msi` |
| **macOS** | `Sobr.ia_<version>_x64.dmg` (Intel) + `_aarch64.dmg` (Apple Silicon) | Tauri bundle `dmg` |
| **Linux** | `sobr-ia_<version>_amd64.deb` + `.AppImage` | Tauri bundle `deb` + `appimage` |
| **Chrome** | `sobria-extension-chrome-v<version>.zip` | Extension build (existe déjà) |
| **Firefox** | `sobria-extension-firefox-v<version>.xpi` | Extension build (existe déjà) |
| **Android** | `sobr-ia_<version>.apk` | Tauri mobile (uniquement tag major, cf. §"Stratégie mobile") |
| **iOS** | À voir App Store / TestFlight | Différé v1.x (cf. §"Stratégie mobile") |

**Affichage** :

- Pour chaque téléchargement disponible : bouton + taille fichier (récupérée au build via GitHub API, mise en cache) + lien "Vérifier SHA-256" qui ouvre un panneau avec le hash + commande `shasum -a 256 <file>` pour vérification.
- Pour Android/iOS non encore buildés : carte avec badge "Bientôt disponible" sans CTA, ou simple opt-in vers la mailing list `/cloud/`.

**Stratégie mobile (Android + iOS)** :

- **Pas de build Android/iOS dans le CI standard.** Cela alourdit beaucoup le pipeline (signing keys, store-ready bundles, etc.) et n'apporte pas de valeur sur chaque tag.
- **Build Android uniquement sur tag major** (v1.0, v1.1, v2.0…) :
  - Trigger : tag matching `v[0-9]+.[0-9]+.0` (zéro patch = major/minor sans patch).
  - Workflow `.github/workflows/mobile-android-release.yml` à créer (différé chantier ultérieur).
  - Signing key Android stockée dans GitHub Secrets (`ANDROID_KEYSTORE_BASE64`).
- **iOS différé v1.x au plus tôt** : nécessite Apple Developer Program ($99/an), TestFlight ou App Store, codesign certificate via Keychain. Pas urgent en v1.0.
- En v0.1 du site : Android + iOS = badge "Bientôt disponible" sans CTA actif.

**Page `/telecharger/` complémentaire** :

- Reste comme page détaillée : versions précédentes (changelog), checksums affichés, signatures GPG futures, instructions de vérification, FAQ install (notamment Gatekeeper macOS, SmartScreen Windows en attendant codesign).

---

### Astro Content Collections — branchement avec `docs/` du repo

**Mécanisme** :
- Dans le build CI (`.github/workflows/site-deploy.yml`), avant `npm run build`, on copie/symlink `docs/` (du repo principal) vers `site/src/content/docs/` et `docs/adr/` vers `site/src/content/adrs/`.
- Astro Content Collections lit ces dossiers via `config.ts` (schemas Zod pour valider front-matter).
- Pages générées dynamiquement via `src/pages/docs/[...slug].astro` + `getStaticPaths()`.
- **Single source of truth** : modification de `docs/` du repo → push → CI rebuild + redeploy. 0 duplication.

**Front-matter standard à ajouter aux MD existants** (rétrocompatible) :

```yaml
---
title: "Titre lisible"
description: "Court extrait pour SEO + cards"
order: 10                  # ordre dans la sidebar (optionnel)
category: "Méthodologie"  # groupement sidebar (optionnel)
---
```

Si front-matter absent → Astro fallback sur le premier H1 et un extrait auto.

---

## 3. Découpage en sous-chantiers

### C33.1 — Bootstrap Astro + design system (1 j)

- `site/package.json` (Astro 5, Svelte 5, Tailwind 4, Three.js, Motion One, Pagefind, MDX).
- Config Astro avec adapter Cloudflare (`@astrojs/cloudflare`) + intégration Svelte + Tailwind.
- Design system Tailwind : palette Sobr.ia (lime `#a0e060`, ambre `#e0c060`, coral `#e07060`, ink `#1a1a1a`, paper `#fafafa`), tokens typo, spacing.
- Composants UI de base (`Button.svelte`, `Card.svelte`, `Badge.svelte`).
- Topbar sticky + footer placeholder.
- Page `index.astro` placeholder avec sections vides numérotées.
- `tests/lighthouse.spec.ts` + `tests/a11y.spec.ts` (squelette).

**DoD C33.1** : `cd site && npm run dev` ouvre une page propre avec la palette correcte, topbar sticky, sections vides.

### C33.2 — Hero 3D + sections statiques (1.5 j)

- **Hero 3D** (`HeroGlobe.svelte`) :
  - Three.js scene : sphère Terre (texture earth basique, ≤ 200 KB jpg).
  - 28 datacenters Europe geo-positionnés (cf. `crates/sobria-geoloc/data/datacenters_demo.json`) avec spots lumineux pulsants.
  - Particules carbone (instanced mesh, 1000 particules) montant lentement vers l'espace.
  - Camera rotation lente (auto-orbit) + interaction souris (drag pour tourner).
  - Lazy-load : Three.js chargé seulement quand la page est mount (pas en SSR).
  - Fallback : si `prefers-reduced-motion` ou WebGL absent → image statique.
- Section 1 (Hero) : globe 3D + headline + sous-headline + 2 CTAs.
- Section 2 (Pour qui) : 5 PersonaCard avec liens vers pages /personas/{id}/.
- Section 3 (Comment ça marche) : 3 étapes avec icônes + texte court.
- Section Footer : ADRs, licences, candidature, contributeurs.

**DoD C33.2** : hero impressionnant qui tourne sans janks à 60 fps + 3 sections statiques propres.

### C33.3 — Sections animées 3D (1.5 j)

- **Section Méthodologies** (`MonteCarloViz.svelte`) :
  - Viz 3D : 10⁴ points (instanced spheres) qui dansent autour d'une courbe gaussienne.
  - Au scroll : la viz s'active, anim de "convergence" sur la valeur P50.
  - Toggle : "AFNOR/Sobr.ia" vs "EcoLogits" → la viz change de courbe (params différents).
- **Section Vendors disclosure** :
  - Table animée 5 vendors avec révélation séquentielle des cellules au scroll.
  - Chiffres clés en gros (1.14 g, 0.03 g, 11 390 t…) avec source en hover.
  - Badges ✅ / ⚠️ / ❌ animés selon disclosure status.
- **Section Territoire FR** (`IrisMap3D.svelte`) :
  - Carte FR en 2.5D (extrusion CSS 3D ou Three.js plane), zones IRIS colorées par densité datacenter.
  - Tooltip survol : sites industriels par maille.
- **Section Entreprises** :
  - 2 cards côte-à-côte : "Mode Équipe self-hosted" (dispo) vs "Cloud managé bientôt" (CTA mailing list).
  - Schéma 3D simple : binaire serveur + flèches vers 3-4 employés (avatars stylisés).

**DoD C33.3** : 4 sections 3D fluides, perf > 50 fps sur laptop standard, fallbacks OK.

### C33.4 — Doc interactive (1.5 j)

- Setup Astro Content Collections : `src/content/config.ts` avec schemas Zod pour `docs/` et `adrs/`.
- Script CI `scripts/sync-docs.sh` (ou Astro plugin) : copie `docs/*.md` + `docs/adr/*.md` du repo vers `site/src/content/docs/` + `adrs/` au build.
- Pages `/docs/`, `/docs/[...slug]`, `/adrs/`, `/adrs/[...slug]` avec sidebar de navigation.
- Pagefind setup pour full-text search (icon search dans topbar).
- Front-matter rétro-ajouté aux MD existants si absent (script `scripts/add-frontmatter.mjs` pour les transformer).
- Composants doc : `<CodeBlock>` avec syntax highlight (Shiki), `<Callout>` (note/tip/warning), `<Tree>` (structure dossiers).
- Sidebar : groupement automatique par dossier + ordre via `order:` front-matter.
- Breadcrumb + "Edit on GitHub" sur chaque page doc.

**DoD C33.4** : `/docs/` navigable avec recherche full-text fonctionnelle, sidebar propre, dark mode.

### C33.5 — Pages dédiées + CTA cloud + workflows release CI (1.5 j)

⚠️ **Périmètre étendu** : audit a révélé qu'il n'y a PAS de workflow CI qui build les binaires Tauri ni l'extension. C33 doit créer ces workflows pour que les téléchargements proposés sur la landing existent vraiment.

#### C33.5.a — Workflow `app-release.yml` (build Tauri Win/macOS/Linux)

- `.github/workflows/app-release.yml` :
  - Trigger : tag matching `v[0-9]+.[0-9]+.[0-9]+` + dispatch manuel.
  - Matrix : `windows-latest`, `macos-latest` (x86_64 + aarch64), `ubuntu-22.04`.
  - Steps : checkout, setup Rust + Node, `cd web && npm ci && npm run build`, `cargo tauri build --bundles <bundle-type>`.
  - Bundles à produire :
    - Windows : `.msi` + `.exe` (NSIS).
    - macOS : `.dmg` (séparé Intel + Apple Silicon).
    - Linux : `.deb` + `.AppImage`.
  - Upload assets sur GitHub Release via `softprops/action-gh-release@v2`.
  - Calcul SHA-256 + upload des `.sha256` à côté de chaque binaire.
- Note : **signing différé** — les binaires ne sont pas codesigned en v0.1.0. Disclaimer Gatekeeper macOS / SmartScreen Windows sur `/telecharger/`.

#### C33.5.b — Workflow `extension-release.yml` (build Chrome zip + Firefox xpi)

- `.github/workflows/extension-release.yml` :
  - Trigger : même tag pattern.
  - `cd extension && npm ci && npm run build && npm run build:firefox && npm run package`.
  - Upload `sobria-extension-chrome-v<version>.zip` + `sobria-extension-firefox-v<version>.xpi` + SHA-256.

#### C33.5.c — Trigger initial pour v0.7.1 (rétroactif)

- Une fois les workflows mergés, **trigger manuel pour le tag v0.7.1** via `workflow_dispatch` pour que la dernière release existante ait les binaires.
- Vérifier : `https://github.com/<owner>/defis-lia-generatif/releases/v0.7.1` doit afficher 8+ assets.
- Liens `latest/download/...` deviennent fonctionnels.

#### C33.5.d — Pages dédiées du site

- `/cloud/` placeholder :
  - Hero "Mode Équipe self-hosted dispo · Cloud managé bientôt".
  - 2 sections : "Self-hosted aujourd'hui" (lien doc team-aggregator) vs "Cloud bientôt" (form mailing list).
  - Form opt-in : email + checkbox RGPD + bouton "Être prévenu·e". MVP = mailto simple.
- `/telecharger/` page détaillée :
  - Liste exhaustive des téléchargements avec versions précédentes (GitHub Releases API au build, fallback statique).
  - Section "Vérification SHA-256" avec commande copy-paste par OS.
  - Section "Avertissements OS" expliquant Gatekeeper macOS et SmartScreen Windows tant que pas codesigned.
  - Section "Source code" : lien GitHub + tag + commit SHA pour reproductibilité.
- `/candidature/` : statut candidature data.gouv.fr + 19 sources auditées (lien `/docs/sources/AUDIT-2026-Q3/`) + ADRs clés (0009, 0012, 0013, 0014).

**DoD C33.5** :
- 2 workflows release fonctionnels (app + extension).
- v0.7.1 dispose de 8+ assets téléchargeables sur GitHub Releases.
- 3 pages secondaires propres avec CTAs fonctionnels.
- Section `#download` sur landing affiche 5 vrais boutons fonctionnels (Win/macOS/Linux/Chrome/Firefox) + 2 "Bientôt" (Android/iOS).

### C33.6 — CI/CD + déploiement self-hosted nginx (1 j)

**Périmètre étendu** vu qu'on passe en self-hosted avec sécurité à durcir.

#### C33.6.a — Provisioning serveur (depuis Claude Code en SSH)

Avant tout déploiement, créer l'environnement sur le serveur Thibault (80.11.20.55, Ubuntu 22.04+, nginx déjà installé) :

- **Créer user dédié `deployer`** (PAS root pour le CI) :
  - `useradd -m -s /bin/bash deployer`
  - Ajouter au groupe `www-data` pour permissions web.
  - Donner sudoers limité : SEULEMENT `systemctl reload nginx` et `systemctl status nginx`. Pas plus.
- **Générer clé SSH dédiée** côté serveur (`ssh-keygen -t ed25519 -f ~/.ssh/sobria_deploy -N ""`) :
  - Clé publique → `/home/deployer/.ssh/authorized_keys`.
  - Clé privée → à mettre dans GitHub Secrets `SOBRIA_DEPLOY_SSH_KEY`.
- **Créer dossier site** :
  - `mkdir -p /var/www/sobria-site`
  - `chown -R deployer:www-data /var/www/sobria-site`
  - `chmod -R 750 /var/www/sobria-site`
- **Server block nginx** dans `/etc/nginx/sites-available/sobria.brilliantstudio.co` (config détaillée dans `docs/operations/site-deploy.md`).
- **Symlink** : `ln -s /etc/nginx/sites-available/sobria.brilliantstudio.co /etc/nginx/sites-enabled/`.
- **DNS** : Thibault ajoute un record A sur `sobria.brilliantstudio.co` → `80.11.20.55` côté registrar du domaine `brilliantstudio.co`.
- **Certbot** : `certbot --nginx -d sobria.brilliantstudio.co` (Let's Encrypt automatique, renouvellement cron).

Voir doc complète `docs/operations/site-deploy.md`.

#### C33.6.b — GitHub Action `.github/workflows/site-deploy.yml`

- Trigger : push sur `main` (paths `site/**` + `docs/**`) ou dispatch manuel.
- Steps :
  1. Checkout repo.
  2. Setup Node 20.
  3. `bash scripts/sync-docs.sh` (copie `docs/` → `site/src/content/`).
  4. `cd site && npm ci`.
  5. `npm run check` (TypeScript strict).
  6. `npm run lint`.
  7. `npm run build` (Astro static → `site/dist/`).
  8. Setup SSH agent avec `SOBRIA_DEPLOY_SSH_KEY` (secret).
  9. `rsync -avz --delete site/dist/ deployer@80.11.20.55:/var/www/sobria-site/`.
  10. SSH reload nginx (optionnel, seulement si config changée).
  11. Post-deploy : `curl -sf https://sobria.brilliantstudio.co/ > /dev/null` smoke test.
- Secrets requis :
  - `SOBRIA_DEPLOY_SSH_KEY` (clé privée ed25519).
  - `SOBRIA_DEPLOY_HOST` (`80.11.20.55`).
  - `SOBRIA_DEPLOY_USER` (`deployer`).
  - `SOBRIA_DEPLOY_PATH` (`/var/www/sobria-site/`).
- Tests CI séparés (avant deploy) :
  - Lighthouse CI ≥ 90 sur preview build local.
  - axe-core a11y sur preview build local.

#### C33.6.c — README + doc

- `site/README.md` : setup local (`npm install && npm run dev`), structure, deploy auto via push.
- `docs/operations/site-deploy.md` (nouveau) : doc opérationnelle complète (provisioning, nginx config, certbot, rotation key, troubleshooting).

**DoD C33.6** : push main → site online en ~3 min sur `https://sobria.brilliantstudio.co/`. Renouvellement TLS auto. Pas de credentials root utilisés.

### C33.7 — Polish + tag site-v0.1.0 (0.5 j)

- Smoke test cross-browser (Chrome, Firefox, Safari, mobile).
- Test `prefers-reduced-motion` actif → vérifier dégradation propre.
- Test WebGL désactivé → fallbacks OK.
- OpenGraph card pour partages réseaux sociaux.
- Sitemap + robots.txt.
- CHANGELOG (du site, dans `site/CHANGELOG.md` séparé du CHANGELOG racine).
- Tag `site-v0.1.0` :

```bash
git tag -a site-v0.1.0 -m "site-v0.1.0 — Premier site internet Sobr.ia (C33)

Site Astro 5 + Svelte 5 islands + Three.js + Motion One déployé sur
Cloudflare Pages. Hero 3D globe Terre + datacenters animés, sections
animées au scroll (méthodologies, vendors disclosure, territoire FR,
entreprises), doc interactive sourcée du repo (Astro Content
Collections), recherche full-text Pagefind.

Lighthouse ≥ 90 sur les 4 catégories. Respect prefers-reduced-motion.
Pas de tracking, pas de Google Fonts, pas d'analytics tiers."
```

---

## 4. Definition of Done globale `site-v0.1.0`

- [ ] `cd site && npm run check && npm run lint && npm run build` OK.
- [ ] Site déployé en HTTPS sur `https://sobria.brilliantstudio.co/`.
- [ ] User `deployer` créé sur le serveur (PAS root), sudoers limité à `systemctl reload nginx`.
- [ ] Certbot Let's Encrypt OK + renouvellement cron configuré.
- [ ] Lighthouse score ≥ 90 sur Performance / Accessibility / Best Practices / SEO.
- [ ] Bundle initial ≤ 100 KB gzip (vérifier avec `astro build` rapport).
- [ ] Bundle 3D lazy-loaded ≤ 300 KB gzip.
- [ ] `prefers-reduced-motion` respecté (3D désactivée + animations remplacées par fade-in).
- [ ] WebGL fallback testé (image statique).
- [ ] `/docs/*` génère bien depuis `docs/*.md` du repo, recherche full-text fonctionnelle.
- [ ] 6 sections principales + 3 pages secondaires (`/cloud/`, `/telecharger/`, `/candidature/`).
- [ ] Form mailing list `/cloud/` fonctionnel (au minimum mailto).
- [ ] CI/CD `.github/workflows/site-deploy.yml` auto-deploy sur push main.
- [ ] Tests `a11y.spec.ts` (axe-core) propres.
- [ ] Cross-browser : Chrome, Firefox, Safari, mobile.
- [ ] OG card pour partages.
- [ ] Tag `site-v0.1.0` poussé.

---

## 5. Anti-périmètre

- Pas de version EN (FR-only en v0.1.0).
- Pas d'analytics tiers (Google Analytics, Plausible cloud) — réfléchir Plausible self-hosted en v0.2.0.
- Pas de CRM / marketing automation.
- Pas de privacy policy détaillée RGPD légale (placeholder OK, à compléter avant cloud beta v1.3+).
- Pas de demo in-browser de l'app (juste lien Télécharger).
- Pas de blog (peut venir en v0.2.0 site).
- Pas de portfolio contributeurs (peut venir plus tard).

---

## 6. Risques + mitigations

| Risque | Mitigation |
|---|---|
| Three.js trop lourd → Lighthouse < 90 | Lazy-load + Intersection Observer + bundle 3D séparé + texture compress |
| `prefers-reduced-motion` mal géré | Tests automatisés en CI + fallback fade-in |
| Doc Content Collections explose au build (trop de fichiers) | Astro build optimisé + sync incrémental dans CI |
| Cloudflare Pages limite free tier | Largement OK pour notre trafic, monitoring quotas en place |
| Domaine `sobr.ia` indisponible ou cher | Backup `sobria.fr` confirmé disponible |
| Form mailing list nécessite backend | Cloudflare Worker + Airtable free tier ou simple mailto au début |
| Trop de scope si on tente perfection visuelle | Stop ship en 7 j max, polish v0.2 site plus tard |

---

## 7. Découpage temporel

| Jour | Sous-chantier | Livrable |
|---|---|---|
| J1 | C33.1 bootstrap Astro + design system | Pages vides propres, palette OK |
| J2 | C33.2 Hero 3D + 3 sections statiques | Hero globe fonctionnel + Pour qui + Comment + Footer |
| J3 | C33.2 fin + C33.3 début | Section #download avec auto-détection OS + 1 section 3D |
| J4-J5 | C33.3 sections 3D animées | Monte-Carlo + vendors + IRIS + entreprises |
| J5.5 | C33.4 doc interactive | /docs/ + search Pagefind |
| J6 | C33.5 workflows release CI + pages dédiées | app-release.yml + extension-release.yml + trigger rétro v0.7.1 |
| J7 | C33.5 fin + C33.6 deploy nginx | /cloud/ + /telecharger/ + /candidature/ + site online HTTPS |
| J8 | C33.7 polish + tag | Smoke cross-browser + tag site-v0.1.0 |

Total : **6,5-8 jours** selon densité (vs 5-7 j initial — étendu pour intégrer les workflows release CI absents).

---

## 8. Domaine — décidé

**Choix arrêté** : `sobria.brilliantstudio.co` (sous-domaine domaine studio Thibault).

Avantages :
- Zéro coût domaine (déjà acquis).
- DNS sous contrôle Thibault, pas de dépendance Cloudflare ou autre.
- Cohérent souveraineté : serveur Thibault + domaine Thibault, pas un service cloud central.
- Si rebrand v1.x : on bouge facilement le sous-domaine sans casser l'existant.

Note : `sobr.ia` et `sobria.fr` non disponibles à l'enregistrement. Rebrand éventuel du PRODUIT vers un nouveau nom = chantier séparé (différé après v1.0 candidature).

---

## 9. Livrables annexes

- `site/README.md` — setup + deploy + structure.
- `site/CHANGELOG.md` — séparé du CHANGELOG racine (site versionné indépendamment).
- `.github/workflows/site-deploy.yml`.
- DNS configuration documentée dans `docs/operations/site-deploy.md`.
- Page `/candidature/` enrichie qui pointe vers ADRs + audit C30 + audit C32.

---

## 10. Et après site-v0.1.0 ?

- **v1.0 candidature data.gouv.fr** : sprint final ~1 semaine, le site est un asset majeur.
- **site-v0.2.0** : version EN, blog, page "Contributeurs", Plausible self-hosted analytics.
- **site-v1.0.0** : cohabite avec le lancement cloud beta v1.3 — landing page d'inscription cloud + dashboard managed embedded.
