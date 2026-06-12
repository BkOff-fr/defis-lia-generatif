# syntax=docker/dockerfile:1
# ─────────────────────────────────────────────────────────────────────────────
# Sobr.ia — Mode Équipe : image Docker de `sobria-team-aggregator` (C40)
#
# ⚠️ Images non testées en CI à ce jour — voir C40.
#
# Multi-stage :
#   1. web-builder  : build du dashboard SvelteKit `web-team/` (Node 22,
#      cf. engines de web-team/package.json). Sortie : web-team/build/.
#   2. rust-builder : compile le binaire. Le crate embarque le dashboard au
#      compile time via rust-embed, chemin `../../web-team/build` relatif à
#      crates/sobria-team-aggregator (cf. src/server/embedded_web.rs) → le
#      bundle web DOIT être copié dans /app/web-team/build AVANT cargo build.
#   3. runtime      : debian:stable-slim, non-root, un seul VOLUME /data.
#
# Build (contexte = RACINE du repo, jamais deploy/team/) :
#   docker build -f deploy/team/Dockerfile -t sobria/team-aggregator:0.9.0 .
# ou, recommandé :
#   docker compose -f deploy/team/docker-compose.yml build
#
# Guide opérateur : docs/operations/deploiement-equipe.md
# ─────────────────────────────────────────────────────────────────────────────

# ── Stage 1 : dashboard SvelteKit (web-team) ────────────────────────────────
# node:22-slim — web-team/package.json exige node >= 22, npm >= 10.
FROM node:22-slim AS web-builder
WORKDIR /app/web-team
# Couche dédiée aux dépendances → cache Docker tant que le lock ne change pas.
COPY web-team/package.json web-team/package-lock.json ./
RUN npm ci --no-audit --no-fund
COPY web-team/ ./
# adapter-static → sortie dans `build/` (cf. web-team/svelte.config.js).
RUN npm run build

# ── Stage 2 : binaire Rust ──────────────────────────────────────────────────
# rust:1-slim = dernière stable (>= 1.79 requis par rust-version du workspace).
# NB glibc : si le runtime affiche « GLIBC_x.y not found », épingler les deux
# stages sur la MÊME release Debian (ex. rust:1-slim-trixie + debian:trixie-slim).
FROM rust:1-slim AS rust-builder
# build-essential : cc/make/binutils pour les build scripts C des dépendances
# (rusqlite `bundled` compile sqlite3.c, ring compile ses sources C/asm).
# Aucune lib système de plus : TLS = rustls (pas d'OpenSSL), SQLite = bundled.
RUN apt-get update \
    && apt-get install -y --no-install-recommends build-essential \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
# Manifests + sources : TOUS les membres du workspace doivent être présents
# pour la résolution Cargo, même si seul sobria-team-aggregator est compilé.
# rust-toolchain.toml est volontairement NON copié : il ferait télécharger
# rustfmt/clippy/rust-analyzer + 5 targets croisés inutiles ici (le canal
# est « stable », identique au toolchain de l'image).
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
# Dashboard buildé au chemin attendu par rust-embed (#[folder]) :
# /app/crates/sobria-team-aggregator + ../../web-team/build = /app/web-team/build
COPY --from=web-builder /app/web-team/build/ web-team/build/
# --locked : respecte Cargo.lock à l'octet près (reproductibilité).
# Profil release du workspace : opt-level="z", lto=true, strip → binaire
# compact (~15 Mo) mais 1ʳᵉ compilation longue (10-25 min).
RUN cargo build --locked --release -p sobria-team-aggregator

# ── Stage 3 : runtime minimal non-root ──────────────────────────────────────
FROM debian:stable-slim AS runtime
LABEL org.opencontainers.image.title="sobria-team-aggregator" \
      org.opencontainers.image.description="Sobr.ia Mode Équipe — agrégateur self-hosted (HTTPS, SQLite, k-anonymat ADR-0015)" \
      org.opencontainers.image.licenses="MIT" \
      org.opencontainers.image.version="0.9.0"
# curl            : HEALTHCHECK ci-dessous.
# sqlite3         : sauvegarde à chaud `.backup` (docs/operations/team-aggregator.md §Sauvegardes).
# ca-certificates : sorties TLS éventuelles (webhooks d'alerte, SMTP — C29.4).
RUN apt-get update \
    && apt-get install -y --no-install-recommends curl sqlite3 ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    # User système non-root, UID fixe (utile pour les bind mounts).
    && useradd --system --uid 10001 --user-group --home-dir /data \
        --shell /usr/sbin/nologin sobria \
    && mkdir -p /data \
    && chown sobria:sobria /data \
    && chmod 700 /data

COPY --from=rust-builder /app/target/release/sobria-team-aggregator /usr/local/bin/sobria-team-aggregator
COPY deploy/team/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod 0755 /usr/local/bin/entrypoint.sh

USER sobria
# Tout l'état du serveur vit ici : team.sqlite (+ -wal/-shm), cert.pem, key.pem.
VOLUME /data
EXPOSE 8443

# -k : cert auto-signé ; -f : échec si statut HTTP >= 400.
# Le port suit SOBRIA_TEAM_PORT si l'opérateur le change (forme shell → expansion).
HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
    CMD curl -fsSk "https://localhost:${SOBRIA_TEAM_PORT:-8443}/health" || exit 1

# entrypoint.sh : `init` automatique au premier démarrage (team.sqlite absent)
# puis `serve --bind 0.0.0.0 --port 8443` sur --data-dir /data.
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD ["serve"]
