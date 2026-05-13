# Prompt Claude Code — M9 Référentiel modèles

> À transmettre après M3 (réutilise les barres + Chart.js déjà introduits).

---

```
Tu es Claude Code, en charge du frontend pour le chantier C18 / module M9.

OBJECTIF
========
Écran "Référentiel modèles" : catalogue browsable des 8 modèles de
référence avec fiches détaillées exposant les valeurs distributionnelles
réelles utilisées par l'estimateur. C'est l'écran de TRANSPARENCE
méthodologique — un chercheur ou journaliste doit pouvoir consulter
exactement quels chiffres Sobr.ia utilise et leur provenance.

CONTEXTE OBLIGATOIRE À LIRE
============================
1. briefs/chantiers/C18-referentiel-modeles.md
2. crates/sobria-app/src/dto.rs — ModelPresetDto, ModelDetailDto, TripletDto.
3. docs/methodology/MODEL-PRESETS.md (méthodologie de calibration).
4. CLAUDE.md §13.

CONTRATS IPC
============
- `list_models() -> ModelPresetDto[]` (déjà existante, ≥ 8 modèles)
- `get_model_detail({id}) -> ModelDetailDto` (NOUVELLE, expose triplets
  epsilon prefill/decode/embodied + baseline)

Erreurs : `not_found` si id de modèle inconnu.

LIVRABLES
=========

A) Route `/m9` layout responsive :
   - Vue grille (par défaut)
   - Vue détaillée (au click sur un modèle, slide-in panneau ou drawer)

B) Vue grille — cards par modèle :
   - Nom + provider (Instrument Serif pour le nom)
   - Badge calibration (validated lime / indicative jaune / extrapolated coral)
   - Badge openness (open / open_weights / closed) avec icône
   - Tagline : "~X B parameters" (en monospace JetBrains)
   - Carte baseline rapide : CO2eq P50 (gros chiffre) sur "100/500 tokens
     de référence"
   - Click → ouvre la vue détaillée

C) Vue détaillée (drawer plein-écran ou panneau side) :
   - Header : nom + provider + family + badges (calibration, openness)
   - Section "Plage de référence" :
     - 3 mini-graphes barres horizontales superposées (style "candles") :
       - ε_prefill (mJ/token) : 3 points P5/P50/P95 avec ligne
       - ε_decode (mJ/token)
       - embodied (g/req)
     - Échelle log si nécessaire (embodied peut être très petit)
   - Section "Baseline contextuel" :
     - "Pour un prompt 100 tokens in / 500 tokens out sur PUE 1.3 + IF FR :"
     - CO2eq P50 + intervalle [P5–P95]
     - Énergie P50 (Wh)
     - Eau P50 (L)
   - Section "Sources documentaires" :
     - Liste cliquable (URL ou citation BibTeX)
     - Si l'URL est valide, ouvre dans le navigateur système via
       `tauri-plugin-shell` (ou tag <a target="_blank"> avec CSP autorisée).
   - Section "Méthodologie" :
     - Phrase explicative selon calibration :
       - validated → "Calibré contre Luccioni 2023 et EcoLogits à ±15%."
       - indicative → "Calibré par ordre de grandeur depuis HF AI Energy Score."
       - extrapolated → "Extrapolé depuis un modèle ouvert comparable."
     - Lien vers /methodo pour le détail.

D) Filtres optionnels en haut de la grille :
   - Filtrer par provider (multi-select)
   - Filtrer par openness (single-select : tous / open / closed)
   - Filtrer par calibration (multi-select)

E) Tri optionnel :
   - Par nom (A-Z)
   - Par paramètres B (croissant / décroissant)
   - Par baseline CO2eq P50 (croissant — frugal en premier)

CONTRAINTES UX
==============
- Design system existant.
- Pas de nouvelle lib externe (Chart.js déjà importée pour les bins en C09).
- A11y :
  - Grille : navigation clavier, focus visible, role="grid" / "gridcell".
  - Drawer : trap focus quand ouvert, esc pour fermer.
  - Tous les graphes : aria-label descriptif + fallback table caché.
- Si l'URL d'une source pointe vers un domaine non autorisé par CSP,
  fallback affichage du texte brut sans lien.

DEFINITION OF DONE
==================
- [ ] Route `/m9` avec grille des 8+ modèles.
- [ ] Drawer détaillé fonctionnel avec sources cliquables.
- [ ] 3 mini-graphes barres P5/P50/P95 par modèle.
- [ ] Baseline contextuel affiché.
- [ ] Filtres + tri opérationnels.
- [ ] Erreurs IPC (`not_found`) gérées proprement.
- [ ] `npm run check && npm run lint` verts.
- [ ] 1 test Playwright "no-mock contract".
- [ ] Screenshot dans commit.

À NE PAS FAIRE
==============
- Pas d'édition / ajout de modèles utilisateur (différé v1.1).
- Pas de comparaison directe entre modèles dans cet écran (c'est M3).
- Pas de recommandation contextuelle (différé v1.1).
- Pas d'envoi de données vers un serveur externe pour valider les sources.

NOTE LEDGER
===========
`get_model_detail` ne journalise PAS dans l'audit ledger (fiche statique
pédagogique). Pas de side-effect à indiquer dans l'UI.
```
