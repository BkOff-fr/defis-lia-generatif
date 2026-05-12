#!/usr/bin/env bash
# Build production — toutes les cibles.
# Voir CDC §9 (multi-plateforme) et NF-03/04 (budget tailles).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_DIR"

OUT_DIR="$REPO_DIR/dist"
mkdir -p "$OUT_DIR"

echo "🏗  Build production Sobr.ia"

# 1. Workspace Rust (release)
echo "→ cargo build --release…"
cargo build --release --workspace

# 2. Frontend SvelteKit
echo "→ Frontend production…"
(cd web && npm ci && npm run build)

# 3. App Tauri (desktop natif)
echo "→ Tauri bundle desktop…"
(cd crates/sobria-app && cargo tauri build) || echo "⚠ Tauri build a échoué (vérifier dépendances système)"

# 4. Extension navigateur (Chrome + Firefox)
echo "→ Extension MV3 (Chrome + Firefox)…"
(cd extension && npm ci && npm run build)

# 5. Build Wasm (démo web)
echo "→ Build Wasm démo…"
cargo build --release --target wasm32-unknown-unknown -p sobria-core || echo "⚠ Wasm build non disponible"

# 6. Vérification tailles (budget frugalité)
echo "→ Vérification budgets de taille…"
for bin in target/release/sobria-*; do
    if [ -f "$bin" ] && [ -x "$bin" ]; then
        size_human=$(du -h "$bin" | cut -f1)
        echo "   $bin : $size_human"
    fi
done

echo ""
echo "✅ Build terminé. Artefacts dans target/release/ et web/build/ et extension/dist/"
