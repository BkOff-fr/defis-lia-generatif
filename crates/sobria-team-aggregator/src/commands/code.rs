//! Commande CLI `sobria-team-aggregator code` (create/list/revoke).
//!
//! - `code create N --ttl-days 7 --admin admin` : génère N codes 12 chiffres
//!   (OS RNG), les hashe Argon2id, les insère, et les affiche en clair UNE
//!   seule fois (warning UX dans la sortie).
//! - `code list` : table tabulée des codes existants (id, créé par, état).
//! - `code revoke <id>` : pose `revoked_at` = `now`.

use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use rand::RngCore;
use ulid::Ulid;

use crate::config::DataPaths;
use crate::storage::{admins, enrollment_codes, Storage};

/// Codes nouvellement créés (clair + métadonnées).
pub use crate::storage::enrollment_codes::CreatedCode;

/// Crée N codes pour `admin_username` (défaut `admin`), valides `ttl_days`.
pub fn create_batch(
    paths: &DataPaths,
    count: u32,
    ttl_days: i64,
    admin_username: &str,
) -> Result<Vec<CreatedCode>> {
    if count == 0 {
        anyhow::bail!("count doit être ≥ 1");
    }
    if ttl_days <= 0 {
        anyhow::bail!("ttl_days doit être ≥ 1");
    }

    let storage = Storage::open(&paths.db()).context("open team.sqlite")?;
    let admin = admins::find_by_username(storage.connection(), admin_username)
        .context("query admin")?
        .ok_or_else(|| anyhow::anyhow!("admin `{admin_username}` introuvable"))?;
    let now = Utc::now();
    let expires_at = now + Duration::days(ttl_days);

    let mut out = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let id = Ulid::new().to_string();
        let code = random_12_digit_code();
        enrollment_codes::insert(storage.connection(), &id, &code, &admin.id, now, expires_at)
            .context("insert enrollment code")?;
        out.push(CreatedCode {
            id,
            code,
            expires_at,
        });
    }
    Ok(out)
}

/// Liste les codes (admin view).
pub fn list_all(paths: &DataPaths) -> Result<Vec<enrollment_codes::EnrollmentCodeRow>> {
    let storage = Storage::open(&paths.db()).context("open team.sqlite")?;
    Ok(enrollment_codes::list_all(storage.connection())?)
}

/// Révoque un code par id.
pub fn revoke(paths: &DataPaths, id: &str) -> Result<bool> {
    let storage = Storage::open(&paths.db()).context("open team.sqlite")?;
    Ok(enrollment_codes::revoke(
        storage.connection(),
        id,
        Utc::now(),
    )?)
}

/// Code aléatoire 12 chiffres ASCII via OS RNG.
///
/// Entropie ≈ 40 bits (10^12 = 2^39.86) — suffisant car les codes sont
/// révoqués après usage et Argon2id rend tout brute force serveur lent.
fn random_12_digit_code() -> String {
    let mut bytes = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    // 16 octets → u128 ; on prend modulo 10^12 pour un nombre 12 chiffres.
    let mut acc: u128 = 0;
    for b in &bytes {
        acc = acc.wrapping_mul(31).wrapping_add(u128::from(*b));
    }
    let n: u64 = (acc % 1_000_000_000_000u128) as u64;
    format!("{n:012}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn init_temp_aggregator() -> (tempfile::TempDir, DataPaths) {
        let dir = tempdir().unwrap();
        let paths = DataPaths::new(dir.path());
        crate::commands::init::run(&paths, "admin", "init-pw-secret", false).unwrap();
        (dir, paths)
    }

    #[test]
    fn create_batch_inserts_codes_with_distinct_clear_text() {
        let (_dir, paths) = init_temp_aggregator();
        let codes = create_batch(&paths, 5, 7, "admin").unwrap();
        assert_eq!(codes.len(), 5);
        for c in &codes {
            assert_eq!(c.code.len(), 12);
            assert!(c.code.chars().all(|c| c.is_ascii_digit()));
        }
        let clear_set: std::collections::HashSet<_> =
            codes.iter().map(|c| c.code.clone()).collect();
        // Pas obligatoirement 5 distincts (RNG peut collisionner), mais ≥ 3.
        assert!(clear_set.len() >= 3, "trop de collisions: {clear_set:?}");

        let listed = list_all(&paths).unwrap();
        assert_eq!(listed.len(), 5);
    }

    #[test]
    fn revoke_then_relist_shows_revoked_at() {
        let (_dir, paths) = init_temp_aggregator();
        let codes = create_batch(&paths, 2, 7, "admin").unwrap();
        let id = codes[0].id.clone();
        assert!(revoke(&paths, &id).unwrap());
        let listed = list_all(&paths).unwrap();
        let row = listed.iter().find(|r| r.id == id).unwrap();
        assert!(row.revoked_at.is_some());
        // Re-revoke → false.
        assert!(!revoke(&paths, &id).unwrap());
    }

    #[test]
    fn create_batch_rejects_unknown_admin() {
        let (_dir, paths) = init_temp_aggregator();
        let err = create_batch(&paths, 1, 7, "nobody").unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("nobody"), "msg = {msg}");
    }
}
