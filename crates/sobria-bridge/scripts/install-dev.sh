#!/usr/bin/env bash
# Sobr.ia bridge — installation dev du manifest natif (C27.5).
#
# Usage : ./install-dev.sh <EXTENSION_ID>
#
# Construit le binaire en release, dépose le manifest natif au bon endroit
# selon l'OS (Chrome + Firefox), et patche {{BRIDGE_PATH}} + {{EXTENSION_ID}}.
#
# L'EXTENSION_ID est visible dans chrome://extensions après chargement de
# l'extension non empaquetée (chaîne hex 32 chars).
#
# Ce script est un fallback dev tant que l'app Sobr.ia desktop n'installe
# pas le manifest automatiquement (C27.5.b — module bridge_install dans
# crates/sobria-app/src/).

set -euo pipefail

EXT_ID="${1:-}"
if [[ -z "$EXT_ID" ]]; then
  echo "Usage : $0 <EXTENSION_ID>"
  exit 1
fi

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
BRIDGE_BIN="$REPO_ROOT/target/release/sobria-bridge"

echo "[install-dev] cargo build --release -p sobria-bridge"
cargo build --release -p sobria-bridge --manifest-path "$REPO_ROOT/Cargo.toml"

if [[ ! -x "$BRIDGE_BIN" ]]; then
  echo "[install-dev] ❌ binaire introuvable : $BRIDGE_BIN"
  exit 1
fi

case "$(uname -s)" in
  Darwin*)
    CHROME_DIR="$HOME/Library/Application Support/Google/Chrome/NativeMessagingHosts"
    FIREFOX_DIR="$HOME/Library/Application Support/Mozilla/NativeMessagingHosts"
    ;;
  Linux*)
    CHROME_DIR="$HOME/.config/google-chrome/NativeMessagingHosts"
    FIREFOX_DIR="$HOME/.mozilla/native-messaging-hosts"
    ;;
  MINGW*|MSYS*|CYGWIN*)
    echo "[install-dev] Windows non supporté par ce script bash — voir install-dev.ps1"
    exit 1
    ;;
  *)
    echo "[install-dev] OS inconnu : $(uname -s)" >&2
    exit 1
    ;;
esac

for DIR in "$CHROME_DIR" "$FIREFOX_DIR"; do
  mkdir -p "$DIR"
  TEMPLATE="$REPO_ROOT/crates/sobria-bridge/manifest.template.json"
  TARGET="$DIR/com.sobria.bridge.json"
  sed -e "s|{{BRIDGE_PATH}}|$BRIDGE_BIN|g" \
      -e "s|{{EXTENSION_ID}}|$EXT_ID|g" \
      "$TEMPLATE" > "$TARGET"
  echo "[install-dev] ✓ $TARGET"
done

echo "[install-dev] Manifest natif déployé. Recharge l'extension pour tester."
