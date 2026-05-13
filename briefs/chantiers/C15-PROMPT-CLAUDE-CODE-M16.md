# Prompt Claude Code — M16 Forecaster 12 mois (bande d'incertitude)

> À transmettre **après** que Claude Code ait fini M13 (qui introduit
> Chart.js distributions, réutilisé ici).

---

```
Tu es Claude Code, en charge du frontend pour le chantier C15 / module M16.

OBJECTIF
========
Implémenter l'écran "Forecaster 12 mois" qui projette l'empreinte cumulée
sur 12 mois avec une bande d'incertitude P5/P50/P95 visible, et permet de
superposer plusieurs scénarios de croissance d'adoption (status quo,
accélération, ralentissement, …).

CONTEXTE OBLIGATOIRE À LIRE
============================
1. briefs/chantiers/C15-forecaster-12-mois.md — spec complète.
2. crates/sobria-app/src/dto.rs — YearlyForecastRequestDto,
   YearlyScenarioDto, YearlyForecastResultDto, YearlyScenarioOutcomeDto.
3. docs/CAHIER-DES-CHARGES-v1.0.md §4 M16 et §4.3 (Bande d'incertitude
   = composant transverse réutilisable).
4. CLAUDE.md §13 — zéro mock, données réelles uniquement.

CONTRAT IPC
===========
- `forecast_yearly_budget({req: YearlyForecastRequestDto}) -> YearlyForecastResultDto`
- Erreurs typées : `unknown_model`, `estimator_error` (bornes : 10
  scénarios max, 1-60 mois, growth ±50%, volume 0-10⁶).

LIVRABLES
=========

A) Route `/m16` layout 2 colonnes :
   - Gauche (1/3) : 3 sliders + sélecteur modèle + bouton "Lancer le forecast"
   - Droite (2/3) : visualisation bande d'incertitude + cumul annuel

B) Inputs côté gauche :
   - Sélecteur modèle (depuis listModels(), défaut gpt-4o-mini)
   - Slider "Volume / jour" (1-10000 prompts/jour, défaut 100)
   - Slider "Tokens out" (50-2000, défaut 500)
   - Slider "Horizon" (3-60 mois, défaut 12)
   - Multi-scénarios : par défaut 3 cards prédéfinis (status quo 0%,
     accélération +5%/mois, ralentissement -3%/mois). L'utilisateur peut
     en ajouter/supprimer (max 10), modifier label + growth%.
   - Bouton "Lancer le forecast" : debounce 500ms après dernier changement,
     OU clic manuel. Loading state pendant l'IPC.

C) Visualisation côté droit :
   - **Graphe principal** : courbes mensuelles superposées par scénario.
     - Axe X : mois (1..N selon horizon)
     - Axe Y : gCO2eq cumulé (avec switch "mensuel | cumulatif")
     - Pour chaque scénario : une courbe P50 + une **bande remplie**
       entre P5 et P95 (alpha 0.15 pour visualiser l'incertitude sans
       surcharger).
     - Légende : pour chaque scénario, couleur + label + annual_p50
       avec intervalle [P5–P95].
     - Hover : crosshair vertical + tooltip avec valeurs pour tous les
       scénarios à ce mois.
   - **Carte annuelle** :
     - Pour chaque scénario, gros chiffre annuel_p50 (Instrument Serif)
       + intervalle d'incertitude en pill (P5–P95).
     - Équivalent parlant (× km voiture, × douches) sous le chiffre.
   - **Toggle "Masquer la bande d'incertitude"** : permet de cacher la
     zone P5-P95 pour comparer plus clairement les P50 entre scénarios.

D) Implémentation graphe :
   - Réutiliser Chart.js (déjà importé en C09 pour les bins).
   - Pour la bande remplie : 2 datasets `fill: '-1'` (P95 dessine au-dessus,
     P5 remplit vers P95).
   - Ou D3 SVG si plus contrôlable — au choix Claude Code, mais 1 lib max.

CONTRAINTES UX
==============
- Design system existant (tokens app.css), Instrument Serif pour les
  chiffres P50 annuels, Geist pour le corps.
- A11y :
  - Sliders : labels associés + aria-valuenow + aria-valuemin/max
  - Graphe : aria-label descriptif + fallback table cachée pour
    screen readers (8-12 lignes max : un row par mois × N scénarios)
- Footer : "Calcul basé sur Monte-Carlo N=10⁴, seed 42. Voir
  Méthodologie." (lien vers /methodo).

CONTRAINTE MÉTHODOLOGIQUE OBLIGATOIRE
======================================
Afficher en haut de la visualisation, en discret mais visible :

> « Cette projection assume une croissance géométrique constante. Elle
>   n'intègre pas la saisonnalité, ni les ruptures technologiques
>   (changement de modèle, gains d'efficacité datacenter). À utiliser
>   comme cadrage budgétaire, pas comme prédiction. »

DEFINITION OF DONE
==================
- [ ] Route `/m16`.
- [ ] 3 sliders + multi-scénarios fonctionnels avec validation.
- [ ] Graphe avec bande d'incertitude P5-P95 visible (alpha 0.15).
- [ ] Toggle "masquer la bande".
- [ ] Carte annuelle par scénario (P50 + intervalle).
- [ ] Tooltip / crosshair fonctionnel.
- [ ] Erreurs IPC typées affichées.
- [ ] Disclaimer méthodologique visible.
- [ ] `npm run check && npm run lint` verts.
- [ ] 1 test Playwright "no-mock contract".
- [ ] Screenshot dans commit.

À NE PAS FAIRE
==============
- Pas de saisonnalité (différé v1.1).
- Pas plus de 10 scénarios par appel IPC.
- Pas de comparaison automatique entre modèles (c'est M3 Comparer, pas M16).
- Pas de prévision au-delà de 60 mois.
- Pas de simulation Monte-Carlo côté front — c'est Rust qui calcule.

NOTE
====
Le forecast journalise le baseline dans l'audit ledger (1 entrée par appel
IPC). Si tu lances 10 forecasts pour explorer des paramètres, tu auras
10 entrées d'audit. C'est intentionnel — chaque forecast est un acte
analytique daté reproductible.
```
