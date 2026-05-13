# Prompt à transmettre à Claude Code — C10 onboarding + module gating

> **Mode d'emploi** : copier tout ce qui suit le séparateur `--- PROMPT ---`
> et le coller dans Claude Code à la racine du repo. Le prompt est
> autonome — Claude Code lit déjà CLAUDE.md à chaque session.
>
> **Pré-requis** : C09 mergé, runtime Tauri actif, lucide migré vers
> `@lucide/svelte`. Étape D (Journal d'audit) livrée et taggée.
> Cowork aura livré la partie Rust de C10 (types, table, IPC) **avant**
> que Claude Code ne démarre la partie frontend décrite ci-dessous.

---

## --- PROMPT ---

```
Tu es Claude Code, en charge du frontend pour le chantier C10
(briefs/chantiers/C10-onboarding-personas.md).

OBJECTIF
========
Implémenter le wizard d'onboarding 4 étapes + le gating des modules dans
l'UI Sobr.ia, en consommant les commandes IPC `get_app_preferences` et
`set_app_preferences` livrées côté Rust par Cowork.

CONTEXTE OBLIGATOIRE À LIRE EN PREMIER
=======================================
1. docs/adr/ADR-0010-personas-and-module-gating.md — décision et schéma data
2. briefs/chantiers/C10-onboarding-personas.md — DoD complète
3. docs/CAHIER-DES-CHARGES-v1.0.md §3 (personas v2) et §4 (25 modules)
4. crates/sobria-core/src/preferences.rs — enums Persona + ModuleId (source de vérité)
5. crates/sobria-app/src/dto.rs — AppPreferencesDto
6. web/src/lib/api.ts — wrapper existant à étendre
7. CLAUDE.md §13 — pas de mock, données réelles uniquement

ENJEUX UX
=========
Sobr.ia sert 5 publics aux exigences opposées :
- l'étudiant·e qui ne veut PAS voir 25 modules le premier jour,
- l'entreprise qui veut TOUT voir, particulièrement le rapport CSRD,
- la collectivité qui veut la cartographie territoriale,
- le chercheur qui veut le batch CSV et l'empreinte projet,
- le pro tech qui veut le workbench et les comparaisons modèles.

L'onboarding est la première impression. Il doit être :
- court (4 étapes max, < 90 secondes),
- non-bloquant (lien "passer" discret),
- réversible (refaire l'onboarding via Paramètres),
- visuellement aligné avec le design system v1 (ink #1A1A1A / lime #D5F265 /
  ivory #F5F0E8, fonts Instrument Serif + Geist + JetBrains Mono).

LIVRABLES ATTENDUS
==================

A) web/src/lib/preferences.ts — store typé strict (modèle dans le brief C10 §3.1)
   - Types `Persona`, `ModuleId`, `AppPreferences` mirrorant exactement le
     DTO Rust (snake_case).
   - `loadPreferences()` au boot via `invoke<AppPreferences>('get_app_preferences')`.
   - `savePreferences(p)` optimistic + rollback IPC.
   - Helpers : `defaultModulesFor(persona)` (mirror Rust), `moduleLabel(id)`,
     `moduleDescription(id)`, `personaLabel(p)`, `personaTagline(p)`.

B) web/src/routes/+layout.svelte — garde de layout
   - `onMount`: `await loadPreferences()`, puis si `!onboarded && route !== '/onboarding'`
     → `goto('/onboarding', { replaceState: true })`.
   - Rail vertical filtre ses entrées via `$preferences.enabled_modules.includes(...)`.
   - Tooltip persistant en bas du rail : « + Ajouter des modules » → `/parametres`.

C) web/src/routes/onboarding/+page.svelte — wizard 4 étapes (state local Svelte 5 runes)

   Étape 1 — Splash (auto-advance après 3s ou clic "Continuer") :
   - logo Sobr.ia centré (logo de design system, pas placeholder)
   - tagline italique : "Mesurez la sobriété de votre IA générative"
   - 1 phrase de mission : "Une mesure scientifique, accessible à tout le monde."
   - bouton primaire lime "Continuer"

   Étape 2 — Persona picker :
   - titre H2 italic : "Vous êtes…"
   - 5 cartes cliquables, layout flex/grid responsive :
     - 🎓 Étudiant·e / Curieux·se — "Comprendre votre impact, apprendre les bons réflexes"
     - 🧑‍💻 Professionnel·le tech — "Optimiser vos prompts, comparer les modèles, exporter pour votre équipe"
     - 🏢 Entreprise — "Piloter votre scope 3 IA, rapport CSRD, forecast budget carbone"
     - 🏛️ Collectivité / Service public — "Suivre votre empreinte territoriale, marchés publics frugaux"
     - 🔬 Chercheur·se / Journaliste — "Reproductibilité, comparaisons inter-modèles, datasets publiables"
   - lien discret en bas : "Je préfère choisir à la carte →" (saute étape 2 avec persona=null)
   - hover state : carte se lève légèrement, contour lime

   Étape 3 — Bundle (après choix persona) :
   - titre H2 : "Voici votre première sélection"
   - sous-titre : "Vous pourrez la modifier à tout moment dans Paramètres."
   - 8-11 checkboxes pré-cochées correspondant au bundle persona (modules avec
     icône lucide + label + 1 ligne de description)
   - section collapsable "+ Plus de modules disponibles" qui révèle les
     25 - bundle restants, non cochés
   - bouton primaire lime "C'est parti", bouton ghost "Précédent"

   Étape 4 — Premier prompt guidé (skippable) :
   - tooltip pointing animé sur le sélecteur de modèle M1
   - "Essayez votre premier prompt. Choisissez un modèle, écrivez 50-200 tokens,
     cliquez Estimer."
   - bouton "Terminer" (ferme onboarding, set onboarded=true, goto '/')
   - lien "Passer cette étape" (idem)

D) web/src/routes/parametres/+page.svelte — paramètres
   - 4 sections :
     1. Persona courant (avec bouton "Changer" → dialog confirmation +
        relance du picker en place, remplace bundle)
     2. Modules activés (liste interactive avec toggle ou checkbox, groupés
        par catégorie : « Estimation », « Visualisation », « Reporting »,
        « Pédagogie »)
     3. Modules disponibles non activés (avec teaser 1 ligne)
     4. Bouton « Refaire l'onboarding » + langue FR/EN (pour préparer i18n)

E) Garde de route minimale pour M1 et M13 (les seuls modules présents au
   moment de C10). Pattern à appliquer dans chaque module futur :

   ```svelte
   <script lang="ts">
     import { onMount } from 'svelte';
     import { goto } from '$app/navigation';
     import { preferences } from '$lib/preferences';
     import { get } from 'svelte/store';

     const MODULE_ID = 'm13';

     onMount(() => {
       if (!get(preferences).enabled_modules.includes(MODULE_ID)) {
         goto('/?disabled=' + MODULE_ID, { replaceState: true });
       }
     });
   </script>
   ```

   Et sur `+page.svelte` (M1), afficher si `$page.url.searchParams.get('disabled')` :
   un bandeau coral discret « Le module XX n'est pas activé. → Activer dans
   Paramètres » (lien cliquable).

F) Tests Playwright dans web/tests/onboarding.spec.ts :
   - test 1 : premier lancement (state vide) → onboarding visible →
     choisir Étudiant → bundle pré-coché contient 8 modules → cliquer
     "C'est parti" → atterrir sur '/' → rail montre les 8 modules.
   - test 2 : depuis /parametres → toggle M22 ON → goto '/' → rail
     contient M22.
   - test 3 : depuis paramètres, modifier persona → confirmation dialog →
     bundle Entreprise remplace bundle Étudiant.
   - test 4 : ouvrir /m13 directement alors que M13 désactivé → redirection
     '/?disabled=m13' + bandeau visible.

G) Mettre à jour web/src/lib/api.ts pour ajouter les wrappers IPC
   `getAppPreferences()` et `setAppPreferences(prefs)` typés.

CONTRAINTES NON-NÉGOCIABLES (rappel CLAUDE.md §13)
====================================================
1. Pas de mock, pas de fallback, pas de données factices. Si IPC indisponible
   (contexte non-Tauri), throw immédiat (cf api.ts existant).
2. TypeScript strict avec `exactOptionalPropertyTypes`.
3. Svelte 5 runes obligatoire ($state, $derived, $effect).
4. A11y : tous les inputs labellés, contraste WCAG AA, focus trap dans
   l'onboarding modal-like.
5. Pas de framework UI lourd. Composants extraits sur le design system
   ink/lime/ivory existant.
6. `@lucide/svelte` pour les icônes (PAS `lucide-svelte` qui est legacy).
7. Aucun localStorage : la persistance passe par l'IPC vers SQLite.

WORKFLOW SUGGÉRÉ
=================
1. Lis ADR-0010 + brief C10 + dto.rs.
2. Étends api.ts (E) — 2 fonctions, 10 lignes.
3. Écris preferences.ts (A) + helpers de labels.
4. Pose la garde de layout (B).
5. Écris la route /onboarding (C) — la plus longue.
6. Écris la route /parametres (D).
7. Pose les gardes de route sur M1 et M13.
8. Tests Playwright (F).
9. Vérifie : `npm run lint && npm run check` verts.
10. Test manuel : `cargo tauri dev`, supprime ton fichier
    `referentiel.sqlite` pour simuler premier lancement, vérifie le flow
    complet, refais l'onboarding, change de persona.
11. Commit unique : `feat(web): onboarding personas + module gating (C10)`.

DEFINITION OF DONE C10 FRONTEND
================================
- [ ] api.ts a getAppPreferences + setAppPreferences typés
- [ ] preferences.ts store optimistic + rollback
- [ ] /onboarding 4 étapes pixel-perfect avec le design system
- [ ] /parametres complet (switch persona, toggle module, refaire onboarding)
- [ ] Rail filtré par enabled_modules
- [ ] Garde de route sur M1 et M13
- [ ] 4 tests Playwright verts
- [ ] npm run lint && npm run check verts
- [ ] Screenshot des 4 étapes onboarding + paramètres dans le commit
- [ ] CHANGELOG entrée v0.3.0-onboarding

Ping-moi quand C10 est livré, je validerai visuellement et on enchaînera
sur C11 (M13 Simulateur « Et si...? »).
```

---

## Notes (hors-prompt — pour toi, Thibault)

- C10 doit être joué **après** que la partie Rust soit livrée (Cowork) + que
  l'étape D Journal d'audit soit livrée par Claude Code (en cours après
  validation lucide).
- Une fois C10 mergé, on tag `v0.3.0-onboarding` et on enchaîne sur C11.
- Si Claude Code rapporte qu'un design Claude Design n'existe pas pour
  l'onboarding/paramètres : il doit composer à partir des tokens existants
  (ink/lime/ivory + Instrument Serif). Pas de redesign visuel structurant
  hors validation.
