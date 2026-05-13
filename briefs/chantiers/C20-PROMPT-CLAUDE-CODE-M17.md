# Prompt Claude Code — M17 Empreinte projet (datasheet Gebru)

> À transmettre après les écrans déjà cadrés (M13, M20, M12, M22, M16, M3, M9,
> M15, M25). M17 est ciblé persona « Chercheur·se / Journaliste ».

---

```
Tu es Claude Code, en charge du frontend pour le chantier C20 / module M17.

OBJECTIF
========
Écran "Empreinte projet" — permet à un·e chercheur·se de documenter une
étude / un article / un benchmark sous forme de **projet** persistant
puis de générer son **datasheet Gebru 2018** au format JSON-LD
(reproductibilité scientifique, standard académique).

CONTEXTE OBLIGATOIRE À LIRE
============================
1. briefs/chantiers/C20-empreinte-projet-datasheet.md — spec complète,
   7 sections Gebru, format JSON-LD.
2. crates/sobria-app/src/dto.rs — ProjectDto, CreateProjectDto,
   UpdateProjectDto, DatasheetDto, CompositionDto.
3. Référence académique : Gebru et al. 2018, arXiv:1803.09010.
4. CLAUDE.md §13 — zéro mock.

CONTRATS IPC
============
- `list_projects() -> ProjectDto[]`
- `get_project({id}) -> ProjectDto`
- `create_project({req: CreateProjectDto}) -> ProjectDto`
- `update_project({id, req: UpdateProjectDto}) -> ProjectDto`
- `delete_project({id}) -> void`
- `generate_project_datasheet({id}) -> DatasheetDto`

Erreurs typées : `not_found`, `invalid_request`.

LIVRABLES
=========

A) Route `/m17` layout 2 colonnes :
   - Gauche (1/3) : liste des projets en cards + bouton "Nouveau projet"
   - Droite (2/3) : panel contextuel — création, édition, ou rendu datasheet
                    selon l'action.

B) Liste projets (gauche) :
   - Card par projet :
     - Nom (Instrument Serif)
     - Description tronquée (2 lignes)
     - Période : "01/01/2026 → 01/04/2026" en monospace
     - Tags (pills lime)
     - Bouton actions : Voir datasheet | Éditer | Supprimer (icône trash)
   - État vide : "Aucun projet. Créez votre premier projet pour générer un
     datasheet selon le standard académique Gebru et al. 2018."

C) Formulaire création (panel droite quand "Nouveau projet") :
   - Input nom (text, requis, max 200 chars, compteur)
   - Textarea description (max 5000 chars, compteur)
   - Date pickers période_début / période_fin
   - Input tags (chips ajoutables, max 10, validation a-z0-9- côté front)
   - Bouton "Créer le projet"
   - Erreur invalid_request : affichage inline du message renvoyé

D) Formulaire édition (panel droite quand "Éditer") :
   - Mêmes champs SAUF dates (read-only, tooltip "Dates immutables pour
     préserver la reproductibilité")
   - Bouton "Enregistrer"

E) Vue datasheet (panel droite quand "Voir datasheet") :
   - **Header** : nom du projet + bouton "Régénérer" + SHA-256 affiché en
     monospace (signature de provenance)
   - **Composition** (card) :
     - total_requests, total_co2eq_g_p50, total_energy_wh_p50, total_water_l_p50
     - Liste des modèles uniques (chips)
     - Date première / dernière entrée
   - **Sections Gebru** (7 cards repliables, dans l'ordre) :
     1. Motivation (sobria:motivation)
     2. Composition (sobria:composition) — résumé visuel
     3. Collection process (sobria:collectionProcess)
     4. Preprocessing (sobria:preprocessing)
     5. Uses (sobria:uses)
     6. Distribution (sobria:distribution) — incl. licences
     7. Maintenance (sobria:maintenance)
   - **Actions** :
     - "Copier le JSON-LD" → copie dans le presse-papier le `jsonld` brut
       (utiliser navigator.clipboard.writeText)
     - "Télécharger en .jsonld" → save dialog Tauri (plugin-dialog) → écrit
       le fichier sur disque
     - "Générer rapport PDF CSRD" → appelle `export_csrd_report` (C14) avec
       les dates du projet → ouvre le PDF résultant. Lien naturel entre
       datasheet (académique) et rapport (compliance).

F) Footer datasheet :
   - "Standard utilisé : Gebru et al. 2018 — Datasheets for Datasets.
     arXiv:1803.09010" (lien externe — vérifier CSP)
   - Liens vers les vocabulaires : schema.org, W3C PROV-O, DCAT

CONTRAINTES UX
==============
- Design system existant.
- A11y :
  - Formulaires : labels associés, aria-describedby pour erreurs
  - Sections repliables : aria-expanded, navigation clavier
  - Bouton "Supprimer" : confirmation modale (dialog HTML5 ou custom)
- Pas de markdown rendering dans description (texte brut suffisant v1.0)
- Cards repliables avec animation slide subtile (200ms ease).

DEFINITION OF DONE
==================
- [ ] Route `/m17` avec liste + panel contextuel.
- [ ] CRUD projets fonctionnel (create, update, delete) avec validation.
- [ ] Vue datasheet rendant les 7 sections Gebru.
- [ ] Copy-to-clipboard du JSON-LD.
- [ ] Save dialog → fichier .jsonld.
- [ ] Bouton "Générer PDF CSRD" qui chain vers C14.
- [ ] Erreurs IPC typées affichées (not_found, invalid_request).
- [ ] État vide géré.
- [ ] `npm run check && npm run lint` verts.
- [ ] 1 test Playwright "no-mock contract".
- [ ] Screenshot dans commit.

À NE PAS FAIRE
==============
- Pas d'édition des dates après création (immutables, voir brief §1.1).
- Pas de versioning du datasheet (différé v1.1).
- Pas de partage vers plateformes (Zenodo, OSF) — différé v1.1.
- Pas de tags hiérarchiques — flat tags suffisants v1.0.
- Pas de duplicate / fork de projet — backlog.

NOTE PERSONA
============
M17 est par défaut dans le bundle du persona "Chercheur·se" (voir
ADR-0010 §Personas). Pour les autres personas, le module reste activable
via Paramètres mais n'apparaît pas dans le rail par défaut.

NOTE MÉTHODOLOGIQUE À AFFICHER
===============================
En tooltip discret sur "Sections Gebru" :
"Format standard pour documenter datasets et modèles ML, publié par
Gebru, Morgenstern, Vecchione et al. en 2018. Adopté par les conférences
NeurIPS, ICML, FAccT et les revues scientifiques majeures. Sobr.ia
génère ce format automatiquement depuis votre ledger d'audit, pour
faciliter la publication scientifique reproductible."
```
