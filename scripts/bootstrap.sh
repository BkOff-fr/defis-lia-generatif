#!/usr/bin/env bash
# Bootstrap Sobr.ia — installe toutes les dépendances nécessaires au dev.
# Compatible Linux, macOS, et WSL2. Sous Windows natif, voir scripts/bootstrap.ps1.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_DIR"

echo "🌱 Bootstrap Sobr.ia — depuis $REPO_DIR"

# ─────────────────────────────────────────────────────────────────────────────
# 1. Rust (via rustup) — utilise rust-toolchain.toml
# ─────────────────────────────────────────────────────────────────────────────
if ! command -v rustup &>/dev/null; then
    echo "→ Installation de rustup…"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
    # shellcheck disable=SC1091
    source "$HOME/.cargo/env"
fi
echo "→ Synchronisation toolchain Rust…"
rustup show

# ─────────────────────────────────────────────────────────────────────────────
# 2. Outils Cargo
# ─────────────────────────────────────────────────────────────────────────────
echo "→ Outils Cargo (cargo-tauri, audit, deny, tarpaulin, watch)…"
cargo install --locked tauri-cli@^2 cargo-audit cargo-deny cargo-tarpaulin cargo-watch || true

# ─────────────────────────────────────────────────────────────────────────────
# 3. Node 22 (via volta si présent, sinon nvm)
# ─────────────────────────────────────────────────────────────────────────────
if ! command -v node &>/dev/null; then
    echo "⚠ Node introuvable. Installer Node 22 via https://nodejs.org ou nvm." >&2
    exit 1
fi

# ─────────────────────────────────────────────────────────────────────────────
# 4. Dépendances frontend + extension
# ─────────────────────────────────────────────────────────────────────────────
if [ -d "$REPO_DIR/web" ]; then
    echo "→ npm ci dans web/…"
    (cd "$REPO_DIR/web" && npm ci)
fi
if [ -d "$REPO_DIR/extension" ]; then
    echo "→ npm ci dans extension/…"
    (cd "$REPO_DIR/extension" && npm ci)
fi

# ─────────────────────────────────────────────────────────────────────────────
# 5. DVC (versionnage des données médaillon — ADR-0007)
# ─────────────────────────────────────────────────────────────────────────────
if ! command -v dvc &>/dev/null; then
    echo "→ Installation de DVC via pipx…"
    if ! command -v pipx &>/dev/null; then
        python3 -m pip install --user pipx
        python3 -m pipx ensurepath
    fi
    pipx install 'dvc[s3]'
fi

# ─────────────────────────────────────────────────────────────────────────────
# 6. Quarto (notebook scientifique reproductible)
# ─────────────────────────────────────────────────────────────────────────────
if ! command -v quarto &>/dev/null; then
    echo "⚠ Quarto non installé. Voir https://quarto.org/docs/get-started/" >&2
fi

# ─────────────────────────────────────────────────────────────────────────────
# 7. Téléchargement automatique des datasets officiels FR (M12 + M20)
#    Données publiques Etalab 2.0 — ODRÉ / RTE — pas de clé API requise.
# ─────────────────────────────────────────────────────────────────────────────
if [ ! -f "$REPO_DIR/crates/sobria-geoloc/data/territoire_fr.json" ]; then
    echo "→ Téléchargement ODRÉ — top 200 sites industriels FR…"
    cargo run -q -p sobria-ingest -- fetch territoire-fr --limit 200 || \
        echo "⚠ fetch territoire-fr en échec — relance manuelle possible"
fi
if [ ! -f "$REPO_DIR/crates/sobria-geoloc/data/rte_mix_fr.json" ]; then
    echo "→ Téléchargement RTE eco2mix — mix électrique national 2023…"
    cargo run -q -p sobria-ingest -- fetch rte-mix --year 2023 || \
        echo "⚠ fetch rte-mix en échec — relance manuelle possible"
fi

# ─────────────────────────────────────────────────────────────────────────────
# 8. Hooks Git (pre-commit fmt + clippy)
# ─────────────────────────────────────────────────────────────────────────────
if [ -d .git ]; then
    cat > .git/hooks/pre-commit <<'HOOK'
#!/usr/bin/env bash
set -e
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
HOOK
    chmod +x .git/hooks/pre-commit
    echo "→ Hook pre-commit installé."
fi

echo ""
echo "✅ Bootstrap terminé."
echo ""
echo "Commandes utiles (voir CLAUDE.md §10) :"
echo "  cargo run -p sobria-app                              # lance l'app Tauri en dev"
echo "  cargo run -p sobria-ingest -- pipeline run           # pipeline médaillon complet"
echo "  cargo run -p sobria-ingest -- fetch territoire-fr    # refresh sites industriels FR"
echo "  cargo run -p sobria-ingest -- fetch rte-mix          # refresh mix élec national"
echo "  cd web && npm run dev                                # SvelteKit en hot reload"
echo "  cd extension && npm run dev                          # extension en dev"
