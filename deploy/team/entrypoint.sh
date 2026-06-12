#!/bin/sh
# ─────────────────────────────────────────────────────────────────────────────
# Sobr.ia Mode Équipe — entrypoint du conteneur Docker (C40).
#
# Rôle :
#   1. Premier démarrage (pas de /data/team.sqlite) → `init` automatique :
#      crée la base SQLite, le certificat TLS auto-signé, la clé JWT et le
#      compte admin. Le mot de passe est lu PAR LE BINAIRE directement dans
#      l'env SOBRIA_TEAM_ADMIN_PASSWORD (jamais passé en argv → invisible
#      dans `ps` et dans l'historique).
#   2. Démarrages suivants → `serve` directement (exec : le serveur devient
#      PID 1, les signaux docker stop/SIGTERM lui parviennent proprement).
#
# Toute sous-commande explicite autre que `serve` est relayée au binaire
# avec le bon --data-dir, ce qui permet :
#   docker compose run --rm sobria-team code create 10 --ttl-days 7
#   docker compose run --rm sobria-team config set k_anonymity_min 8
#   docker compose run --rm sobria-team admin reset-password admin
#
# Variables d'environnement :
#   SOBRIA_TEAM_DATA_DIR        défaut /data   (le VOLUME de l'image)
#   SOBRIA_TEAM_BIND            défaut 0.0.0.0
#   SOBRIA_TEAM_PORT            défaut 8443
#   SOBRIA_TEAM_ADMIN_USERNAME  défaut admin   (init uniquement)
#   SOBRIA_TEAM_ADMIN_PASSWORD  REQUIS au premier démarrage uniquement
# ─────────────────────────────────────────────────────────────────────────────
set -eu

BIN="sobria-team-aggregator"
DATA_DIR="${SOBRIA_TEAM_DATA_DIR:-/data}"
BIND="${SOBRIA_TEAM_BIND:-0.0.0.0}"
PORT="${SOBRIA_TEAM_PORT:-8443}"
ADMIN_USERNAME="${SOBRIA_TEAM_ADMIN_USERNAME:-admin}"

# Passthrough : sous-commande explicite != serve (code / admin / config / init…).
if [ "$#" -gt 0 ] && [ "$1" != "serve" ]; then
    exec "$BIN" --data-dir "$DATA_DIR" "$@"
fi
# Consomme le « serve » éventuel ; le reste de $@ = flags additionnels
# (ex. `docker compose run --rm sobria-team serve --regen-cert`).
if [ "$#" -gt 0 ]; then
    shift
fi

# ── Premier démarrage : initialisation du data dir ──────────────────────────
if [ ! -f "${DATA_DIR}/team.sqlite" ]; then
    if [ -z "${SOBRIA_TEAM_ADMIN_PASSWORD:-}" ]; then
        cat >&2 <<'MSG'
[entrypoint] ERREUR — premier démarrage détecté (/data/team.sqlite absent)
mais SOBRIA_TEAM_ADMIN_PASSWORD n'est pas définie.

  → Créez un fichier .env à côté de docker-compose.yml :

      SOBRIA_TEAM_ADMIN_PASSWORD=une-passphrase-longue-32-caracteres-mini

    puis relancez :  docker compose up -d

  La variable n'est nécessaire QU'À cette initialisation : retirez-la du
  .env ensuite (voir docs/operations/deploiement-equipe.md, étape 5).
MSG
        exit 1
    fi
    echo "[entrypoint] Premier démarrage : init du data dir ${DATA_DIR} (admin: ${ADMIN_USERNAME})…"
    "$BIN" --data-dir "$DATA_DIR" init --admin-username "$ADMIN_USERNAME"
    echo "[entrypoint] Init terminé. Retirez SOBRIA_TEAM_ADMIN_PASSWORD du .env puis \`docker compose up -d\`."
fi

echo "[entrypoint] Lancement : serve --bind ${BIND} --port ${PORT} $*"
exec "$BIN" --data-dir "$DATA_DIR" serve --bind "$BIND" --port "$PORT" "$@"
