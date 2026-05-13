# ADR-0010 — Personas et gating modulaire par préférences utilisateur

- **Statut** : Accepted
- **Date** : 2026-05-13
- **Décideurs** : Thibault, Cowork
- **Contexte** : Sprint S5 (post-C09 Estimer, avant déploiement des modules dataviz et reporting)

## Contexte et énoncé du problème

Sobr.ia doit servir cinq publics aux exigences très différentes :

- un·e **étudiant·e curieux·se** qui veut comprendre l'empreinte de ses propres prompts ChatGPT,
- un·e **professionnel·le tech** qui veut comparer des modèles, exporter pour son équipe et installer une extension navigateur,
- une **entreprise** (DSI, RSE) qui pilote son scope 3 IA, prépare un rapport CSRD et forecaste son budget carbone,
- une **collectivité** (région, métropole, ministère) qui suit son empreinte IA territoriale et orchestre des marchés publics frugaux,
- un·e **chercheur·se / journaliste** qui exploite Sobr.ia pour la reproductibilité d'études et la production d'articles sourcés.

Trois symptômes pratiques en découlent :

1. **Saturation cognitive** côté étudiant : afficher d'emblée le rail à 25 modules (cf. CDC v1.3 §4) noie le débutant et trahit la promesse de frugalité cognitive.
2. **Sous-équipement** côté entreprise : ouvrir uniquement les écrans grand public laisse les décideurs sans rapport CSRD ni forecast budget carbone — donc sans cas d'usage défendable.
3. **Pertes de focus produit** : pousser tout le monde vers la même expérience moyenne ne fait personne heureux et complique notre messaging dossier candidature.

Le besoin : permettre à chaque utilisateur de **n'activer que les modules qu'il/elle veut voir**, sans pour autant en faire un projet à plusieurs binaires. Une seule application, mais une **expérience composée** par profil.

## Décision

Adopter un **gating modulaire** côté frontend, piloté par une **préférence utilisateur persistée** côté backend (`AppState`), et amorcé par un **onboarding** au premier lancement qui propose un bundle préréglé par persona.

### Principes

1. **Un persona à la fois, librement modulable.**
   L'utilisateur choisit un persona unique au démarrage (5 options). Le persona précoche un bundle de modules. L'utilisateur peut ensuite **ajouter ou retirer librement** n'importe quel module à la carte. Le persona peut être changé à tout moment dans Paramètres (réamorce le bundle, en alertant que la sélection actuelle sera remplacée).

2. **Gating frontend uniquement, jamais backend.**
   Les commandes Tauri IPC restent toutes disponibles et testables — aucun module n'est désactivé côté Rust. Le gating se limite à :
   - filtrer les entrées visibles dans le rail de navigation,
   - garder les routes correspondantes derrière un check de préférences (redirection vers `/onboarding` ou message « Module désactivé — l'activer dans Paramètres »).

   Raison : on ne crée pas de surface de désactivation backend qui complexifierait les tests, le versionning d'API et la maintenance.

3. **Préférences persistées dans SQLite référentiel, pas dans LocalStorage.**
   Sobr.ia étant une app native multi-fenêtres potentiellement, et l'audit ledger exigeant une cohérence transactionnelle, les préférences vivent dans la base SQLite existante `referentiel.sqlite` (table `app_preferences`). Cela permet :
   - export propre des préférences avec le reste du référentiel,
   - cohérence ACID avec le reste de l'état app,
   - portabilité (copier la SQLite recrée l'expérience).

4. **Onboarding non-bloquant.**
   L'utilisateur peut **passer l'onboarding** à tout moment (lien discret) — il atterrit alors sur le bundle "Pro tech" par défaut (le plus équilibré), avec un tooltip "Personnaliser dans Paramètres". On ne barre jamais l'accès au produit.

5. **Pas de personas multiples ou hiérarchisés.**
   Pas de combinaisons (Étudiant + Chercheur). Pas de niveaux (Étudiant débutant vs Étudiant avancé). Un persona = un bundle = un changement explicite si on veut switcher. Simplification UX prioritaire sur la flexibilité maximale.

### Personas v2

| ID | Nom | Cible | Bundle par défaut |
|----|-----|-------|-------------------|
| `student` | Étudiant·e / Curieux·se | Apprentissage, usage personnel | M1, M8, M11, M13, M14, M15, M24, M25 |
| `pro_tech` | Professionnel·le tech | Dev, ML eng, intégrateur | M1, M2, M3, M5, M7, M9, M11, M18, M21 |
| `enterprise` | Entreprise (DSI, RSE) | Conformité, scope 3, forecast | M1, M2, M5, M6, M7, M10, M12, M16, M19, M21, M22 |
| `public_sector` | Collectivité / Service public | Territorial, achats responsables | M1, M5, M6, M7, M8, M12, M16, M20, M22, M23 |
| `researcher` | Chercheur·se / Journaliste | Reproductibilité, comparaisons | M1, M3, M5, M8, M14, M17, M18 |

(IDs de modules : voir CDC v1.3 §4.)

### Modèle de données

Une nouvelle table dans `referentiel.sqlite` :

```sql
CREATE TABLE app_preferences (
    key         TEXT PRIMARY KEY,    -- 'persona', 'enabled_modules', 'onboarded', 'lang'
    value       TEXT NOT NULL,       -- JSON sérialisé
    updated_at  TEXT NOT NULL        -- RFC 3339 UTC
);
```

Quatre clés réservées en v1.0 :
- `persona` → `"student" | "pro_tech" | "enterprise" | "public_sector" | "researcher" | null`
- `enabled_modules` → `["m1", "m13", "m22", ...]` (lowercase, ordonné)
- `onboarded` → `"true" | "false"`
- `lang` → `"fr" | "en"`

Tout autre `key` est ignorée par le frontend (forward-compatibility).

### Surface IPC

Deux nouvelles commandes Tauri dans `sobria-app` :

```rust
#[tauri::command]
fn get_app_preferences(state: tauri::State<'_, AppState>) -> IpcResult<AppPreferencesDto>;

#[tauri::command]
fn set_app_preferences(
    prefs: AppPreferencesDto,
    state: tauri::State<'_, AppState>,
) -> IpcResult<()>;
```

DTO partagé :

```rust
pub struct AppPreferencesDto {
    pub persona: Option<Persona>,
    pub enabled_modules: Vec<ModuleId>,  // lowercase, stable
    pub onboarded: bool,
    pub lang: String,                    // "fr" | "en"
}
```

Validation côté Rust (avant écriture SQLite) :
- `persona` doit être l'une des 5 valeurs connues ou `None`,
- `enabled_modules` doit contenir uniquement des IDs reconnus (set fermé v1.0 : 25 IDs),
- `lang` doit appartenir à `["fr", "en"]`.

Toute violation → `IpcError { code: "invalid_request", ... }`.

### Surface frontend

- **Store** `web/src/lib/preferences.ts` chargé au boot via `get_app_preferences()`, écrit via `set_app_preferences()` après mutation. Optimistic update local + rollback si IPC échoue.
- **Route** `/onboarding` (4 étapes, voir brief C10).
- **Garde de layout** `+layout.svelte` : si `!$preferences.onboarded && route !== '/onboarding'` → redirige vers `/onboarding`.
- **Filtre rail** : les entrées du rail sont filtrées via `$preferences.enabled_modules.includes(moduleId)`.
- **Garde de route** : chaque route `+page.svelte` d'un module commence par un `if (!enabled) goto('/?disabled=' + moduleId)`.
- **Paramètres** : page dédiée pour relancer l'onboarding, switcher persona, toggler modules à la carte.

## Conséquences

### Positives

- Une expérience qui s'adapte sans démultiplier les binaires ni les distributions.
- Un onboarding qui sert de **pitch produit incarné** : en présentant les 5 personas, on raconte la promesse Sobr.ia.
- Une surface de paramétrage propre, persistée, exportable.
- Pas d'impact sur les tests Rust : tous les modules restent compilés et testables.
- Story dossier candidature renforcée : un même outil sert étudiant ET ministère.

### Négatives / Risques

- **Risque de cacher la valeur** : un utilisateur qui désactive M22 Rapport CSRD ne saura pas qu'il existe. Mitigation : Paramètres a une section « Modules disponibles non activés » avec teaser.
- **Coût UX du picker** : 25 cases à cocher peut intimider. Mitigation : on présente d'abord 5-10 modules du bundle persona, le reste est dans une section "Plus de modules…" collapsable.
- **Drift entre bundle de persona et liste modules** : si on ajoute un nouveau module en v1.1, on doit décider s'il rentre dans les bundles existants. Process : tout nouveau module pousse un mini-ADR sur son inclusion par défaut.
- **Multiplicité des routes orphelines** : un module désactivé reste accessible via URL directe. Mitigation : garde de route obligatoire dans chaque `+page.svelte` (cf brief C10).

### Neutres

- Le binaire reste de taille constante (gating au runtime).
- Pas d'impact sur les capabilities Tauri ou le CSP.

## Alternatives écartées

| Alternative | Raison du rejet |
|---|---|
| **Multi-binaires** (sobria-app-student, sobria-app-enterprise) | Démultiplie maintenance, CI, release notes, signatures. Anti-pattern frugal. |
| **Persona multiple** (Étudiant + Chercheur) | Complexifie résolution des bundles (union ? intersection ?), surcharge l'UX. |
| **Pas de gating, tout visible** | Sature le rail, dilue la promesse, perd les étudiants. |
| **Gating backend** (commandes IPC désactivées par persona) | Double le code de test, casse l'isomorphisme prod ↔ tests. |
| **Préférences en LocalStorage** | Pas testable côté Rust, pas portable, pas auditable. |
| **Préférences dans audit.sqlite** | Pollue le ledger (réservé aux estimations), brise l'isolation transactionnelle. |

## Plan de mise en œuvre

1. **C10** — Onboarding + module picker + IPC + store frontend (chantier dédié, brief associé).
2. **C11-C18** — Implémentation des 13 nouveaux modules (M13, M12, M15-M25), un par chantier ou groupés par bundle thématique.
3. **Test d'intégration** : chaque nouveau module ajoute une entrée à la grille du test e2e qui valide « si module désactivé → route inaccessible ».

## Références

- CDC v1.3 §3 (personas v2) et §4 (25 modules).
- Brief `briefs/chantiers/C10-onboarding-personas.md`.
- ADR-0001 (Rust + Tauri 2), ADR-0002 (SvelteKit), ADR-0003 (SQLite/DuckDB).
