# Prompt à donner à Claude Design — Sobr.ia (itération 2)

> **Usage** : copier le bloc « Prompt à coller » ci-dessous et le re-soumettre à Claude Design pour obtenir une version complète des composants. Le prompt fait référence à la maquette UI textuelle [`MAQUETTE-UI-TEXTUELLE.md`](MAQUETTE-UI-TEXTUELLE.md) qui doit être jointe ou copiée en pièce jointe.

---

## Prompt à coller

```
Tu travailles sur Sobr.ia, une application desktop+mobile+web (Tauri 2 + SvelteKit)
qui mesure et visualise l'impact environnemental de l'IA générative pour le
défi data.gouv.fr. Le projet est candidat sur la rigueur scientifique ET la
frugalité visuelle — c'est notre différenciant.

PREMIÈRE ITÉRATION : tu as déjà produit une base de composants. Cette deuxième
itération doit compléter la palette en couvrant TOUS les composants spécifiques
listés ci-dessous. Plusieurs composants dataviz manquent, notamment des
diagrammes Sankey, des histogrammes Monte-Carlo, des bandes d'incertitude et
une carte choroplèthe IRIS.

═══════════════════════════════════════════════════════════════════════════════
1. CONTRAINTES DESIGN SYSTEM (à respecter strictement)
═══════════════════════════════════════════════════════════════════════════════

PALETTE (mode sombre par défaut — économie énergétique écrans OLED) :
- Fond principal       : #0d1117
- Surfaces (cartes)    : #161b22
- Bordures subtiles    : #30363d (1 px)
- Texte primaire       : #c9d1d9
- Texte secondaire     : #8b949e
- Accent vert sobriété : #3fb950   (succès, valeurs basses, "bon")
- Accent ambre         : #d29922   (vigilance, valeurs moyennes)
- Erreur / haut impact : #f85149   (rouge, valeurs hautes, alertes)
- Accent info (lien)   : #58a6ff

PALETTE MODE CLAIR (alternative — utilisateur peut basculer) :
- Fond #ffffff, surface #f6f8fa, bordures #d0d7de, texte #24292f

TYPOGRAPHIE :
- UI : Inter Variable (web font auto-hébergée), poids 400-700
- Nombres et code : JetBrains Mono Variable
- Tailles : 12 / 14 / 16 / 20 / 28 / 40 px (Major Second)

ESPACEMENTS : grille 4 px — utiliser 4 / 8 / 16 / 24 / 40 / 64.

COINS ARRONDIS : 4 px (inputs), 8 px (cartes), 12 px (modales).

ANIMATIONS : 150-250 ms ease-out maximum. Désactivables via
`prefers-reduced-motion`.

ICONOGRAPHIE : lucide-icons stroke 1.5, jamais bicolore.

ACCESSIBILITÉ : RGAA AA / WCAG 2.1 AA obligatoire. Contraste ≥ 4.5:1 pour
texte, ≥ 3:1 pour grandes valeurs et bordures d'interaction. Tous les graphes
ont une description textuelle (aria-label) et un mode tableau alternatif
accessible par les lecteurs d'écran.

DATAVIZ : palettes daltoniens uniquement (Viridis, Cividis, ColorBrewer
"YlGnBu" inversé pour "moins c'est mieux"). Jamais de rouge/vert seul.

PHILOSOPHIE : "frugalité visuelle" — pas de skeuomorphisme, pas d'ombres
portées massives, pas de dégradés tape-à-l'œil. L'interface incarne le sujet
qu'elle traite. Inspiration : Linear, Vercel dashboard, Stripe Atlas, le tout
moins lumineux.

═══════════════════════════════════════════════════════════════════════════════
2. COMPOSANTS DATAVIZ MANQUANTS (priorité absolue)
═══════════════════════════════════════════════════════════════════════════════

2.1 — SankeyEnergy (D3-based)
─────────────────────────────
Flux énergétique d'un prompt :
  Compute (3.2 Wh) ──┐
                     ├─► PUE 1.3 ─► Total 4.87 Wh
  Idle (0.4 Wh) ─────┤
  Networking (0.1)  ─┘
- Largeur des flux proportionnelle à la valeur.
- Couleurs : Viridis du sombre au clair selon la valeur.
- Tooltip au survol avec la valeur exacte + intervalle P5-P95.
- Légende cliquable pour filtrer.
- Mode tableau alternatif a11y.

2.2 — HistogramMC (histogramme Monte-Carlo)
───────────────────────────────────────────
Restitue les 10 000 tirages Monte-Carlo d'une estimation :
- Histogramme à 30-50 barres.
- Lignes verticales pointillées sur P5, P50 (médiane), P95.
- Bande P5-P95 surlignée en arrière-plan (rectangle semi-transparent).
- Axe X : valeur de l'indicateur (avec son unité).
- Axe Y : densité (pas nombre brut).
- Annotation flottante : "P5 = 1.68 / P50 = 2.14 / P95 = 2.74 gCO2eq".

2.3 — UncertaintyBand (courbe avec bande d'incertitude)
───────────────────────────────────────────────────────
Pour les projections temporelles (simulateur de scénarios) :
- Ligne médiane (P50) en accent vert.
- Bande P5-P95 remplie semi-transparente autour.
- Axe X : temps (mois ou années).
- Axe Y : valeur cumulée de l'indicateur.
- Annotations sur les pics ou jalons (ex: "2030 — cumul N tonnes CO2eq").
- Variants : aires empilées si plusieurs scénarios comparés côte à côte.

2.4 — ChoroplethMap IRIS France (D3 + GeoJSON IRIS)
───────────────────────────────────────────────────
Carte de France au niveau IRIS (~50 000 polygones — performance critique !) :
- Projection Lambert 93 (officielle pour la France).
- Coloration par classe (quantiles 5-7 buckets) selon l'indicateur sélectionné.
- Palette Viridis (faible → fort).
- Possibilité de zoom (Île-de-France, AURA, Occitanie…) via picker.
- Hover : tooltip avec code IRIS, nom commune, valeur.
- Click : sélection IRIS, panneau latéral avec détails.
- Légende discrète bas-droite.
- Mode "outliers only" qui n'affiche que les 100 IRIS top consommation.
- Fallback raster (PNG) si performance vectorielle insuffisante.

2.5 — HeatmapModels (matrice modèles × indicateurs)
───────────────────────────────────────────────────
Matrice pour le comparateur (M5) :
- Lignes : modèles sélectionnés (2 à 8).
- Colonnes : indicateurs (Énergie, CO2eq, Eau, Embodied, Coût, Latency).
- Cellules colorées par valeur normalisée (palette divergente Viridis).
- Indication "données manquantes" : motif diagonal hachuré.
- Tri par colonne au clic header.
- Mode "normalisation" : par colonne (relatif à max), absolu, ou %.
- Annotations valeurs absolues dans la cellule (toggle).

2.6 — Treemap (consommation par catégorie)
──────────────────────────────────────────
Pour le workbench M3 : répartition des modèles par provider × taille × usage.
- Squarified treemap classique.
- 3 niveaux : provider → famille → modèle.
- Couleur selon l'indicateur dominant.
- Drill-down au clic.

2.7 — Ridge Plot (distribution modèles)
───────────────────────────────────────
Distributions empilées des Wh/req par modèle (workbench, vue dense).
- Une ligne par modèle, distribution KDE.
- Tri par médiane.
- Mode "normalize y" : chaque ligne à la même hauteur visuelle.

═══════════════════════════════════════════════════════════════════════════════
3. COMPOSANTS UI TRANSVERSAUX MANQUANTS
═══════════════════════════════════════════════════════════════════════════════

3.1 — MetricCard
────────────────
Carte affichant une métrique avec son incertitude.
Props : indicator (CO2eq | Energy | Water | …), p5, p50, p95, unit, sparkline?
État vide : "Pas encore estimé".
État chargement : skeleton shimmer doux.
État alerte : bordure ambre/rouge selon seuil.

3.2 — SourcePopover
───────────────────
Pop-over cliquable depuis une valeur ou hypothèse.
Affiche : titre source, auteurs, année, DOI/URL cliquable, citation
formatée APA, bouton "Copier la citation".

3.3 — EquivalentBadge
─────────────────────
Badge inline pour les équivalents parlants.
Ex: "≈ 17 m en voiture thermique" avec icône 🚗 (lucide).
Hover : source de l'équivalent.

3.4 — LedgerEntry
─────────────────
Ligne d'audit (Module M7).
Affiche : timestamp UTC court, modèle, tokens in/out, indicateur principal,
hash SHA-256 tronqué (cliquable → copie complète).
État : "intégrité ✓" en vert / "compromise" en rouge.

3.5 — HypothesisChip
────────────────────
Chip cliquable représentant une hypothèse utilisée.
Texte : clé courte (ex: "PUE 1.3"), couleur selon catégorie
(hardware / énergie / méthodologie).
Click : ouvre SourcePopover.

3.6 — LocalIndicator (bandeau permanent)
────────────────────────────────────────
Bandeau bas d'écran shell :
  🔒 100 % local • Référentiel YYYY.MM.DD • N alertes
- Discret mais toujours visible.
- Clic sur "alertes" → panneau de diagnostic.

3.7 — UncertaintyTooltip
────────────────────────
Tooltip standard pour toute valeur incertaine.
Format : "2.14 gCO2eq [1.68–2.74]" avec icône (i) info.
Hover prolongé : "Intervalle de confiance à 90 % issu de 10 000 simulations
Monte-Carlo (seed 42)."

3.8 — IntervalSlider
────────────────────
Slider double-poignée pour filtrer par plage de valeurs (workbench).
Affiche min/max + valeur courante.

3.9 — ConfidenceIndicator
─────────────────────────
Petit indicateur visuel de confiance des données :
  ▓░░░░░ Faible | ▓▓▓░░░ Moyenne | ▓▓▓▓▓░ Élevée | ▓▓▓▓▓▓ Très élevée
Hover : explication de la note.

3.10 — DatasheetCallout
───────────────────────
Encadré "Datasheet for Datasets" (Gebru et al.) cliquable sur le bouton
"Exporter dataset" → ouvre la fiche complète.

═══════════════════════════════════════════════════════════════════════════════
4. ÉCRANS À PRODUIRE (vue d'ensemble — détail dans MAQUETTE-UI-TEXTUELLE.md)
═══════════════════════════════════════════════════════════════════════════════

Pour CHACUN des 9 écrans + onboarding + extension overlay :
- Layout desktop 1280×800 + variants tablette 1024×768 et mobile 390×844
- États : vide / chargement / résultat / erreur
- Mockups haute-fidélité + spec interactive (focus/hover/active)
- Annotations d'accessibilité (ordre tabulation, aria-labels)

ÉCRANS :
1. Estimer un prompt          (Module M2)
2. Workbench                  (Module M3)
3. Comparer                   (Module M5)
4. Simuler des scénarios      (Module M4)
5. Importer logs entreprise   (Module M10)
6. Géolocaliser datacenter    (Module M9)
7. Rapports & exports         (Module M6)
8. Journal d'audit            (Module M7)
9. Méthodologie & aide        (Module M8)
10. Territoire français       (Module M12 — NOUVEAU, avec ChoroplethMap)
11. Onboarding 4 étapes
12. Extension navigateur (overlay sur ChatGPT/Claude/Mistral)

ÉCRAN PRIORITAIRE M12 (Territoire français — nouveau, ajouté en v1.2) :
- ChoroplethMap IRIS France plein écran
- Panneau latéral droit : filtres + KPI agrégés (total MWh, top 10 IRIS)
- Toolbar haut : sélecteur d'indicateur (consommation, intensité carbone,
  ratio élec/gaz, croisement avec usage LLM)
- Sélection région via picker carte ou liste
- Mode "scénario national" : superposer projection ComparIA + RTE IRIS

═══════════════════════════════════════════════════════════════════════════════
5. CONTRAINTES TECHNIQUES (à anticiper dans le design)
═══════════════════════════════════════════════════════════════════════════════

- Composants livrables : Svelte 5 (runes), TypeScript strict
- Pas de framework UI lourd : tout est custom (Skeleton CSS-only ou rien).
- Imports JS autorisés : Observable Plot, D3 v7, lucide-svelte. Rien d'autre.
- Bundle final ≤ 200 Ko gzip (frontend total).
- Cible RGAA AA validée par axe-core en CI.
- Mode hors-ligne : tous les graphes doivent fonctionner sans réseau (données
  pré-chargées via stores Svelte).
- i18n : tous les textes en clés `kebab-case` consommables par svelte-i18n.

═══════════════════════════════════════════════════════════════════════════════
6. LIVRABLES ATTENDUS POUR CETTE ITÉRATION
═══════════════════════════════════════════════════════════════════════════════

Pour chaque composant nouveau ou révisé :
1. Composant Svelte fonctionnel (.svelte) avec props typés.
2. Variants documentés (au moins : default, hover, focus, error, empty).
3. Storybook / page de démo avec exemples réalistes (utiliser des valeurs
   plausibles : GPT-4o-mini ≈ 2 gCO2eq, mix FR ≈ 56 gCO2eq/kWh).
4. Spec d'accessibilité (rôle ARIA, aria-label dynamique, ordre de tabulation,
   contrastes vérifiés).
5. Notes de performance si pertinent (notamment ChoroplethMap IRIS — 50k
   polygones).

═══════════════════════════════════════════════════════════════════════════════
7. RÉFÉRENCE AUTORITAIRE
═══════════════════════════════════════════════════════════════════════════════

La spec textuelle exhaustive est dans MAQUETTE-UI-TEXTUELLE.md ci-jointe.
En cas de conflit entre ce prompt et la maquette, la maquette prévaut pour
les écrans, et ce prompt prévaut pour le design system et la liste des
composants à produire.

═══════════════════════════════════════════════════════════════════════════════

Commence par les 7 composants dataviz manquants (§2) — ce sont les plus
différenciants pour notre candidature. Ensuite les composants UI transversaux
(§3). Termine par les écrans (§4) en intégrant les composants déjà produits.
```

---

## Notes pour Thibault

1. **Joins la maquette textuelle** (`MAQUETTE-UI-TEXTUELLE.md`) au prompt — Claude Design a besoin de la spec écran par écran pour ne rien rater.
2. **Insiste sur les graphes dataviz** : c'est le différenciant majeur. Un Sankey énergétique et un histogramme Monte-Carlo bien faits = effet "wow" garanti devant le jury.
3. **Mentionne explicitement M12** : c'est le module Territoire français ajouté en v1.2, Claude Design ne le connaît pas si la première itération a été lancée avant le pivot.
4. **Demande une charte de variants** : pour chaque composant, l'état default + hover + focus + active + disabled + error + empty. Ça t'évitera des allers-retours pour faire passer la a11y plus tard.
5. **Cible RGAA AA dès maintenant** : c'est dans NF-12, plus tard ça coûte beaucoup plus cher de revenir corriger.

Une fois la nouvelle itération reçue, on intégrera les composants Svelte dans `web/src/lib/components/`. Si tu veux, je peux préparer l'arborescence cible dès maintenant pour qu'il n'y ait plus qu'à copier-coller.

[Prompt complet à copier-coller](computer://C:\Users\NR2201ZE\Desktop\defis-lia-generatif\docs\ux\PROMPT-CLAUDE-DESIGN.md)