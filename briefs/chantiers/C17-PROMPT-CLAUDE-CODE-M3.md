# Prompt Claude Code — M3 Comparer modèles

> À transmettre dans l'ordre M13 → M20 → M12 → M22 → M16 → **M3**.
> M3 réutilise les composants barres + bins déjà introduits par M13.

---

```
Tu es Claude Code, en charge du frontend pour le chantier C17 / module M3.

OBJECTIF
========
Écran "Comparer modèles" : sélectionner N modèles (1-20), lancer un
benchmark sur un prompt commun, visualiser le classement par CO2eq /
énergie / eau avec barres horizontales + fiche calibration par modèle.

CONTEXTE OBLIGATOIRE À LIRE
============================
1. briefs/chantiers/C17-comparer-modeles.md — spec complète.
2. crates/sobria-app/src/dto.rs — BenchmarkRequestDto, BenchmarkResultDto,
   BenchmarkOutcomeDto.
3. docs/CAHIER-DES-CHARGES-v1.0.md §4 M3.
4. CLAUDE.md §13.

CONTRAT IPC
===========
- `benchmark_models({req: BenchmarkRequestDto}) -> BenchmarkResultDto`
- Erreurs : `invalid_request` (empty/duplicate/too many), `unknown_model`.

LIVRABLES
=========

A) Route `/m3` layout 2 colonnes :
   - Gauche (1/3) : configuration prompt + sélecteur modèles
   - Droite (2/3) : résultats benchmark

B) Configuration (gauche) :
   - Slider "tokens_in" (1-2000, défaut 100)
   - Slider "tokens_out" (1-5000, défaut 500)
   - Multi-sélecteur modèles : checklist depuis listModels() avec :
     - Nom du modèle + provider
     - Badge calibration (validated / indicative / extrapolated) avec
       couleur tokens design (lime/jaune/coral)
     - Badge openness (open / open_weights / closed)
   - Compteur "X / 20 sélectionnés"
   - Bouton "Lancer le benchmark" (disabled si < 1 ou > 20).

C) Résultats (droite) :
   - **Carte synthèse** :
     - Modèle gagnant CO2eq (en gros, lime)
     - Modèle le moins performant (en coral)
     - Ratio gagnant/perdant (× ou %)
   - **3 graphes barres horizontales** (CO2eq / Énergie / Eau) :
     - Une barre par modèle, longueur = P50, segments lighter pour P5-P95
     - Triées du meilleur au pire pour chaque indicateur
     - Couleur fixe par modèle (palette stable entre les 3 graphes)
     - Hover : tooltip avec P5/P50/P95 + unité
     - Click : ouvre une drawer avec les détails complets du modèle
       (provider, family, sources de calibration)
   - **Table récapitulative** (optionnelle, bonus) :
     - Lignes = modèles, colonnes = (rang CO2eq, rang énergie, rang eau,
       openness, calibration)
     - Tri par n'importe quelle colonne

D) Insight UX :
   - Si l'écart entre P5/P95 du gagnant chevauche le P50 du suivant,
     affiche un avertissement discret : « Écart non significatif
     statistiquement entre modèles X et Y. »
   - (Cf. C12 méthodologique : pour les petits prompts, l'embodied
     domine, les écarts entre modèles peuvent être bruités.)

CONTRAINTES UX
==============
- Design system existant (tokens app.css).
- Réutiliser Chart.js (déjà importé en C09 pour les bins).
- A11y :
  - Multi-sélecteur : checkboxes vraies + label associé.
  - Graphes : aria-label descriptif + fallback table cachée.
- Pas de plus de 1 lib externe nouvelle.

DEFINITION OF DONE
==================
- [ ] Route `/m3`.
- [ ] Multi-sélecteur modèles fonctionnel avec badges.
- [ ] 3 graphes barres + tooltip.
- [ ] Drawer détail au click modèle.
- [ ] Carte synthèse gagnant/perdant.
- [ ] Avertissement écarts non-significatifs.
- [ ] Erreurs IPC typées.
- [ ] `npm run check && npm run lint` verts.
- [ ] 1 test Playwright "no-mock contract".
- [ ] Screenshot dans commit.

À NE PAS FAIRE
==============
- Pas de comparaison sur N prompts (différé v1.1).
- Pas de coût € par modèle (différé — pas de source publique fiable).
- Pas de recommandation automatique "le meilleur pour ton cas" — laisser
  l'utilisateur arbitrer.
- Pas plus de 20 modèles par appel IPC.

NOTE LEDGER
===========
Chaque benchmark crée N entrées d'audit (1 par modèle). L'utilisateur le
sait : afficher discrètement « N entrées d'audit journalisées » sous
la carte synthèse.
```
