//! Schéma SQLite v1 du `sobria-team-aggregator`.
//!
//! Voir `briefs/chantiers/C28-mode-equipe-self-hosted.md` §2 pour le détail.
//! La migration est idempotente (`CREATE TABLE IF NOT EXISTS`) et pilotée par
//! la `PRAGMA user_version` SQLite. C28.1 installe la **totalité** des tables
//! v1 ; les sous-chantiers suivants n'auront qu'à les peupler.

use rusqlite::Connection;

use crate::error::AggregatorResult;

/// Version de schéma cible.
pub const SCHEMA_VERSION: u32 = 1;

/// DDL complet v1.
///
/// - `config` : KV store interne (JWT signing key, schema_installed_at…).
/// - `admins` : comptes administrateurs (login + Argon2id PHC).
/// - `enrollment_codes` : jetons single-use 12 chiffres distribués aux employés.
/// - `users` : employés enrollés (1 enrollment_code → 1 user).
/// - `tokens` : refresh tokens hashés Argon2id (admin OU user, jamais les deux).
/// - `estimations` : événements remontés du client REST.
pub const DDL_V1: &str = r"
CREATE TABLE IF NOT EXISTS config (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
) STRICT;

CREATE TABLE IF NOT EXISTS admins (
    id              TEXT PRIMARY KEY,
    username        TEXT UNIQUE NOT NULL,
    password_hash   TEXT NOT NULL,
    created_at      TEXT NOT NULL,
    last_login_at   TEXT
) STRICT;

CREATE TABLE IF NOT EXISTS enrollment_codes (
    id                TEXT PRIMARY KEY,
    code_hash         TEXT NOT NULL,
    created_by        TEXT NOT NULL REFERENCES admins(id),
    created_at        TEXT NOT NULL,
    expires_at        TEXT NOT NULL,
    used_at           TEXT,
    used_by_user_id   TEXT,
    revoked_at        TEXT
) STRICT;
CREATE INDEX IF NOT EXISTS idx_enrollment_codes_expires ON enrollment_codes(expires_at);
CREATE INDEX IF NOT EXISTS idx_enrollment_codes_used    ON enrollment_codes(used_at);

CREATE TABLE IF NOT EXISTS users (
    id                  TEXT PRIMARY KEY,
    enrollment_code_id  TEXT REFERENCES enrollment_codes(id),
    fingerprint         TEXT UNIQUE NOT NULL,
    password_hash       TEXT NOT NULL,
    display_name        TEXT,
    created_at          TEXT NOT NULL,
    last_seen_at        TEXT
) STRICT;
CREATE INDEX IF NOT EXISTS idx_users_fingerprint ON users(fingerprint);

CREATE TABLE IF NOT EXISTS tokens (
    id                 TEXT PRIMARY KEY,
    user_id            TEXT REFERENCES users(id),
    admin_id           TEXT REFERENCES admins(id),
    refresh_token_hash TEXT NOT NULL,
    issued_at          TEXT NOT NULL,
    expires_at         TEXT NOT NULL,
    revoked_at         TEXT,
    CHECK ((user_id IS NULL) <> (admin_id IS NULL))
) STRICT;
CREATE INDEX IF NOT EXISTS idx_tokens_user  ON tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_tokens_admin ON tokens(admin_id);

CREATE TABLE IF NOT EXISTS estimations (
    id               TEXT PRIMARY KEY,
    user_id          TEXT NOT NULL REFERENCES users(id),
    ts               TEXT NOT NULL,
    method           TEXT NOT NULL,
    model_id         TEXT NOT NULL,
    tokens_in        INTEGER NOT NULL,
    tokens_out       INTEGER NOT NULL,
    gco2eq_p50       REAL NOT NULL,
    gco2eq_p5        REAL,
    gco2eq_p95       REAL,
    water_ml         REAL NOT NULL,
    energy_wh        REAL NOT NULL,
    region           TEXT,
    raw_payload_json TEXT NOT NULL,
    received_at      TEXT NOT NULL
) STRICT;
CREATE INDEX IF NOT EXISTS idx_estimations_user_ts ON estimations(user_id, ts);
CREATE INDEX IF NOT EXISTS idx_estimations_ts      ON estimations(ts);
CREATE INDEX IF NOT EXISTS idx_estimations_model   ON estimations(model_id);
";

/// Applique le schéma v1 sur la connexion et pose `PRAGMA user_version = 1`.
///
/// L'opération est idempotente : on peut la rappeler sur une base déjà
/// migrée sans effet. Si une `user_version` future (≥ 2) est détectée,
/// la fonction la laisse intacte (compat ascendante).
pub fn install(conn: &Connection) -> AggregatorResult<()> {
    conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")?;

    let current: u32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if current >= SCHEMA_VERSION {
        return Ok(());
    }

    conn.execute_batch(DDL_V1)?;
    conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        install(&conn).unwrap();
        install(&conn).unwrap();
        let v: u32 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(v, SCHEMA_VERSION);
    }

    #[test]
    fn install_creates_all_v1_tables() {
        let conn = Connection::open_in_memory().unwrap();
        install(&conn).unwrap();
        for table in [
            "config",
            "admins",
            "enrollment_codes",
            "users",
            "tokens",
            "estimations",
        ] {
            let exists: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(exists, 1, "table {table} manquante");
        }
    }

    #[test]
    fn tokens_check_enforces_xor_admin_user() {
        let conn = Connection::open_in_memory().unwrap();
        install(&conn).unwrap();
        // user_id et admin_id tous deux NULL → CHECK violé
        let err = conn.execute(
            "INSERT INTO tokens (id, user_id, admin_id, refresh_token_hash, issued_at, expires_at)
             VALUES ('t1', NULL, NULL, 'h', '2026-01-01T00:00:00Z', '2026-01-08T00:00:00Z')",
            [],
        );
        assert!(err.is_err(), "le CHECK XOR doit rejeter NULL/NULL");

        // Les deux renseignés → CHECK violé aussi
        let err = conn.execute(
            "INSERT INTO tokens (id, user_id, admin_id, refresh_token_hash, issued_at, expires_at)
             VALUES ('t2', 'u', 'a', 'h', '2026-01-01T00:00:00Z', '2026-01-08T00:00:00Z')",
            [],
        );
        assert!(
            err.is_err(),
            "le CHECK XOR doit rejeter user_id ET admin_id"
        );
    }
}
