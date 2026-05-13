# Chantier #10 — Onboarding personas + module gating

> **Pré-requis** : v0.2.0-estimer mergé (C09 complet, Tauri runtime actif).
> **Crates touchées** : `sobria-app` (préférences + IPC), `sobria-core` (Persona/ModuleId enums).
> **Frontend** : `web/` (route `/onboarding`, store `preferences.ts`, garde de layout, garde de routes).
> **Durée cible** : 2-3 jours.
> **Référence ADR** : [ADR-0010](../../docs/adr/ADR-0010-personas-and-module-gating.md).
> **Référence CDC** : v1.3 §3 (personas) + §4 (25 modules).

---

## 0. Objectif

Implémenter le **gating modulaire** de Sobr.ia : un seul binaire, 25 modules,
5 personas qui précochent un bundle, et un utilisateur libre de composer son
expérience. Au premier lancement, un **wizard d'onboarding** propose les
5 personas et permet de personnaliser. Les préférences sont persistées
dans `referentiel.sqlite` (table `app_preferences`). Le rail UI filtre
automatiquement, les routes des modules désactivés redirigent.

## 1. Types `Persona` et `ModuleId` (sobria-core)

Nouveau module `sobria-core/src/preferences.rs`. Deux enums fermés, sérialisables.

```rust
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Cinq personas figés en v1.3 (cf. ADR-0010 + CDC §3).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Persona {
    Student,
    ProTech,
    Enterprise,
    PublicSector,
    Researcher,
}

/// Identifiants stables des 25 modules (CDC §4.1).
/// L'ordre déclaré ici est l'ordre par défaut dans le rail UI.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ModuleId {
    M1, M2, M3, /* M4 réservé */
    M5, M6, M7, M8, M9, M10, M11, M12, M13, M14, M15,
    M16, M17, M18, M19, M20, M21, M22, M23, M24, M25,
}

impl Persona {
    /// Bundle par défaut associé à un persona.
    #[must_use]
    pub fn default_modules(self) -> Vec<ModuleId> {
        use ModuleId::*;
        match self {
            Persona::Student =>
                vec![M1, M8, M11, M13, M14, M15, M24, M25],
            Persona::ProTech =>
                vec![M1, M2, M3, M5, M7, M9, M11, M18, M21],
            Persona::Enterprise =>
                vec![M1, M2, M5, M6, M7, M10, M12, M16, M19, M21, M22],
            Persona::PublicSector =>
                vec![M1, M5, M6, M7, M8, M12, M16, M20, M22, M23],
            Persona::Researcher =>
                vec![M1, M3, M5, M8, M14, M17, M18],
        }
    }
}
```

**Tests** (5+) :
- chaque persona produit un bundle non vide,
- M1 (Estimer) est présent dans **tous** les bundles (point fixe),
- pas de doublons dans un bundle,
- l'ID `M4` n'apparaît jamais (réservé, retiré v1.3),
- `serde` round-trip JSON sur les 5 personas et les 25 modules.

Re-export dans `sobria-core/src/lib.rs` :

```rust
pub use preferences::{ModuleId, Persona};
```

## 2. State côté `sobria-app`

### 2.1 Table SQLite

Ajouter dans `AppState::init` la création de la table si absente :

```sql
CREATE TABLE IF NOT EXISTS app_preferences (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);
```

Cette table vit dans `referentiel.sqlite` (pas dans `audit.sqlite`).
Si le fichier référentiel n'existe pas encore au premier boot (cas v0.2),
on le crée à côté de `audit.sqlite` dans `data_root`.

### 2.2 DTO IPC

Nouveau fichier `crates/sobria-app/src/dto.rs` (extension de l'existant) :

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppPreferencesDto {
    pub persona: Option<Persona>,
    pub enabled_modules: Vec<ModuleId>,
    pub onboarded: bool,
    pub lang: String, // "fr" | "en"
}
```

### 2.3 Commandes IPC

Deux commandes dans `main.rs` (déléguant à `logic::*`) :

```rust
#[tauri::command]
fn get_app_preferences(state: tauri::State<'_, AppState>) -> IpcResult<AppPreferencesDto>;

#[tauri::command]
fn set_app_preferences(
    prefs: AppPreferencesDto,
    state: tauri::State<'_, AppState>,
) -> IpcResult<()>;
```

Comportement de `get_app_preferences` :
- Si la table `app_preferences` est vide → retourne valeurs par défaut :
  `{ persona: None, enabled_modules: Persona::ProTech.default_modules(), onboarded: false, lang: "fr" }`.
- Sinon → reconstitue depuis les 4 clés `persona`, `enabled_modules`, `onboarded`, `lang`. Les clés manquantes utilisent les défauts.

Comportement de `set_app_preferences` :
- Valide : persona dans le set des 5, modules dans le set des 25, lang ∈ {"fr","en"}.
- Écrit les 4 clés en transaction (UPSERT).
- Met à jour `updated_at` à `Utc::now().to_rfc3339()`.

### 2.4 Tests Rust

Ajouter dans `logic::tests` :
- `get_returns_defaults_on_empty_db`,
- `set_then_get_round_trips`,
- `set_rejects_unknown_module`,
- `set_rejects_unknown_persona`,
- `set_rejects_unknown_lang`,
- `set_overwrites_previous`,
- `default_bundle_for_each_persona_includes_m1`.

## 3. Frontend SvelteKit

### 3.1 Store de préférences

`web/src/lib/preferences.ts` :

```ts
import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export type Persona = 'student' | 'pro_tech' | 'enterprise' | 'public_sector' | 'researcher';

export type ModuleId =
  | 'm1' | 'm2' | 'm3' | 'm5' | 'm6' | 'm7' | 'm8' | 'm9' | 'm10'
  | 'm11' | 'm12' | 'm13' | 'm14' | 'm15' | 'm16' | 'm17' | 'm18'
  | 'm19' | 'm20' | 'm21' | 'm22' | 'm23' | 'm24' | 'm25';

export interface AppPreferences {
  persona: Persona | null;
  enabled_modules: ModuleId[];
  onboarded: boolean;
  lang: 'fr' | 'en';
}

const initial: AppPreferences = {
  persona: null,
  enabled_modules: [],
  onboarded: false,
  lang: 'fr',
};

export const preferences = writable<AppPreferences>(initial);

export async function loadPreferences(): Promise<void> {
  const p = await invoke<AppPreferences>('get_app_preferences');
  preferences.set(p);
}

export async function savePreferences(p: AppPreferences): Promise<void> {
  preferences.set(p); // optimistic
  try {
    await invoke('set_app_preferences', { prefs: p });
  } catch (e) {
    // rollback : recharger depuis le backend
    await loadPreferences();
    throw e;
  }
}
```

### 3.2 Garde de layout

`web/src/routes/+layout.svelte` ajoute en `onMount` :
- charge les préférences,
- si `!onboarded && route !== '/onboarding'` → `goto('/onboarding')`.

Le rail filtre ses entrées via `$preferences.enabled_modules.includes(moduleId)`.
Tooltip persistant en bas du rail : « + Ajouter des modules » → ouvre `/parametres`.

### 3.3 Route `/onboarding`

4 étapes en wizard (state local, pas de route nestée) :

1. **Splash** (≈ 4 sec ou clic « Continuer ») : logo + tagline + mission 1 phrase.
2. **Persona picker** : 5 cartes cliquables (illustration discrète, titre, 1 phrase).
   Lien discret « Je préfère choisir à la carte → ».
3. **Bundle** : checkboxes des modules du persona (pré-cochés) + section
   collapsable « Plus de modules disponibles » avec les 25 - bundle.
   Description courte par module (1 ligne).
4. **Premier prompt guidé** (optionnel, skippable) : tooltip contextuel
   pointant le sélecteur de modèle sur M1.

Au clic « C'est parti », on appelle :
```ts
await savePreferences({
  persona: chosenPersona,
  enabled_modules: chosenModules,
  onboarded: true,
  lang: 'fr',
});
goto('/');
```

### 3.4 Route `/parametres`

Nouvelle route (toujours visible dans le rail, pas de gating) :
- Switch persona (avec dialog de confirmation : « Remplacer la sélection actuelle ? »).
- Toggle module à la carte.
- Bouton « Refaire l'onboarding » → set `onboarded = false`, `goto('/onboarding')`.
- Switch langue FR/EN (pour préparation chantier i18n).
- Section « Modules disponibles non activés » avec teaser 1-liner.

### 3.5 Garde par route module

Chaque `+page.svelte` d'un module désactivable commence par :

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

La page d'accueil `/` affiche, si `?disabled=mXX` :
> « Le module XX n'est pas activé. → Activer dans Paramètres ».

## 4. Definition of Done

### Rust
- [ ] `Persona` et `ModuleId` dans `sobria-core` avec 7+ tests.
- [ ] Table `app_preferences` créée à `AppState::init`.
- [ ] 2 commandes IPC `get_app_preferences` / `set_app_preferences`.
- [ ] 7 tests `logic::tests` couvrant defaults, round-trip, rejets.
- [ ] `cargo clippy -p sobria-core -p sobria-app -- -D warnings` propre.

### Frontend
- [ ] Store `preferences.ts` typé strictement, optimistic + rollback.
- [ ] Garde de layout → redirige vers `/onboarding` si pas onboardé.
- [ ] Route `/onboarding` à 4 étapes, design extrait du système v1 (ink/lime/ivory).
- [ ] Rail filtre selon `enabled_modules`.
- [ ] Garde de route sur **chaque** route module (au moins M1 et M13 en C10, les autres lors de leur chantier).
- [ ] Route `/parametres` fonctionnelle (switch persona, toggle module, refaire onboarding).
- [ ] `npm run lint && npm run check` verts.

### E2E
- [ ] 1 test Playwright : premier lancement → onboarding visible → choix Étudiant → bundle pré-coché → `goto('/')` → rail montre 8 entrées (bundle étudiant).
- [ ] 1 test Playwright : depuis paramètres → toggle M22 → revenir au rail → M22 apparaît.
- [ ] 1 test Playwright : ouvrir `/m13` alors que M13 désactivé → redirige sur `/?disabled=m13`.

### Doc
- [ ] CHANGELOG : entrée v0.3.0 « Onboarding personas + module gating (ADR-0010) ».
- [ ] `briefs/chantiers/C10-RETROSPECTIVE.md` après livraison.

## 5. Non-objectifs (différés)

- **Bundles partagés / export-import** (ex: « partager mon bundle avec un collègue ») → chantier C11+.
- **i18n EN complète** : on prépare la structure mais la traduction est différée.
- **Tutoriel interactif** (au-delà du tooltip étape 4) → backlog v1.1.
- **Mode multi-utilisateurs** (plusieurs profils par installation) → backlog v1.1.

## 6. Risques

| Risque | Probabilité | Parade |
|---|---|---|
| Drift entre IDs Rust (`ModuleId`) et IDs TS (`ModuleId` string) | Moyen | Test e2e qui valide `Object.keys(EnabledMap)` côté TS == set Rust |
| Migration de la table `app_preferences` lors d'un futur changement de schéma | Faible (v1.0) | Clé `schema_version` ajoutée plus tard |
| Onboarding intrusif → l'utilisateur l'évite | Moyen | Lien « passer » discret + bundle pro_tech par défaut |
| 25 cases à cocher saturent l'étape 3 | Moyen | Bundle pré-coché en haut, autres modules dans section collapsable |

## 7. Séquençage

C10 est le **chantier débloqueur** pour C11 (M13), C12 (M12), C13 (M20),
C14 (M22), etc. Tous les futurs modules ajoutent leur ID à `ModuleId` et
leur entrée dans le rail, mais le wiring du gating est posé une bonne fois
ici.

---

*Brief rédigé par Cowork. Exécution en C10.1 (Rust + IPC) puis C10.2
(SvelteKit + design). Validation finale par Thibault sur l'écran
onboarding rendu.*
