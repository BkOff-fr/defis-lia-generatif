# Prompts Claude Code — M15 Dashboard + M25 Eco-budget

> Deux écrans liés méthodologiquement (mêmes données = ledger d'audit)
> mais routes séparées. Ordre suggéré : M15 d'abord (plus simple),
> puis M25 (s'appuie sur les concepts du dashboard).

---

## Prompt 1 — M15 Dashboard personnel

```
Tu es Claude Code, en charge du frontend pour le chantier C19 / module M15.

OBJECTIF
========
Écran "Dashboard personnel" — vue récapitulative de l'usage IA d'un·e
utilisateur·rice sur 5 périodes prédéfinies : Aujourd'hui, 7 derniers jours,
Ce mois-ci, Mois précédent, Cette année. Affiche métriques cumulées, top
modèles, time series journalière et comparaison vs période précédente.

CONTEXTE OBLIGATOIRE À LIRE
============================
1. briefs/chantiers/C19-dashboard-eco-budget.md (§1 M15)
2. crates/sobria-app/src/dto.rs — DashboardSummaryDto, DashboardComparisonDto,
   TopModelDto, DailySeriesPointDto.
3. CLAUDE.md §13 — zéro mock.

CONTRAT IPC
===========
- `get_dashboard_summary({period: string}) -> DashboardSummaryDto`
- period ∈ "today" | "last_7_days" | "this_month" | "last_month" | "this_year"
- Erreur : `invalid_request` si période inconnue.

LIVRABLES
=========

A) Route `/m15` :
   - Header avec **switch périodes** (5 boutons / tabs) :
     "Aujourd'hui | 7 derniers jours | Ce mois-ci | Mois précédent | Cette année"
   - Reload silencieux sur changement de période.

B) Section "Synthèse" (cards 4 colonnes) :
   - **Total requêtes** (gros chiffre Instrument Serif)
     - Sous-texte : delta_requests_pct vs période précédente (badge
       lime si -, coral si +)
   - **CO2eq total** (gros chiffre, unité gCO2eq ou kgCO2eq selon échelle)
     - Sous-texte : delta_co2eq_pct
   - **Énergie** (Wh ou kWh)
   - **Eau** (L ou mL)

C) Section "Évolution journalière" :
   - Chart.js bar chart : 1 barre par jour (date sur X, requêtes sur Y)
   - Tooltip au hover : date, count, CO2eq, énergie, eau pour ce jour
   - Couleur lime pour les barres < moyenne, coral pour les barres > moyenne × 1.5

D) Section "Top 5 modèles" :
   - Tableau ou liste cards :
     - Rank (1-5)
     - Nom du modèle (avec badge calibration)
     - request_count
     - total_co2eq_g_p50 (barre horizontale proportionnelle au gagnant=100%)
   - Click sur un modèle → ouvre la fiche M9 (?id=...)

E) Pied de page : "Période affichée : [period_start] → [period_end]"

CONTRAINTES UX
==============
- Design system existant (tokens app.css)
- Réutiliser Chart.js (importée en C09)
- A11y :
  - Tabs/switch périodes : aria-selected, navigation flèches
  - Chart bar : aria-label + fallback table
  - Cards stats : aria-live="polite" pour les màj au changement de période
- Si vs_previous est null → afficher juste les totaux sans delta.

DEFINITION OF DONE
==================
- [ ] Route `/m15` avec 5 périodes
- [ ] 4 cards stats avec deltas % colorés
- [ ] Time series bar chart avec tooltip
- [ ] Top 5 modèles avec lien M9
- [ ] Erreurs IPC typées affichées
- [ ] `npm run check && npm run lint` verts
- [ ] 1 test Playwright "no-mock contract"
- [ ] Screenshot dans commit

À NE PAS FAIRE
==============
- Pas de période custom (différé v1.1)
- Pas de drilldown par jour (différé)
- Pas de comparaison N périodes (1 vs N-1 seulement)
```

---

## Prompt 2 — M25 Eco-budget personnel

```
Tu es Claude Code, en charge du frontend pour le chantier C19 / module M25.

OBJECTIF
========
Écran "Objectifs & habitudes" — permet à l'utilisateur de définir des
objectifs (budgets) personnels par indicateur et période, et visualiser
en temps réel sa consommation vs ses budgets avec niveaux d'alerte
(ok / warning / exceeded).

CONTEXTE OBLIGATOIRE À LIRE
============================
1. briefs/chantiers/C19-dashboard-eco-budget.md (§2 M25)
2. crates/sobria-app/src/dto.rs — PersonalGoalDto, BudgetStatusDto.
3. CLAUDE.md §13.

CONTRATS IPC
============
- `list_personal_goals() -> PersonalGoalDto[]`
- `set_personal_goal({goal: PersonalGoalDto}) -> void`
- `delete_personal_goal({indicator, period}) -> void`
- `get_budget_status() -> BudgetStatusDto[]`

Erreurs typées : `invalid_request` (indicateur/période/unit/value invalides).

LIVRABLES
=========

A) Route `/m25` layout 2 colonnes :
   - Gauche (1/3) : formulaire d'ajout/modification
   - Droite (2/3) : liste des objectifs actifs + statut

B) Formulaire (gauche) :
   - Select indicateur : 3 options
     - "CO₂eq" (unité : gCO2eq)
     - "Énergie" (unité : Wh)
     - "Eau" (unité : L)
   - Select période : 3 options
     - Quotidien (par jour)
     - Hebdomadaire (par semaine)
     - Mensuel (par mois calendaire)
   - Input value_max (number, > 0)
   - Unit auto-affichée d'après indicateur (lecture seule)
   - Bouton "Enregistrer l'objectif"
   - L'unité est validée côté Rust (rejet si incohérent).

C) Liste objectifs (droite) :
   - Pour chaque objectif (récup via get_budget_status) :
     - Titre : "CO2eq / Mensuel" + bouton supprimer (icône trash)
     - Barre de progression horizontale :
       - Remplissage = current_value / value_max
       - Couleur selon status : lime (ok), jaune (warning), coral (exceeded)
       - Texte au-dessus : "X gCO2eq / Y gCO2eq (Z%)"
     - Sous-texte : "Période [period_start] → [period_end]"
     - Si status = "exceeded" : badge "Dépassé de N gCO2eq" en coral
     - Si status = "warning" : badge "Plus que N gCO2eq restant" en jaune

D) État vide :
   - Si aucun objectif : illustration discrète + texte
     "Aucun objectif défini. Commencez par définir un budget mensuel
     CO2eq pour suivre votre empreinte IA."

E) Edition d'un objectif existant :
   - Click sur la barre d'un objectif → préremplit le formulaire
   - Submit avec mêmes (indicateur, période) → UPSERT (replace en backend)

CONTRAINTES UX
==============
- Design system existant
- Pas de notification système (différé v1.1, voir M21 Alertes)
- A11y :
  - Form fields avec labels associés
  - Barres de progression : role="progressbar" + aria-valuenow/min/max
  - Trash button avec aria-label "Supprimer objectif {indicateur} {période}"
  - Status badges avec aria-live="polite"

DEFINITION OF DONE
==================
- [ ] Route `/m25`
- [ ] Formulaire avec validation côté front (value > 0)
- [ ] Liste des objectifs avec barres de progression colorées
- [ ] Click barre → édition
- [ ] Suppression avec confirmation (dialog native)
- [ ] État vide géré
- [ ] Erreurs IPC typées affichées
- [ ] `npm run check && npm run lint` verts
- [ ] 1 test Playwright "no-mock contract"
- [ ] Screenshot dans commit

À NE PAS FAIRE
==============
- Pas de notifications système (M21 Alertes, différé)
- Pas d'objectifs partagés équipe (c'est M19)
- Pas de périodes custom (jour 15-30) — strictement daily/weekly/monthly
- Pas d'historique des dépassements (différé v1.1)

NOTE MÉTHODOLOGIQUE À AFFICHER
===============================
En footer du formulaire ou en tooltip discret :
"Les périodes weekly suivent ISO 8601 (lundi 00:00 → dimanche 23:59).
Les valeurs sont calculées en P50 (médiane Monte-Carlo) — voir
Méthodologie pour le détail."
```
