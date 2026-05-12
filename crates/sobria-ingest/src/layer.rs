//! Trait fondateur du pipeline médaillon — voir ADR-0009.
//!
//! Chaque source du référentiel implémente `DataLayer` et est enregistrée
//! dans le [`LayerRegistry`](crate::registry::LayerRegistry).

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::context::Context;

/// Métadonnées descriptives d'une source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMeta {
    /// Identifiant stable et unique.
    pub id: &'static str,
    /// Nom humainement lisible.
    pub name: String,
    /// URL principale documentaire.
    pub url: String,
    /// Licence des données.
    pub license: String,
    /// Fréquence amont de mise à jour (ex: `"trimestrielle"`).
    pub update_frequency: String,
    /// Tier (1, 2 ou 3) — voir `docs/sources/CATALOGUE-SOURCES.md`.
    pub tier: u8,
}

/// Référence à un snapshot Copper figé.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopperSnapshot {
    /// Source d'origine.
    pub source_id: String,
    /// Horodatage UTC de la récupération.
    pub fetched_at: DateTime<Utc>,
    /// Chemin sur disque (relatif à `data/copper/<source>/<YYYY-MM-DD>/`).
    pub path: String,
    /// Hash SHA-256 du contenu brut, signé dans le manifest.
    pub sha256: String,
    /// Licence détectée à l'ingestion.
    pub license: String,
    /// URL d'origine.
    pub source_url: String,
}

/// Une entité Silver écrite en Parquet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SilverEntity {
    /// Nom de l'entité (ex: `"comparia_conversations"`).
    pub name: String,
    /// Chemin du Parquet écrit (relatif à `data/silver/<source>/`).
    pub path: String,
    /// Version du schéma utilisé (ex: `"v1"`).
    pub schema_version: String,
    /// Lignée — hashes Copper d'origine.
    pub copper_hashes: Vec<String>,
    /// Nombre de lignes écrites (sanity check).
    pub row_count: u64,
}

/// Contribution d'une source au Gold final.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldContribution {
    /// Source contributrice.
    pub source_id: String,
    /// Tables Gold enrichies par cette source.
    pub tables_touched: Vec<String>,
    /// Notes méthodologiques pour la datasheet finale.
    pub notes: Vec<String>,
}

/// Trait unique — voir ADR-0009 §"Implémentation automatique".
#[async_trait]
pub trait DataLayer: Send + Sync {
    /// Identifiant stable et unique.
    fn id(&self) -> &'static str;

    /// Métadonnées.
    fn meta(&self) -> SourceMeta;

    /// Étape 1 — récupération brute → Copper.
    ///
    /// Doit être idempotente : un même contenu en amont doit produire le même hash.
    async fn ingest_copper(&self, ctx: &Context) -> anyhow::Result<CopperSnapshot>;

    /// Étape 2 — promotion vers Silver (Parquet typé, validé).
    async fn promote_silver(
        &self,
        snapshot: &CopperSnapshot,
        ctx: &Context,
    ) -> anyhow::Result<Vec<SilverEntity>>;

    /// Étape 3 — contribution à Gold (jointures, agrégations).
    async fn contribute_gold(
        &self,
        silver: &[SilverEntity],
        ctx: &Context,
    ) -> anyhow::Result<GoldContribution>;
}
