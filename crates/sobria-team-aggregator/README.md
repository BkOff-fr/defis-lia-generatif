# sobria-team-aggregator

Binaire HTTPS self-hosted du **Mode Équipe** Sobr.ia — ADR-0013 Phase 2,
v0.7.0 (chantier C28).

Une entreprise déploie ce binaire chez elle (poste admin, NAS, VPS interne)
pour agréger les estimations carbone de ses N employés (extension navigateur
+ app Tauri). **Aucun cloud Sobr.ia n'est impliqué.**

## Statut

**C28.1 livré** — bootstrap minimal :

- CLI `init` (crée `team.sqlite` v1 + TLS auto-signé via rcgen + admin Argon2id).
- CLI `serve` (axum + rustls + ring, route `/health`).
- Schéma SQLite v1 complet (`admins`, `enrollment_codes`, `users`, `tokens`, `estimations`, `config`).

Les sous-chantiers à venir (cf. `briefs/chantiers/C28-mode-equipe-self-hosted.md`) :

- C28.2 — auth JWT + API REST core (enroll, login, refresh, estimations).
- C28.3 — API admin + analytics agrégés.
- C28.4 — dashboard Svelte servi en statique (rust-embed).
- C28.5 — exports CSRD PDF + PROV-O + CSV.
- C28.6 — section Mode Équipe dans extension + app Tauri.
- C28.7 — doc déploiement + packaging multi-OS.

## Quickstart

```bash
# 1. Initialise un data dir (DB + cert TLS + admin)
cargo run -p sobria-team-aggregator -- \
    --data-dir ./team-data \
    init \
    --admin-username admin \
    --admin-password 'change-me-quickly'

# 2. Lance le serveur HTTPS
cargo run -p sobria-team-aggregator -- \
    --data-dir ./team-data \
    serve --port 8443

# 3. Vérifie /health depuis un autre terminal
curl -k https://localhost:8443/health
# → {"ok":true,"version":"0.6.0"}
```

Le data dir contient :

- `team.sqlite` — base SQLite (WAL + foreign keys).
- `cert.pem` — certificat auto-signé (validité 10 ans, SANs : `localhost`,
  `127.0.0.1`, `::1`, hostname OS).
- `key.pem` — clé privée ECDSA-P256 (chmod 600 sur Unix).

## Sécurité

- Aucune dépendance OpenSSL (rustls + ring uniquement).
- Argon2id PHC pour tous les hashs (password admin, futurs codes / tokens).
- Le data dir doit être protégé par les ACLs OS (cf. `docs/operations/team-aggregator.md`).
- Pas de tracking vers Sobr.ia depuis le binaire — voir CLAUDE.md §7.

## Voir aussi

- ADR-0013 — décision d'architecture (extension + pairing + mode équipe).
- `briefs/chantiers/C28-mode-equipe-self-hosted.md` — brief complet du chantier.
- `crates/sobria-app/src/pairing.rs` — pattern Argon2id partagé (C27 v0.6.0).
