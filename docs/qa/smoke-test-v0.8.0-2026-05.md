# Smoke-test v0.8.0 — 5 personas onboarding

> **But** : valider manuellement avant publication GitHub que les 5
> parcours persona produisent le résultat attendu après C32 (clarté
> produit). Ce document est un **runbook** : à exécuter en
> environnement de dev avant le tag final, et à archiver avec son
> résultat dans ce même fichier (section « Résultats ») après chaque
> release polish.
>
> **Cible** : Thibault (ou tout opérateur qui prépare la candidature
> data.gouv.fr).
>
> **Pré-requis** : `cargo tauri dev` lancé localement, base SQLite
> fraîche (`rm crates/sobria-app/data.sqlite*` avant chaque persona).

---

## Pré-vol

1. Build Tauri à jour :
   ```bash
   cargo tauri dev
   ```
2. Pour chaque persona, réinitialiser les préférences :
   ```bash
   rm -f data/app.sqlite data/app.sqlite-wal data/app.sqlite-shm
   ```
   (ou utiliser le bouton « Réinitialiser l'onboarding » dans
   `/parametres` quand il sera livré v1.x).
3. Vider le localStorage du WebView (DevTools → Application → Local
   Storage → tout supprimer) pour repartir d'un état neutre pour la
   bannière « Et après ? » et le flag `welcome_skipped`.

---

## Critères à vérifier (par persona)

Pour **chaque** persona ci-dessous, valider la séquence :

| # | Étape | Attendu | Statut |
|---|---|---|---|
| 1 | Lancement app | Onboarding wizard s'affiche, splash + brand mark | ⬜ |
| 2 | Splash auto-advance (3 s) ou clic « Continuer » | Étape 2 « Sobr.ia en 30 secondes » visible | ⬜ |
| 3 | Étape 2 — Schéma SVG d'équivalence | « 1 prompt typique = 1,14 g CO₂eq ≈ 5 m voiture » visible | ⬜ |
| 4 | Clic « Continuer » | Étape 3 Persona Picker visible avec 5 cartes | ⬜ |
| 5 | Sélection persona | Étape 4 Bundle avec modules pré-cochés du persona | ⬜ |
| 6 | Tooltip survol module dans bundle | Texte « Pourquoi ce module ? » s'affiche (persona-spécifique) | ⬜ |
| 7 | Clic « Continuer » | Étape 5 Ready avec phrase d'accueil + bouton « Ouvrir l'atelier » | ⬜ |
| 8 | Clic « Ouvrir l'atelier » | Redirection vers `/` (M1 Atelier) avec rail filtré au bundle persona | ⬜ |
| 9 | 1er prompt + clic « Estimer l'impact » | ResultBlock + ligne équivalence humaine sous le bloc | ⬜ |
| 10 | Bannière « Et après ? » apparaît | 3 cartes Comparer / Dashboard / Eco-budget visibles | ⬜ |
| 11 | Clic dismiss bannière (×) | Bannière disparaît | ⬜ |
| 12 | Refresh page | Bannière ne réapparaît pas (localStorage persisté) | ⬜ |

---

## Persona 1 — 🎓 Étudiant·e / Curieux·se

**Bundle attendu** (5 modules, mirror Rust) :

- M1 Estimer un prompt
- M8 Méthodologie
- M13 Simulateur « Et si...? »
- M15 Tableau de bord personnel
- M25 Eco-budget

**Tooltips attendus** (sample) :

- M1 → « Mesurer chaque prompt pour comprendre l'ordre de grandeur (CO₂, eau). »
- M25 → « Fixer un objectif mensuel + alerte quand tu le dépasses. »

**Test exécuté le** : `__/__/____` par `_______________`
**Résultat** : ⬜ Pass · ⬜ Fail (cause :  `_______________`)

---

## Persona 2 — 💻 Pro Tech (dev, ML eng)

**Bundle attendu** (6 modules) :

- M1 Estimer un prompt
- M3 Comparer modèles
- M7 Journal d'audit
- M8 Méthodologie
- M9 Bibliothèque de modèles
- M13 Simulateur « Et si...? »

**Tooltips attendus** (sample) :

- M3 → « Comparer 3 modèles côte-à-côte pour choisir le plus frugal. »
- M9 → « Catalogue 25+ modèles avec P5/P50/P95 + vendor disclosure (Mistral × ADEME, Google, Meta). »

**Test exécuté le** : `__/__/____` par `_______________`
**Résultat** : ⬜ Pass · ⬜ Fail (cause :  `_______________`)

---

## Persona 3 — 🏢 Entreprise (DSI, RSE)

**Bundle attendu** (8 modules) :

- M1 Estimer un prompt
- M7 Journal d'audit
- M12 Datacenters Europe
- M15 Tableau de bord personnel
- M17 Datasheet scientifique
- M20 Territoire FR
- M22 Rapport réglementaire (CSRD/AGEC)
- M25 Eco-budget

**Tooltips attendus** (sample) :

- M22 → « Rapport CSRD/AGEC trimestriel signé + PROV-O. »
- M17 → « Datasheet Gebru pour reproductibilité scientifique. »

**Vérifications additionnelles spécifiques** :

- `/parametres` → section Mode Équipe → panneau **« Activer Mode Équipe »** visible (URL non configurée).
- Clic « Activer Mode Équipe (mon entreprise) » → dialog 3 étapes s'ouvre, ESC le ferme.
- Étape 2 du dialog : bloc `<pre><code>` avec 3 commandes copy-paste.
- Lien « Voir le guide complet (5 minutes) » → ouvre `team-aggregator.md` sur GitHub.

**Test exécuté le** : `__/__/____` par `_______________`
**Résultat** : ⬜ Pass · ⬜ Fail (cause :  `_______________`)

---

## Persona 4 — 🏛️ Public Sector (collectivité, service public)

**Bundle attendu** (6 modules) :

- M1 Estimer un prompt
- M8 Méthodologie
- M12 Datacenters Europe
- M17 Datasheet scientifique
- M20 Territoire FR
- M22 Rapport réglementaire (CSRD/AGEC)

**Tooltips attendus** (sample) :

- M20 → « Empreinte par IRIS RTE : différenciateur FR unique de Sobr.ia. »
- M8 → « Méthodologie AFNOR SPEC 2314 + sources officielles FR (ADEME, RTE). »

**Vérifications additionnelles spécifiques** :

- `/territoire` → carte IRIS chargée (200 sites industriels), Sankey
  énergétique national rendu.
- `/rapport-csrd` → titre « Rapport réglementaire (CSRD/AGEC) »
  (renommé en C32.1, pas l'ancien « Rapport CSRD/AGEC »).

**Test exécuté le** : `__/__/____` par `_______________`
**Résultat** : ⬜ Pass · ⬜ Fail (cause :  `_______________`)

---

## Persona 5 — 🔬 Researcher / Journaliste

**Bundle attendu** (6 modules) :

- M1 Estimer un prompt
- M3 Comparer modèles
- M7 Journal d'audit
- M8 Méthodologie
- M9 Bibliothèque de modèles
- M17 Datasheet scientifique

**Tooltips attendus** (sample) :

- M1 → « Atelier reproductible (seed SOBRIA_SEED=42). »
- M9 → « Catalogue P5/P50/P95 + vendor disclosure (transparence multi-méthodo). »

**Vérifications additionnelles spécifiques** :

- `/m9` → table comparaison vendor disclosure visible (Mistral / Google
  / Meta / Anthropic / OpenAI), Anthropic + OpenAI marqués
  « Pas de disclosure officielle ».
- Clic sur Mistral Large 2 → fiche détaillée avec encadré « Données
  vendor disclosure » (3 cartes : training tCO₂eq, training m³ eau,
  inference gCO₂eq/400 tokens).
- `notebook/validation.qmd` reste reproductible : `quarto render` sans
  erreur (testé hors-app).

**Test exécuté le** : `__/__/____` par `_______________`
**Résultat** : ⬜ Pass · ⬜ Fail (cause :  `_______________`)

---

## Vérifications transverses (à faire 1 fois)

| # | Vérification | Statut |
|---|---|---|
| T1 | Aucun label « M1 », « M3 », « M9 »... visible dans l'UI utilisateur (rail nav, breadcrumbs, titres de page, module-rows) | ⬜ |
| T2 | Section « Méthodologies disponibles » du README préservée intacte | ⬜ |
| T3 | M14 « À propos » reste accessible via le rail (`/a-propos`) même si retiré des bundles | ⬜ |
| T4 | EquivalenceCarbon affiche bien des sources au survol de chaque équivalent (ADEME / Shift Project) | ⬜ |
| T5 | M15 Dashboard → ligne « Cette période, vous avez consommé l'équivalent de : … » visible | ⬜ |
| T6 | M25 Eco-budget → équivalence sous chaque progress bar de budget actif | ⬜ |
| T7 | M9 page principale → table comparaison vendor disclosure visible | ⬜ |
| T8 | Cargo tests verts : `cargo test --workspace` (peut être skip si CI verte) | ⬜ |
| T9 | Web checks : `cd web && npm run check && npm run lint` clean | ⬜ |
| T10 | DOI Zenodo : `.zenodo.json` présent à la racine, lisible (JSON valide) | ⬜ |

---

## Quand re-jouer ce smoke-test

- Avant le tag `v0.8.0` final (cf. C32.5).
- Avant chaque release `v0.X.Y` qui touche à l'onboarding, aux bundles
  persona, ou aux 5 modules d'entrée (M1 / M8 / M15 / M22 / M25).
- Après tout merge de PR qui modifie `web/src/routes/onboarding/` ou
  `crates/sobria-core/src/preferences.rs`.

---

## Résultats

> Compléter au fil des passes. Garder l'historique pour traçabilité.

### Passe 1 — 2026-05-17 (pré-tag v0.8.0)

- Opérateur : Thibault
- Build : `cargo tauri dev` sur Windows 11
- Résumé : *à compléter après exécution*
- Findings éventuels : *à compléter*
- Décision de ship : ⬜ Go · ⬜ No-go
