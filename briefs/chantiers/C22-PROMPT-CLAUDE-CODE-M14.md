# Prompt Claude Code — M14 À propos / Crédits

> Petit chantier de fermeture : page statique « À propos » pour boucler
> les 13 modules essentiels v1.0. Aucun backend nouveau requis (utilise
> `meta_info` existant).

---

```
Tu es Claude Code, en charge du frontend pour le chantier C22 / module M14.

OBJECTIF
========
Écran "À propos / Crédits" — page statique légale et de transparence
qui boucle les 13 modules essentiels v1.0 (cf. ADR-0011). Module
**présent dans tous les bundles persona** par défaut.

CONTEXTE OBLIGATOIRE À LIRE
============================
1. docs/adr/ADR-0011-reduction-perimetre-v1-0.md — périmètre v1.0
2. docs/CAHIER-DES-CHARGES-v1.0.md v1.4 §4 (table modules)
3. CLAUDE.md §13 — zéro mock
4. crates/sobria-app/src/dto.rs — MetaInfo (déjà disponible)

CONTRAT IPC (existant, à réutiliser)
=====================================
- `meta_info() -> MetaInfo` (déjà livré C09)
  - app_version, estimator_seed, estimator_n, audit_path, data_root

LIVRABLES
=========

A) Route `/a-propos` (slug FR cohérent avec /journal, /methodo) OU
   `/m14` (slug mNN cohérent avec /m9, /m15, /m25). Choix : prends
   le slug **/a-propos** pour rester naturel pour les utilisateurs.

B) Layout simple, single column, max-width 800px, scroll vertical :

   - **Header** : Logo Sobr.ia + version (depuis meta_info)
   - **Mission** : 1 paragraphe court
     "Sobr.ia mesure l'impact environnemental de l'usage des LLMs avec
     rigueur scientifique. Open source, frugal, transparent. Candidat au
     défi data.gouv.fr 'L'impact environnemental de l'IA générative'."

   - **Section "Méthodologie"** :
     - "Notre estimateur applique l'AFNOR SPEC 2314"
     - "Monte-Carlo N=10⁴ tirages, seed déterministe (42)"
     - "Validation croisée à ±15% contre Luccioni 2023 et EcoLogits 2024"
     - Lien : "Voir la méthodologie complète →" vers /methodo

   - **Section "Licences"** :
     - Sobr.ia (code) : **MIT** — lien GitHub repo
     - Données ODRÉ (RTE/NaTran/Teréga) : **Etalab 2.0**
     - Données AFNOR SPEC 2314 : publique
     - Polices : **SIL Open Font License 1.1** (Geist, Instrument Serif, JetBrains Mono)
     - Documentation : **CC-BY 4.0**

   - **Section "Sources des données"** :
     - HF AI Energy Score (calibration ε prefill/decode)
     - RTE eco2mix (mix élec FR annuel)
     - Electricity Maps + AIB (mix élec EU annuel par pays)
     - ADEME Base Empreinte (équivalents parlants)
     - Mytton 2021 (water usage effectiveness)
     - Luccioni et al. 2023 (validation modèles)
     - Gebru et al. 2018, arXiv:1803.09010 (standard datasheet)

   - **Section "Contributeurs"** :
     - Thibault (auteur / mainteneur)
     - Cowork (assistance architecture)
     - Claude Code (assistance code)
     - Ouvert aux contributions externes — lien GitHub Issues

   - **Section "Mentions légales"** :
     - Aucune donnée envoyée à un serveur externe
     - Audit ledger local en SQLite + chiffrement WAL
     - RGPD : droit à l'oubli implémenté (commande IPC `purge_audit_before`)
     - Aucune télémétrie, aucun tracking

   - **Section "État technique"** (depuis meta_info) :
     - Version : v{app_version}
     - Seed Monte-Carlo : {estimator_seed}
     - N tirages : {estimator_n}
     - Chemin ledger : {audit_path} (en monospace, copiable)
     - Racine données : {data_root} (idem)

   - **Footer discret** :
     - "© 2026 Sobr.ia · MIT · Made in France"
     - Build hash si dispo

CONTRAINTES UX
==============
- Design system existant (tokens app.css)
- A11y : headings hiérarchiques h1>h2, liens externes avec rel="noopener noreferrer"
- Liens cliquables (URLs publiques) : ouvrir dans le navigateur système
  via `tauri-plugin-shell::open()` si l'URL pointe hors-app, sinon `<a target="_blank">`
- Pas de markdown rendering nécessaire (texte structuré HTML suffit)

DEFINITION OF DONE
==================
- [ ] Route `/a-propos` avec contenu structuré
- [ ] meta_info appelé au mount, version + chemins affichés
- [ ] Liens URL vers GitHub, sources scientifiques fonctionnels
- [ ] Boutons "copier" sur les chemins audit/data (clipboard)
- [ ] `npm run check && npm run lint` verts
- [ ] 1 test Playwright minimal (la route charge sans erreur)
- [ ] Ajouter l'entrée dans `+layout.svelte` rail (section "Audit") :
      ```ts
      { label: 'À propos', icon: Info, href: '/a-propos', moduleId: 'm14' }
      ```
      (importer Info depuis @lucide/svelte)
- [ ] Screenshot dans commit

À NE PAS FAIRE
==============
- Pas d'animation tape-à-l'œil (page statique posée)
- Pas de formulaire de contact (différé v1.1)
- Pas d'i18n (FR par défaut v1.0)

DURÉE ESTIMÉE
=============
30-45 minutes (page statique + texte + intégration meta_info)
```
