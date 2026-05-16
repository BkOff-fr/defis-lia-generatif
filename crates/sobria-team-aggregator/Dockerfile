# Dockerfile bonus — sobria-team-aggregator (C28.7).
#
# Cible : un admin qui veut isoler le binaire dans un container plutôt
# que l'installer en bare metal. Le binaire est self-contained donc
# l'image runtime ne contient que glibc + ca-certificates.
#
# Usage :
#   docker build -t sobria-team-aggregator -f crates/sobria-team-aggregator/Dockerfile .
#   docker run -d --name sobria-team \
#       -p 8443:8443 \
#       -v sobria-data:/data \
#       sobria-team-aggregator init --admin-password "$ADMIN_PW"
#   docker exec sobria-team \
#       /usr/local/bin/sobria-team-aggregator --data-dir /data serve --bind 0.0.0.0
#
# Volume `/data` persiste team.sqlite + cert.pem + key.pem.
# Le healthcheck appelle /health via curl (ignore le cert auto-signé).

# ─── Build stage ──────────────────────────────────────────────────────────────
FROM node:22-bookworm-slim AS web-builder
WORKDIR /build
COPY web-team/package.json web-team/package-lock.json ./web-team/
RUN cd web-team && npm ci
COPY web-team/ ./web-team/
RUN cd web-team && npm run build

FROM rust:1.79-slim-bookworm AS builder
WORKDIR /build
# Deps système minimales pour rusqlite (bundled) + rustls (ring).
RUN apt-get update && apt-get install -y --no-install-recommends \
        pkg-config build-essential \
    && rm -rf /var/lib/apt/lists/*

# Cache Cargo : on copie d'abord les manifests et un main.rs vide pour
# warm-up le cache des deps avant de copier les sources réelles.
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/
# Le frontend buildé est attendu par rust-embed au moment du compile.
COPY --from=web-builder /build/web-team/build/ ./web-team/build/

RUN cargo build --release -p sobria-team-aggregator \
    && strip target/release/sobria-team-aggregator

# ─── Runtime stage ────────────────────────────────────────────────────────────
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system sobria --gid 1000 \
    && useradd --system --uid 1000 --gid sobria --shell /usr/sbin/nologin sobria

COPY --from=builder /build/target/release/sobria-team-aggregator /usr/local/bin/sobria-team-aggregator

RUN mkdir -p /data && chown sobria:sobria /data
VOLUME ["/data"]

USER sobria
EXPOSE 8443

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD curl -fsSk https://127.0.0.1:8443/health || exit 1

ENTRYPOINT ["/usr/local/bin/sobria-team-aggregator", "--data-dir", "/data"]
CMD ["serve", "--bind", "0.0.0.0", "--port", "8443"]
