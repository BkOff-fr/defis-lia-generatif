//! Entrée du ledger d'audit.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Sentinel placé dans le champ payload après purge RGPD.
pub const PURGED_SENTINEL: &str = "PURGED";

/// Une entrée du journal d'audit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditEntry {
    /// Identifiant SQLite auto-incrément.
    pub id: i64,
    /// Horodatage UTC RFC 3339.
    pub timestamp: DateTime<Utc>,
    /// `EstimationResult` sérialisé en JSON, ou [`PURGED_SENTINEL`] après purge RGPD.
    pub estimation_result_json: String,
    /// Signature de l'entrée précédente (chaînage). `""` pour la genesis.
    pub prev_sig: String,
    /// Signature SHA-256 de cette entrée :
    /// `SHA256(timestamp || estimation_result_json || prev_sig)`.
    pub sig: String,
    /// Horodatage de purge RGPD, ou `None` si l'entrée n'a jamais été purgée.
    pub purged_at: Option<DateTime<Utc>>,
}

impl AuditEntry {
    /// Calcule la signature attendue de cette entrée à partir de ses
    /// champs `timestamp`, `estimation_result_json` et `prev_sig`.
    ///
    /// Note importante : après une purge, le payload est remplacé par
    /// [`PURGED_SENTINEL`], **mais la signature originale est conservée**
    /// pour préserver la chaîne. Donc `compute_sig()` ne correspondra
    /// plus au `sig` stocké pour les entrées purgées — c'est attendu.
    /// La vérification de chaîne tient compte de ce cas.
    #[must_use]
    pub fn compute_sig(&self) -> String {
        let payload = format!(
            "{}|{}|{}",
            self.timestamp.to_rfc3339(),
            self.estimation_result_json,
            self.prev_sig
        );
        hex_encode(&Sha256::digest(payload.as_bytes()))
    }

    /// `true` si l'entrée a été purgée (payload remplacé par sentinel).
    #[must_use]
    pub fn is_purged(&self) -> bool {
        self.estimation_result_json == PURGED_SENTINEL && self.purged_at.is_some()
    }
}

/// Encode un slice de bytes en hexadécimal minuscule.
fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = Vec::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize]);
        out.push(HEX[(b & 0x0f) as usize]);
    }
    String::from_utf8(out).expect("hex chars ASCII")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn sample() -> AuditEntry {
        AuditEntry {
            id: 1,
            timestamp: Utc.with_ymd_and_hms(2026, 5, 12, 14, 32, 8).unwrap(),
            estimation_result_json: r#"{"co2eq_p50": 2.14}"#.into(),
            prev_sig: String::new(),
            sig: String::new(),
            purged_at: None,
        }
    }

    #[test]
    fn compute_sig_is_deterministic() {
        let e = sample();
        assert_eq!(e.compute_sig(), e.compute_sig());
    }

    #[test]
    fn compute_sig_changes_when_payload_changes() {
        let mut a = sample();
        let sig_a = a.compute_sig();
        a.estimation_result_json = r#"{"co2eq_p50": 99.0}"#.into();
        let sig_b = a.compute_sig();
        assert_ne!(sig_a, sig_b);
    }

    #[test]
    fn compute_sig_is_64_hex_chars() {
        let s = sample().compute_sig();
        assert_eq!(s.len(), 64);
        assert!(s.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn is_purged_returns_correctly() {
        let mut e = sample();
        assert!(!e.is_purged());
        e.estimation_result_json = PURGED_SENTINEL.into();
        e.purged_at = Some(Utc::now());
        assert!(e.is_purged());
    }
}
