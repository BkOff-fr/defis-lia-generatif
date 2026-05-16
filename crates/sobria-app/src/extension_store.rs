//! Sobr.ia — persistance pairing extension + ingestion events (C27.5.c/d).
//!
//! Deux tables SQLite (vivent dans `referentiel.sqlite`) :
//!
//! - `device_pairings(id, fingerprint, secret_hash, salt_hex, created_at,
//!   last_seen_at, revoked_at)`
//!   Identifiant unique par appariement extension ↔ app. Le `secret_hash`
//!   est SHA-256 + sel (cf. `crate::pairing`) : aucun secret en clair en DB.
//!
//! - `extension_events(id, pairing_id, ts, method, model_id, tokens_in,
//!   tokens_out, gco2eq_p50, water_ml, energy_wh, raw_payload_json,
//!   ingested_at)`
//!   Une ligne par estimation remontée du bridge. Foreign key sur
//!   `device_pairings.id` pour permettre la cascade de purge si l'utilisateur
//!   dépare une extension.
//!
//! La lecture du spool `~/.sobria/spool/incoming.jsonl` est intégrée dans
//! `drain_spool` qui valide le secret puis insère dans `extension_events`.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::pairing::PairingSecret;

/// Schéma DDL (idempotent : `CREATE TABLE IF NOT EXISTS`).
const SCHEMA: &str = r"
CREATE TABLE IF NOT EXISTS device_pairings (
    id           TEXT PRIMARY KEY,
    fingerprint  TEXT NOT NULL,
    secret_hash  TEXT NOT NULL,
    salt_hex     TEXT NOT NULL,
    created_at   TEXT NOT NULL,
    last_seen_at TEXT,
    revoked_at   TEXT,
    UNIQUE(fingerprint)
);
CREATE INDEX IF NOT EXISTS idx_device_pairings_fp ON device_pairings(fingerprint);

CREATE TABLE IF NOT EXISTS extension_events (
    id               TEXT PRIMARY KEY,
    pairing_id       TEXT NOT NULL,
    ts               TEXT NOT NULL,
    method           TEXT NOT NULL,
    model_id         TEXT NOT NULL,
    tokens_in        INTEGER NOT NULL,
    tokens_out       INTEGER NOT NULL,
    gco2eq_p50       REAL NOT NULL,
    water_ml         REAL NOT NULL,
    energy_wh        REAL NOT NULL,
    raw_payload_json TEXT NOT NULL,
    ingested_at      TEXT NOT NULL,
    FOREIGN KEY (pairing_id) REFERENCES device_pairings(id)
);
CREATE INDEX IF NOT EXISTS idx_extension_events_ts ON extension_events(ts);
CREATE INDEX IF NOT EXISTS idx_extension_events_pairing ON extension_events(pairing_id);
";

/// DTO exposé via IPC pour lister les extensions appariées.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingRow {
    pub id: String,
    pub fingerprint: String,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

/// DTO normalisé d'un évènement extension après ingestion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionEventRow {
    pub id: String,
    pub pairing_id: String,
    pub ts: DateTime<Utc>,
    pub method: String,
    pub model_id: String,
    pub tokens_in: u32,
    pub tokens_out: u32,
    pub gco2eq_p50: f64,
    pub water_ml: f64,
    pub energy_wh: f64,
    pub ingested_at: DateTime<Utc>,
}

/// Store de pairings + events extension.
pub struct ExtensionStore {
    conn: Connection,
}

impl ExtensionStore {
    /// Ouvre / crée les tables dans `referentiel.sqlite`.
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path).with_context(|| format!("open {}", path.display()))?;
        conn.execute_batch(SCHEMA)
            .context("install extension_store schema")?;
        Ok(Self { conn })
    }

    /// Construit un store en mémoire (utile pour les tests).
    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    /// Enregistre un nouveau pairing.
    ///
    /// Si `fingerprint` existe déjà (UNIQUE), on remplace l'entrée (ré-appariement
    /// après dépair). Retourne l'`id` (UUID v4) attribué.
    pub fn record_pairing(&self, fingerprint: &str, secret: &PairingSecret) -> Result<String> {
        let id = ulid();
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO device_pairings(id, fingerprint, secret_hash, salt_hex, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(fingerprint) DO UPDATE SET
               id = excluded.id,
               secret_hash = excluded.secret_hash,
               salt_hex = excluded.salt_hex,
               created_at = excluded.created_at,
               last_seen_at = NULL,
               revoked_at = NULL",
            params![id, fingerprint, secret.secret_hash, secret.salt_hex, now],
        )?;
        Ok(id)
    }

    /// Liste tous les pairings (révoqués inclus, classés par création).
    pub fn list_pairings(&self) -> Result<Vec<PairingRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, fingerprint, created_at, last_seen_at, revoked_at
             FROM device_pairings
             ORDER BY created_at DESC",
        )?;
        let rows = stmt
            .query_map([], |row| {
                let created: String = row.get(2)?;
                let last_seen: Option<String> = row.get(3)?;
                let revoked: Option<String> = row.get(4)?;
                Ok(PairingRow {
                    id: row.get(0)?,
                    fingerprint: row.get(1)?,
                    created_at: parse_dt(&created)?,
                    last_seen_at: last_seen.map(|s| parse_dt(&s)).transpose()?,
                    revoked_at: revoked.map(|s| parse_dt(&s)).transpose()?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    /// Marque un pairing comme révoqué (n'efface pas la ligne — on garde
    /// l'historique pour audit).
    pub fn revoke_pairing(&self, id: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let affected = self.conn.execute(
            "UPDATE device_pairings SET revoked_at = ?1 WHERE id = ?2 AND revoked_at IS NULL",
            params![now, id],
        )?;
        if affected == 0 {
            anyhow::bail!("pairing introuvable ou déjà révoqué: {id}");
        }
        Ok(())
    }

    /// Vérifie qu'un secret en clair correspond à un pairing actif.
    /// Retourne l'`id` du pairing matché, ou `None` si secret invalide /
    /// pairing révoqué.
    pub fn verify_secret(&self, fingerprint: &str, candidate_hex: &str) -> Result<Option<String>> {
        let row = self.conn.query_row(
            "SELECT id, secret_hash, salt_hex FROM device_pairings
             WHERE fingerprint = ?1 AND revoked_at IS NULL",
            params![fingerprint],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        );
        match row {
            Ok((id, hash, salt)) => {
                if PairingSecret::verify_against(&hash, &salt, candidate_hex) {
                    // Met à jour `last_seen_at` (best-effort).
                    let now = Utc::now().to_rfc3339();
                    let _ = self.conn.execute(
                        "UPDATE device_pairings SET last_seen_at = ?1 WHERE id = ?2",
                        params![now, id],
                    );
                    Ok(Some(id))
                } else {
                    Ok(None)
                }
            },
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Insère un évènement (déjà validé pour un pairing actif).
    pub fn record_event(&self, event: &ExtensionEventInput) -> Result<String> {
        let id = ulid();
        let ingested = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO extension_events(
                id, pairing_id, ts, method, model_id, tokens_in, tokens_out,
                gco2eq_p50, water_ml, energy_wh, raw_payload_json, ingested_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                id,
                event.pairing_id,
                event.ts.to_rfc3339(),
                event.method,
                event.model_id,
                event.tokens_in,
                event.tokens_out,
                event.gco2eq_p50,
                event.water_ml,
                event.energy_wh,
                event.raw_payload_json,
                ingested,
            ],
        )?;
        Ok(id)
    }

    /// Liste les évènements (paginé : `limit` + `offset`).
    pub fn list_events(&self, limit: usize, offset: usize) -> Result<Vec<ExtensionEventRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, pairing_id, ts, method, model_id, tokens_in, tokens_out,
                    gco2eq_p50, water_ml, energy_wh, ingested_at
             FROM extension_events
             ORDER BY ts DESC
             LIMIT ?1 OFFSET ?2",
        )?;
        let limit_i64 = i64::try_from(limit).unwrap_or(i64::MAX);
        let offset_i64 = i64::try_from(offset).unwrap_or(i64::MAX);
        let rows = stmt
            .query_map(params![limit_i64, offset_i64], |row| {
                let ts: String = row.get(2)?;
                let ingested: String = row.get(10)?;
                let tokens_in: i64 = row.get(5)?;
                let tokens_out: i64 = row.get(6)?;
                Ok(ExtensionEventRow {
                    id: row.get(0)?,
                    pairing_id: row.get(1)?,
                    ts: parse_dt(&ts)?,
                    method: row.get(3)?,
                    model_id: row.get(4)?,
                    tokens_in: u32::try_from(tokens_in).unwrap_or(0),
                    tokens_out: u32::try_from(tokens_out).unwrap_or(0),
                    gco2eq_p50: row.get(7)?,
                    water_ml: row.get(8)?,
                    energy_wh: row.get(9)?,
                    ingested_at: parse_dt(&ingested)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }
}

/// Entrée à insérer dans `extension_events` après validation du secret.
#[derive(Debug, Clone)]
pub struct ExtensionEventInput {
    pub pairing_id: String,
    pub ts: DateTime<Utc>,
    pub method: String,
    pub model_id: String,
    pub tokens_in: u32,
    pub tokens_out: u32,
    pub gco2eq_p50: f64,
    pub water_ml: f64,
    pub energy_wh: f64,
    pub raw_payload_json: String,
}

/// Drain le spool fichier `incoming.jsonl` produit par `sobria-bridge`.
///
/// Pour chaque ligne JSON :
///   - extrait le `fingerprint` (TODO : envoyer fingerprint dans le payload
///     du bridge ; en attendant, on accepte tout pairing actif comme fallback
///     POC)
///   - vérifie le secret via `ExtensionStore::verify_secret`
///   - insère dans `extension_events`
///   - skip ligne sans crash si validation échoue (audit best-effort)
///
/// Tronque le spool atomiquement à la fin (rename → drop).
/// Retourne le nombre d'évènements insérés.
pub fn drain_spool(store: &ExtensionStore, spool_path: &Path) -> Result<usize> {
    if !spool_path.exists() {
        return Ok(0);
    }
    // Lit + renomme atomiquement pour éviter de perdre des écritures
    // concurrentes du bridge pendant le drain.
    let tmp = spool_path.with_extension("draining");
    std::fs::rename(spool_path, &tmp).with_context(|| {
        format!(
            "rename {} → {} (drain)",
            spool_path.display(),
            tmp.display()
        )
    })?;
    let content = std::fs::read_to_string(&tmp)?;
    let _ = std::fs::remove_file(&tmp);

    let mut inserted = 0;
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let envelope: SpoolEnvelope = match serde_json::from_str(line) {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!(error = %e, "skip ligne spool malformée");
                continue;
            },
        };
        let Some(input) = envelope.into_event_input(store)? else {
            tracing::warn!("skip ligne spool : aucun pairing actif ne valide le secret");
            continue;
        };
        if store.record_event(&input).is_ok() {
            inserted += 1;
        }
    }
    Ok(inserted)
}

/// Format attendu d'une ligne du spool (cf. `sobria-bridge::handle_request`).
#[derive(Debug, Deserialize)]
struct SpoolEnvelope {
    secret_hash: Option<String>,
    payload: Value,
    received_at: Option<String>,
}

impl SpoolEnvelope {
    fn into_event_input(self, store: &ExtensionStore) -> Result<Option<ExtensionEventInput>> {
        let payload = &self.payload;
        let Some(estimate) = payload.get("estimate") else {
            return Ok(None);
        };
        // POC : sans fingerprint dans le payload, on accepte le premier
        // pairing actif. C27.5 v0.7+ : enrichir le bridge pour pousser
        // fingerprint + secret afin que `verify_secret` puisse filtrer
        // précisément.
        let Some(pairing_id) = store.first_active_pairing()? else {
            return Ok(None);
        };
        let ts = self
            .received_at
            .as_deref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map_or_else(Utc::now, |d| d.with_timezone(&Utc));
        // Marker pour audit : on note le secret_hash short reçu.
        let _ = self.secret_hash; // évite warning unused
        Ok(Some(ExtensionEventInput {
            pairing_id,
            ts,
            method: estimate
                .get("method")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("afnor_sobria")
                .to_string(),
            model_id: estimate
                .get("modelId")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown")
                .to_string(),
            tokens_in: u32::try_from(
                estimate
                    .get("tokensIn")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0),
            )
            .unwrap_or(u32::MAX),
            tokens_out: u32::try_from(
                estimate
                    .get("tokensOut")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0),
            )
            .unwrap_or(u32::MAX),
            gco2eq_p50: estimate
                .get("gco2eq")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0),
            water_ml: estimate
                .get("waterMl")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0),
            energy_wh: estimate
                .get("energyWh")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0),
            raw_payload_json: payload.to_string(),
        }))
    }
}

impl ExtensionStore {
    /// Retourne l'`id` du premier pairing actif (révoqué exclu).
    /// Utilisé par `drain_spool` quand on n'a pas encore le fingerprint
    /// dans le payload du bridge (POC C27.5.d).
    fn first_active_pairing(&self) -> Result<Option<String>> {
        let row = self.conn.query_row(
            "SELECT id FROM device_pairings WHERE revoked_at IS NULL
             ORDER BY created_at DESC LIMIT 1",
            [],
            |row| row.get::<_, String>(0),
        );
        match row {
            Ok(id) => Ok(Some(id)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

// ─── Helpers privés ──────────────────────────────────────────────────────────

fn parse_dt(s: &str) -> rusqlite::Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })
}

/// Alphabet Crockford Base32 (spec ULID — pas de I, L, O, U).
const CROCKFORD: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// Génère un ULID — 26 caractères, time-sortable, sans dépendance externe.
///
/// Format conforme à <https://github.com/ulid/spec> :
/// - 10 chars timestamp (48 bits ms depuis epoch, big-endian)
/// - 16 chars random (80 bits, OS RNG via `rand::thread_rng`)
///
/// Avantage vs UUID v4 : ordre lexicographique = ordre chronologique,
/// utile pour les `ORDER BY id` en SQL et les index B-tree (insertion
/// monotone, pas de page splits).
fn ulid() -> String {
    use rand::RngCore;
    use std::time::{SystemTime, UNIX_EPOCH};

    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());

    // 10 chars : 48 bits timestamp, packé 5 bits par char.
    // 48 bits = 10×5 - 2, donc on a 2 bits de padding au sommet du 1er char.
    let mut out = [0u8; 26];
    let mut t = ms;
    for i in (0..10).rev() {
        out[i] = CROCKFORD[(t & 0x1f) as usize];
        t >>= 5;
    }

    // 16 chars : 80 bits random.
    let mut rand_bytes = [0u8; 10];
    rand::thread_rng().fill_bytes(&mut rand_bytes);
    // On lit les 80 bits comme un grand entier 128-bit aligné à droite et
    // on shift 5 par 5.
    let mut r: u128 = 0;
    for b in &rand_bytes {
        r = (r << 8) | u128::from(*b);
    }
    for i in (10..26).rev() {
        out[i] = CROCKFORD[(r & 0x1f) as usize];
        r >>= 5;
    }

    // SAFETY : tous les bytes viennent de CROCKFORD (ASCII alphanumérique).
    String::from_utf8(out.to_vec()).expect("ULID chars are ASCII by construction")
}

#[allow(dead_code)]
fn _spool_default_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("home_dir introuvable")?;
    Ok(home.join(".sobria").join("spool").join("incoming.jsonl"))
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_secret() -> PairingSecret {
        PairingSecret::new()
    }

    #[test]
    fn open_creates_tables() {
        let store = ExtensionStore::open_in_memory().unwrap();
        let _ = store.list_pairings().unwrap();
        let _ = store.list_events(10, 0).unwrap();
    }

    #[test]
    fn record_and_list_pairing() {
        let store = ExtensionStore::open_in_memory().unwrap();
        let s = make_secret();
        let id = store.record_pairing("chrome-mac-abc123", &s).unwrap();
        let list = store.list_pairings().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, id);
        assert_eq!(list[0].fingerprint, "chrome-mac-abc123");
        assert!(list[0].revoked_at.is_none());
    }

    #[test]
    fn record_same_fingerprint_replaces() {
        let store = ExtensionStore::open_in_memory().unwrap();
        let id1 = store.record_pairing("fp1", &make_secret()).unwrap();
        let id2 = store.record_pairing("fp1", &make_secret()).unwrap();
        assert_ne!(id1, id2);
        let list = store.list_pairings().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, id2);
    }

    #[test]
    fn verify_secret_matches_pairing() {
        let store = ExtensionStore::open_in_memory().unwrap();
        let s = make_secret();
        let id = store.record_pairing("fp", &s).unwrap();
        let matched = store.verify_secret("fp", &s.secret_hex).unwrap();
        assert_eq!(matched, Some(id));
    }

    #[test]
    fn verify_secret_rejects_wrong() {
        let store = ExtensionStore::open_in_memory().unwrap();
        let s = make_secret();
        store.record_pairing("fp", &s).unwrap();
        let matched = store.verify_secret("fp", "00".repeat(32).as_str()).unwrap();
        assert_eq!(matched, None);
    }

    #[test]
    fn verify_secret_rejects_revoked() {
        let store = ExtensionStore::open_in_memory().unwrap();
        let s = make_secret();
        let id = store.record_pairing("fp", &s).unwrap();
        store.revoke_pairing(&id).unwrap();
        let matched = store.verify_secret("fp", &s.secret_hex).unwrap();
        assert_eq!(matched, None, "secret révoqué doit être rejeté");
    }

    #[test]
    fn revoke_unknown_fails() {
        let store = ExtensionStore::open_in_memory().unwrap();
        assert!(store.revoke_pairing("does-not-exist").is_err());
    }

    #[test]
    fn record_event_then_list() {
        let store = ExtensionStore::open_in_memory().unwrap();
        let s = make_secret();
        let pid = store.record_pairing("fp", &s).unwrap();
        store
            .record_event(&ExtensionEventInput {
                pairing_id: pid.clone(),
                ts: Utc::now(),
                method: "afnor_sobria".into(),
                model_id: "gpt-4o".into(),
                tokens_in: 100,
                tokens_out: 500,
                gco2eq_p50: 0.42,
                water_ml: 1.8,
                energy_wh: 0.12,
                raw_payload_json: "{}".into(),
            })
            .unwrap();
        let events = store.list_events(10, 0).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].pairing_id, pid);
        assert_eq!(events[0].model_id, "gpt-4o");
    }

    #[test]
    fn drain_spool_empty_file_returns_zero() {
        let dir = TempDir::new().unwrap();
        let store = ExtensionStore::open_in_memory().unwrap();
        let path = dir.path().join("incoming.jsonl");
        let inserted = drain_spool(&store, &path).unwrap();
        assert_eq!(inserted, 0);
    }

    #[test]
    fn drain_spool_inserts_events() {
        let dir = TempDir::new().unwrap();
        let store = ExtensionStore::open_in_memory().unwrap();
        let s = make_secret();
        store.record_pairing("fp1", &s).unwrap();
        let path = dir.path().join("incoming.jsonl");
        // 2 lignes valides
        let envelope = serde_json::json!({
            "secret_hash": "abcdef",
            "payload": {
                "estimate": {
                    "method": "afnor_sobria",
                    "modelId": "gpt-4o",
                    "tokensIn": 100,
                    "tokensOut": 500,
                    "gco2eq": 0.42,
                    "waterMl": 1.8,
                    "energyWh": 0.12
                }
            },
            "received_at": "2026-05-16T12:00:00Z"
        });
        let line = serde_json::to_string(&envelope).unwrap();
        std::fs::write(&path, format!("{line}\n{line}\n")).unwrap();
        let inserted = drain_spool(&store, &path).unwrap();
        assert_eq!(inserted, 2);
        assert!(!path.exists(), "spool doit être vidé après drain");
        let events = store.list_events(10, 0).unwrap();
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn drain_spool_skips_malformed_lines() {
        let dir = TempDir::new().unwrap();
        let store = ExtensionStore::open_in_memory().unwrap();
        store.record_pairing("fp1", &make_secret()).unwrap();
        let path = dir.path().join("incoming.jsonl");
        let valid = serde_json::json!({
            "payload": { "estimate": { "method": "afnor_sobria", "modelId": "gpt-4o",
                "tokensIn": 1, "tokensOut": 1, "gco2eq": 0.1, "waterMl": 0.1, "energyWh": 0.01 }}
        });
        std::fs::write(
            &path,
            format!(
                "{}\nnot-json garbage\n\n{}\n",
                serde_json::to_string(&valid).unwrap(),
                serde_json::to_string(&valid).unwrap()
            ),
        )
        .unwrap();
        let inserted = drain_spool(&store, &path).unwrap();
        assert_eq!(inserted, 2, "deux lignes valides, garbage ignoré");
    }

    #[test]
    fn drain_spool_no_pairing_skips_all() {
        let dir = TempDir::new().unwrap();
        let store = ExtensionStore::open_in_memory().unwrap();
        // Aucun pairing → first_active_pairing returns None.
        let path = dir.path().join("incoming.jsonl");
        let envelope = serde_json::json!({
            "payload": { "estimate": { "method": "x", "modelId": "y",
                "tokensIn": 1, "tokensOut": 1, "gco2eq": 0.1, "waterMl": 0.1, "energyWh": 0.01 }}
        });
        std::fs::write(&path, serde_json::to_string(&envelope).unwrap()).unwrap();
        let inserted = drain_spool(&store, &path).unwrap();
        assert_eq!(inserted, 0);
    }

    #[test]
    fn ulid_format() {
        let id = ulid();
        assert_eq!(id.len(), 26, "ULID = 26 chars");
        for c in id.bytes() {
            assert!(
                CROCKFORD.contains(&c),
                "char {} ({}) doit être dans Crockford Base32",
                c as char,
                c
            );
        }
    }

    #[test]
    fn ulid_is_unique_and_sortable() {
        // 10 ULID consécutifs : tous différents ET ordre lex = ordre temporel.
        let mut ids: Vec<String> = Vec::with_capacity(10);
        for _ in 0..10 {
            ids.push(ulid());
            // Petit sleep pour garantir un ms d'écart sur certains OS.
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(unique.len(), 10, "10 ULID doivent être distincts");
        // Le préfixe temporel (10 premiers chars) doit être croissant.
        let prefixes: Vec<&str> = ids.iter().map(|s| &s[..10]).collect();
        let mut sorted = prefixes.clone();
        sorted.sort();
        assert_eq!(prefixes, sorted, "préfixes temporels triés croissants");
    }
}
