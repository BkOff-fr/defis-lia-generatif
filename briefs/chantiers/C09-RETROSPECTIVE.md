# Rétrospective — Chantier C09 (Intégration Tauri + UI desktop v0.2)

> **Date** : 13 mai 2026
> **Sprints couverts** : S6 (UI MVP, façade IPC, écrans M2 + M7 + M3 + M5 + M8)
> **Statut** : jalon stable, taggé `v0.2.0-estimer`.
> **Référence brief** : `briefs/chantiers/C09-tauri-integration.md` + `C09-PROMPT-CLAUDE-CODE.md`.

---

## Ce qui a été livré

### Vue d'ensemble

L'application Tauri ouvre une fenêtre sur l'écran *Estimer*, exécute une vraie estimation Monte-Carlo via IPC, journalise l'entrée dans le ledger d'audit ACID, et expose **cinq écrans fonctionnels + quatre stubs documentés + un écran Paramètres** — le shell de navigation est complet, plus un seul `404` dans le rail.

```
┌────────────────────────────────────────────────────────────────────┐
│                                                                    │
│   SvelteKit 2 + Svelte 5 runes                                     │
│   ├─ Design System v2 (ink / lime / ivory · Instrument Serif       │
│   │   italic + Geist + JetBrains Mono · 8 WOFF2 self-host)         │
│   ├─ Wrapper IPC typé `web/src/lib/api.ts` (6 commandes)           │
│   ├─ 10 routes :                                                   │
│   │    /          Estimer (M2)            ✓ IPC réel               │
│   │    /workbench Workbench (M3)          ✓ IPC réel               │
│   │    /comparer  Comparer (M5, M15 UI)   ✓ IPC réel (fan-out)     │
│   │    /journal   Journal d'audit (M7)    ✓ IPC réel (4 cmds)      │
│   │    /methodo   Méthodologie (M8)       ✓ statique               │
│   │    /parametres Paramètres             ✓ IPC `meta_info`        │
│   │    /simuler   Simuler (M4)            ⚠ stub <ComingSoon>      │
│   │    /importer  Importer (M10)          ⚠ stub                   │
│   │    /territoire Géoloc (M9+M12)        ⚠ stub                   │
│   │    /exporter  Exporter (M6)           ⚠ stub                   │
│   └─ 12 tests Playwright « no-mock contract »                      │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

### Commits ordonnés

| # | Hash | Titre |
|---|------|-------|
| C09.A | `e673758` | feat(app): wrapper IPC typé pour sobria-app |
| C09.A | `c923b7e` | chore(web): align eslint deps and add prettierignore |
| C09.B | `dfd5cd3` | feat(app): écran Estimer immersif + design system v2 |
| C09.B | `8e44e9a` | fix(web): migrate lucide-svelte → @lucide/svelte (svelte 5 runes) |
| C09.B | `0c5d28f` | chore(web): add favicon.svg |
| C09.B | `c6d1a32` | fix(web): bins log-normales + tokenizer FR + licences fonts + e2e no-mock |
| C09.B | `0aaf87c` | fix(web): format numérique en chiffres significatifs + scroll vers résultat |
| C09.B | `1fc3157` | fix(web): auto-rescale unités (g→mg→µg, Wh→mWh→µWh, L→mL→µL) |
| C09.B | `33311e0` | fix(web): hypothèses en cartes empilées, plus de débordement |
| C09.D | `13d04b8` | feat(app): écran Journal d'audit (chantier C09.D) |
| C09.D | `67a7753` | fix(web): alignement des colonnes du Journal d'audit |
| C09.E | `6ca0b21` | feat(app): écran Méthodologie (M8) — clôt les liens /methodo |
| C09.E | `4ebaaf9` | feat(app): écran Workbench (M3) — exploration du référentiel |
| C09.E | `a4ad793` | feat(app): écran Comparer (M5) — matrice + score composite |
| C09.E | `0601fa0` | feat(app): comparer par prompt + visualisation contributions + CTA Estimer→Comparer |
| C09.E | `085dfca` | feat(app): Comparer redessiné façon M15 (verdict + cards) + fix bouton Composer |
| C09.E | `3442b87` | feat(app): Paramètres fonctionnel + stubs Simuler/Importer/Territoire/Exporter |

### Métriques

| Métrique | Valeur |
|---|---|
| Routes implémentées | 10 / 10 (6 fonctionnelles + 4 stubs) |
| Composants Svelte | 5 (Composer, ResultBlock, HypothesisBlock, ComingSoon, +layout) |
| Tests Playwright | **12 passed (5 s)** |
| Erreurs `npm run check` | **0** / 3 784 fichiers |
| Lint `npm run lint` | **OK** (prettier + eslint) |
| Polices auto-hébergées | 8 WOFF2 (~155 ko total) |
| Tokens design system | ~80 (couleurs / type / spacing / radii / durées) |
| Lignes Svelte / TS livrées | ~5 800 |
| Commits | 17 |

---

## Décisions méthodologiques notables

### 1. Aucun mock côté front, ledger as source of truth

CLAUDE.md §13 imposait **zéro mock**. Conséquence : un test Playwright dédié vérifie sur chaque écran fonctionnel que, hors contexte Tauri, l'UI **refuse de servir un faux résultat** et affiche une bannière `tauri_unavailable` explicite avec mention `cargo run -p sobria-app`. Ce contrat est verrouillé par `tests/*.spec.ts` (12 specs).

Bénéfice : impossible d'introduire silencieusement un fallback qui mentirait à l'utilisateur sur la véracité des chiffres.

### 2. Design system v2 (ink/lime/ivory) plutôt que la maquette v1 (Inter/#0d1117)

La maquette texte initiale (`docs/ux/MAQUETTE-UI-TEXTUELLE.md`) proposait Inter + palette `#0d1117 / #3fb950`. Le handoff Claude Design (`sobr-ia-design-system/`) a livré une direction beaucoup plus éditoriale : **Instrument Serif italic** pour les chiffres et titres, **Geist** pour l'UI, **JetBrains Mono** pour les hashes ; palette **ink `#0a0d0b` + lime `#c5f04a` + ivory `#f0ece3`**. On a basculé là-dessus, avec auto-hébergement des fontes pour respecter `default-src 'self'`.

### 3. Bins log-normales dans le DTO + payload audit (option A1)

Plutôt que de re-fitter une analytique côté TS, on a fait remonter dans `IndicatorValue.bins: Option<DistributionBins>` (au niveau `sobria-core`, donc journalisé dans l'audit ledger) un histogramme équi-width de 50 bins issu directement des 10⁴ tirages Monte-Carlo. L'UI dessine la vraie courbe (queue droite log-normale). Coût : ~600 B / entrée d'audit. Rétro-compat : `Option<>` + `#[serde(default)]`.

Décision actée avec Cowork en cours de chantier (cf. discussion sur la colinéarité des indicateurs).

### 4. Tokenizer FR à 3,3 chars/token (heuristique pré-v0.3)

`prompt.length / 3.3` au lieu de `4` (ratio anglo-saxon par défaut). Gain ~25 % de précision pour les utilisateurs FR. Tooltip explicite que le tokenizer réel (BPE / tiktoken-wasm) arrivera en chantier outillage C10/C11.

### 5. Auto-rescale d'unité par P50 médian

`pickScale(p50, baseUnit)` choisit l'unité d'affichage la plus naturelle (kg/g/mg/µg/ng CO₂eq, etc.) et l'applique à P5/P50/P95 + axes — cohérence inter-percentile garantie. Empêche les `0,00 g` sur les petits modèles.

### 6. Pivot Nutri-Score → design M15 (verdict + cards)

Premier essai d'un barème A-F avec poids ajustables : **mathématiquement faux**. Les 3 indicateurs sont colinéaires (Energy → CO₂eq via IF constante, Energy → Water via WUE constante) tant que le datacenter est partagé. Re-design complet du Comparer selon `preview/34-m15-comparator.html` : **verdict éditorial « X est Y× plus sobre que Z »** + cards par modèle avec barres normalisées. Plus honnête, plus lisible, scalable à N modèles.

À débloquer en v0.3 : mapping provider → datacenter (PUE / WUE / IF par fournisseur) — les axes divergeront vraiment.

### 7. Stubs `<ComingSoon>` plutôt que pages cachées

Les 4 routes du rail qui dépendaient d'IPC non encore exposés (Simuler, Importer, Territoire, Exporter) ont reçu chacune une page stub qui **affiche explicitement la liste des IPC Rust attendus** + le chantier prévu + les EF du CDC couvertes. Le rail est complet et honnête.

---

## Ce qui a été appris / piégeux

### Pièges techniques résolus

1. **`lucide-svelte` (legacy Svelte 3/4) crashait Vite** en runes mode → migré vers `@lucide/svelte` (nouveau namespace officiel Svelte 5). Audit deps : code applicatif déjà runes-clean, `svelte-i18n@4` flagué pour Svelte 5 mais non importé.
2. **Cache Vite `optimizeDeps`** parfois sale après changement de dep → ajout d'un script `npm run clean` qui nuke `.vite + .svelte-kit + build + test-results + playwright-report`.
3. **Playwright workers parallèles** se masquaient mutuellement quand Vite re-optimisait → `workers: 1` (séquentiel, 5 s pour 12 tests, acceptable).
4. **`<td>{display: inline-flex}`** cassait `table-layout` du tableau du Journal — flex sur span enfant + `table-layout: fixed` avec largeurs explicites.
5. **`$app/stores`** ne résolvait pas dans le tsconfig SvelteKit généré (versionning) → bascule sur `window.location.pathname` + `popstate` listener autonome dans `+layout`.
6. **Format numérique `maximumFractionDigits: 2`** affichait `0,00` pour les petits modèles → bascule vers `maximumSignificantDigits: 3` + auto-rescale d'unité.

### Décisions de design rejetées en cours de route

- Score composite à poids ajustables → trompeur, indicateurs colinéaires.
- Barème Nutri-Score A-F → même problème, même cause.
- Heatmap normalisée par colonne du Comparer → remplacée par cards éditoriales pour gain de lisibilité.

### Hypothèses non vérifiées

- **Mapping provider → datacenter** côté Rust : indispensable pour que les 3 axes (Énergie / CO₂eq / Eau) divergent par modèle. Sans ça, le comparateur reste dominé par `N_params` et la note composite n'a pas de sens.
- **Validation paper-based** : aucun modèle au statut `Validated` dans `model_presets.rs`, tous en `Indicative` ou `Extrapolated`. Calibration vs Luccioni 2023 / Patterson 2021 / EcoLogits 2024 prévue en C07 (déjà ouvert).

---

## Reste à faire (chantiers ultérieurs)

### Court terme (C10 — outillage)

- **`run_scenario(req)`** dans `sobria-estimator` pour la route `/simuler` (projection temporelle 5 ans).
- **`render_report_pdf(template, payload, path)`** + `render_notebook_qmd` pour `/exporter`.
- **Tokenizer BPE** (tiktoken-wasm ou rust-bpe via WASM) pour remplacer l'heuristique `prompt.length / 3.3`.
- **Loader Markdown front** pour tirer le glossaire `docs/methodology/GLOSSAIRE.md` directement (zéro duplication contenu).
- **IPC `get_audit_entry(id)`** pour enrichir le drawer du Journal avec le payload complet (équivalents, hypothèses individuelles).

### Moyen terme (C11 — entreprise / RSE)

- **`parse_usage_log` / `apply_mapping` / `estimate_batch` / `export_rse_report`** pour `/importer` (cf. EF-M10-01..05).
- **Loader CSV / Parquet** + détection schéma + anonymisation locale.
- **Rapport CSRD-ready 12 pages** (livrable L4).

### Long terme (C12 — Tier 1 défi)

- **`list_iris` / `load_iris_consumption` / `detect_datacenters_candidates`** pour `/territoire` (cf. EF-M9-01..05 + EF-M12-01..07).
- **GeoJSON IRIS rendu** côté front (D3 ou MapLibre offline) — pas Leaflet, CDN tiles incompatibles avec la CSP `default-src 'self'`.
- **Croisement ComparIA × IRIS** : volume LLM par bassin de population.

### Validation méthodologique (C07 ré-ouvert)

- **Mapping provider → datacenter** : OpenAI → US-East PUE 1,3 / IF 412 g/kWh ; Anthropic → US-West PUE 1,15 / IF 200 ; Mistral → FR PUE 1,2 / IF 56 ; Google Hamina → FI PUE 1,1 / IF 71. Sans cela, le comparateur n'a pas de pertinence.
- **Calibration paper-based** des modèles `Indicative` → `Validated`.
- **Per-token coefficient** par modalité (texte / vision / audio) si on garde l'angle texte-only en v1.0.

---

## Métriques de qualité

| Item | Cible CDC | Mesure |
|---|---|---|
| Temps de calcul moyen estimation | < 50 ms | non mesuré (à instrumenter post-tag) |
| Empreinte binaire desktop | < 20 Mo | non mesuré (cargo bundle non lancé) |
| Couverture tests Rust | ≥ 80 % | hors scope C09 (couverte par C04-C08 + C07) |
| Conformité RGAA AA | obligatoire | a11y partielle : `aria-sort`, focus auto drawer, Esc fermeture, alt sur tous les SVG décoratifs. **Audit axe-playwright à exécuter en C10.** |
| i18n FR + EN | obligatoire | FR uniquement à ce stade — EN à programmer (`@inlang/paraglide`, C10). |
| Privacy by design | obligatoire | ✓ aucun appel réseau externe sauf au runtime (Tauri local, polices auto-hébergées, IPC localhost). |

---

## Tag

`v0.2.0-estimer` (jalon C09 figé, app desktop multi-écrans avec Estimer + Journal + Workbench + Comparer + Méthodologie + Paramètres + 4 stubs documentés).

---

*Rédigé le 13 mai 2026 après le dernier commit C09 (`3442b87`).*
