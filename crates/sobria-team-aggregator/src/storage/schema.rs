//! Schéma SQLite v1 du `sobria-team-aggregator`.
//!
//! Voir `briefs/chantiers/C28-mode-equipe-self-hosted.md` §2 pour le détail.
//! La migration est idempotente (`CREATE TABLE IF NOT EXISTS`) et pilotée par
//! la `PRAGMA user_version` SQLite. C28.1 installe la **totalité** des tables
//! v1 ; les sous-chantiers suivants n'auront qu'à les peupler.

use rusqlite::Connection;

use crate::error::AggregatorResult;

/// Version de schéma cible.
///
/// - v1 (C28.1) : tables auth + estimations.
/// - v2 (C29.4) : alertes seuils (`alert_thresholds`, `alert_triggers`).
pub const SCHEMA_VERSION: u32 = 4;

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

/// DDL incrémental v1 → v2 (C29.4 — alertes seuils).
///
/// - `alert_thresholds` : règle "si la conso de gco2eq d'un (user OU team)
///   sur une période (daily/weekly/monthly) dépasse `gco2eq_max`, notifier".
/// - `alert_triggers`   : journal des déclenchements (1 trigger par
///   (threshold_id, period_start) garanti par l'index UNIQUE).
pub const DDL_V2_ALERTS: &str = r"
CREATE TABLE IF NOT EXISTS alert_thresholds (
    id                    TEXT PRIMARY KEY,
    scope                 TEXT NOT NULL CHECK (scope IN ('user', 'team')),
    target_id             TEXT,
    period                TEXT NOT NULL CHECK (period IN ('daily', 'weekly', 'monthly')),
    gco2eq_max            REAL NOT NULL,
    notify_kind           TEXT NOT NULL CHECK (notify_kind IN ('webhook', 'email', 'log_only')),
    notify_target         TEXT,
    created_by_admin_id   TEXT NOT NULL REFERENCES admins(id),
    created_at            TEXT NOT NULL,
    disabled_at           TEXT,
    CHECK (
        (scope = 'team' AND target_id IS NULL)
        OR (scope = 'user' AND target_id IS NOT NULL)
    )
) STRICT;
CREATE INDEX IF NOT EXISTS idx_alert_thresholds_active
    ON alert_thresholds(disabled_at)
    WHERE disabled_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_alert_thresholds_target
    ON alert_thresholds(scope, target_id);

CREATE TABLE IF NOT EXISTS alert_triggers (
    id                TEXT PRIMARY KEY,
    threshold_id      TEXT NOT NULL REFERENCES alert_thresholds(id),
    period_start      TEXT NOT NULL,
    period_end        TEXT NOT NULL,
    observed_gco2eq   REAL NOT NULL,
    triggered_at      TEXT NOT NULL,
    notified_at       TEXT,
    notify_error      TEXT
) STRICT;
CREATE UNIQUE INDEX IF NOT EXISTS idx_alert_triggers_unique
    ON alert_triggers(threshold_id, period_start);
CREATE INDEX IF NOT EXISTS idx_alert_triggers_ts
    ON alert_triggers(triggered_at);
";

/// DDL incrémental v2 → v3 (C38 — ADR-0015 privacy Mode Équipe).
///
/// - `users.share_identified` : consentement OPT-IN du salarié à apparaître
///   nommément dans les vues admin (classements, totaux). Défaut 0 : seuls
///   les agrégats k-anonymes sont visibles côté admin.
/// - `config.k_anonymity_min` : seuil k (nombre minimal d'utilisateurs
///   actifs dans la fenêtre) en dessous duquel les analytics équipe sont
///   bloqués. Plancher dur appliqué côté code : `max(3, valeur)`.
pub const DDL_V3_PRIVACY: &str = r"
ALTER TABLE users ADD COLUMN share_identified INTEGER NOT NULL DEFAULT 0;
INSERT OR IGNORE INTO config (key, value) VALUES ('k_anonymity_min', '5');
";

/// DDL incrémental v3 → v4 (C44 — ADR-0016 politique + dimension projet).
///
/// - `estimations.project` : étiquette projet optionnelle (taguée par
///   conversation côté extension). NULL = hors projet.
/// - `config.visibility_policy` : `anonymous` | `opt_in` (défaut) |
///   `identified` (exige une attestation, cf. ADR-0016).
pub const DDL_V4_PROJECTS: &str = r"
ALTER TABLE estimations ADD COLUMN project TEXT;
CREATE INDEX IF NOT EXISTS idx_estimations_project ON estimations(project);
INSERT OR IGNORE INTO config (key, value) VALUES ('visibility_policy', 'opt_in');
";

/// Applique le schéma cible sur la connexion et pose `PRAGMA user_version`.
///
/// Migrations idempotentes et progressives :
///
/// - `current == 0` (DB neuve) → applique `DDL_V1`, `DDL_V2_ALERTS`, `DDL_V3_PRIVACY`.
/// - `current == 1` (DB C28) → applique `DDL_V2_ALERTS` puis `DDL_V3_PRIVACY`.
/// - `current == 2` (DB C29) → applique uniquement `DDL_V3_PRIVACY` (C38).
/// - `current >= SCHEMA_VERSION` → no-op (compat ascendante).
pub fn install(conn: &Connection) -> AggregatorResult<()> {
    conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")?;

    let current: u32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if current >= SCHEMA_VERSION {
        return Ok(());
    }

    if current < 1 {
        conn.execute_batch(DDL_V1)?;
    }
    if current < 2 {
        conn.execute_batch(DDL_V2_ALERTS)?;
    }
    if current < 3 {
        conn.execute_batch(DDL_V3_PRIVACY)?;
    }
    if current < 4 {
        conn.execute_batch(DDL_V4_PROJECTS)?;
    }
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
    fn migration_v3_adds_share_identified_with_default_zero() {
        let conn = Connection::open_in_memory().unwrap();
        install(&conn).unwrap();
        // La colonne existe, défaut 0, et le k par défaut est posé.
        conn.execute(
            "INSERT INTO users (id, fingerprint, password_hash, created_at)
             VALUES ('u-1', 'fp', 'h', '2026-06-12T00:00:00Z')",
            [],
        )
        .unwrap();
        let share: i64 = conn
            .query_row(
                "SELECT share_identified FROM users WHERE id = 'u-1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(share, 0);
        let k: String = conn
            .query_row(
                "SELECT value FROM config WHERE key = 'k_anonymity_min'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(k, "5");
    }

    #[test]
    fn migration_v4_adds_project_and_policy_default() {
        let conn = Connection::open_in_memory().unwrap();
        install(&conn).unwrap();
        let cols: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('estimations') WHERE name='project'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(cols, 1);
        let policy: String = conn
            .query_row(
                "SELECT value FROM config WHERE key = 'visibility_policy'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(policy, "opt_in");
    }

    #[test]
    fn install_creates_all_tables() {
        let conn = Connection::open_in_memory().unwrap();
        install(&conn).unwrap();
        for table in [
            "config",
            "admins",
            "enrollment_codes",
            "users",
            "tokens",
            "estimations",
            "alert_thresholds",
            "alert_triggers",
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
    fn migrate_from_v1_to_v2_adds_alert_tables() {
        let conn = Connection::open_in_memory().unwrap();
        // Simule une base C28 (v1) puis applique la migration v2.
        conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")
            .unwrap();
        conn.execute_batch(DDL_V1).unwrap();
        conn.pragma_update(None, "user_version", 1u32).unwrap();

        install(&conn).unwrap();

        let v: u32 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(v, SCHEMA_VERSION);
        for table in ["alert_thresholds", "alert_triggers"] {
            let exists: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(exists, 1, "table {table} manquante post-migration");
        }
    }

    #[test]
    fn alert_thresholds_scope_target_check_constraint() {
        let conn = Connection::open_in_memory().unwrap();
        install(&conn).unwrap();
        // Setup admin foreign key.
        conn.execute(
            "INSERT INTO admins (id, username, password_hash, created_at)
             VALUES ('a', 'admin', 'h', '2026-01-01T00:00:00Z')",
            [],
        )
        .unwrap();
        // scope=user sans target_id → REJECT
        let err = conn.execute(
            "INSERT INTO alert_thresholds (id, scope, target_id, period, gco2eq_max,
                notify_kind, notify_target, created_by_admin_id, created_at)
             VALUES ('t1', 'user', NULL, 'daily', 100.0, 'log_only', NULL, 'a',
                '2026-01-01T00:00:00Z')",
            [],
        );
        assert!(err.is_err(), "scope=user sans target_id doit être rejeté");
        // scope=team avec target_id → REJECT
        let err = conn.execute(
            "INSERT INTO alert_thresholds (id, scope, target_id, period, gco2eq_max,
                notify_kind, notify_target, created_by_admin_id, created_at)
             VALUES ('t2', 'team', 'u1', 'daily', 100.0, 'log_only', NULL, 'a',
                '2026-01-01T00:00:00Z')",
            [],
        );
        assert!(err.is_err(), "scope=team avec target_id doit être rejeté");
        // scope=user + target_id → OK
        conn.execute(
            "INSERT INTO alert_thresholds (id, scope, target_id, period, gco2eq_max,
                notify_kind, notify_target, created_by_admin_id, created_at)
             VALUES ('t3', 'user', 'u1', 'daily', 100.0, 'log_only', NULL, 'a',
                '2026-01-01T00:00:00Z')",
            [],
        )
        .unwrap();
    }

    #[test]
    fn alert_triggers_unique_per_threshold_period() {
        let conn = Connection::open_in_memory().unwrap();
        install(&conn).unwrap();
        conn.execute(
            "INSERT INTO admins (id, username, password_hash, created_at)
             VALUES ('a', 'admin', 'h', '2026-01-01T00:00:00Z')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO alert_thresholds (id, scope, target_id, period, gco2eq_max,
                notify_kind, notify_target, created_by_admin_id, created_at)
             VALUES ('t1', 'team', NULL, 'daily', 100.0, 'log_only', NULL, 'a',
                '2026-01-01T00:00:00Z')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO alert_triggers (id, threshold_id, period_start, period_end,
                observed_gco2eq, triggered_at)
             VALUES ('tr1', 't1', '2026-01-01T00:00:00Z', '2026-01-01T23:59:59Z',
                123.0, '2026-01-01T12:00:00Z')",
            [],
        )
        .unwrap();
        // Même threshold + même period_start → UNIQUE violé
        let err = conn.execute(
            "INSERT INTO alert_triggers (id, threshold_id, period_start, period_end,
                observed_gco2eq, triggered_at)
             VALUES ('tr2', 't1', '2026-01-01T00:00:00Z', '2026-01-01T23:59:59Z',
                150.0, '2026-01-01T13:00:00Z')",
            [],
        );
        assert!(err.is_err(), "double trigger même période doit être rejeté");
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
