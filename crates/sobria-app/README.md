# sobria-app

Binaire Tauri 2 — point d'entrée de l'application native Sobr.ia.

## Architecture

- **`src/lib.rs`** — façade publique de la lib (testable sans Tauri).
- **`src/main.rs`** — binaire : enregistre les commandes IPC et démarre la
  fenêtre.
- **`src/dto.rs`** — Data Transfer Objects Rust ↔ TypeScript.
- **`src/error.rs`** — `IpcError` + mapping `AppError` → `IpcError`.
- **`src/logic.rs`** — logique métier (testable).
- **`src/state.rs`** — `AppState` partagé via `tauri::State`.

## Commandes IPC exposées (C09)

| Commande | Entrée | Sortie |
|----------|--------|--------|
| `meta_info` | — | `MetaInfo` |
| `list_models` | — | `Vec<ModelPresetDto>` |
| `estimate_prompt` | `EstimationRequestDto` | `EstimationResultDto` |
| `verify_audit` | — | `IntegrityReportDto` |
| `list_audit_entries` | `limit, offset` | `Vec<AuditEntrySummaryDto>` |
| `export_audit_ndjson` | `path` | `usize` (nb lignes) |

Voir [`briefs/chantiers/C09-tauri-integration.md`](../../briefs/chantiers/C09-tauri-integration.md)
pour les contrats détaillés.

## Dev

```bash
# Tests Rust (sans Tauri runtime)
cargo test -p sobria-app

# Lance l'app (nécessite web/ buildé ou en dev SvelteKit)
cargo run -p sobria-app

# Lint
cargo clippy -p sobria-app -- -D warnings
```

Voir [CLAUDE.md](../../CLAUDE.md) et [ADR-0001](../../docs/adr/ADR-0001-rust-tauri.md).
