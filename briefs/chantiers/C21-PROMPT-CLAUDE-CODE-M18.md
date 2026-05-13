# Prompt Claude Code — M18 Batch CSV → rapport agrégé

> À transmettre après M17 (synergie naturelle : batch CSV alimente un projet).

---

```
Tu es Claude Code, en charge du frontend pour le chantier C21 / module M18.

OBJECTIF
========
Écran "Batch CSV" — l'utilisateur charge un CSV de N prompts, lance
l'estimation en lot, voit un rapport agrégé, et peut exporter les
résultats dans un CSV de sortie.

CONTEXTE OBLIGATOIRE À LIRE
============================
1. briefs/chantiers/C21-batch-csv.md — format CSV, validations, bornes.
2. crates/sobria-app/src/dto.rs — BatchRequestDto, BatchResultDto,
   BatchAggregateDto, BatchModelAggregateDto.
3. CLAUDE.md §13 — zéro mock.

CONTRAT IPC
===========
- `run_batch_from_csv({req: BatchRequestDto}) -> BatchResultDto`
- Erreurs typées : `invalid_request` (fichier absent, format, > 1000 lignes,
  > 50% rejected), `io_error`.

FORMAT CSV ATTENDU (à afficher en aide)
========================================
```
model_id,tokens_in,tokens_out,datacenter_id
gpt-4o-mini,100,500,
claude-3-5-sonnet,200,1000,aws-eu-west-3-paris
```

LIVRABLES
=========

A) Route `/m18` layout vertical :
   1. Section "Charger un CSV"
   2. Section "Aperçu" (visible après chargement)
   3. Section "Résultats" (visible après run)

B) Section "Charger un CSV" :
   - **Drag-and-drop zone** : zone visible (border dashed) avec
     "Glissez votre CSV ici ou cliquez pour sélectionner"
   - Alternative : input file (accept=".csv")
   - **Quand fichier déposé** :
     - Lecture native côté front avec FileReader pour aperçu
     - Validation simple : .csv extension + < 5 MB
     - Affichage chemin du fichier (le chemin absolu sera utilisé pour
       l'IPC — Tauri 2 expose le chemin via `webkitRelativePath` ou
       `Tauri::path`)
   - Bouton "Lancer le batch" (disabled tant qu'aucun fichier)
   - Toggle "Exporter les résultats" + champ chemin (optionnel)
     - Si activé, ouvre save dialog Tauri pour choisir output_csv_path

C) Section "Aperçu" (avant lancement) :
   - Tableau des 10 premières lignes du CSV (lecture front via FileReader)
   - Compteur "X lignes au total"
   - Validation visuelle :
     - Headers reconnus en vert (lime)
     - Tokens hors bornes ou modèles non listés (highlight coral)
   - Note : "Max 1000 lignes par batch."

D) Section "Résultats" (après run_batch_from_csv) :
   - **Carte synthèse** :
     - rows_processed (gros chiffre, lime si > 0)
     - rows_rejected (gros chiffre, coral si > 0)
     - total_co2eq_g_p50, total_energy_wh_p50, total_water_l_p50
     - avg / min / max CO2eq P50
   - **Graphe barres horizontales — Top modèles par CO2eq** :
     - 1 barre par modèle dans by_model (trié desc)
     - Couleur tokens design system
     - Tooltip : count + total + moyenne
   - **Lien fichier de sortie** :
     - Si output_csv_path présent : bouton "Ouvrir le CSV de sortie"
       (utiliser plugin-shell pour ouvrir avec l'app par défaut, ou copier
       chemin dans presse-papier)
   - **Lien audit** :
     - "Cette analyse a journalisé X entrées d'audit (id {first} → {last}).
        Voir Journal d'audit →" (lien vers /m7 si configuré)
   - **Bouton "Créer un projet à partir de ce batch"** : 
     - Pré-remplit le formulaire M17 avec :
       - name : "Batch du {date}"
       - description : "{N} prompts importés depuis {filename}"
       - period_start : audit entry first timestamp
       - period_end : audit entry last timestamp + 1 minute
     - Navigation vers /m17 avec query params

CONTRAINTES UX
==============
- Design system existant.
- A11y :
  - Drag-and-drop : alternative clavier (input file accessible)
  - aria-live="polite" sur la section résultats pour annoncer le succès
  - Tableau aperçu : role="table", focus visible
- Performance : si > 100 lignes, afficher progress bar pendant l'IPC
  (loading state simple v1.0 — pas de streaming réel)

DEFINITION OF DONE
==================
- [ ] Route `/m18`
- [ ] Drag-and-drop + file picker fonctionnels
- [ ] Aperçu 10 premières lignes
- [ ] Lancement run_batch_from_csv avec loading state
- [ ] Affichage résultats avec graphe top modèles
- [ ] Export CSV optionnel (save dialog Tauri)
- [ ] Bouton "Créer un projet" qui chain vers /m17
- [ ] Erreurs IPC typées affichées (file not found, format, rejections, etc.)
- [ ] `npm run check && npm run lint` verts
- [ ] 1 test Playwright "no-mock contract"
- [ ] Screenshot dans commit

À NE PAS FAIRE
==============
- Pas de support Parquet en entrée (différé v1.1)
- Pas plus de 1000 lignes (cap strict côté Rust)
- Pas de streaming pour gros fichiers (différé)
- Pas d'estimation parallèle (séquentielle côté Rust v1.0)
- Pas de format CSV alternatif (header strict v1.0)

NOTE MÉTHODOLOGIQUE À AFFICHER
===============================
Sous la section "Charger un CSV", en tooltip :
"Chaque ligne du CSV génère une estimation Monte-Carlo (N=10⁴ tirages)
journalisée dans le ledger d'audit. Un batch de 1000 lignes prend ~10s
et produit 1000 entrées d'audit. Pour des datasets plus larges, utilisez
le module M17 Empreinte projet pour grouper sous un même libellé."
```
