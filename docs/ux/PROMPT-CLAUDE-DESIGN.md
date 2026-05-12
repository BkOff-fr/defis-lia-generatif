# Prompt à donner à Claude Design — Sobr.ia (itération 2)

> **Usage** : copier le bloc « Prompt à coller » ci-dessous. Le visuel (palette, typo, formes, animations) est volontairement laissé libre — Thibault gère le langage graphique. Le but de ce prompt est de **décrire le contenu, les données, les interactions et la richesse fonctionnelle** pour que les composants livrés soient complets et immersifs.

---

## Prompt à coller

```
Tu travailles sur Sobr.ia, une application Tauri (desktop, mobile, web) qui
mesure et visualise l'impact environnemental de l'IA générative pour le défi
data.gouv.fr.

Le langage visuel (couleurs, formes, typographie, animations) est défini en
dehors de toi — ne fais aucune proposition d'esthétique. Tu te concentres
exclusivement sur :

  - Le CONTENU exact qu'affiche chaque composant.
  - Les DONNÉES qu'il consomme (avec exemples plausibles).
  - Les INTERACTIONS attendues.
  - Les ÉTATS à gérer (vide, chargement, succès, erreur, dégradé).
  - L'INTENTION NARRATIVE — comment l'utilisateur comprend ce qu'il voit.

═══════════════════════════════════════════════════════════════════════════════
1. ITÉRATION PRÉCÉDENTE
═══════════════════════════════════════════════════════════════════════════════

Tu as déjà produit une première base. Cette itération doit COMPLÉTER, pas
remplacer. Plusieurs composants critiques manquent — notamment des
visualisations de données (Sankey énergétique, histogramme Monte-Carlo,
bande d'incertitude, carte choroplèthe au niveau IRIS).

═══════════════════════════════════════════════════════════════════════════════
2. CONTEXTE D'IMMERSION
═══════════════════════════════════════════════════════════════════════════════

L'utilisateur typique veut comprendre, en quelques secondes, "combien coûte"
en environnement le fait d'utiliser une IA générative. Il oscille entre
plusieurs sentiments : curiosité, scepticisme ("vos chiffres viennent d'où ?"),
et besoin de justification (RSE, presse, décisions publiques).

Notre promesse narrative : "rien n'est caché, tout est mesuré honnêtement,
chaque valeur a une source cliquable, chaque incertitude est explicitée."

Trois fils rouges traversent toute l'interface :

  A. INCERTITUDE EXPLICITE — aucune valeur n'est affichée sans son intervalle
     P5-P95 issu du Monte-Carlo. L'utilisateur doit comprendre intuitivement
     qu'on lui dit une fourchette, pas une vérité.

  B. TRAÇABILITÉ — chaque chiffre, chaque hypothèse est cliquable et mène à
     la source (URL, DOI, datasheet, paper). Pas de "boîte noire".

  C. LOCAL — l'utilisateur garde le contrôle. Aucune donnée ne part en
     dehors de son appareil sauf consentement explicite. C'est rappelé en
     permanence quelque part dans le shell.

═══════════════════════════════════════════════════════════════════════════════
3. COMPOSANTS DATAVIZ MANQUANTS (priorité absolue)
═══════════════════════════════════════════════════════════════════════════════

3.1 — Diagramme de Sankey énergétique
─────────────────────────────────────
QUOI : visualise comment l'énergie est distribuée pour un prompt unitaire.

DONNÉES TYPIQUES :
  Compute       3.2 Wh ─┐
  Idle          0.4 Wh  ├─► × PUE 1.3 ─► Cooling+overhead ─► Total 4.87 Wh
  Networking    0.1 Wh ─┘

CONTENU :
  - Sources (gauche) : compute, idle, networking, embodied amorti.
  - Étape intermédiaire : facteur PUE.
  - Destination (droite) : énergie totale livrée.
  - Chaque flux porte sa valeur (Wh) + son intervalle (P5-P95 entre crochets).
  - Le PUE est annoté avec sa source (datacenter X, ADEME, etc.).

INTERACTIONS :
  - Survol d'un flux : tooltip avec valeur exacte, intervalle, source.
  - Clic sur une étape : panneau latéral qui détaille les hypothèses.
  - Légende cliquable pour masquer/afficher chaque flux.
  - Mode "tableau" alternatif (a11y) — toggle visible.

ÉTATS :
  - Vide ("aucune estimation").
  - Chargement (recalcul en cours).
  - Succès.
  - Erreur ("source PUE indisponible pour ce datacenter").

INTENTION : montrer qu'un prompt qui consomme N Wh "compute" en consomme en
réalité PUE × N à la sortie du datacenter, et que la part embodied (fabrication
hardware) est rarement nulle.


3.2 — Histogramme Monte-Carlo
─────────────────────────────
QUOI : la distribution des 10 000 simulations Monte-Carlo qui ont produit
l'estimation.

DONNÉES TYPIQUES :
  - 10 000 valeurs en gCO2eq pour un prompt
  - Médiane (P50) = 2.14 g, intervalle [1.68, 2.74]
  - Forme typique : log-normale légèrement asymétrique à droite

CONTENU :
  - Histogramme à 30-50 classes.
  - Trois lignes verticales : P5, P50, P95, étiquetées avec leur valeur.
  - Bande P5-P95 surlignée derrière l'histogramme.
  - Axe X : valeur de l'indicateur + son unité (gCO2eq, Wh, L, etc.).
  - Sous-titre : "10 000 tirages Monte-Carlo, seed 42 (reproductible)".

INTERACTIONS :
  - Hover sur une classe : nombre de tirages dans cette classe.
  - Bouton "exporter les 10 000 tirages" (CSV).
  - Switch indicateur si plusieurs ont été calculés (CO2eq, énergie, eau).

INTENTION : prouver visuellement que la médiane n'est pas une vérité
ponctuelle. L'utilisateur "voit" l'incertitude au lieu de la lire.


3.3 — Courbe avec bande d'incertitude (projection temporelle)
─────────────────────────────────────────────────────────────
QUOI : projection sur 5 ans du cumul d'un scénario macro (simulateur M4).

DONNÉES TYPIQUES :
  - Population : 50 000 fonctionnaires
  - Taux d'adoption : 60 % avec croissance +5%/an
  - Modèle : mix 60% GPT-4o-mini + 40% Claude 3.5
  - Projection : cumul CO2eq mensuel sur 2026-01 → 2030-12
  - Pour chaque mois : P5, P50, P95

CONTENU :
  - Ligne médiane (P50) bien visible.
  - Bande P5-P95 remplie semi-transparente autour.
  - Axe X : temps (mois ou années).
  - Axe Y : valeur cumulée + unité.
  - Annotations sur jalons importants ("fin 2027 = 200 t CO2eq cumulés").
  - Petit panneau récapitulatif : "cumul 5 ans ≈ 1 850 t [1 320 – 2 580]".

INTERACTIONS :
  - Sliders au-dessus du graphe pour ajuster taux, fréquence, modèle —
    le graphe se recalcule en live (≤ 1 s).
  - Comparaison : superposer 2 ou 3 scénarios côte à côte.
  - Mode "linéaire vs log Y" toggle.

VARIANTES :
  - Aires empilées si plusieurs scénarios.
  - Aire seule sans P5-P95 si donnée déterministe (rare).

INTENTION : faire comprendre que les projections macro sont des
intervalles, pas des courbes lisses.


3.4 — Carte choroplèthe IRIS de France
──────────────────────────────────────
QUOI : carte de France au niveau IRIS (~50 000 polygones, plus petite maille
INSEE). Spécifique au module M12 "Territoire français".

DONNÉES TYPIQUES :
  - Source primaire : RTE/NaTran/Teréga "Consommation annuelle IRIS sites
    industriels raccordés au transport" (data.gouv.fr).
  - Pour chaque IRIS : consommation élec annuelle (MWh), consommation gaz
    annuelle (MWh), nombre de sites industriels.
  - Croisement avec ComparIA pour les scénarios d'usage LLM.

CONTENU :
  - Carte projetée en Lambert 93 (la projection officielle française).
  - Coloration par classe en quantiles (5-7 buckets).
  - Indicateur affiché sélectionnable : conso élec, conso gaz, ratio
    élec/gaz, "candidat datacenter" (heuristique), croisement avec usage LLM.
  - Légende avec les bornes des classes.
  - Sous-titre : "Référentiel IRIS INSEE 2023, données 2024".

INTERACTIONS :
  - Hover sur un IRIS : tooltip avec code IRIS, nom commune, valeur exacte.
  - Clic : panneau latéral avec fiche détaillée (top 5 sites industriels,
    estimation d'usage LLM si on croise ComparIA).
  - Zoom sur région via picker (IDF, AURA, Occitanie, etc.).
  - Mode "top 100" : n'affiche que les 100 IRIS les plus consommateurs,
    les autres en grisé.
  - Mode "écart à la moyenne nationale" (palette divergente).
  - Exporter en GeoJSON enrichi ou PNG haute résolution.

ÉTATS :
  - Chargement (long potentiellement : 50k polygones).
  - Données partielles (certains IRIS en secret statistique → motif hachuré).
  - Erreur (référentiel non chargé).

PERFORMANCE : critique à anticiper. Suggestion : tuiles vectorielles ou
simplification topologique selon le niveau de zoom.

INTENTION : passer du chiffre national désincarné au territoire concret.
"Regarde, c'est cet IRIS-là qui consomme N MWh, à côté de chez toi."


3.5 — Heatmap modèles × indicateurs
───────────────────────────────────
QUOI : matrice comparative pour le module M5 (comparateur).

DONNÉES TYPIQUES :
  - Lignes : 2 à 8 modèles sélectionnés (GPT-4o, GPT-4o-mini, Claude 3.5,
    Mistral L, Llama 70B, Llama 8B).
  - Colonnes : Énergie, CO2eq, Eau, Embodied, Coût, Latence.
  - Valeurs : intervalle P5-P95 dans chaque cellule.
  - Normalisation : par défaut absolue ; toggle relatif à la meilleure valeur.

CONTENU :
  - Cellule : valeur (P50) + petit indicateur d'incertitude en filigrane.
  - Indicateur "données manquantes" pour les modèles fermés sans mesure
    fiable (motif hachuré + tooltip explicatif).
  - Score composite paramétrable (sliders au-dessus pour pondérer les
    indicateurs).
  - Classement automatique sous la matrice (top 3 modèles selon le score).

INTERACTIONS :
  - Tri par colonne au clic header.
  - Ajout/retrait de modèle en drag-and-drop ou via picker.
  - Mode "normalize" toggle (absolu / % du max / écart à la médiane).
  - Annotations valeurs absolues ON/OFF dans la cellule.

INTENTION : permettre une décision rapide ("quel modèle choisir pour mon
SaaS ?") avec transparence sur les hypothèses (sources cliquables).


3.6 — Treemap consommation
──────────────────────────
QUOI : vue dense pour le workbench M3 — répartition des modèles par catégorie.

DONNÉES TYPIQUES :
  - 100-200 modèles du référentiel.
  - 3 niveaux : provider → famille → modèle.
  - Aire proportionnelle à l'indicateur sélectionné (Wh/req par défaut).

CONTENU :
  - Squarified treemap classique.
  - Étiquettes lisibles aux 2 premiers niveaux.

INTERACTIONS :
  - Drill-down au clic.
  - Switch indicateur (Wh, gCO2eq, eau).
  - Breadcrumb retour navigation.

INTENTION : montrer "qui pèse" dans l'écosystème (OpenAI vs Mistral vs
modèles ouverts).


3.7 — Ridge plot des distributions
──────────────────────────────────
QUOI : distributions empilées (KDE) des Wh/req par modèle — vue dense pour
le workbench.

CONTENU :
  - Une ligne par modèle, distribution KDE remplie.
  - Tri par médiane (du moins gourmand au plus gourmand).
  - Mode "normalize y" : chaque distribution à la même hauteur visuelle.
  - Mode "absolute" : hauteurs proportionnelles.

INTERACTIONS :
  - Hover sur une ligne : surligne et montre la médiane.
  - Filtre par taille de modèle, provider, modalité.

INTENTION : voir d'un coup d'œil la dispersion entre modèles, pas seulement
les médianes.

═══════════════════════════════════════════════════════════════════════════════
4. COMPOSANTS UI TRANSVERSAUX MANQUANTS
═══════════════════════════════════════════════════════════════════════════════

4.1 — MetricCard (carte d'indicateur)
─────────────────────────────────────
CONTENU : un indicateur (CO2eq, énergie, eau, etc.) avec sa valeur P50,
son intervalle P5-P95, son unité, optionnellement une sparkline d'historique,
et un badge "équivalent parlant" (ex: "≈ 17 m voiture").
ÉTATS : vide, chargement, succès, erreur, valeur dégradée (proxy).
INTERACTIONS : clic sur la carte → vue détaillée de l'indicateur.

4.2 — SourcePopover
───────────────────
CONTENU : ouvre depuis n'importe quelle valeur ou hypothèse. Affiche
titre source, auteurs, année, DOI/URL cliquable, citation formatée APA,
bouton "Copier la citation BibTeX".
INTENTION : "rien n'est caché, tout est sourcé".

4.3 — EquivalentBadge
─────────────────────
CONTENU : équivalent parlant inline (ex: "≈ 17 m en voiture thermique",
"≈ 0.5 s de douche chaude", "≈ 3 écrans-heures").
INTERACTION : hover affiche la source de la conversion (ADEME, etc.).

4.4 — LedgerEntry (ligne d'audit)
─────────────────────────────────
CONTENU : timestamp UTC, modèle, tokens in/out, indicateur principal,
hash SHA-256 tronqué cliquable (copie complète au clic).
ÉTAT : "chaîne intègre ✓" ou "compromise" en cas d'altération détectée.
INTENTION : démontrer la traçabilité réglementaire (CSRD).

4.5 — HypothesisChip
────────────────────
CONTENU : chip représentant une hypothèse utilisée par le moteur, avec
une clé courte (ex: "PUE 1.3", "ε_decode 1.8 mJ/tok").
INTERACTION : clic → SourcePopover.
INTENTION : rendre les rouages visibles, jamais cachés.

4.6 — LocalIndicator (bandeau permanent du shell)
─────────────────────────────────────────────────
CONTENU : "🔒 100 % local • Référentiel YYYY.MM.DD • N alertes".
INTERACTION : clic sur "alertes" → panneau de diagnostic réseau / données.
INTENTION : rassurer en permanence sur la confidentialité.

4.7 — UncertaintyTooltip (standard pour toute valeur incertaine)
────────────────────────────────────────────────────────────────
CONTENU : "2.14 gCO2eq [1.68–2.74]" + icône info.
HOVER PROLONGÉ : "Intervalle de confiance à 90 % issu de 10 000 simulations
Monte-Carlo (seed 42). Cliquer pour voir l'histogramme complet."
INTERACTION : clic → ouvre HistogramMC dans une modale.

4.8 — IntervalSlider
────────────────────
CONTENU : slider double-poignée pour filtrer par plage (utilisé dans le
workbench pour ne voir que les modèles consommant entre X et Y Wh/req).
ÉTAT : valeurs min, max et courantes affichées.

4.9 — ConfidenceIndicator
─────────────────────────
CONTENU : indicateur visuel de confiance des données (faible / moyenne /
élevée / très élevée).
HOVER : explication de la note ("3 sources concordantes, mesures directes
sur hardware standardisé").
INTENTION : honnêteté radicale — "voici ce qu'on sait vraiment".

4.10 — DatasheetCallout
───────────────────────
CONTENU : encadré "Datasheet for Datasets" (Gebru et al.) accessible
depuis le bouton "Exporter dataset" → ouvre la fiche complète (motivation,
composition, collecte, recommended uses, limitations, etc.).
INTENTION : rigueur scientifique communiquée à l'utilisateur final.

═══════════════════════════════════════════════════════════════════════════════
5. ÉCRANS À PRODUIRE
═══════════════════════════════════════════════════════════════════════════════

12 écrans au total. Détail exhaustif dans la maquette UI textuelle
(MAQUETTE-UI-TEXTUELLE.md) jointe en référence.

Pour CHAQUE écran :
  - Layout pour desktop, tablette et mobile.
  - Tous les états (vide, chargement, succès, erreur, hors-ligne).
  - Annotations d'accessibilité (ordre de tabulation, libellés ARIA).

LISTE :
  1. Estimer un prompt              (M2)
  2. Workbench (référentiel)        (M3)
  3. Comparateur                     (M5)
  4. Simulateur de scénarios         (M4)
  5. Importer logs entreprise        (M10)
  6. Géolocaliser datacenter         (M9)
  7. Rapports & exports              (M6)
  8. Journal d'audit                 (M7)
  9. Méthodologie & aide             (M8)
 10. Territoire français             (M12 — nouveau, avec ChoroplethMap)
 11. Onboarding 4 étapes
 12. Extension navigateur (overlay sur ChatGPT / Claude / Mistral / Gemini)

ÉCRAN PRIORITAIRE M12 (nouveau en v1.2) :
  - ChoroplethMap IRIS plein écran (composant §3.4).
  - Panneau latéral : filtres + KPI agrégés (total MWh, top 10 IRIS).
  - Toolbar : sélecteur d'indicateur, mode "scénario national" qui
    superpose ComparIA + RTE IRIS.
  - Récit guidé en bas de page : "Voici comment lire cette carte" (3 étapes).

═══════════════════════════════════════════════════════════════════════════════
6. CONTRAINTES TECHNIQUES (à anticiper dans la conception)
═══════════════════════════════════════════════════════════════════════════════

  - Bundle frontend final ≤ 200 Ko gzip (cf. CDC NF).
  - Tous les composants doivent fonctionner hors-ligne (données chargées
    via stores Svelte / IndexedDB).
  - i18n : tous les textes affichés via clés kebab-case (svelte-i18n).
    Pas de texte en dur dans les composants.
  - Conformité RGAA AA / WCAG 2.1 AA obligatoire.
  - Pas de framework UI lourd : composants custom.
  - Imports JS autorisés pour la dataviz : Observable Plot, D3 v7,
    lucide-svelte. Rien d'autre.

═══════════════════════════════════════════════════════════════════════════════
7. CE QUE TU NE FAIS PAS
═══════════════════════════════════════════════════════════════════════════════

  - Pas de palette imposée. Pas de couleurs nommées.
  - Pas de typographie nommée.
  - Pas d'espacements en pixels.
  - Pas de coins arrondis suggérés.
  - Pas de durées d'animation.
  - Pas de référence à d'autres produits comme inspiration esthétique.

Concentre-toi sur le contenu, les données, les interactions, et la
narration. Le visuel est décidé en dehors de toi.

═══════════════════════════════════════════════════════════════════════════════
8. LIVRABLES ATTENDUS
═══════════════════════════════════════════════════════════════════════════════

Pour chaque composant nouveau ou révisé :
  1. Spécification fonctionnelle complète (contenu, données, états,
     interactions, narration).
  2. Exemples de données plausibles (valeurs réalistes : GPT-4o-mini ≈
     2 gCO2eq, mix FR ≈ 56 gCO2eq/kWh, etc.).
  3. Squelette Svelte (.svelte) avec props typés mais STYLES VIDES — le
     style sera ajouté hors de ton périmètre.
  4. Notes d'accessibilité (rôle ARIA, ordre de tabulation, alternatives
     textuelles pour les graphes).
  5. Notes de performance si pertinent (notamment ChoroplethMap IRIS).

ORDRE DE PRIORITÉ :
  - §3 — composants dataviz manquants (différenciants majeurs)
  - §4 — composants UI transversaux
  - §5 — écrans (en intégrant les composants déjà produits)

═══════════════════════════════════════════════════════════════════════════════
```

---

## Notes pour Thibault

1. **Joins la maquette textuelle en pièce jointe** ([MAQUETTE-UI-TEXTUELLE.md](MAQUETTE-UI-TEXTUELLE.md)) — c'est la source autoritaire pour les écrans, le prompt ci-dessus ne reprend que la liste.
2. **Insiste sur les §3 dataviz** — le Sankey énergétique et l'histogramme Monte-Carlo sont les composants les plus différenciants devant le jury, et ce sont aussi les plus susceptibles d'être oubliés.
3. **Le module M12 (Territoire français) est nouveau** depuis le pivot ComparIA — si la première itération date d'avant, Claude Design ne le connaît pas du tout.
4. **Le prompt précise explicitement « pas de couleurs, pas de typo, pas de formes »** — comme ça il ne se croira pas tenu de proposer un visuel.
5. Quand tu reçois la nouvelle itération, dis-le moi : je prépare l'arborescence `web/src/lib/components/` pour qu'il n'y ait plus qu'à coller les composants Svelte.
