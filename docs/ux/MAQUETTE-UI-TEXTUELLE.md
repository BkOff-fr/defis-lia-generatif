# Maquette UI textuelle — Sobr.ia

> **Version** : v1.0 (cadrage)
> **Statut** : storyboard de référence pour les sprints S6-S8.
> **Cible** : Claude Code l'utilise pour aligner l'implémentation Svelte.
> **Conventions ASCII** : `[ ]` = champ input, `( )` = radio, `[x]` = checkbox, `«»` = bouton, `┃` = bordure, `▣` = composant Svelte custom, `╌╌` = état désactivé.

---

## Principes UX directeurs

1. **Frugalité visuelle** — pas de skeuomorphisme, palettes neutres, dataviz généreuse.
2. **Honnêteté scientifique** — toute valeur affichée avec intervalle d'incertitude (P5-P95) ; les sources sont à un clic.
3. **Local-first** — un indicateur permanent rappelle « 100 % local, aucune donnée envoyée ».
4. **Accessibilité prioritaire** — RGAA AA, lecteur d'écran, raccourcis clavier visibles.
5. **i18n FR/EN** — sélecteur de langue dans le coin haut-droit, FR par défaut.
6. **Mode sombre par défaut** — économies écran OLED ; clair en option.

---

## Layout général (shell de l'app)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  Sobr.ia                                              FR ▾   ☾ ─ ☐ ✕         │  ← barre titre
├──────────────────────────────────────────────────────────────────────────────┤
│┃ 🧮 Estimer       ┃                                                          │
│┃ 📚 Workbench     ┃              [ ZONE DE CONTENU PRINCIPAL ]              │
│┃ ⚖  Comparer      ┃                                                          │
│┃ 📈 Simuler       ┃                                                          │
│┃ 📥 Importer      ┃                                                          │
│┃ 🌍 Géolocaliser  ┃                                                          │
│┃ 📤 Exporter      ┃                                                          │
│┃ 🗂 Journal audit ┃                                                          │
│┃ 📖 Méthodologie  ┃                                                          │
│┃                  ┃                                                          │
│┃ ⚙  Paramètres    ┃                                                          │
│┃ ❓ Aide          ┃                                                          │
│┃                  ┃                                                          │
│┃ ● Local • v0.1.0 ┃                                                          │
├──────────────────────────────────────────────────────────────────────────────┤
│  🔒 100 % local, aucune donnée envoyée  •  Référentiel YYYY.MM.DD  •  3 alertes │
└──────────────────────────────────────────────────────────────────────────────┘
```

Largeur cible **1024×640 px** mini, optimisée jusqu'à 1920×1080. Mobile = layout vertical (drawer pour la sidebar).

---

## Écran 1 — Estimer un prompt  (Module M2)

**Objectif** : l'utilisateur saisit un usage unitaire, obtient une estimation chiffrée avec incertitudes et sources.

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  🧮  Estimer un prompt                                              ? Aide   │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ▣ MODÈLE                                                                    │
│  [ GPT-4o-mini                          ▾ ]   Provider : OpenAI              │
│                                                                              │
│  ▣ PROMPT                                                                    │
│  ┌────────────────────────────────────────────────────────────────────┐     │
│  │ Écris-moi un résumé de 500 mots sur la photosynthèse…             │     │
│  │                                                                    │     │
│  │                                                                    │     │
│  └────────────────────────────────────────────────────────────────────┘     │
│  Tokens entrée : 23 ± 2    Tokens sortie estimés : 720 ± 80   ⚙ Ajuster    │
│                                                                              │
│  ▣ DATACENTER  ( détecté auto, modifiable )                                  │
│  ☑ Auto via géoloc → US-East (Virginie)         «  Modifier  »              │
│                                                                              │
│  ▣ MIX ÉLECTRIQUE                                                            │
│  ☑ Live (Electricity Maps) → 412 gCO₂eq/kWh    ( ) Annuel moyen             │
│                                                                              │
│                                  «  Estimer  »                               │
└──────────────────────────────────────────────────────────────────────────────┘
```

**État résultat (après clic Estimer)** :

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  📊 Résultat                                                       🔗 Partager│
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   CO₂eq         ÉNERGIE         EAU            MÉTAUX                        │
│   ────────      ────────        ────           ──────                        │
│     2,14 g       4,87 Wh        0,12 L         0,03 mg                      │
│   [1,68–2,74]  [3,80–6,18]    [0,09–0,16]    [0,02–0,05]   ← P5-P95         │
│                                                                              │
│  ▣ Histogramme Monte-Carlo (N=10⁴)                                           │
│  ┌────────────────────────────────────────────────────────────────────┐     │
│  │             ▁▁▁▂▃▄▆█▇▅▃▂▁▁▁                                       │     │
│  │  ╌╌╌╌╌╌╌╌╌╌P5═════════════P95╌╌╌╌╌╌╌╌╌╌╌                          │     │
│  │  1,5  1,7  1,9  2,1  2,3  2,5  2,7  2,9  g CO₂eq                  │     │
│  └────────────────────────────────────────────────────────────────────┘     │
│                                                                              │
│  ▣ Équivalents parlants                                                      │
│  ≈ 17 m en voiture thermique  •  ≈ 0,5 s de douche chaude                   │
│                                                                              │
│  ▣ Sankey énergétique  (compute → cooling → losses)               🔍 Détail  │
│  ┌────────────────────────────────────────────────────────────────────┐     │
│  │ Compute ████████████████ 3,2 Wh ──┐                               │     │
│  │                                    ├─► PUE 1,3 ─► Total 4,87 Wh    │     │
│  │ Idle    ██ 0,4 Wh ────────────────┘                                │     │
│  └────────────────────────────────────────────────────────────────────┘     │
│                                                                              │
│  ▣ Hypothèses utilisées (cliquables)                                         │
│  • ε_decode = 1,8 mJ/token  [HF AI Energy Score, 2026]                       │
│  • PUE = 1,3 [moyenne datacenter Virginie, ADEME]                            │
│  • IF élec = 412 gCO₂eq/kWh [Electricity Maps, live 14h32 UTC]               │
│  • Embodied amorti = 0,02 gCO₂eq/req [Gupta et al., 2022]                    │
│                                                                              │
│  «  Sauvegarder  »  «  Exporter PDF  »  «  Comparer  »                       │
│                                                                              │
│  🗂 Journalisé dans l'audit ledger — hash 7a3f9b…                            │
└──────────────────────────────────────────────────────────────────────────────┘
```

**États** :
- *Vide* : message d'accueil + tutorial « Saisis ton premier prompt ».
- *Calcul en cours* : barre de progression + « 10 000 simulations Monte-Carlo en cours… » (objectif < 200 ms perçus).
- *Erreur* : message inline (modèle manquant, mix indisponible) + suggestion.
- *Hors-ligne* : bandeau indiquant que le mix électrique sera en valeur moyenne historique.

**Raccourcis clavier** : `Ctrl+Enter` = estimer, `Ctrl+S` = sauvegarder, `Esc` = vider.

**EF couvertes** : EF-M2-01, EF-M2-02, EF-M2-03, EF-M2-04, EF-M2-05.

---

## Écran 2 — Workbench  (Module M3)

**Objectif** : explorer le référentiel, filtrer, comparer rapidement, lire les fiches modèles.

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  📚  Workbench — Référentiel des modèles                Maj : 2026-05-10     │
├──────────────────────────────────────────────────────────────────────────────┤
│ FILTRES                                                                      │
│ Provider : [Tous▾]   Taille : [▶─────●─────] 1B–500B   Modalité : [✓ Texte]│
│ Licence : [Toutes▾]  Score énergie : [A][B][C][D][E][F]                     │
│ 🔍 [ Rechercher : llama, claude…                                         ]  │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│ ▣ Tableau (88 modèles affichés / 142 total)                                  │
│ ┌────────────┬──────────┬─────────┬───────────┬──────────┬──────────────┐  │
│ │ Modèle     │ Provider │ Params  │ Wh/req P50│ gCO2/req │ Score énergie│  │
│ ├────────────┼──────────┼─────────┼───────────┼──────────┼──────────────┤  │
│ │ GPT-4o     │ OpenAI   │ ~200B   │ 14,2      │ 5,8      │ ▓▓▓▓▓░ C    │  │
│ │ GPT-4o-mini│ OpenAI   │ ~8B     │ 4,9       │ 2,0      │ ▓▓░░░░ B    │  │
│ │ Claude 3.5 │ Anthropic│ ~200B*  │ 13,7      │ 5,4      │ ▓▓▓▓▓░ C    │  │
│ │ Mistral L  │ Mistral  │ ~123B   │ 9,8       │ 3,9      │ ▓▓▓▓░░ B    │  │
│ │ Llama 3.1  │ Meta     │ 70B     │ 7,1       │ 2,8      │ ▓▓▓░░░ B    │  │
│ │ Llama 3.1  │ Meta     │ 8B      │ 1,2       │ 0,5      │ ▓░░░░░ A    │  │
│ │ ▼ + 82 modèles                                                           │  │
│ └────────────┴──────────┴─────────┴───────────┴──────────┴──────────────┘  │
│   * = estimation (paramètres non publics)                                    │
│                                                                              │
│ ✓ 3 modèles sélectionnés  «  Comparer  »  «  Exporter  »  «  Fiche détail »│
└──────────────────────────────────────────────────────────────────────────────┘
```

**Fiche détail d'un modèle (sous-écran)** :

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  📄  Mistral Large 2                                              ← Retour   │
├──────────────────────────────────────────────────────────────────────────────┤
│ Provider : Mistral AI    Sortie : 2024-07-24    Licence : Mistral Research  │
│ Params (publié) : 123 B  Architecture : Dense decoder  Contexte : 128 k    │
│                                                                              │
│ ▣ INDICATEURS (P50, P5-P95)                                                  │
│ ┌──────────────────────────┬───────────────────────┬─────────────────────┐  │
│ │ Énergie/req (Wh)         │ CO₂eq/req (g)         │ Eau/req (mL)        │  │
│ │ 9,8  [7,1–13,4]          │ 3,9  [2,8–5,4]        │ 24  [17–34]         │  │
│ └──────────────────────────┴───────────────────────┴─────────────────────┘  │
│                                                                              │
│ ▣ SOURCES (lineage)                                                          │
│ • Hugging Face AI Energy Score — fetched 2026-05-08, hash 9e2f…              │
│ • EcoLogits modèle ID `mistral-large-2`, snapshot 2026-04-30                 │
│ • Mistral AI tech report (2024), DOI:…                                       │
│                                                                              │
│ ▣ HYPOTHÈSES                                                                 │
│ • ε_decode log-normale (μ=1,5 mJ, σ=0,3)                                     │
│ • Amortissement embodied : 5 ans, 10⁹ req                                    │
│                                                                              │
│ «  Estimer un prompt avec ce modèle  »  «  Comparer  »  «  Ajouter scénario »│
└──────────────────────────────────────────────────────────────────────────────┘
```

**EF couvertes** : EF-M3-01 → EF-M3-05.

---

## Écran 3 — Comparateur  (Module M5)

**Objectif** : comparer 2 à 8 modèles côte à côte, score composite paramétrable.

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  ⚖  Comparer des modèles                                                     │
├──────────────────────────────────────────────────────────────────────────────┤
│  Modèles : [GPT-4o] [GPT-4o-mini] [Claude 3.5] [Mistral L] [Llama 70B] [+]  │
│                                                                              │
│  ▣ HEATMAP normalisée  (clair = mieux)                                       │
│  ┌────────────┬──────────┬─────────┬───────┬─────────┬──────┬───────┐       │
│  │            │ Énergie  │ CO₂eq   │ Eau   │ Embodied│ Coût │ Latency│      │
│  ├────────────┼──────────┼─────────┼───────┼─────────┼──────┼───────┤       │
│  │ GPT-4o     │ ░░░░░░░░ │ ░░░░░░░░│ ▓░░░░ │ ▒▒▒░░░░ │ ░░░░ │ ▓▓░░░ │      │
│  │ GPT-4o-mini│ ████████ │ ████████│ ███▒▒ │ ████░░░ │ ████ │ █████ │      │
│  │ Claude 3.5 │ ░░░░░░░░ │ ░░░░░░░░│ ▒░░░░ │ ▓▓▓░░░░ │ ░░░░ │ ▓▓░░░ │      │
│  │ Mistral L  │ ▓▓▓▓░░░░ │ ▓▓▓▓░░░░│ ████▓ │ ▓▓▓▓░░░ │ ▓▓▓▓ │ ▓▓▓▓░ │      │
│  │ Llama 70B  │ ████▓▓░░ │ ████▓▓░░│ ███▓░ │ ████░░░ │ —    │ █████ │      │
│  └────────────┴──────────┴─────────┴───────┴─────────┴──────┴───────┘       │
│                                                                              │
│  ▣ SCORE COMPOSITE  (ajuste les poids)                                       │
│  CO₂eq    [▶───────●──] 70 %                                                 │
│  Eau      [▶──●──────] 15 %                                                 │
│  Embodied [▶─●───────] 10 %                                                 │
│  Coût     [▶●────────]  5 %                                                 │
│                                                                              │
│  🏆 Classement                                                               │
│  1. GPT-4o-mini  (score 87)                                                  │
│  2. Llama 8B     (score 82)                                                  │
│  3. Mistral L    (score 64)                                                  │
│  4. Claude 3.5   (score 41)                                                  │
│  5. GPT-4o       (score 39)                                                  │
│                                                                              │
│  «  Exporter matrice (CSV)  »   «  Exporter PDF  »                           │
└──────────────────────────────────────────────────────────────────────────────┘
```

**EF couvertes** : EF-M5-01 → EF-M5-05.

---

## Écran 4 — Simulateur de scénarios  (Module M4)

**Objectif** : projections macro (entreprise, secteur, pays) sur plusieurs années.

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  📈  Simulateur de scénarios                                                 │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Scénario : [ Mon entreprise — 2026-2030          ▾ ]  «  Nouveau  »         │
│                                                                              │
│  ▣ POPULATION                                                                │
│  Effectif : [ 50 000 ]  Taux adoption : [▶────────●──] 60 %                  │
│  Croissance adoption : [ +5 % / an ]                                         │
│                                                                              │
│  ▣ USAGE                                                                     │
│  Requêtes/jour/utilisateur : [▶─────●─────] 12                               │
│  Tokens moyens : entrée [ 80 ]  sortie [ 350 ]                               │
│  Modèle : [ Mix : 60 % GPT-4o-mini + 40 % Claude 3.5 ]                       │
│                                                                              │
│  ▣ INFRASTRUCTURE                                                            │
│  Datacenter : [ Mixte FR + US ]    PUE moyen : [ 1,25 ]                      │
│  Mix électrique : ( ) Live   (●) Trajectoire RTE 2030                        │
│                                                                              │
│  ▣ PÉRIODE                                                                   │
│  Du [ 2026-06 ]  au  [ 2030-12 ]                                             │
│                                                                              │
│                                «  Lancer la simulation  »                    │
├──────────────────────────────────────────────────────────────────────────────┤
│  ▣ PROJECTION CO₂eq (P5-P95)                                                 │
│  ┌────────────────────────────────────────────────────────────────────┐     │
│  │ 800 t ┃                                                ┌──────┐    │     │
│  │       ┃                                       ┌────────┘ ╳P95  │   │     │
│  │ 600 t ┃                              ┌────────┘            ╳P50 │   │     │
│  │       ┃                     ┌────────┘             ╳        ╳P5 │   │     │
│  │ 400 t ┃             ┌───────┘             ╳   ╳ ╳   ╳ ╳   ╳    │   │     │
│  │       ┃     ┌───────┘                ╳ ╳ ╳                     │   │     │
│  │ 200 t ┃─────┘  ╳ ╳ ╳ ╳ ╳ ╳ ╳ ╳ ╳ ╳                            │   │     │
│  │       ┃                                                            │     │
│  │ 0 t   ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛│     │
│  │       2026   2027   2028   2029   2030                            │     │
│  └────────────────────────────────────────────────────────────────────┘     │
│                                                                              │
│  Cumul 5 ans  :  ≈ 1 850 t CO₂eq  [1 320 – 2 580]                            │
│  Équivalent   :  ≈ 9 200 vols Paris-NY  •  ≈ 1 200 voitures/an              │
│                                                                              │
│  «  Comparer scénarios  »   «  Exporter (JSON)  »   «  Rapport PDF  »        │
└──────────────────────────────────────────────────────────────────────────────┘
```

**EF couvertes** : EF-M4-01 → EF-M4-06.

---

## Écran 5 — Importer des logs entreprise  (Module M10)

**Objectif** : import CSV/JSONL d'un journal d'usage anonymisé → rapport RSE.

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  📥  Importer un journal d'usage                                             │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│   ┌──────────────────────────────────────────────────────────────────┐      │
│   │                  📂                                              │      │
│   │       Glisser-déposer votre fichier ici                          │      │
│   │       ou  «  Parcourir  »                                        │      │
│   │                                                                  │      │
│   │       CSV, JSONL, Parquet  •  jusqu'à 1 Go  •  100 % local       │      │
│   └──────────────────────────────────────────────────────────────────┘      │
│                                                                              │
│   Formats reconnus : OpenAI usage export, Anthropic, Mistral, générique     │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

**Après dépôt — mapping interactif** :

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  📥  Importer — Mapper les colonnes  (43 217 lignes détectées)               │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ▣ Aperçu (premières lignes)                                                 │
│  ┌──────────────┬───────────┬──────────────┬──────────┬───────────────┐    │
│  │ timestamp    │ user_hash │ model        │ tokens_in│ tokens_out    │    │
│  ├──────────────┼───────────┼──────────────┼──────────┼───────────────┤    │
│  │ 2026-05-08…  │ a3f9…     │ gpt-4o-mini  │ 120      │ 420           │    │
│  │ 2026-05-08…  │ b1e2…     │ claude-3-5   │ 80       │ 1200          │    │
│  └──────────────┴───────────┴──────────────┴──────────┴───────────────┘    │
│                                                                              │
│  ▣ Mapping détecté automatiquement  (modifiable)                             │
│  Sobr.ia                  ←  Votre fichier                                   │
│  Timestamp UTC            ←  [ timestamp     ▾ ]  ✓                          │
│  Utilisateur (hash)       ←  [ user_hash     ▾ ]  ✓                          │
│  Modèle                   ←  [ model         ▾ ]  ✓                          │
│  Tokens entrée            ←  [ tokens_in     ▾ ]  ✓                          │
│  Tokens sortie            ←  [ tokens_out    ▾ ]  ✓                          │
│  Équipe (optionnel)       ←  [ — non mappé   ▾ ]                             │
│                                                                              │
│  ⚠ 3 lignes avec timestamp invalide seront ignorées.                         │
│  ⚠ 12 lignes avec modèle inconnu seront mises en « modèle générique ».      │
│                                                                              │
│  «  Annuler  »                                          «  Importer  »       │
└──────────────────────────────────────────────────────────────────────────────┘
```

**EF couvertes** : EF-M10-01 → EF-M10-05.

---

## Écran 6 — Géolocalisation datacenter  (Module M9)

**Objectif** : détecter ou choisir manuellement le datacenter probable et son mix électrique.

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  🌍  Datacenter & mix électrique                                             │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ▣ DÉTECTION AUTOMATIQUE                                                     │
│  IP détectée (locale) : 81.x.x.x  →  France, Île-de-France                  │
│  Confiance : ▓▓▓▓▓░ Élevée                                                   │
│                                                                              │
│  ▣ DATACENTER PROBABLE  (heuristique provider → zone)                        │
│  Si vous utilisez :                                                          │
│  • OpenAI / Azure   → US-East (Virginie)   PUE 1,30   IF 412 gCO₂/kWh       │
│  • Anthropic        → US-West (Oregon)     PUE 1,15   IF 200 gCO₂/kWh       │
│  • Mistral          → France (Paris)       PUE 1,20   IF  56 gCO₂/kWh       │
│  • Google / Gemini  → US-Iowa              PUE 1,12   IF 320 gCO₂/kWh       │
│                                                                              │
│  ▣ CARTE MONDIALE (interactive)                                              │
│  ┌────────────────────────────────────────────────────────────────────┐     │
│  │                  ▓▓▓ (intensité carbone live)                     │     │
│  │  [carte choroplèthe — clic = sélection datacenter]                │     │
│  │   FR : 56 gCO₂/kWh  •  DE : 380  •  US-VA : 412  •  IS : 28        │     │
│  └────────────────────────────────────────────────────────────────────┘     │
│                                                                              │
│  ▣ OVERRIDE MANUEL                                                           │
│  Datacenter : [ — choisir —                              ▾ ]                 │
│  PUE        : [ 1,30 ]   IF : [ 412 ] gCO₂eq/kWh                             │
│                                                                              │
│  «  Utiliser ce paramétrage par défaut  »                                    │
└──────────────────────────────────────────────────────────────────────────────┘
```

**EF couvertes** : EF-M9-01 → EF-M9-05.

---

## Écran 7 — Rapports & exports  (Module M6)

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  📤  Rapports & exports                                                      │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ▣ RAPPORT PDF                                                               │
│  ( ) Rapport synthétique 1 page                                              │
│  (●) Rapport détaillé 4-8 pages   ☑ Inclure hypothèses                       │
│  ( ) Rapport CSRD-ready 12 pages (entreprises)                               │
│  Langue : (●) FR  ( ) EN  ( ) Bilingue                                       │
│  «  Générer PDF  »                                                           │
│                                                                              │
│  ▣ EXPORT DONNÉES                                                            │
│  ☑ CSV  ☑ Parquet  ☐ JSON-LD (audit CSRD)  ☐ Observable Notebook            │
│  «  Exporter sélection  »                                                    │
│                                                                              │
│  ▣ NOTEBOOK QUARTO REPRODUCTIBLE                                             │
│  Le notebook contient le pipeline complet (Copper → Silver → Gold → calculs). │
│  «  Exporter .qmd  »   «  Exporter HTML  »                                   │
│                                                                              │
│  ▣ AUDIT LEDGER                                                              │
│  ☑ Export NDJSON signé  ☐ Filtré par période  ☐ Filtré par utilisateur      │
│  «  Exporter ledger  »                                                       │
└──────────────────────────────────────────────────────────────────────────────┘
```

**EF couvertes** : EF-M6-01 → EF-M6-05.

---

## Écran 8 — Journal d'audit  (Module M7)

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  🗂  Journal d'audit                                Intégrité : ✓ vérifiée   │
├──────────────────────────────────────────────────────────────────────────────┤
│  🔍 [ Rechercher                                              ]              │
│  Période : [ 2026-04-01 ] → [ 2026-05-12 ]   Modèle : [ Tous ▾ ]            │
│                                                                              │
│  ┌────────────┬──────────────┬──────────┬───────────┬──────────┐            │
│  │ Date       │ Modèle       │ Tokens   │ gCO₂eq    │ Hash     │            │
│  ├────────────┼──────────────┼──────────┼───────────┼──────────┤            │
│  │ 05-12 14:32│ gpt-4o-mini  │ 23/720   │ 2,14      │ 7a3f9b…  │            │
│  │ 05-12 14:18│ claude-3-5   │ 80/1200  │ 5,42      │ 2c8e0a…  │            │
│  │ 05-12 14:05│ mistral-l    │ 120/450  │ 3,89      │ b91d44…  │            │
│  └────────────┴──────────────┴──────────┴───────────┴──────────┘            │
│                                                                              │
│  Total période : 1 247 estimations  •  4,8 kg CO₂eq  •  Chaîne intègre ✓     │
│                                                                              │
│  «  Exporter NDJSON signé  »   «  Vérifier intégrité  »   «  Purger (RGPD) » │
└──────────────────────────────────────────────────────────────────────────────┘
```

**EF couvertes** : EF-M7-01 → EF-M7-05.

---

## Écran 9 — Méthodologie & aide  (Module M8)

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  📖  Méthodologie & aide                                                     │
├──────────────────────────────────────────────────────────────────────────────┤
│  GUIDE                  MÉTHODE                   GLOSSAIRE                 │
│  • Premiers pas         • Formule de référence    • CO₂eq                   │
│  • 5 minutes pour…      • Sources d'incertitude   • PUE                     │
│  • Cas d'usage RSE      • AFNOR SPEC 2314         • WUE                     │
│  • Cas dev SaaS         • Validation croisée      • Embodied carbon         │
│                         • Limitations connues     • ...                     │
│                                                                              │
│  RÉFÉRENCES NORMATIVES                                                       │
│  • AFNOR SPEC 2314 — IA frugale (Ecolab, 2024)              ↗ lien officiel │
│  • ISO/IEC 21031:2024 — Méthodologie environnementale ICT   ↗               │
│  • ITU-T L.1410 — LCA pour les TIC                          ↗               │
│  • GHG Protocol Scope 3                                     ↗               │
│                                                                              │
│  BIBLIOGRAPHIE  (≥ 30 entrées)                            «  Voir tout  »   │
│                                                                              │
│  À PROPOS  •  Version v0.1.0 — référentiel 2026.05.10                       │
│            •  Licences : MIT (code), Etalab 2.0 (données), CC-BY (docs)      │
│            •  Sources : ADEME, RTE, Hugging Face, Data for Good, MaxMind…   │
└──────────────────────────────────────────────────────────────────────────────┘
```

**EF couvertes** : EF-M8-01 → EF-M8-04.

---

## Extension navigateur — UI overlay  (Module M11)

```
Sur chatgpt.com / claude.ai / mistral.ai / gemini.google.com / chat.lechat.ai :

┌─────────────────────────────────────┐
│  ChatGPT                            │
│                                     │
│  [ … fil de conversation … ]        │
│                                     │
│  ┌──────────────────────────────┐   │
│  │ Tape ton message…           │   │
│  └──────────────────────────────┘   │
│                                     │
│        ┌──────────────────────┐    │
│        │  🌱 Sobr.ia          │    │
│        │  Aujourd'hui :       │    │
│        │  • 47 prompts        │    │
│        │  • 89 g CO₂eq        │    │
│        │  • ≈ 0,7 km voiture  │    │
│        │  📊 Voir détail      │    │
│        └──────────────────────┘    │
└─────────────────────────────────────┘

Popover détail (clic sur le badge) :

┌────────────────────────────────────────────┐
│ 🌱 Sobr.ia — Vie réelle                    │
├────────────────────────────────────────────┤
│ Aujourd'hui :    47 prompts, 89 g CO₂eq    │
│ Cette semaine :  211 prompts, 412 g        │
│ Ce mois :        842 prompts, 1,67 kg      │
│                                            │
│ Top modèle : ChatGPT-4o (62 % de l'impact) │
│ Hypothèse mix élec : US-East live          │
│                                            │
│ ☑ Notification hebdo bilan                 │
│ ☑ Bridge vers l'app Sobr.ia (local)        │
│ ☐ Mode privé (pas de stockage)             │
│                                            │
│ «  Ouvrir l'app  »    «  Paramètres  »     │
│                                            │
│ 🔒 100 % local — aucune donnée envoyée     │
└────────────────────────────────────────────┘
```

**EF couvertes** : EF-M11-01 → EF-M11-07.

---

## Onboarding (premier lancement)

```
Étape 1/4 — Bienvenue
┌────────────────────────────────────────┐
│ 🌱 Sobr.ia                             │
│ Mesurez la sobriété de votre IA.       │
│                                        │
│ • 100 % local                          │
│ • Méthodologie AFNOR SPEC 2314         │
│ • Open source                          │
│                                        │
│           «  Commencer  »              │
└────────────────────────────────────────┘

Étape 2/4 — Détection de votre zone
┌────────────────────────────────────────┐
│ 🌍 On a détecté : France (Île-de-Fr)   │
│ Mix électrique de référence : RTE      │
│                                        │
│ ☑ Utiliser cette détection             │
│                                        │
│        «  Suivant  »                   │
└────────────────────────────────────────┘

Étape 3/4 — Profil
┌────────────────────────────────────────┐
│ Quel est votre cas d'usage principal ? │
│                                        │
│ ( ) Curiosité personnelle              │
│ (●) Reporting RSE entreprise           │
│ ( ) Recherche / journalisme            │
│ ( ) Dev intégrant un LLM               │
│ ( ) Décision publique                  │
│                                        │
│        «  Suivant  »                   │
└────────────────────────────────────────┘

Étape 4/4 — Première estimation guidée
┌────────────────────────────────────────┐
│ Essayez : « Résume-moi la photosynthèse│
│  en 500 mots » avec GPT-4o-mini        │
│                                        │
│          «  Estimer  »                 │
│        «  Passer  »                    │
└────────────────────────────────────────┘
```

---

## Composants Svelte transversaux à isoler

| Composant | Rôle | Réutilisé dans |
|-----------|------|----------------|
| `<MetricCard>` | Affiche une métrique avec intervalle | Estimer, Comparer, Workbench |
| `<UncertaintyBand>` | Bande P5-P95 sur un graphe | Estimer, Simuler |
| `<HistogramMC>` | Histogramme Monte-Carlo | Estimer |
| `<SankeyEnergy>` | Sankey énergétique (D3) | Estimer, Workbench |
| `<HeatmapModels>` | Heatmap comparative | Comparer |
| `<ChoroplethMap>` | Carte mix électrique | Géoloc, Simuler |
| `<SourcePopover>` | Pop-up source cliquable | partout |
| `<EquivalentBadge>` | "≈ X km voiture" | Estimer, Simuler |
| `<LocalIndicator>` | Bandeau "100 % local" | Shell |
| `<LedgerHash>` | Affiche un hash audit | Estimer, Journal |

---

## Spécifications visuelles (Skeleton CSS custom)

- **Palette principale (mode sombre par défaut)** : fond `#0d1117`, surface `#161b22`, texte `#c9d1d9`, accent vert `#3fb950` (sobriété), accent ambre `#d29922` (vigilance), erreur `#f85149`.
- **Palette claire** : fond `#ffffff`, surface `#f6f8fa`, texte `#24292f`.
- **Typographie** : Inter Variable (UI) + JetBrains Mono (chiffres et code). Auto-hébergées.
- **Espacements** : grille 4 px, hiérarchie 4/8/16/24/40/64.
- **Coins arrondis** : 4 px sur les inputs, 8 px sur les cartes.
- **Ombres** : aucune ombre portée massive — bordures 1 px subtiles.
- **Animations** : 150-250 ms `ease-out` max, désactivables (préférence utilisateur).
- **Dataviz** : palettes Viridis / Cividis (daltoniens) ; vert sobr.ia → rouge en gradient.

---

## États globaux à anticiper

- **Pas de connexion** : bandeau jaune en haut, données mix élec = moyennes annuelles.
- **Référentiel obsolète (> 7 j)** : suggère mise à jour, bouton « Mettre à jour ».
- **Première utilisation** : onboarding 4 étapes.
- **Données entreprise importées** : badge « Mode entreprise » dans le shell.
- **Audit ledger compromis** : alerte rouge + impossibilité de générer rapports tant que pas réinitialisé.

---

## Inventaire des écrans

| # | Écran | Module | Sprint cible | Priorité |
|---|-------|--------|--------------|----------|
| 1 | Estimer | M2 | S6 | P0 |
| 2 | Workbench | M3 | S7 | P0 |
| 3 | Comparer | M5 | S7 | P0 |
| 4 | Simuler | M4 | S8 | P0 |
| 5 | Importer | M10 | S7 | P0 |
| 6 | Géolocaliser | M9 | S6 | P0 |
| 7 | Exporter | M6 | S10 | P0 |
| 8 | Journal | M7 | S5-S10 | P1 |
| 9 | Méthodologie | M8 | S10 | P1 |
| — | Onboarding | shell | S10 | P1 |
| — | Extension overlay | M11 | S8 | P0 |

---

*Cette maquette est une référence visuelle pour Claude Code. Toute déviation = ADR.*
