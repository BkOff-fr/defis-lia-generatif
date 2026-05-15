//! Stockage des projets (M17 / C20).
//!
//! Un projet est une entité persistante nommée qui regroupe une période
//! d'estimations dans le ledger d'audit, pour produire un **datasheet
//! Gebru 2018** reproductible.
//!
//! Table `projects` dans `referentiel.sqlite` (à côté de `app_preferences`
//! et `personal_goals`).

use std::path::Path;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::error::AppError;

/// Longueur max du `name`.
pub const PROJECT_NAME_MAX: usize = 200;
/// Longueur max de la `description`.
pub const PROJECT_DESCRIPTION_MAX: usize = 5000;
/// Nombre max de tags par projet.
pub const PROJECT_TAGS_MAX: usize = 10;
/// Longueur max d'un tag.
pub const PROJECT_TAG_MAX: usize = 50;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
pub struct NewProject {
    pub name: String,
    pub description: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ProjectUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

pub struct ProjectStore {
    conn: Connection,
}

impl ProjectStore {
    /// Ouvre (ou crée) la table `projects` dans la base donnée.
    pub fn open(path: &Path) -> Result<Self, AppError> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            r"
            PRAGMA journal_mode = WAL;
            CREATE TABLE IF NOT EXISTS projects (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                name            TEXT NOT NULL,
                description     TEXT NOT NULL DEFAULT '',
                period_start    TEXT NOT NULL,
                period_end      TEXT NOT NULL,
                tags            TEXT NOT NULL DEFAULT '[]',
                created_at      TEXT NOT NULL,
                updated_at      TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_projects_period
                ON projects(period_start, period_end);
            ",
        )?;
        info!(path = %path.display(), "projects: ouverture du store");
        Ok(Self { conn })
    }

    pub fn list_all(&self) -> Result<Vec<Project>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, period_start, period_end, tags, created_at, updated_at \
             FROM projects ORDER BY id DESC",
        )?;
        let rows = stmt.query_map([], row_to_project)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn get(&self, id: i64) -> Result<Option<Project>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, period_start, period_end, tags, created_at, updated_at \
             FROM projects WHERE id = ?1",
        )?;
        let p = stmt.query_row(params![id], row_to_project).optional()?;
        Ok(p)
    }

    pub fn create(&mut self, p: NewProject) -> Result<Project, AppError> {
        validate_name(&p.name)?;
        validate_description(&p.description)?;
        validate_period(p.period_start, p.period_end)?;
        validate_tags(&p.tags)?;
        let now = Utc::now();
        let tags_json = serde_json::to_string(&p.tags)?;
        self.conn.execute(
            "INSERT INTO projects \
             (name, description, period_start, period_end, tags, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            params![
                p.name,
                p.description,
                p.period_start.to_rfc3339(),
                p.period_end.to_rfc3339(),
                tags_json,
                now.to_rfc3339(),
            ],
        )?;
        let id = self.conn.last_insert_rowid();
        debug!(id, name = %p.name, "project: created");
        Ok(Project {
            id,
            name: p.name,
            description: p.description,
            period_start: p.period_start,
            period_end: p.period_end,
            tags: p.tags,
            created_at: now,
            updated_at: now,
        })
    }

    /// Met à jour un projet existant. Les dates ne sont **pas** modifiables
    /// (voir brief §1.1). Au moins un champ doit être fourni.
    pub fn update(&mut self, id: i64, update: ProjectUpdate) -> Result<Project, AppError> {
        if update.name.is_none() && update.description.is_none() && update.tags.is_none() {
            return Err(AppError::InvalidRequest(
                "au moins un champ à modifier doit être fourni".into(),
            ));
        }
        let mut current = self
            .get(id)?
            .ok_or_else(|| AppError::InvalidRequest(format!("projet {id} inconnu")))?;
        if let Some(name) = update.name {
            validate_name(&name)?;
            current.name = name;
        }
        if let Some(desc) = update.description {
            validate_description(&desc)?;
            current.description = desc;
        }
        if let Some(tags) = update.tags {
            validate_tags(&tags)?;
            current.tags = tags;
        }
        let now = Utc::now();
        let tags_json = serde_json::to_string(&current.tags)?;
        self.conn.execute(
            "UPDATE projects SET name = ?1, description = ?2, tags = ?3, updated_at = ?4 \
             WHERE id = ?5",
            params![
                current.name,
                current.description,
                tags_json,
                now.to_rfc3339(),
                id
            ],
        )?;
        current.updated_at = now;
        Ok(current)
    }

    /// Supprime un projet. Idempotent — pas d'erreur si déjà absent.
    pub fn delete(&mut self, id: i64) -> Result<(), AppError> {
        self.conn
            .execute("DELETE FROM projects WHERE id = ?1", params![id])?;
        debug!(id, "project: deleted");
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Validations
// ─────────────────────────────────────────────────────────────────────────────

fn validate_name(name: &str) -> Result<(), AppError> {
    if name.trim().is_empty() {
        return Err(AppError::InvalidRequest(
            "name ne doit pas être vide".into(),
        ));
    }
    if name.chars().count() > PROJECT_NAME_MAX {
        return Err(AppError::InvalidRequest(format!(
            "name trop long (> {PROJECT_NAME_MAX} caractères)"
        )));
    }
    Ok(())
}

fn validate_description(desc: &str) -> Result<(), AppError> {
    if desc.chars().count() > PROJECT_DESCRIPTION_MAX {
        return Err(AppError::InvalidRequest(format!(
            "description trop longue (> {PROJECT_DESCRIPTION_MAX} caractères)"
        )));
    }
    Ok(())
}

fn validate_period(start: DateTime<Utc>, end: DateTime<Utc>) -> Result<(), AppError> {
    if end <= start {
        return Err(AppError::InvalidRequest(
            "period_end doit être strictement après period_start".into(),
        ));
    }
    Ok(())
}

fn validate_tags(tags: &[String]) -> Result<(), AppError> {
    if tags.len() > PROJECT_TAGS_MAX {
        return Err(AppError::InvalidRequest(format!(
            "trop de tags : {} (max {PROJECT_TAGS_MAX})",
            tags.len()
        )));
    }
    for tag in tags {
        if tag.chars().count() > PROJECT_TAG_MAX {
            return Err(AppError::InvalidRequest(format!(
                "tag '{tag}' trop long (> {PROJECT_TAG_MAX} caractères)"
            )));
        }
        if !tag
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(AppError::InvalidRequest(format!(
                "tag '{tag}' invalide (attendu : a-z, 0-9, -)"
            )));
        }
        if tag.is_empty() {
            return Err(AppError::InvalidRequest("tag vide".into()));
        }
    }
    Ok(())
}

fn row_to_project(row: &rusqlite::Row<'_>) -> rusqlite::Result<Project> {
    let id: i64 = row.get(0)?;
    let name: String = row.get(1)?;
    let description: String = row.get(2)?;
    let period_start_str: String = row.get(3)?;
    let period_end_str: String = row.get(4)?;
    let tags_str: String = row.get(5)?;
    let created_at_str: String = row.get(6)?;
    let updated_at_str: String = row.get(7)?;
    let parse_dt = |s: String, col: usize| -> rusqlite::Result<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(&s)
            .map(|d| d.with_timezone(&Utc))
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    col,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })
    };
    let tags: Vec<String> = serde_json::from_str(&tags_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(e))
    })?;
    Ok(Project {
        id,
        name,
        description,
        period_start: parse_dt(period_start_str, 3)?,
        period_end: parse_dt(period_end_str, 4)?,
        tags,
        created_at: parse_dt(created_at_str, 6)?,
        updated_at: parse_dt(updated_at_str, 7)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn open_temp() -> (tempfile::TempDir, ProjectStore) {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("referentiel.sqlite");
        let store = ProjectStore::open(&path).unwrap();
        (tmp, store)
    }

    fn new_project(name: &str) -> NewProject {
        NewProject {
            name: name.into(),
            description: "Étude test".into(),
            period_start: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            period_end: Utc.with_ymd_and_hms(2026, 4, 1, 0, 0, 0).unwrap(),
            tags: vec!["q1-2026".into(), "claude".into()],
        }
    }

    #[test]
    fn empty_store_lists_empty() {
        let (_tmp, store) = open_temp();
        assert!(store.list_all().unwrap().is_empty());
    }

    #[test]
    fn create_then_list_round_trip() {
        let (_tmp, mut store) = open_temp();
        let p = store.create(new_project("Étude Q1")).unwrap();
        assert!(p.id >= 1);
        let list = store.list_all().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "Étude Q1");
        assert_eq!(list[0].tags, vec!["q1-2026", "claude"]);
    }

    #[test]
    fn get_known_id_returns_project() {
        let (_tmp, mut store) = open_temp();
        let p = store.create(new_project("X")).unwrap();
        let got = store.get(p.id).unwrap().unwrap();
        assert_eq!(got.name, "X");
    }

    #[test]
    fn get_unknown_id_returns_none() {
        let (_tmp, store) = open_temp();
        assert!(store.get(999).unwrap().is_none());
    }

    #[test]
    fn create_rejects_empty_name() {
        let (_tmp, mut store) = open_temp();
        let mut p = new_project("");
        p.name = "   ".into();
        assert!(store.create(p).is_err());
    }

    #[test]
    fn create_rejects_inverted_period() {
        let (_tmp, mut store) = open_temp();
        let mut p = new_project("X");
        std::mem::swap(&mut p.period_start, &mut p.period_end);
        assert!(store.create(p).is_err());
    }

    #[test]
    fn create_rejects_too_many_tags() {
        let (_tmp, mut store) = open_temp();
        let mut p = new_project("X");
        p.tags = (0..11).map(|i| format!("tag-{i}")).collect();
        assert!(store.create(p).is_err());
    }

    #[test]
    fn create_rejects_invalid_tag_chars() {
        let (_tmp, mut store) = open_temp();
        let mut p = new_project("X");
        p.tags = vec!["UPPERCASE".into()];
        assert!(store.create(p).is_err());
    }

    #[test]
    fn update_at_least_one_field_required() {
        let (_tmp, mut store) = open_temp();
        let p = store.create(new_project("X")).unwrap();
        let err = store.update(p.id, ProjectUpdate::default()).unwrap_err();
        assert!(format!("{err}").contains("au moins un champ"));
    }

    #[test]
    fn update_partial_fields() {
        let (_tmp, mut store) = open_temp();
        let p = store.create(new_project("X")).unwrap();
        let updated = store
            .update(
                p.id,
                ProjectUpdate {
                    name: Some("Étude renommée".into()),
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(updated.name, "Étude renommée");
        assert_eq!(updated.description, p.description, "desc inchangée");
        assert_eq!(updated.tags, p.tags, "tags inchangés");
        // updated_at doit avoir bougé
        assert!(updated.updated_at >= p.updated_at);
    }

    #[test]
    fn update_unknown_id_returns_error() {
        let (_tmp, mut store) = open_temp();
        let err = store
            .update(
                999,
                ProjectUpdate {
                    name: Some("X".into()),
                    ..Default::default()
                },
            )
            .unwrap_err();
        assert!(format!("{err}").contains("inconnu"));
    }

    #[test]
    fn delete_idempotent() {
        let (_tmp, mut store) = open_temp();
        store.delete(999).unwrap();
        let p = store.create(new_project("X")).unwrap();
        store.delete(p.id).unwrap();
        assert!(store.get(p.id).unwrap().is_none());
        store.delete(p.id).unwrap();
    }

    #[test]
    fn persists_across_reopen() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("referentiel.sqlite");
        let id = {
            let mut s = ProjectStore::open(&path).unwrap();
            let p = s.create(new_project("Persistent")).unwrap();
            p.id
        };
        let s2 = ProjectStore::open(&path).unwrap();
        let p = s2.get(id).unwrap().unwrap();
        assert_eq!(p.name, "Persistent");
    }
}
