//! Commandes CLI `admin` (C29.2 — brief §C29.2).
//!
//! - [`reset_password`] : réinitialise le mot de passe d'un admin existant
//!   (Argon2id PHC + révocation des tokens actifs).
//! - [`list_admins`]    : énumère les admins (id, username, dates).

use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::config::DataPaths;
use crate::crypto::password;
use crate::storage::{admins, tokens, Storage};

/// Vue condensée d'un admin pour la sortie CLI `admin list`.
#[derive(Debug, Clone)]
pub struct AdminSummary {
    pub id: String,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

/// Liste tous les admins du data dir donné (CLI `admin list`).
pub fn list_admins(paths: &DataPaths) -> Result<Vec<AdminSummary>> {
    let storage = Storage::open(&paths.db())?;
    let rows = admins::list_all(storage.connection())?;
    Ok(rows
        .into_iter()
        .map(|r| AdminSummary {
            id: r.id,
            username: r.username,
            created_at: r.created_at,
            last_login_at: r.last_login_at,
        })
        .collect())
}

/// Résultat d'un reset-password : nb tokens révoqués + nouvel admin row.
#[derive(Debug, Clone)]
pub struct ResetOutcome {
    pub admin_id: String,
    pub username: String,
    pub revoked_tokens: usize,
}

/// Réinitialise le mot de passe d'un admin (CLI `admin reset-password`).
///
/// Le mot de passe doit faire ≥ 8 caractères. Le hash Argon2id PHC remplace
/// l'ancien et `last_login_at` est mis à NULL. Tous les tokens admin actifs
/// sont révoqués (un nouvel `/login` est alors nécessaire).
///
/// Retourne `Err` si l'admin n'existe pas.
pub fn reset_password(
    paths: &DataPaths,
    username: &str,
    new_password: &str,
) -> Result<ResetOutcome> {
    if new_password.len() < 8 {
        anyhow::bail!("le mot de passe doit faire au moins 8 caractères");
    }
    let storage = Storage::open(&paths.db())?;

    let admin = admins::find_by_username(storage.connection(), username)?
        .ok_or_else(|| anyhow::anyhow!("admin `{username}` introuvable"))?;

    let new_hash = password::hash_password(new_password)?;
    let updated = admins::set_password_hash(storage.connection(), username, &new_hash)?;
    if updated == 0 {
        anyhow::bail!("admin `{username}` introuvable lors du UPDATE");
    }

    let revoked = tokens::revoke_all_for_admin(storage.connection(), &admin.id, Utc::now())?;

    Ok(ResetOutcome {
        admin_id: admin.id,
        username: admin.username,
        revoked_tokens: revoked,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::init;
    use chrono::Duration;
    use tempfile::tempdir;

    fn setup() -> (tempfile::TempDir, DataPaths) {
        let dir = tempdir().unwrap();
        let paths = DataPaths::new(dir.path());
        init::run(&paths, "alice", "alice-initial-pw", false).unwrap();
        (dir, paths)
    }

    #[test]
    fn reset_password_rewrites_hash_and_revokes_tokens() {
        let (_dir, paths) = setup();

        // Pré-condition : ajoute 2 tokens admin actifs en base.
        let storage = Storage::open(&paths.db()).unwrap();
        let admin = admins::find_by_username(storage.connection(), "alice")
            .unwrap()
            .unwrap();
        let now = Utc::now();
        for (i, secret) in ["s1", "s2"].iter().enumerate() {
            let hash = password::hash_password(secret).unwrap();
            tokens::insert(
                storage.connection(),
                &format!("t-{i}"),
                None,
                Some(&admin.id),
                &hash,
                now,
                now + Duration::days(7),
            )
            .unwrap();
        }
        let initial_hash = admin.password_hash.clone();
        drop(storage);

        let out = reset_password(&paths, "alice", "new-password-strong").unwrap();
        assert_eq!(out.username, "alice");
        assert_eq!(out.revoked_tokens, 2);

        // Vérifie le nouveau hash en relisant la base.
        let storage = Storage::open(&paths.db()).unwrap();
        let after = admins::find_by_username(storage.connection(), "alice")
            .unwrap()
            .unwrap();
        assert_ne!(after.password_hash, initial_hash);
        assert!(password::verify_password(
            &after.password_hash,
            "new-password-strong"
        ));
        assert!(after.last_login_at.is_none());

        // Et les tokens sont bien révoqués.
        let t1 = tokens::find_by_id(storage.connection(), "t-0")
            .unwrap()
            .unwrap();
        let t2 = tokens::find_by_id(storage.connection(), "t-1")
            .unwrap()
            .unwrap();
        assert!(t1.revoked_at.is_some());
        assert!(t2.revoked_at.is_some());
    }

    #[test]
    fn reset_password_rejects_short_password() {
        let (_dir, paths) = setup();
        let err = reset_password(&paths, "alice", "short");
        assert!(err.is_err());
    }

    #[test]
    fn reset_password_unknown_admin_fails() {
        let (_dir, paths) = setup();
        let err = reset_password(&paths, "bob", "long-enough-pw");
        assert!(err.is_err());
    }

    #[test]
    fn list_admins_returns_seed() {
        let (_dir, paths) = setup();
        let list = list_admins(&paths).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].username, "alice");
    }
}
