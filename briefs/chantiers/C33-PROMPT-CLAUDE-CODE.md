# C33 — Prompt Claude Code (site internet Sobr.ia)

> **Mode d'emploi** : copier-coller le bloc ci-dessous dans une nouvelle session Claude Code (CLI) à la racine du repo. Le prompt démarre par `/using-superpower`.
>
> **Pré-requis** : v0.8.0 shippée (C32 Clarté produit), wordings finaux dispo.

---

```
/using-superpower

# Mission : C33 — Site internet Sobr.ia (site-v0.1.0)

Tu vas livrer le site internet officiel Sobr.ia, déployé en HTTPS sur
**serveur self-hosted Thibault** (nginx + Let's Encrypt sur Ubuntu 22.04+,
IP `80.11.20.55`), avec Hero 3D Three.js, 6 sections animées scroll, doc
interactive sourcée du repo (Astro Content Collections), CI/CD auto-deploy
via SSH rsync.

Périmètre : 5-7 jours. Stack : Astro 5 + Svelte 5 islands + Three.js +
**Motion One verrouillé** + Tailwind 4 + Pagefind + nginx self-hosted.

**Domaine arrêté** : `sobria.brilliantstudio.co` (sous-domaine du
domaine studio Thibault, DNS déjà configuré côté registrar — record A
vers 80.11.20.55).

## Contexte à charger AVANT toute action

Lis ces fichiers dans l'ordre :

1. `CLAUDE.md` — règles, anti-patterns, privacy by design, frugalité §8.
2. **`briefs/chantiers/C33-site-internet.md`** — brief complet, source
   de vérité pour le périmètre + DoD + découpage en 7 sous-chantiers.
3. **`docs/operations/site-deploy.md`** — doc opérationnelle déploiement
   complète (provisioning serveur, nginx server block, certbot, SSH key,
   GitHub Secrets, troubleshooting). À suivre SCRUPULEUSEMENT pour C33.6.
4. **`docs/product/AUDIT-PRODUIT-2026-Q3.md`** — wordings, value
   proposition, taglines persona à réutiliser SUR le site (cohérence
   avec C32 v0.8.0).
5. `docs/sources/AUDIT-2026-Q3.md` — vendors disclosure (Mistral × ADEME,
   Google, Meta, etc.) à présenter dans la section dédiée.
6. `docs/adr/ADR-0014-dual-track-local-cloud.md` — narratif cloud opt-in
   à respecter scrupuleusement dans la page `/cloud/` (Coming soon, pas
   d'engagement commercial avant v1.3).
7. `README.md` (v2 post-C32) — réutiliser le ton et les sections.
8. `web/src/lib/styles/` ou équivalent — palette Sobr.ia (lime/ambre/coral).
9. `crates/sobria-geoloc/data/datacenters_demo.json` — coordonnées des
   28 datacenters Europe pour le globe 3D.

## Stratégie + garde-fous

- **Frugalité incarnée** : Three.js lazy-load + Intersection Observer.
  Bundle initial ≤ 100 KB gzip. Bundle 3D ≤ 300 KB gzip. Lighthouse ≥ 90.
- **Privacy by design** : ZÉRO tracking, ZÉRO Google Fonts (system fonts
  ou self-hosted), ZÉRO analytics tiers en v0.1.0.
- **Accessibilité critique** : `prefers-reduced-motion` respecté
  (3D désactivée + fade-in), WebGL fallback image statique, axe-core CI.
- **Single source of truth doc** : `docs/*.md` du repo → Astro Content
  Collections via script `sync-docs.sh` en CI. PAS de duplication.
- **Site dans monorepo** : `/site/` au root, versionné indépendamment
  (`site/package.json` 0.1.0, `site/CHANGELOG.md` séparé).
- **Pas de bump version app** : C33 ne touche pas à Cargo.toml workspace
  ni à l'app Tauri. Seul `site/package.json` et tag `site-v0.1.0`.
- **DEMANDER** si tu hésites sur :
  - Choix du domaine (sobr.ia vs sobria.fr).
  - Wordings Hero (réutilise C32 value prop par défaut).
  - Forme du form mailing list `/cloud/` (mailto simple OK pour v0.1.0).

## Plan d'exécution

### C33.1 — Bootstrap Astro + design system (1 j)

- Créer `/site/` avec :
  - `package.json` : astro@5, @astrojs/svelte, @astrojs/tailwind,
    @astrojs/cloudflare, @astrojs/mdx, three@~0.170, motion (motion-v),
    pagefind, @astrojs/check, prettier, prettier-plugin-astro.
  - `astro.config.mjs` adapter Cloudflare, intégrations Svelte + Tailwind + MDX.
  - `tsconfig.json` strict, extends astro default.
  - `tailwind.config.mjs` palette Sobr.ia (lime `#a0e060`, ambre `#e0c060`,
    coral `#e07060`, ink `#1a1a1a`, paper `#fafafa`), Inter ou system
    fonts (PAS Google Fonts).
- Composants UI base : `Button.svelte`, `Card.svelte`, `Badge.svelte`
  dans `src/components/ui/`.
- `Topbar.astro` sticky + footer placeholder dans `src/components/`.
- `src/pages/index.astro` avec 8 sections vides numérotées (Hero, Pour
  qui, Comment ça marche, Méthodos, Vendors, Territoire FR, Entreprises,
  Footer).
- `tests/lighthouse.spec.ts` + `tests/a11y.spec.ts` (squelette
  Playwright).

DoD C33.1 : `cd site && npm run dev` ouvre page propre avec palette,
topbar sticky, sections vides.

### C33.2 — Hero 3D + 3 sections statiques (1.5 j)

- **`src/components/three/HeroGlobe.svelte`** :
  - Three.js scene chargée DYNAMIQUEMENT via `import()` (pas dans le
    bundle initial). Mount uniquement si pas reduced-motion + WebGL OK.
  - Scene : SphereGeometry r=2, MeshStandardMaterial avec texture earth
    basique (img 1024x512 jpg ≤ 200 KB, à trouver ou générer).
  - 28 datacenters Europe geo-positionnés via lat/lon → vec3, spots
    lumineux pulsants (small spheres émissives + cycle alpha sinus).
  - Particules carbone : `InstancedMesh` 1000 instances, montée
    lente vers Z+ avec wrap-around. Couleur ambre semi-transparent.
  - Camera auto-orbit lent (axe Y) + drag souris pour rotate manuel
    (PointerEvents).
  - `ResizeObserver` pour responsive.
  - `requestAnimationFrame` boucle, cleanup au destroy.
  - Fallback : `<picture>` avec image statique pré-rendue si WebGL absent
    ou reduced-motion.
- Section Hero : globe 3D fullscreen + overlay headline + sous-headline
  (réutilise value prop C32) + 2 CTAs (Télécharger lien anchor /
  /telecharger, Voir la démo lien GitHub).
- Section "Pour qui ?" : 5 `PersonaCard.svelte` avec icônes Lucide
  (GraduationCap, Code2, Building2, Landmark, Microscope) + lien
  `/personas/{id}/`. Wordings : taglines C32.
- Section "Comment ça marche ?" : 3 étapes avec icônes (Download, Type,
  Activity), animation fade-in séquentielle à l'entrée viewport.
- Footer : 4 colonnes (Sobr.ia / Produit / Doc / Légal) + ADRs +
  candidature data.gouv.fr badge.

DoD C33.2 : Hero impressionnant > 60 fps sur laptop, fallback OK, 3
sections statiques propres mobile + desktop.

### C33.3 — 4 sections 3D animées (1.5 j)

- **`MonteCarloViz.svelte`** : viz Monte-Carlo 3D.
  - 10⁴ instances de petites spheres, distribution gaussienne autour
    d'un axe X.
  - Au scroll-entry : les points dansent puis convergent vers la mean
    et la P5/P95 visible.
  - Toggle "AFNOR/Sobr.ia" vs "EcoLogits" → params différents (mean +
    sigma) → transition smooth via lerp.
- **`VendorDisclosure.svelte`** : table 5 vendors animée.
  - Lignes : Mistral × ADEME / Google Gemini / Meta Llama 3.x /
    Anthropic / OpenAI.
  - Colonnes : Prompt-level disclosure ? / Training disclosure ? /
    Source.
  - Apparition séquentielle des cellules au scroll (stagger Motion One).
  - Chiffres clés en gros (1.14 g, 0.03 g, 11 390 t), badges ✅/⚠️/❌.
- **`IrisMap3D.svelte`** : carte FR 2.5D.
  - Plane GeoJSON FR (départements ou régions) avec extrusion CSS 3D
    ou Three.js. Hauteur proportionnelle à conso élec datacenters.
  - Tooltip survol : nom région + chiffre clé.
  - Si trop complexe Three.js, fallback CSS 3D transform.
- **`Entreprises.svelte`** :
  - 2 cards côte-à-côte : "Mode Équipe self-hosted (disponible
    aujourd'hui)" + "Cloud managé (bientôt disponible)".
  - Schéma 3D simple côté self-hosted : binaire serveur + 3-4 avatars
    employés stylisés.
  - Card cloud : CTA "Être prévenu·e" → lien vers `/cloud/`.

DoD C33.3 : 4 sections 3D fluides, perf > 50 fps, fallbacks `prefers-
reduced-motion` OK.

### C33.4 — Doc interactive Astro Content Collections (1.5 j)

- `src/content/config.ts` : 2 collections `docs` + `adrs` avec schemas
  Zod (title, description, order, category optionnels).
- `scripts/sync-docs.sh` : copie `../docs/**/*.md` (sans `adr/`) vers
  `src/content/docs/` et `../docs/adr/*.md` vers `src/content/adrs/`.
  Exécuté avant chaque `npm run build`.
- `src/pages/docs/[...slug].astro` + `getStaticPaths` qui lit la
  collection `docs`.
- `src/pages/adrs/[...slug].astro` idem pour `adrs`.
- `src/pages/docs/index.astro` : sommaire avec groupement par
  `category`, ordre par `order`.
- Composants doc :
  - `<DocLayout>` avec sidebar gauche (tree) + content + breadcrumb +
    "Edit on GitHub" link.
  - `<CodeBlock>` avec syntax highlight Shiki.
  - `<Callout type="note|tip|warning">`.
- Pagefind setup : `pagefind` installé en dev, build hook qui indexe
  `dist/docs/**` + `dist/adrs/**`. Search bar dans Topbar.
- Front-matter rétro-compatible : si MD source n'a pas de front-matter,
  utilise le premier H1 comme title + extrait auto comme description.

DoD C33.4 : `/docs/` navigable avec sidebar, recherche full-text
fonctionne, dark mode supporté.

### C33.5 — Workflows release CI + section #download + pages (1.5 j)

⚠️ **Périmètre étendu** : il n'y a actuellement PAS de workflow CI qui
build les binaires Tauri ni l'extension. Les tags v0.4.0 → v0.7.1
existent mais sans assets dans GitHub Releases. C33 doit créer ces
workflows pour que les téléchargements proposés sur la landing soient
fonctionnels.

**C33.5.a — `.github/workflows/app-release.yml`** :
- Trigger : tag `v[0-9]+.[0-9]+.[0-9]+` + dispatch manuel.
- Matrix : `windows-latest`, `macos-latest` (x86_64 + aarch64),
  `ubuntu-22.04`.
- Build Tauri avec bundles `.msi`, `.exe` (NSIS), `.dmg` (Intel + ARM),
  `.deb`, `.AppImage`.
- Upload assets sur GitHub Release via `softprops/action-gh-release@v2`
  + fichiers `.sha256` pour vérification utilisateur.
- Pas de signing en v0.1.0 (différé), disclaimer côté site.

**C33.5.b — `.github/workflows/extension-release.yml`** :
- Même trigger tag.
- Build Chrome zip + Firefox xpi via `cd extension && npm run build &&
  npm run package` (vérifier scripts existants extension/package.json).
- Upload assets + SHA-256.

**C33.5.c — Trigger rétroactif v0.7.1** :
- Après merge des 2 workflows, `workflow_dispatch` pour le tag v0.7.1
  afin qu'il dispose de tous les binaires.
- Vérifier `https://github.com/<owner>/defis-lia-generatif/releases/v0.7.1`
  affiche 8+ assets.
- Les URLs `releases/latest/download/<filename>` deviennent fonctionnelles.

**C33.5.d — Section `#download` sur landing + pages dédiées** :
- `<DownloadSection />` Svelte sur `src/pages/index.astro`, ancre
  `#download` :
  - Détection OS via `navigator.userAgent` + `userAgentData.platform`.
  - Highlight bouton détecté (bordure lime + badge "Recommandé").
  - 7 cartes : Windows / macOS / Linux / Chrome / Firefox / Android
    (badge "Bientôt") / iOS (badge "Bientôt").
  - Chaque carte : version, taille, lien "Vérifier SHA-256".
  - URLs stables `https://github.com/<owner>/defis-lia-generatif/releases/latest/download/<filename>`.
- `/telecharger/` détaillée : versions précédentes, checksums,
  signatures futures, FAQ install (Gatekeeper macOS, SmartScreen
  Windows tant que pas codesigned).
- `/cloud/` placeholder : Hero + 2 sections (Self-hosted vs Cloud
  bientôt) + form mailto MVP.
- `/candidature/` : statut candidature + 19 sources audit + ADRs clés
  (0009, 0012, 0013, 0014).

**Stratégie mobile (Android + iOS)** :
- **Pas dans le CI standard.** Build Android = tag major (v1.0, v1.1,
  v2.0) uniquement, workflow `mobile-android-release.yml` à créer
  **plus tard** (chantier ultérieur, pas C33).
- iOS différé v1.x au plus tôt (Apple Developer Program, codesign).
- En v0.1 du site : badges "Bientôt disponible" sans CTA actif.

DoD C33.5 :
- 2 workflows release fonctionnels (app + extension).
- Tag v0.7.1 dispose de 8+ assets téléchargeables.
- Section `#download` opérationnelle sur landing avec auto-détection OS.
- 3 pages secondaires propres avec CTAs fonctionnels.
- Curl test sur chaque URL `releases/latest/download/...` retourne 302
  → 200.

### C33.6 — CI/CD self-hosted nginx (1 j)

⚠️ **À faire en suivant SCRUPULEUSEMENT `docs/operations/site-deploy.md`.**
Cette doc contient le détail exact des commandes SSH, config nginx,
certbot, GitHub Secrets, etc. Si quelque chose n'est pas clair dans
cette doc, demande à Thibault avant de modifier la config serveur.

**Décomposition** :

C33.6.a — **Provisioning serveur** (via SSH `root@80.11.20.55`,
session unique au début du sprint) :
1. Vérifier prérequis : OS, nginx, ports, certbot. Voir doc §2.3.
2. Installer certbot si absent : `apt install -y certbot python3-certbot-nginx`.
3. Créer user `deployer` + sudoers limité + SSH directory. Voir doc §3.2 + §3.3.
4. Générer clé SSH ed25519 dédiée, ajouter publique dans
   `/home/deployer/.ssh/authorized_keys`, garder privée pour GitHub
   Secret. Voir doc §3.4.
5. Tester accès `deployer` depuis local : `ssh -i <key> deployer@... 'whoami'`.
6. Créer `/var/www/sobria-site` avec permissions deployer:www-data 750
   + placeholder `index.html`. Voir doc §3.7.

C33.6.b — **Configuration nginx** :
1. Créer `/etc/nginx/sites-available/sobria.brilliantstudio.co` avec
   le server block fourni doc §4.1 (HTTP → HTTPS redirect + HTTPS avec
   security headers + cache assets + Astro routing).
2. `ln -s ... /etc/nginx/sites-enabled/`.
3. `nginx -t` puis `systemctl reload nginx`.
4. Vérifier que HTTP répond (placeholder visible).

C33.6.c — **Certbot Let's Encrypt** :
1. `certbot --nginx -d sobria.brilliantstudio.co --non-interactive
   --agree-tos --email thibault@brilliantstudio.co` (à ajuster email
   avec Thibault).
2. Vérifier renouvellement auto : `systemctl status certbot.timer`.
3. Tester `certbot renew --dry-run`.
4. Curl HTTPS pour valider.

C33.6.d — **GitHub Actions workflow `.github/workflows/site-deploy.yml`** :
1. Copier le workflow YAML fourni doc §6.2.
2. Documenter les 5 secrets à créer (SOBRIA_DEPLOY_SSH_KEY,
   SOBRIA_DEPLOY_HOST, SOBRIA_DEPLOY_USER, SOBRIA_DEPLOY_PATH,
   SOBRIA_DEPLOY_KNOWN_HOSTS) — voir doc §6.1.
3. Premier deploy manuel via rsync depuis local Thibault pour valider
   le pipeline avant d'activer le workflow.
4. Trigger workflow_dispatch pour test, puis activer push trigger.

C33.6.e — **README + doc**
1. `site/README.md` setup local + deploy + structure.
2. Référence vers `docs/operations/site-deploy.md` pour les ops.

**DoD C33.6** :
- HTTPS opérationnel sur `https://sobria.brilliantstudio.co/`.
- Cert Let's Encrypt valide, renouvellement auto activé.
- User `deployer` actif, pas de credentials root utilisés en CI.
- Workflow `site-deploy.yml` testé et fonctionnel (push → deploy).
- Post-deploy smoke test (curl) retourne 200.

### C33.7 — Polish + tag site-v0.1.0 (0.5 j)

- Smoke test cross-browser (Chrome, Firefox, Safari, mobile).
- Test forcer `prefers-reduced-motion` actif → 3D désactivée, fade-in OK.
- Test forcer WebGL désactivé → fallbacks images statiques OK.
- OG card 1200x630 pour partages (image générée ou statique).
- Sitemap + robots.txt.
- `site/CHANGELOG.md` initial avec entrée `[0.1.0] — YYYY-MM-DD —
  Premier site Sobr.ia (C33)`.
- Tag `site-v0.1.0` (préfixe `site-` pour ne pas se confondre avec les
  tags app `v0.x.y`).

```bash
git tag -a site-v0.1.0 -m "site-v0.1.0 — Premier site internet Sobr.ia (C33)
...message complet cf. brief §C33.7..."
```

## DoD globale

- [ ] `cd site && npm run check && npm run lint && npm run build` OK.
- [ ] Site déployé HTTPS sur `https://sobria.brilliantstudio.co/`.
- [ ] User `deployer` créé (PAS root pour CI), sudoers limité à reload/restart nginx.
- [ ] Certbot Let's Encrypt OK + renouvellement auto.
- [ ] Lighthouse ≥ 90 sur Performance / Accessibility / Best Practices / SEO.
- [ ] Bundle initial ≤ 100 KB gzip.
- [ ] Bundle 3D lazy-loaded ≤ 300 KB gzip.
- [ ] `prefers-reduced-motion` respecté + tests CI.
- [ ] WebGL fallback testé.
- [ ] `/docs/*` génère depuis `docs/*.md` du repo, search Pagefind OK.
- [ ] 6 sections principales + 3 pages secondaires.
- [ ] Form `/cloud/` opt-in fonctionnel (minimum mailto).
- [ ] CI/CD auto-deploy sur push main.
- [ ] Cross-browser : Chrome, Firefox, Safari, mobile.
- [ ] OG card.
- [ ] Tag `site-v0.1.0` poussé.

## Convention de commit

```
chore(site): C33.1 bootstrap Astro 5 + Svelte 5 + Tailwind 4 + Three.js
feat(site): C33.2 Hero 3D globe + 3 sections statiques (Pour qui, Comment, Footer)
feat(site): C33.3 sections 3D animées (Monte-Carlo, Vendors, Territoire FR, Entreprises)
feat(site): C33.4 doc interactive Astro Content Collections + Pagefind
feat(site): C33.5 pages dédiées (cloud, telecharger, candidature)
ci(site): C33.6 GitHub Actions deploy Cloudflare Pages
chore(site): C33.7 polish + tag site-v0.1.0
```

## Garde-fous

- **JAMAIS** ajouter de tracking ou analytics tiers en v0.1.0.
- **JAMAIS** utiliser Google Fonts (system fonts ou WOFF2 self-hosted).
- **JAMAIS** dépasser Lighthouse 90 — si une feature 3D fait chuter le
  score, simplifier ou différer.
- **JAMAIS** dupliquer le contenu de `docs/` — single source of truth
  via Astro Content Collections + sync script.
- **JAMAIS** bumper la version de l'app Tauri / Cargo.toml / web/ /
  extension/ / web-team/ : C33 vit dans son propre univers `site/` avec
  son propre tag `site-v0.1.0`.
- **JAMAIS** utiliser `root@80.11.20.55` dans le CI ou dans le code.
  Toujours via user `deployer` dédié avec clé SSH propre. Le SSH root
  ne sert QUE pour le provisioning initial one-shot.
- **JAMAIS** commit la clé SSH privée, ni l'IP serveur dans les fichiers
  du repo. Tout passe par GitHub Secrets.
- **TOUJOURS** respecter `prefers-reduced-motion` + fallbacks WebGL.
- **TOUJOURS** réutiliser les wordings C32 (value prop, taglines persona)
  — cohérence produit ↔ site critique.
- **TOUJOURS** suivre `docs/operations/site-deploy.md` pour C33.6 — pas
  d'improvisation sur le provisioning serveur ou la config nginx.
- **DEMANDER** si tu hésites sur :
  - L'email Thibault pour Let's Encrypt (probablement
    `thibault@brilliantstudio.co`, à confirmer).
  - La forme du form `/cloud/` (mailto simple OK pour v0.1.0).
  - Le wording Hero (réutilise C32 value prop par défaut).
  - Avant de désactiver le login root SSH (étape optionnelle §3.6 doc) :
    valider avec Thibault qu'il a une session backup.

Bonne mission. Commence par C33.1 (bootstrap), puis C33.2 Hero (le wow
factor visuel), puis C33.4 doc (l'utilité brute), puis C33.3 sections
3D animées (le polish), puis C33.5+C33.6+C33.7 (ship).
```

---

## Notes pour Thibault

- Sprint 5-7 jours. Plus visuel et créatif que les sprints précédents.
- Au retour, comme d'habitude : `git log --oneline -15` + on review
  ensemble avant tag site-v0.1.0.
- **Décision domaine à prendre rapidement** : sobr.ia ou sobria.fr ?
  Recommandation Cowork = sobria.fr pour cohérence pitch souverain
  + prix bas + AFNIC FR. Mais le branding sobr.ia est plus mémorable
  côté IA. À toi.
- **Comptes Cloudflare** à créer si pas déjà fait :
  - Cloudflare Pages (gratuit illimité).
  - Cloudflare DNS (gratuit) pour pointer le domaine.
  - Compte AFNIC ou OVH pour acheter `sobria.fr` (~10€/an).
- **Test critique avant tag** : ouvre le site en mode incognito sur ton
  tel, vérifie que :
  - Le Hero 3D tourne sur mobile sans tuer la batterie.
  - La doc est lisible.
  - Le bouton "Cloud bientôt" mène à un form qui marche.
- Après site-v0.1.0 → **v1.0 candidature data.gouv.fr** : le site sera
  ton asset principal dans le dossier candidature.
