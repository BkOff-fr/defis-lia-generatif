# Chantier #9 — Intégration Tauri 2 + design Claude Design

> **Pré-requis** : v0.1.2-audit mergé (C01-C08 verts).
> **Crates touchées** : `sobria-app` (squelette IPC) + nouveau dossier `web/` (SvelteKit).
> **Durée cible** : 3-5 jours.
> **Référence CDC** : §6 (UX), §7 (architecture), §10 (multi-plateforme).
> **Référence ADR** : ADR-0001 (Tauri 2), ADR-0008 (Frontend SvelteKit).

---

## 0. Objectif

Câbler le backend Rust (`sobria-estimator`, `sobria-audit`, `sobria-referentiel`,
etc.) avec une UI **SvelteKit 2 + TypeScript strict** hébergée par **Tauri 2**,
en réutilisant le **design produit par Claude Design** :

> https://api.anthropic.com/v1/design/h/pzqXRiIjWHO1dkAt2b91cw

Le résultat : une app native Windows / macOS / Linux qui démarre, affiche
l'écran « estimer un prompt », appelle le moteur Monte-Carlo via IPC,
visualise les indicateurs (CO₂eq / énergie / eau) avec intervalles P5-P95,
et journalise dans le ledger d'audit.

## 1. Architecture cible

```
┌────────────────────────────────────────────────────────┐
│  SvelteKit 2 (TypeScript strict) — web/               │
│  ├── routes/+page.svelte           (Estimer)          │
│  ├── routes/audit/+page.svelte     (Audit)            │
│  ├── routes/territoire/+page.svelte (FR — M12)        │
│  ├── lib/api.ts                    (wrapper IPC)      │
│  └── lib/components/…              (design Claude)    │
│                ↑ invoke()                              │
└──────────────────┼─────────────────────────────────────┘
                   │  Tauri IPC (JSON-RPC, capabilities)
┌──────────────────┼─────────────────────────────────────┐
│  sobria-app (Rust + Tauri 2)                          │
│  ├── main.rs        builder + run                     │
│  ├── state.rs       AppState (AuditLedger, …)         │
│  ├── error.rs       AppError → IpcError JSON          │
│  └── commands/      #[tauri::command]                 │
│      ├── estimate.rs                                  │
│      ├── models.rs                                    │
│      ├── audit.rs                                     │
│      └── meta.rs                                      │
│                ↓                                       │
│  sobria-estimator · sobria-audit · sobria-core         │
└────────────────────────────────────────────────────────┘
```

**Principes** :

1. **Frontend découplé** : la UI ne sait pas que c'est SQLite ou Rust derrière —
   elle voit une API typée `EstimationResult`, `AuditEntry`, etc.
2. **Capabilities minimales** : pas de FS arbitraire, pas de shell, pas de
   net. Seulement les commandes `#[tauri::command]` listées.
3. **AppState partagé** via `tauri::State<AppState>`. `AuditLedger` est
   derrière un `Mutex<>` (les écritures sont sérielles).
4. **Erreurs typées** : `AppError` → `IpcError { code, message }` JSON
   pour que le frontend puisse afficher des messages cohérents.

## 2. Découpage Cowork ↔ Claude Code

| Bloc | Qui | Output |
|---|---|---|
| **C09.1 — Brief** | Cowork | Ce document |
| **C09.2 — Squelette Tauri Rust** | Cowork | `crates/sobria-app/src/{main,state,error,commands/*}.rs`, `tauri.conf.json`, `capabilities/default.json` |
| **C09.3 — Prompt Claude Code** | Cowork | `briefs/chantiers/C09-PROMPT-CLAUDE-CODE.md` avec URL design + cahier des charges UI |
| **C09.4 — Implémentation frontend** | Claude Code | `web/` SvelteKit complet : routes, composants, `lib/api.ts`, tests Playwright |
| **C09.5 — Wiring final + e2e** | Mixte | `cargo run -p sobria-app` lance l'app, écran *Estimer* fonctionne bout-en-bout |
| **C09.6 — Rétro + tag v0.2.0-app** | Cowork | `briefs/chantiers/C09-RETROSPECTIVE.md`, `CHANGELOG.md`, tag git |

## 3. Commandes IPC (contrats publics)

Toutes les commandes retournent `Result<T, IpcError>` côté Rust. Côté
TypeScript, on récupère `T` (rejet de la promesse si erreur).

### 3.1 `meta_info()`

```rust
#[tauri::command]
fn meta_info(state: tauri::State<'_, AppState>) -> Result<MetaInfo, IpcError>;
```

```ts
type MetaInfo = {
  app_version: string;     // ex: "0.2.0"
  estimator_seed: number;  // 42
  estimator_n: number;     // 10_000
  audit_path: string;      // chemin du ledger (info, pas FS access)
  data_root: string;       // racine ~/.sobria/
};
```

But : afficher le footer "Sobr.ia 0.2.0 — seed 42 — N=10⁴" et permettre
de localiser le ledger pour le bouton « ouvrir l'audit ».

### 3.2 `list_models()`

```rust
#[tauri::command]
fn list_models() -> Result<Vec<ModelPresetDto>, IpcError>;
```

```ts
type ModelPresetDto = {
  id: string;
  display_name: string;
  provider: string;
  family: string;
  approx_params_billions: number;
  openness: "open" | "open_weights" | "closed";
  calibration: "validated" | "indicative" | "extrapolated";
  sources: string[];
};
```

But : peupler le `<select>` du formulaire d'estimation, badger les modèles
extrapolés vs validés (transparence méthodologique).

### 3.3 `estimate_prompt(req)`

```rust
#[tauri::command]
fn estimate_prompt(
    req: EstimationRequestDto,
    state: tauri::State<'_, AppState>,
) -> Result<EstimationResultDto, IpcError>;
```

```ts
type EstimationRequestDto = {
  model_id: string;
  tokens_in: number;
  tokens_out_estimated: number;
  datacenter_id?: string;
};

type EstimationResultDto = {
  request: EstimationRequestDto & { timestamp: string };
  indicators: { indicator: "co2eq" | "energy" | "water";
                p5: number; p50: number; p95: number;
                unit: string }[];
  equivalents: { label: string; value: number; source: string }[];
  hypotheses:   { key: string; value: unknown; source: string }[];
  computed_at: string;
  seed: number;
  audit_id: number;      // entrée du ledger (chaînage cliquable)
};
```

Comportement :
1. Validation de la requête (`tokens_in ≤ 10⁶`, `model_id` connu).
2. Lookup `EstimationParams` via `EstimationParams::for_model(model_id)`.
3. `MonteCarloEngine::default().estimate(&req, &params)`.
4. Append au ledger (`AuditLedger::append`) → on récupère `audit_id`.
5. Retourne le DTO complet.

Erreurs typiques : `UnknownModel`, `InvalidRequest`, `EstimatorError`,
`AuditError`. Toutes mappées vers `IpcError { code, message }`.

### 3.4 `verify_audit()`

```rust
#[tauri::command]
fn verify_audit(state: tauri::State<'_, AppState>)
  -> Result<IntegrityReportDto, IpcError>;
```

```ts
type IntegrityReportDto = {
  total_entries: number;
  valid: boolean;
  first_invalid_id?: number;
  message: string;
};
```

But : bouton « Vérifier la chaîne d'audit » dans l'écran *Audit*.

### 3.5 `list_audit_entries({ limit, offset })`

```rust
#[tauri::command]
fn list_audit_entries(
    limit: u32,
    offset: u32,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<AuditEntrySummaryDto>, IpcError>;
```

```ts
type AuditEntrySummaryDto = {
  id: number;
  timestamp: string;
  model_id: string;       // extrait du payload
  co2eq_p50: number;      // pour affichage rapide
  sig_short: string;      // 16 premiers char du sig
  purged: boolean;
};
```

But : alimenter le tableau de la page *Audit* (pagination).

### 3.6 `export_audit_ndjson(path)`

```rust
#[tauri::command]
fn export_audit_ndjson(
    path: String,
    state: tauri::State<'_, AppState>,
) -> Result<usize, IpcError>; // nb lignes écrites
```

Note : le `path` est fourni par un dialog `tauri::api::dialog::save_file`
côté frontend (capability `dialog:default` activée).

### 3.7 `purge_audit_before(iso_datetime)` *(v0.3 — skip pour C09)*

Listée pour mémoire, **non implémentée en C09**. RGPD = chantier C11.

## 4. State + erreurs

### 4.1 `AppState`

```rust
pub struct AppState {
    pub ledger: Mutex<AuditLedger>,
    pub data_root: PathBuf,
    pub estimator: MonteCarloEngine,  // immutable, partageable
}
```

Initialisation au démarrage :
1. `data_root = dirs::data_dir().join("sobria")` (création si absent).
2. `ledger = AuditLedger::open(data_root.join("audit.sqlite"))`.
3. `estimator = MonteCarloEngine::default()` (seed=42, N=10⁴).

### 4.2 `IpcError`

```rust
#[derive(Debug, Serialize)]
pub struct IpcError {
    pub code: &'static str,        // ex: "unknown_model"
    pub message: String,
    pub details: Option<serde_json::Value>,
}
```

Codes définis :
- `unknown_model`
- `invalid_request`
- `estimator_error`
- `audit_error`
- `io_error`
- `internal`

## 5. Capabilities Tauri (`capabilities/default.json`)

Whitelist stricte — pas de FS arbitraire ni de net :

```json
{
  "identifier": "default",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:default",
    "core:event:default",
    "core:webview:default",
    "core:app:default"
  ]
}
```

**Phase C09.2 (Cowork — squelette)** : seules les permissions `core:*` sont
listées, parce que le plugin `tauri-plugin-dialog` n'est pas encore une
dépendance Cargo. Sans le plugin installé, Tauri rejette les permissions
`dialog:*` au build (« Permission not found »).

**Phase C09.3 (Claude Code — plugin dialog)** : à l'ajout de
`tauri-plugin-dialog = "2"` dans `Cargo.toml` + `Builder::default().plugin(tauri_plugin_dialog::init())`
dans `main.rs`, **ajouter** :

```json
    "dialog:default",
    "dialog:allow-save",
    "dialog:allow-open"
```

**Pas de** : `fs:*`, `shell:*`, `http:*`, `os:*` (sauf strictement nécessaire).
Toute extension nécessite un ADR.

## 6. Cible UI minimale (C09)

Trois écrans nécessaires pour démontrer la chaîne complète :

1. **Estimer** (`/`) — formulaire (modèle, tokens in/out) → carte résultat
   avec 3 indicateurs P5-P50-P95, équivalents parlants, badge calibration,
   lien « voir les hypothèses » (drawer).
2. **Audit** (`/audit`) — liste paginée des entrées + bouton « Vérifier
   la chaîne » + bouton « Exporter NDJSON ».
3. **À propos / méthodo** (`/methodo`) — lien vers les docs locales
   (méthodologie, sources, ADR-0004).

Le design exact est défini par Claude Design — Claude Code récupère le
brief UI et le réalise.

## 7. Definition of Done

- [ ] `cargo build -p sobria-app` compile sur Win / macOS / Linux.
- [ ] `cargo test -p sobria-app` ≥ 6 tests passent (commandes mockées).
- [ ] `cargo clippy -p sobria-app -- -D warnings` propre.
- [ ] `cargo run -p sobria-app` ouvre une fenêtre Tauri avec l'UI Svelte.
- [ ] L'écran *Estimer* fait un aller-retour IPC complet :
      `gpt-4o-mini`, 100/500 tokens → résultat affiché en moins de 200 ms.
- [ ] L'écran *Audit* affiche l'entrée créée à l'étape précédente.
- [ ] Bouton « Vérifier la chaîne » retourne `valid: true` après N appels.
- [ ] Bouton « Exporter NDJSON » écrit un fichier valide.
- [ ] Au moins 1 test Playwright e2e (formulaire → résultat).
- [ ] `cd web && npm run lint && npm run check` verts.
- [ ] `briefs/chantiers/C09-RETROSPECTIVE.md` rédigé.
- [ ] `CHANGELOG.md` mis à jour, tag `v0.2.0-app`.

## 8. Tests Rust (C09.2)

| Test | Description |
|------|-------------|
| `meta_info_returns_version` | `meta_info` renvoie une version non vide |
| `list_models_returns_registry` | ≥ 8 modèles, schéma conforme |
| `estimate_unknown_model_fails` | `unknown_model` côté erreur |
| `estimate_happy_path_journalise` | append au ledger, `audit_id` ≥ 1 |
| `verify_audit_after_append_is_valid` | chaîne valide après N appels |
| `list_audit_entries_pagination` | offset/limit fonctionnent |

Les tests instancient `AppState` directement (pas besoin de lancer Tauri) :
on teste les fonctions internes appelées par les commandes.

## 9. Risques + parades

| Risque | Probabilité | Parade |
|---|---|---|
| **Capabilities trop permissives** par défaut | Moyen | Whitelist + revue avant tag |
| **Bloque la UI thread** sur Monte-Carlo (5-20 ms) | Faible | OK à 10⁴ tirages, sinon `spawn_blocking` |
| **Path ledger non créable** sur disque | Moyen | Fallback `tempdir` + log d'erreur |
| **DTO drift** Rust ↔ TS | Élevé | `ts-rs` ou génération manuelle + tests d'intégration |
| **Concurrence sur le ledger** | Faible | `Mutex<AuditLedger>` (toutes écritures sérielles) |

## 10. Non-objectifs (v2)

- Mode offline / sync cloud — hors périmètre C09.
- Auto-update Tauri (signé) — chantier C12.
- Internationalisation (FR + EN) — chantier C10.
- Drag & drop d'un fichier prompts JSON → batch — chantier C10.

---

*Brief rédigé par Cowork — à exécuter en C09.2 (Rust) puis C09.3 (prompt
Claude Code). Validation finale par Thibault.*
