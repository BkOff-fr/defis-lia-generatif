//! Ledger ACID chaîné — voir `briefs/chantiers/C08-audit-ledger.md`.

use std::{io::Write, path::Path};

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use sobria_core::EstimationResult;
use tracing::{debug, info};

use crate::{
    entry::{AuditEntry, PURGED_SENTINEL},
    error::AuditResult,
};

/// Rapport d'intégrité produit par [`AuditLedger::verify_chain`].
#[derive(Debug, Clone)]
pub struct IntegrityReport {
    /// Nombre total d'entrées vérifiées.
    pub total_entries: usize,
    /// `true` si la chaîne est intacte.
    pub valid: bool,
    /// ID de la première entrée fautive si `valid == false`.
    pub first_invalid_id: Option<i64>,
    /// Message humain résumant le résultat.
    pub message: String,
}

/// Ledger ACID chaîné SHA-256.
pub struct AuditLedger {
    conn: Connection,
}

impl AuditLedger {
    /// Ouvre (ou crée) un ledger sur disque en mode WAL.
    pub fn open(path: &Path) -> AuditResult<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            r"
            PRAGMA journal_mode = WAL;
            CREATE TABLE IF NOT EXISTS audit_entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                estimation_result_json TEXT NOT NULL,
                prev_sig TEXT NOT NULL,
                sig TEXT NOT NULL,
                purged_at TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_audit_timestamp
                ON audit_entries(timestamp);
            ",
        )?;
        Ok(Self { conn })
    }

    /// Ajoute une nouvelle entrée chaînée à la signature de la précédente.
    ///
    /// Retourne l'entrée écrite (avec son `id` et son `sig` calculés).
    pub fn append(&mut self, result: &EstimationResult) -> AuditResult<AuditEntry> {
        let payload = serde_json::to_string(result)?;
        let timestamp = Utc::now();
        let prev_sig = self.last_sig()?.unwrap_or_default();

        let mut tentative = AuditEntry {
            id: 0, // remplacé par last_insert_rowid()
            timestamp,
            estimation_result_json: payload,
            prev_sig,
            sig: String::new(),
            purged_at: None,
        };
        tentative.sig = tentative.compute_sig();

        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO audit_entries \
             (timestamp, estimation_result_json, prev_sig, sig, purged_at) \
             VALUES (?1, ?2, ?3, ?4, NULL)",
            params![
                tentative.timestamp.to_rfc3339(),
                tentative.estimation_result_json,
                tentative.prev_sig,
                tentative.sig,
            ],
        )?;
        let id = tx.last_insert_rowid();
        tx.commit()?;
        tentative.id = id;
        debug!(id, sig = %&tentative.sig[..16], "audit: entrée écrite");
        Ok(tentative)
    }

    /// Vérifie l'intégrité complète de la chaîne.
    pub fn verify_chain(&self) -> AuditResult<IntegrityReport> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, estimation_result_json, prev_sig, sig, purged_at \
             FROM audit_entries ORDER BY id ASC",
        )?;
        let rows = stmt.query_map([], row_to_entry)?;
        let mut total = 0;
        let mut prev_sig = String::new();
        for row in rows {
            let entry = row?;
            total += 1;
            // Continuité de chaîne
            if entry.prev_sig != prev_sig {
                return Ok(IntegrityReport {
                    total_entries: total,
                    valid: false,
                    first_invalid_id: Some(entry.id),
                    message: format!(
                        "entrée {} : prev_sig {:?} != sig précédent {:?}",
                        entry.id,
                        &entry.prev_sig[..entry.prev_sig.len().min(16)],
                        &prev_sig[..prev_sig.len().min(16)]
                    ),
                });
            }
            // Signature interne :
            //   - Si purgée : on saute la vérification du payload, mais
            //     on garde sig original pour le chaînage.
            //   - Sinon : recalcul attendu == stocké.
            if !entry.is_purged() {
                let expected = entry.compute_sig();
                if expected != entry.sig {
                    return Ok(IntegrityReport {
                        total_entries: total,
                        valid: false,
                        first_invalid_id: Some(entry.id),
                        message: format!(
                            "entrée {} : sig stocké != sig recalculé (probable tampering)",
                            entry.id
                        ),
                    });
                }
            }
            prev_sig = entry.sig;
        }
        Ok(IntegrityReport {
            total_entries: total,
            valid: true,
            first_invalid_id: None,
            message: format!("chaîne intègre : {total} entrée(s)"),
        })
    }

    /// Nombre d'entrées dans le ledger.
    pub fn len(&self) -> AuditResult<usize> {
        let n: i64 =
            self.conn.query_row("SELECT COUNT(*) FROM audit_entries", [], |r| r.get(0))?;
        Ok(usize::try_from(n).unwrap_or(0))
    }

    /// `true` si le ledger est vide.
    pub fn is_empty(&self) -> AuditResult<bool> {
        Ok(self.len()? == 0)
    }

    /// Exporte toutes les entrées en NDJSON (une entrée JSON par ligne).
    pub fn export_ndjson(&self, writer: &mut impl Write) -> AuditResult<usize> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, estimation_result_json, prev_sig, sig, purged_at \
             FROM audit_entries ORDER BY id ASC",
        )?;
        let rows = stmt.query_map([], row_to_entry)?;
        let mut count = 0;
        for row in rows {
            let entry = row?;
            let json = serde_json::to_string(&entry)?;
            writer.write_all(json.as_bytes())?;
            writer.write_all(b"\n")?;
            count += 1;
        }
        Ok(count)
    }

    /// Purge RGPD : remplace le payload des entrées antérieures à `before`
    /// par un sentinel, **sans toucher au sig** (la chaîne reste valide).
    /// Retourne le nombre d'entrées purgées.
    pub fn purge_before(&mut self, before: DateTime<Utc>) -> AuditResult<usize> {
        let tx = self.conn.transaction()?;
        let purged_at = Utc::now().to_rfc3339();
        let n = tx.execute(
            "UPDATE audit_entries \
             SET estimation_result_json = ?1, purged_at = ?2 \
             WHERE timestamp < ?3 AND purged_at IS NULL",
            params![PURGED_SENTINEL, purged_at, before.to_rfc3339()],
        )?;
        tx.commit()?;
        info!(count = n, before = %before, "audit: purge RGPD effectuée");
        Ok(n)
    }

    /// Signature de la dernière entrée (ou `None` si ledger vide).
    fn last_sig(&self) -> AuditResult<Option<String>> {
        let sig: Option<String> = self
            .conn
            .query_row(
                "SELECT sig FROM audit_entries ORDER BY id DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .optional()?;
        Ok(sig)
    }
}

/// Convertit une ligne SQLite en `AuditEntry`.
fn row_to_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<AuditEntry> {
    let id: i64 = row.get(0)?;
    let ts_str: String = row.get(1)?;
    let ts = DateTime::parse_from_rfc3339(&ts_str)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(1, rusqlite::types::Type::Text, Box::new(e)))?
        .with_timezone(&Utc);
    let payload: String = row.get(2)?;
    let prev_sig: String = row.get(3)?;
    let sig: String = row.get(4)?;
    let purged_at_str: Option<String> = row.get(5)?;
    let purged_at = match purged_at_str {
        Some(s) => Some(
            DateTime::parse_from_rfc3339(&s)
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(e))
                })?
                .with_timezone(&Utc),
        ),
        None => None,
    };
    Ok(AuditEntry {
        id,
        timestamp: ts,
        estimation_result_json: payload,
        prev_sig,
        sig,
        purged_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use sobria_core::{
        EstimationRequest, Hypothesis, Indicator, IndicatorValue, UncertaintyInterval,
    };

    fn sample_result() -> EstimationResult {
        EstimationResult {
            request: EstimationRequest {
                model_id: "gpt-4o-mini".into(),
                tokens_in: 100,
                tokens_out_estimated: 500,
                datacenter_id: None,
                timestamp: Utc::now(),
            },
            indicators: vec![IndicatorValue {
                indicator: Indicator::Co2Eq,
                interval: UncertaintyInterval::new(1.0, 2.0, 3.0).unwrap(),
                unit: "gCO2eq".into(),
            }],
            equivalents: vec![],
            hypotheses: vec![Hypothesis {
                key: "x".into(),
                value: serde_json::json!(1.0),
                source: "test".into(),
            }],
            computed_at: Utc::now(),
            seed: 42,
        }
    }

    fn open_temp() -> (tempfile::TempDir, AuditLedger) {
        let tmp = tempfile::tempdir().unwrap();
        let ledger = AuditLedger::open(&tmp.path().join("audit.sqlite")).unwrap();
        (tmp, ledger)
    }

    #[test]
    fn empty_ledger_is_valid() {
        let (_tmp, ledger) = open_temp();
        assert!(ledger.is_empty().unwrap());
        let report = ledger.verify_chain().unwrap();
        assert!(report.valid);
        assert_eq!(report.total_entries, 0);
    }

    #[test]
    fn append_one_and_verify() {
        let (_tmp, mut ledger) = open_temp();
        let entry = ledger.append(&sample_result()).unwrap();
        assert_eq!(entry.id, 1);
        assert_eq!(entry.prev_sig, "");
        assert_eq!(entry.sig.len(), 64);
        let report = ledger.verify_chain().unwrap();
        assert!(report.valid, "{}", report.message);
        assert_eq!(report.total_entries, 1);
    }

    #[test]
    fn append_many_and_verify() {
        const N: usize = 50;
        let (_tmp, mut ledger) = open_temp();
        let mut prev_sig = String::new();
        for i in 1..=N {
            let entry = ledger.append(&sample_result()).unwrap();
            assert_eq!(usize::try_from(entry.id).unwrap(), i);
            assert_eq!(entry.prev_sig, prev_sig);
            prev_sig = entry.sig;
        }
        let report = ledger.verify_chain().unwrap();
        assert!(report.valid, "{}", report.message);
        assert_eq!(report.total_entries, N);
    }

    #[test]
    fn tampering_payload_breaks_chain() {
        let (_tmp, mut ledger) = open_temp();
        ledger.append(&sample_result()).unwrap();
        ledger.append(&sample_result()).unwrap();
        // Altère le payload de l'entrée 1 directement via SQL — simule
        // une intrusion.
        ledger
            .conn
            .execute(
                "UPDATE audit_entries SET estimation_result_json = '{\"tampered\":true}' \
                 WHERE id = 1",
                [],
            )
            .unwrap();
        let report = ledger.verify_chain().unwrap();
        assert!(!report.valid);
        assert_eq!(report.first_invalid_id, Some(1));
    }

    #[test]
    fn tampering_prev_sig_breaks_chain() {
        let (_tmp, mut ledger) = open_temp();
        ledger.append(&sample_result()).unwrap();
        ledger.append(&sample_result()).unwrap();
        // Altère le prev_sig de l'entrée 2 — casse la continuité.
        ledger
            .conn
            .execute(
                "UPDATE audit_entries SET prev_sig = 'badbadbad' WHERE id = 2",
                [],
            )
            .unwrap();
        let report = ledger.verify_chain().unwrap();
        assert!(!report.valid);
        assert_eq!(report.first_invalid_id, Some(2));
    }

    #[test]
    fn export_ndjson_produces_one_line_per_entry() {
        let (_tmp, mut ledger) = open_temp();
        for _ in 0..3 {
            ledger.append(&sample_result()).unwrap();
        }
        let mut buf: Vec<u8> = Vec::new();
        let count = ledger.export_ndjson(&mut buf).unwrap();
        assert_eq!(count, 3);
        let text = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        for line in &lines {
            let parsed: serde_json::Value = serde_json::from_str(line).unwrap();
            assert!(parsed["id"].is_i64());
            assert!(parsed["sig"].is_string());
        }
    }

    #[test]
    fn purge_keeps_chain_valid() {
        let (_tmp, mut ledger) = open_temp();
        // Insère 3 entrées
        for _ in 0..3 {
            ledger.append(&sample_result()).unwrap();
        }
        // Purge tout ce qui est antérieur à *demain* → purge les 3
        let n = ledger.purge_before(Utc::now() + Duration::days(1)).unwrap();
        assert_eq!(n, 3);
        // Chaîne toujours valide
        let report = ledger.verify_chain().unwrap();
        assert!(report.valid, "{}", report.message);
        // Toutes les entrées sont purgées
        let mut buf: Vec<u8> = Vec::new();
        ledger.export_ndjson(&mut buf).unwrap();
        let text = String::from_utf8(buf).unwrap();
        for line in text.lines() {
            let parsed: serde_json::Value = serde_json::from_str(line).unwrap();
            assert_eq!(parsed["estimation_result_json"], PURGED_SENTINEL);
            assert!(parsed["purged_at"].is_string());
        }
    }

    #[test]
    fn purge_before_past_no_effect() {
        let (_tmp, mut ledger) = open_temp();
        ledger.append(&sample_result()).unwrap();
        // Purge avant un point passé qui ne couvre rien
        let n = ledger
            .purge_before(Utc::now() - Duration::days(365))
            .unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn ledger_is_persisted_across_open() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("audit.sqlite");
        {
            let mut l = AuditLedger::open(&path).unwrap();
            l.append(&sample_result()).unwrap();
            l.append(&sample_result()).unwrap();
        }
        let l2 = AuditLedger::open(&path).unwrap();
        assert_eq!(l2.len().unwrap(), 2);
        assert!(l2.verify_chain().unwrap().valid);
    }
}
