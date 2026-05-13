# Sobr.ia — Design System

> *Mesurez la sobriété de votre IA générative.*
> *Make generative AI's footprint visible.*

**Sobr.ia** est une application native multi-plateforme (Rust + Tauri 2 + SvelteKit), une extension navigateur (WebExtension MV3) et un dataset ouvert qui mesurent et visualisent l'impact environnemental de l'usage des LLMs. Le projet est candidat au défi *« L'impact environnemental de l'IA générative »* (data.gouv.fr) et applique le référentiel **AFNOR SPEC 2314** sur l'IA frugale.

Ce projet contient le **design system Sobr.ia** : fondations visuelles, type, couleurs, composants, et UI kits de l'application desktop et de l'extension navigateur.

---

## Sources de référence

Tout le système est dérivé des documents projet suivants :

- **Repo GitHub** : `BkOff-fr/defis-lia-generatif` (main)
  - `README.md`, `CLAUDE.md` (contexte produit & contraintes)
  - `docs/CAHIER-DES-CHARGES-v1.0.md` (CDC v1.1 — 11 modules)
  - `docs/ROADMAP.md` (12 sprints S0-S12)
  - `docs/adr/ADR-0002-sveltekit.md` (Skeleton CSS custom)
  - `docs/adr/ADR-0008-observable-plot.md` (dataviz)
- **Maquette UI textuelle** (storyboard ASCII de référence pour les sprints S6-S8) — fournie par le porteur du projet (Thibault) et figée comme guide d'implémentation Svelte. Couvre 9 écrans application + extension + onboarding + composants Svelte transversaux.

> ⚠️ Le projet réel est en phase de cadrage (S0) — aucun code applicatif n'est encore écrit dans le repo. Le design system est donc **construit à partir de la maquette textuelle et des spécifications visuelles** du CDC, et non à partir d'un codebase existant. Toute itération devra être validée contre l'implémentation Svelte/Skeleton réelle dès qu'elle existera.

---

## Identité produit

| Élément | Valeur |
|---|---|
| **Nom** | Sobr.ia |
| **Étymologie** | *Sobriété* + *IA* — l'outil incarne son sujet |
| **Tagline FR** | « Mesurez la sobriété de votre IA générative » |
| **Tagline EN** | « Make generative AI's footprint visible » |
| **Stack** | Rust + Tauri 2 + SvelteKit + Skeleton CSS custom + Observable Plot |
| **Plateformes** | Win / macOS / Linux (1ʳᵉ classe), Web/Wasm (démo), Android/iOS (bonus) + Extension Chrome/Firefox MV3 |
| **Licences** | MIT (code), Etalab 2.0 (données), CC-BY 4.0 (docs), CC-BY-SA 4.0 (identité) |
| **Domaine** | sobr.ia / sobria.fr |

---

## Index du design system

Manifest racine — tout est à plat sauf les UI kits.

| Fichier / dossier | Rôle |
|---|---|
| `README.md` | Ce fichier — contexte, content fundamentals, visual foundations, iconography, index |
| `SKILL.md` | Agent Skill packaging (cross-compatible Claude Code) |
| `colors_and_type.css` | Tokens : couleurs (dark + light), typographie, espacements, rayons, durées + composants atomiques (.btn .input .badge .card) |
| `fonts/` | Inter Variable + JetBrains Mono — auto-hébergement en production (CDC §3.2) |
| `assets/` | Logo SVG + favicon ; pas d'illustrations externes (visuel généré par dataviz) |
| `preview/` | Cards du Design System tab : type, couleurs, spacing/radii, composants, brand |
| `ui_kits/app/` | UI kit application desktop Tauri+Svelte — 5 écrans interactifs (Estimer, Workbench, Comparer, Simuler, Journal) |
| `ui_kits/extension/` | UI kit extension navigateur MV3 — badge flottant + popover détail injectés sur un hôte LLM tiers |

### Démarrer

```bash
# UI kit application desktop
open ui_kits/app/index.html

# UI kit extension navigateur
open ui_kits/extension/index.html
```

### Reuse pattern

Tous les écrans consomment `colors_and_type.css` à la racine via `<link rel="stylesheet" href="../../colors_and_type.css">`. Les composants JSX sont exposés sur `window` pour cross-file sharing entre scripts Babel.

---

## Personas (rappel)

- **P1 — Claire, chargée RSE** : reporting carbone scope 3, rapports CSRD-ready.
- **P2 — Marc, agent Ecolab/ADEME** : scénarios à l'échelle nationale.
- **P3 — Léa, étudiante data journalisme** : viz citables + sources.
- **P4 — Thomas, dev SaaS** : matrice comparative avant déploiement.
- **P5 — Sami, utilisateur curieux** : badge live extension, bilan hebdo.

---

## Index du design system

### Fondations
- [`colors_and_type.css`](./colors_and_type.css) — tokens CSS (couleurs, type, espacements, rayons, ombres, durées)
- [`fonts/`](./fonts/) — Inter Variable + JetBrains Mono (chargés via Google Fonts CDN — voir caveats)
- [`README.md`](./README.md) — ce fichier (CONTENT FUNDAMENTALS + VISUAL FOUNDATIONS + ICONOGRAPHY ci-dessous)

### Assets
- [`assets/logo.svg`](./assets/logo.svg) — logo principal (wordmark + glyphe feuille)
- [`assets/logo-mark.svg`](./assets/logo-mark.svg) — glyphe seul (favicon, badge extension)
- [`assets/icons/`](./assets/icons/) — set d'icônes Lucide (CDN) + iconographie projet (état local, alertes, score énergie)

### Aperçus design system (cartes)
- [`preview/`](./preview/) — fiches type, couleurs, spacing, composants, brand (visibles dans l'onglet Design System)

### UI kits
- [`ui_kits/app/`](./ui_kits/app/) — application desktop Tauri 2 + SvelteKit (9 écrans : Estimer, Workbench, Comparer, Simuler, Importer, Géoloc, Exporter, Journal, Méthodologie + Onboarding)
- [`ui_kits/extension/`](./ui_kits/extension/) — extension navigateur MV3 (badge overlay + popover détail) sur fonds simulés ChatGPT / Claude / Mistral / Gemini

### Skill
- [`SKILL.md`](./SKILL.md) — point d'entrée Agent Skills, cross-compatible Claude Code

---

## CONTENT FUNDAMENTALS

Sobr.ia s'adresse à des publics très variés (RSE, agents publics, data journalistes, devs, grand public curieux) avec une exigence constante : **être lisible sans simplifier la science**. Le ton est sobre, factuel, militant *par exemplarité* — pas par exhortation.

### Voix & posture

- **Vouvoiement** par défaut côté app et docs (« Vous saisissez un prompt… », « Saisissez votre premier prompt »). Le tutoiement n'est utilisé que dans les CTA d'onboarding très courts (« Essayez : "Résume-moi…" »).
- **Phrases courtes**, voix active, jargon scientifique explicité dès la première occurrence (lien vers le glossaire). Exemple : « PUE = Power Usage Effectiveness, ratio énergie totale / IT d'un datacenter. »
- **Honnêteté scientifique avant tout** — chaque valeur est affichée avec son intervalle d'incertitude P5-P95, et les sources sont à un clic. Jamais de chiffre nu.
- **Bilingue FR/EN obligatoire** sur la doc et les écrans clés (sélecteur de langue en haut-droite, FR par défaut). Pas d'écran en anglais seul.

### Casing et ponctuation

- **Titres d'écran** : *Sentence case* avec capitale initiale uniquement (« Estimer un prompt », « Workbench — Référentiel des modèles », « Journal d'audit »).
- **Boutons** : commencent par un verbe à l'infinitif (« Estimer », « Comparer », « Sauvegarder », « Exporter PDF »). Pas d'ALL CAPS, pas de Title Case.
- **Labels de champ** : *SMALL CAPS* ou bold avec préfixe glyphe ▣ pour les sections (« ▣ MODÈLE », « ▣ PROMPT », « ▣ DATACENTER »).
- **Nombres** : virgule décimale FR (`2,14 g`), espace fine insécable avant unité (`4,87 Wh`, `412 gCO₂eq/kWh`). Versions EN : point décimal.
- **Unités** : toujours présentes, jamais omises. Indices Unicode (`CO₂eq`, `m²`).
- **Intervalles** : `2,14 g  [1,68–2,74]` — tiret demi-cadratin entre min et max, intervalle entre crochets, P5–P95 annoté visuellement à proximité.

### Vocabulaire signature

| Terme | Usage |
|---|---|
| **Sobriété** | Cœur de la marque. Toujours préféré à « efficacité énergétique » ou « écologie ». |
| **Estimer / Estimation** | Toujours préféré à « calculer ». Reflète l'humilité scientifique. |
| **Hypothèses** | Section explicite et cliquable sous chaque résultat. |
| **P5–P95** | Notation systématique de l'incertitude. |
| **100 % local** | Mantra présent en bandeau permanent et dans chaque CTA d'import / extension. |
| **Référentiel** | Le dataset interne (jamais « base de données » côté UI). |
| **Équivalents parlants** | Section qui traduit les chiffres en métaphores familières (km voiture, douche). |
| **Frugal / frugalité** | Présent dans la doc, jamais comme reproche, toujours comme posture. |

### Exemples de copywriting (extraits réels de la maquette)

- Bandeau permanent : `🔒 100 % local, aucune donnée envoyée  •  Référentiel YYYY.MM.DD  •  3 alertes`
- Onboarding étape 1 : *« Sobr.ia — Mesurez la sobriété de votre IA. 100 % local. Méthodologie AFNOR SPEC 2314. Open source. »*
- État de calcul : *« 10 000 simulations Monte-Carlo en cours… »* (objectif < 200 ms perçus)
- Équivalents : *« ≈ 17 m en voiture thermique • ≈ 0,5 s de douche chaude »*
- Hypothèse cliquable : *« ε_decode = 1,8 mJ/token  [HF AI Energy Score, 2026] »*
- Géoloc : *« IP détectée (locale) : 81.x.x.x → France, Île-de-France. Confiance : ▓▓▓▓▓░ Élevée »*
- Audit : *« Journalisé dans l'audit ledger — hash 7a3f9b… »*

### Émoji et glyphes

- **Émoji UNICODE autorisés** uniquement comme **icônes de navigation** dans la sidebar et les titres d'écran (🧮 Estimer, 📚 Workbench, ⚖ Comparer, 📈 Simuler, 📥 Importer, 🌍 Géolocaliser, 📤 Exporter, 🗂 Journal, 📖 Méthodologie, ⚙ Paramètres, ❓ Aide). C'est un choix assumé du CDC et de la maquette — sobriété, lisibilité immédiate, zéro dépendance icône.
- **Glyphes ASCII** dans la maquette (`▣`, `«»`, `┃`, `╌╌`) sont des conventions de la maquette texte uniquement — ne PAS les rendre tels quels dans l'app. Les transcrire en composants Svelte (cartes, boutons, bordures).
- **Pas d'émoji décoratif** dans le corps des textes ou dans les boutons. Les émoji disent toujours quelque chose : un module, un statut (✓ validé, ⚠ alerte, 🔒 local, 🌱 sobr.ia signature).
- **Caractères Unicode pour la dataviz** : `▓ ▒ ░` pour score énergétique en bloc, `─ ┃ ┌ └` pour bordures de tableau dans les exports texte / ledger NDJSON.

### Tone matrix

| Surface | Tonalité | Exemple |
|---|---|---|
| **Onboarding** | Accueillant, court, rassurant | « 🌱 Sobr.ia — Mesurez la sobriété de votre IA. » |
| **Résultats** | Factuel, sourcé, sans dramatisation | « 2,14 g CO₂eq [1,68–2,74] ≈ 17 m en voiture » |
| **Erreurs** | Diagnostique + suggestion d'action | « Mix électrique indisponible — bascule sur la moyenne annuelle 2025 » |
| **Bandeaux système** | Statique, informationnel | « Données entreprise importées — Mode entreprise actif » |
| **Méthodo / docs** | Pédagogique, sourcé, références BibTeX | « Embodied amorti = 0,02 gCO₂eq/req [Gupta et al., 2022] » |
| **Policy brief / rapport** | Cadré CSRD, formel, bilingue | « Conformément à l'AFNOR SPEC 2314… » |

---

## VISUAL FOUNDATIONS

### Posture esthétique

> **Frugalité visuelle.** Pas de skeuomorphisme, pas d'ornementation, pas de gradients enveloppants. La dataviz est généreuse, le chrome est minimal. L'outil incarne son sujet : son interface elle-même doit être sobre.

### Couleurs

**Mode sombre par défaut** (économies écran OLED — choix produit du CDC §spec visuelles). Mode clair en option.

```
Mode sombre
  fond        #0d1117   ← background principal app
  surface     #161b22   ← cartes, panneaux, sidebar
  surface-2   #1c2128   ← surface élevée (modal, dropdown)
  border      #30363d   ← bordures 1 px subtiles
  text        #c9d1d9   ← texte principal
  text-muted  #8b949e   ← légendes, labels secondaires
  text-faint  #6e7681   ← placeholders, désactivé

Mode clair
  fond        #ffffff
  surface     #f6f8fa
  surface-2   #ffffff
  border      #d0d7de
  text        #24292f
  text-muted  #57606a
  text-faint  #8c959f

Accents sémantiques (communs aux 2 modes)
  green       #3fb950  ← sobriété, succès, score A/B
  amber       #d29922  ← vigilance, score C/D
  red         #f85149  ← alerte, erreur, score E/F, audit compromis
  blue        #58a6ff  ← liens, focus, info neutre
  purple      #a371f7  ← scénarios alternatifs (rare)
```

**Dataviz** : palettes **Viridis** (continu, daltoniens-friendly) et **Cividis** (alternative haute lisibilité). Pour le score énergétique : gradient vert sobr.ia → ambre → rouge. Pour les bandes d'incertitude P5–P95 : surface accent à 24 % d'opacité, ligne médiane P50 pleine.

**Pas de gradient enveloppant** sur les cartes ou les boutons. Le seul gradient autorisé est dans la dataviz (heatmaps, choroplèthes, gradient de score énergétique).

### Typographie

- **Inter Variable** — toute l'UI (corps, titres, boutons, labels). Poids utilisés : 400 (corps), 500 (labels, boutons), 600 (titres section), 700 (titres écran).
- **JetBrains Mono** — chiffres dans les métriques chiffrées (`2,14 g`), code, hashes audit (`7a3f9b…`), valeurs tableau, badges chiffrés extension. C'est le choix imposé pour que l'œil aligne immédiatement les colonnes de tableaux numériques.
- **Tabular nums** (`font-variant-numeric: tabular-nums`) actif partout où Inter est utilisé pour afficher des chiffres en colonne.

Échelle type (16 px base, ratio 1.25 / *major third*) :

```
display     32 px / 40   weight 700   titre marketing / onboarding
h1          24 px / 32   weight 700   titre d'écran
h2          20 px / 28   weight 600   titre de section
h3          16 px / 24   weight 600   sous-section / titre de carte
body        14 px / 20   weight 400   corps de texte UI
body-sm     13 px / 18   weight 400   labels, légendes
caption     12 px / 16   weight 500   méta, hashes, timestamps
metric-xl   28 px / 32   weight 600   chiffre principal (CO₂eq dans MetricCard) — Mono
metric      18 px / 24   weight 500   chiffre secondaire — Mono
```

### Espacements (grille 4 px)

`4 / 8 / 12 / 16 / 24 / 32 / 40 / 64` — tokens `--space-1` à `--space-9`. Pas de valeur intermédiaire libre.

### Coins arrondis

- **4 px** sur les inputs et boutons (`--radius-sm`).
- **8 px** sur les cartes et conteneurs (`--radius-md`).
- **12 px** sur les modals / popovers (`--radius-lg`).
- **9999 px** pour les badges pilule (`--radius-pill`).
- **0** sur les barres système (bandeau bas), tableaux denses, ledger.

### Bordures vs ombres

> **Bordures 1 px subtiles, pas d'ombre portée massive.** C'est un choix explicite du CDC.

Les cartes sont délimitées par `1px solid var(--border)`. Les modals et popovers ont une ombre douce **uniquement** pour la profondeur Z (`0 8px 24px rgba(0,0,0,0.4)` en sombre, `0 8px 24px rgba(0,0,0,0.12)` en clair) — jamais d'ombre interne, jamais de glow coloré.

### Animations

- **Durées 150–250 ms**, easing `cubic-bezier(0.2, 0.8, 0.2, 1)` (ease-out perceptible).
- **Désactivables** via `prefers-reduced-motion` (a11y obligatoire).
- **Transitions admises** : opacity, transform (translate, scale), background-color. Pas de morphing, pas de bounce, pas de parallax.
- **Apparition de résultat Monte-Carlo** : fade-in 200 ms + petit slide-up 4 px, c'est tout.

### États interactifs

```
hover    surface s'éclaircit (alpha +6 %), bordure s'éclaircit (alpha +12 %)
active   surface s'assombrit légèrement (alpha -4 %)
focus    contour 2 px var(--blue) avec offset 2 px (visible clavier)
disabled opacity 0.4, cursor not-allowed, pas de hover
loading  spinner 12 px JetBrains Mono `…` clignotant, label « Monte-Carlo en cours… »
```

Pas de press = shrink. Pas de hover = couleur radicalement différente. Les changements sont subtils — l'utilisateur RSE doit pouvoir prendre des screenshots professionnels.

### Layout

- Shell : sidebar gauche fixe (200–240 px), zone de contenu fluide, **bandeau bas permanent** (« 🔒 100 % local »).
- Largeur cible : **1024 × 640 px** mini, optimisé jusqu'à 1920 × 1080. Au-dessus de 1440 : marge latérale auto sur le contenu (max-width 1280 px).
- Mobile : layout vertical, drawer pour la sidebar, bottom-sheet pour les actions multiples.
- **Densité moyenne-haute** — tableaux de référentiel à 14 px/20, padding cellulaire 8/12. Sobr.ia n'est pas un produit grand public spectaculaire, c'est un outil d'analyse.

### Imagerie

- **Pas d'illustrations vectorielles décoratives.** Le seul contenu non-data est le wordmark, le glyphe feuille, et les diagrammes méthodologie (Sankey, médaillon Copper/Silver/Gold).
- **Pas d'arrière-plans photographiques.** Surfaces neutres uniquement.
- **Cartes choroplèthes** : Mapbox-style sobre, fond `var(--surface)`, frontières `var(--border)`, remplissage Viridis selon intensité carbone.
- **Émoji système** comme seul élément un peu ludique (sidebar nav) — encadré, contrôlé, pas d'émoji décoratif libre.

### Transparence et blur

- **Backdrop blur** uniquement sur le modal overlay (8 px) en mode sombre, pour faire ressortir le modal sans masquer complètement le contexte.
- **Pas de glassmorphism**. Pas de transparence sur les cartes.

### Composants Svelte transversaux à isoler (cf. maquette §Composants)

| Composant | Rôle | Réutilisé dans |
|---|---|---|
| `<MetricCard>` | Affiche une métrique avec intervalle P5–P95 | Estimer, Comparer, Workbench |
| `<UncertaintyBand>` | Bande P5–P95 sur un graphe | Estimer, Simuler |
| `<HistogramMC>` | Histogramme Monte-Carlo (N=10⁴) | Estimer |
| `<SankeyEnergy>` | Sankey énergétique (compute → cooling → losses) | Estimer, Workbench |
| `<HeatmapModels>` | Heatmap comparative normalisée | Comparer |
| `<ChoroplethMap>` | Carte mix électrique | Géoloc, Simuler |
| `<SourcePopover>` | Pop-up source cliquable (DOI, URL) | Partout |
| `<EquivalentBadge>` | « ≈ X km voiture » | Estimer, Simuler |
| `<LocalIndicator>` | Bandeau « 100 % local » | Shell |
| `<LedgerHash>` | Hash audit tronqué (`7a3f9b…`) cliquable | Estimer, Journal |
| `<EnergyScore>` | Score A-F en blocs `▓░` | Workbench, Fiche modèle |

### Variables CSS de référence

Voir [`colors_and_type.css`](./colors_and_type.css) pour la déclaration complète des tokens et de leurs usages sémantiques (`--fg-1`, `--fg-2`, `--bg`, `--surface`, `--accent-green`, `--metric`, etc.).

---

## ICONOGRAPHY

### Principes

Sobr.ia adopte **deux registres iconographiques distincts**, assumés et complémentaires :

1. **Émoji Unicode** pour la navigation principale (sidebar) et les titres d'écran. C'est un choix produit du CDC et de la maquette : zéro dépendance, lisibilité multi-plateforme, sobriété immédiate. Liste figée :
   - 🧮 Estimer · 📚 Workbench · ⚖ Comparer · 📈 Simuler · 📥 Importer
   - 🌍 Géolocaliser · 📤 Exporter · 🗂 Journal audit · 📖 Méthodologie
   - ⚙ Paramètres · ❓ Aide · 🌱 Logo / signature · 🔒 Sécurité / local
   - ✓ Validé · ⚠ Alerte · ▾ Dropdown · ← Retour · ↗ Lien externe
2. **Set d'icônes traits 1.5 px** (style Lucide / Heroicons outline) pour les actions secondaires : copier, partager, fermer, plus, moins, filtre, recherche, paramètres fins. Stroke `currentColor`, taille 16/20/24 px.

### Substitution flaggée

Le projet réel n'a **pas encore choisi de set d'icônes** — le CDC mentionne « Skeleton CSS custom léger uniquement » sans préciser. Ce design system propose **Lucide** (MIT, CDN, stroke 1.5 px, plus de 1400 icônes, cohérent avec l'esthétique trait minimal de Sobr.ia).

⚠️ **Substitution à valider avec Thibault** : si l'équipe préfère Heroicons, Phosphor (regular) ou Tabler Icons, l'échange est trivial — tous suivent la même grille et le même stroke. La feuille de style ne change pas, juste l'import.

→ Lucide est chargé via CDN dans les UI kits : `https://unpkg.com/lucide-static@latest/icons/<name>.svg`.

### Logos & assets

- [`assets/logo.svg`](./assets/logo.svg) — wordmark complet (glyphe feuille + « sobr.ia »).
- [`assets/logo-mark.svg`](./assets/logo-mark.svg) — glyphe feuille seul (favicon, badge extension, icône taskbar).
- Le logo n'a **pas encore d'identité figée** côté projet (cadrage S0). Le wordmark proposé ici est une interprétation cohérente avec la posture : étymologie *sobriété + IA* → feuille minimaliste 1.5 px stroke + wordmark JetBrains Mono / Inter mix, ton vert sobriété.
- ⚠️ **À valider avec Thibault**. Le projet prévoit que l'identité finale sera CC-BY-SA 4.0.

### Émoji vs Unicode dans la dataviz

- Score énergétique : blocs Unicode `▓ ▒ ░` (composent un score A-F lisible en table).
- État local / sécurité : `🔒` strictement réservé au bandeau permanent et aux mentions de privacy.
- Audit intègre : `✓ vérifiée` / `✕ compromis`.
- Pas de drapeaux nationaux émoji (utiliser les codes ISO + nom de pays).

---

## Caveats / questions ouvertes

1. **Pas de codebase applicatif réel** — le repo `defis-lia-generatif` est en cadrage. Tous les UI kits ici sont des **recréations spéculatives** basées sur la maquette texte et les spécifications visuelles du CDC. À reconfronter dès que les premières crates Svelte existent (S6).
2. **Identité visuelle (logo)** — pas figée côté projet, proposition à valider.
3. **Icônes traits** — Lucide choisi par défaut, à valider (Heroicons / Phosphor / Tabler équivalents).
4. **Fonts** — Inter Variable + JetBrains Mono **chargés depuis Google Fonts CDN**, pas auto-hébergés dans ce projet. Le CDC précise « auto-hébergées » côté app — il faudra rapatrier les TTF/WOFF2 dans `static/fonts/` à l'implémentation. *Voir le dossier [`fonts/README.md`](./fonts/README.md) pour les commandes.*
5. **Composants dataviz** (HistogramMC, SankeyEnergy, ChoroplethMap, HeatmapModels) — esquissés en SVG/CSS dans les UI kits, à remplacer par les vraies implémentations Observable Plot + D3 dès S7.
6. **i18n** — toute la copie est en FR dans ce kit. Doublure EN à produire à la passe S10.

---

*Design system v0.1 — référentiel d'identité Sobr.ia, aligné CDC v1.1 et maquette UI textuelle v1.0. Toute modification structurante = bump version + note dans CHANGELOG du repo applicatif.*
