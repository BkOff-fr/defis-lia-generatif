//! Orchestre toutes les sources du pipeline médaillon — voir ADR-0009.
//!
//! Le registry exécute séquentiellement Copper → Silver → Gold pour
//! l'ensemble des sources enregistrées. Les erreurs sont **collectées**,
//! pas propagées immédiatement : si une source échoue, les autres
//! continuent. Le [`PipelineReport`] final permet de diagnostiquer.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use tracing::{info, warn};

use crate::{
    context::Context,
    error::IngestResult,
    gold::{assemble_gold, GoldArtifacts},
    layer::{
        CopperSnapshot, DataLayer, GoldContribution, HealthReport, SilverEntity,
    },
    lineage::{GoldLineage, SilverLineage},
};

/// Résultat d'une étape pour une source donnée.
#[derive(Debug, Clone)]
pub struct StepResult<T> {
    /// Identifiant de la source concernée.
    pub source_id: String,
    /// Résultat de l'étape.
    pub result: Result<T, String>,
}

impl<T> StepResult<T> {
    /// Helper constructeur succès.
    pub fn ok(source_id: impl Into<String>, value: T) -> Self {
        Self { source_id: source_id.into(), result: Ok(value) }
    }

    /// Helper constructeur échec.
    pub fn err(source_id: impl Into<String>, error: impl ToString) -> Self {
        Self { source_id: source_id.into(), result: Err(error.to_string()) }
    }

    /// `true` si l'étape a réussi.
    pub fn is_ok(&self) -> bool {
        self.result.is_ok()
    }
}

/// Rapport complet d'une exécution de pipeline.
#[derive(Debug, Clone)]
pub struct PipelineReport {
    /// Début de l'exécution.
    pub started_at: DateTime<Utc>,
    /// Fin de l'exécution.
    pub finished_at: DateTime<Utc>,
    /// Résultat Copper par source.
    pub copper: Vec<StepResult<CopperSnapshot>>,
    /// Résultat Silver par source.
    pub silver: Vec<StepResult<Vec<SilverEntity>>>,
    /// Contributions Gold par source.
    pub gold_contributions: Vec<StepResult<GoldContribution>>,
    /// Lignée Gold finale.
    pub gold_lineage: GoldLineage,
    /// Artefacts Gold produits (chemins sur disque), si l'assemblage a réussi.
    pub gold_artifacts: Option<GoldArtifacts>,
}

impl PipelineReport {
    /// Nombre de sources ayant réussi toute la chaîne.
    #[must_use]
    pub fn fully_successful_count(&self) -> usize {
        let copper_ok: std::collections::BTreeSet<&str> =
            self.copper.iter().filter(|r| r.is_ok()).map(|r| r.source_id.as_str()).collect();
        let silver_ok: std::collections::BTreeSet<&str> =
            self.silver.iter().filter(|r| r.is_ok()).map(|r| r.source_id.as_str()).collect();
        let gold_ok: std::collections::BTreeSet<&str> = self
            .gold_contributions
            .iter()
            .filter(|r| r.is_ok())
            .map(|r| r.source_id.as_str())
            .collect();
        copper_ok.intersection(&silver_ok).filter(|s| gold_ok.contains(*s)).count()
    }

    /// Liste des sources ayant échoué à au moins une étape.
    #[must_use]
    pub fn failed_sources(&self) -> Vec<String> {
        let mut failed = std::collections::BTreeSet::new();
        for r in &self.copper {
            if r.result.is_err() {
                failed.insert(r.source_id.clone());
            }
        }
        for r in &self.silver {
            if r.result.is_err() {
                failed.insert(r.source_id.clone());
            }
        }
        for r in &self.gold_contributions {
            if r.result.is_err() {
                failed.insert(r.source_id.clone());
            }
        }
        failed.into_iter().collect()
    }
}

/// Liste de toutes les sources connues.
pub struct LayerRegistry {
    sources: Vec<Arc<dyn DataLayer>>,
}

impl LayerRegistry {
    /// Crée un registre vide.
    #[must_use]
    pub fn new() -> Self {
        Self { sources: Vec::new() }
    }

    /// Construit le registre standard.
    /// TODO(sobria-003) : instancier ComparIA, RTE IRIS, ADEME, ...
    #[must_use]
    pub fn standard() -> Self {
        Self::new()
    }

    /// Enregistre une source.
    pub fn register(&mut self, source: Arc<dyn DataLayer>) {
        self.sources.push(source);
    }

    /// Itère sur les sources enregistrées.
    pub fn sources(&self) -> impl Iterator<Item = &Arc<dyn DataLayer>> {
        self.sources.iter()
    }

    /// Nombre de sources.
    #[must_use]
    pub fn len(&self) -> usize {
        self.sources.len()
    }

    /// `true` si aucune source.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }

    /// Health check séquentiel de toutes les sources.
    pub async fn health_check_all(&self, ctx: &Context) -> Vec<StepResult<HealthReport>> {
        let mut out = Vec::with_capacity(self.sources.len());
        for s in &self.sources {
            let id = s.id();
            match s.health_check(ctx).await {
                Ok(r) => out.push(StepResult::ok(id, r)),
                Err(e) => out.push(StepResult::err(id, e)),
            }
        }
        out
    }

    /// Étape 1 — ingestion brute pour toutes les sources.
    pub async fn run_copper(&self, ctx: &Context) -> Vec<StepResult<CopperSnapshot>> {
        let mut out = Vec::with_capacity(self.sources.len());
        for s in &self.sources {
            let id = s.id();
            info!(source = id, "copper: ingestion");
            match s.ingest_copper(ctx).await {
                Ok(snap) => {
                    info!(source = id, files = snap.files.len(), "copper: ok");
                    out.push(StepResult::ok(id, snap));
                },
                Err(e) => {
                    warn!(source = id, error = %e, "copper: échec");
                    out.push(StepResult::err(id, e));
                },
            }
        }
        out
    }

    /// Étape 2 — promotion Silver pour toutes les sources ayant un Copper OK.
    pub async fn run_silver(
        &self,
        ctx: &Context,
        copper: &[StepResult<CopperSnapshot>],
    ) -> Vec<StepResult<Vec<SilverEntity>>> {
        let mut out = Vec::with_capacity(copper.len());
        for c in copper {
            let id = c.source_id.clone();
            let Some(source) = self.find(&id) else {
                out.push(StepResult::err(&id, "source non enregistrée"));
                continue;
            };
            let Ok(ref snap) = c.result else {
                out.push(StepResult::err(&id, "copper échoué, silver impossible"));
                continue;
            };
            info!(source = %id, "silver: promotion");
            match source.promote_silver(snap, ctx).await {
                Ok(entities) => {
                    info!(source = %id, entities = entities.len(), "silver: ok");
                    out.push(StepResult::ok(&id, entities));
                },
                Err(e) => {
                    warn!(source = %id, error = %e, "silver: échec");
                    out.push(StepResult::err(&id, e));
                },
            }
        }
        out
    }

    /// Étape 3 — contributions Gold pour toutes les sources.
    pub async fn run_gold(
        &self,
        ctx: &Context,
        silver: &[StepResult<Vec<SilverEntity>>],
    ) -> (Vec<StepResult<GoldContribution>>, GoldLineage) {
        let mut contribs = Vec::with_capacity(silver.len());
        let mut lineage = GoldLineage::empty();
        lineage.add_artifact("referentiel.sqlite");
        lineage.add_artifact("analytics.parquet");
        lineage.add_artifact("datasheet.jsonld");

        for s in silver {
            let id = s.source_id.clone();
            let Some(source) = self.find(&id) else {
                contribs.push(StepResult::err(&id, "source non enregistrée"));
                continue;
            };
            let Ok(ref entities) = s.result else {
                contribs.push(StepResult::err(&id, "silver échoué, gold impossible"));
                continue;
            };
            for entity in entities {
                lineage.add_silver(SilverLineage {
                    entity: entity.name.clone(),
                    schema_version: entity.schema_version.clone(),
                    silver_path: entity.path.clone(),
                    copper_refs: entity.copper_refs.clone(),
                    row_count: entity.row_count,
                    written_at: Utc::now(),
                });
            }
            info!(source = %id, "gold: contribution");
            match source.contribute_gold(entities, ctx).await {
                Ok(c) => contribs.push(StepResult::ok(&id, c)),
                Err(e) => {
                    warn!(source = %id, error = %e, "gold: échec");
                    contribs.push(StepResult::err(&id, e));
                },
            }
        }
        (contribs, lineage)
    }

    /// Pipeline complet Copper → Silver → Gold + assemblage des artefacts.
    pub async fn run_full_pipeline(&self, ctx: &Context) -> IngestResult<PipelineReport> {
        let started_at = Utc::now();
        info!(source_count = self.sources.len(), "pipeline médaillon : démarrage");

        let copper = self.run_copper(ctx).await;
        let silver = self.run_silver(ctx, &copper).await;
        let (gold_contributions, gold_lineage) = self.run_gold(ctx, &silver).await;

        // Assemblage Gold : produit referentiel.sqlite, analytics.parquet,
        // datasheet.jsonld, MANIFEST.sha256. Voir chantier C04.
        let sources_meta: Vec<crate::layer::SourceMeta> =
            self.sources.iter().map(|s| s.meta()).collect();
        let gold_artifacts =
            match assemble_gold(ctx, &silver, &sources_meta, &gold_lineage).await {
                Ok(artifacts) => {
                    info!("gold: assemblage terminé");
                    Some(artifacts)
                },
                Err(e) => {
                    warn!(error = %e, "gold: assemblage échoué");
                    None
                },
            };

        let finished_at = Utc::now();
        let report = PipelineReport {
            started_at,
            finished_at,
            copper,
            silver,
            gold_contributions,
            gold_lineage,
            gold_artifacts,
        };
        info!(
            succeeded = report.fully_successful_count(),
            failed = report.failed_sources().len(),
            duration_ms = (finished_at - started_at).num_milliseconds(),
            "pipeline médaillon : terminé"
        );
        Ok(report)
    }

    /// Cherche une source par son identifiant.
    fn find(&self, id: &str) -> Option<&Arc<dyn DataLayer>> {
        self.sources.iter().find(|s| s.id() == id)
    }
}

impl Default for LayerRegistry {
    fn default() -> Self {
        Self::standard()
    }
}

#[cfg(test)]
mod tests {
    fn temp_ctx() -> (tempfile::TempDir, Context) {
        let tmp = tempfile::tempdir().expect("tempdir");
        let ctx = Context {
            data_root: tmp.path().to_path_buf(),
            incremental: false,
            seed: 42,
        };
        (tmp, ctx)
    }

    use std::path::PathBuf;

    use async_trait::async_trait;

    use super::*;
    use crate::{
        error::{IngestError, IngestResult},
        layer::{CopperSnapshot, GoldContribution, SilverEntity, SourceMeta},
        lineage::CopperRef,
    };

    /// Source de test : OK à toutes les étapes.
    struct OkSource {
        id_static: &'static str,
    }

    #[async_trait]
    impl DataLayer for OkSource {
        fn id(&self) -> &'static str {
            self.id_static
        }
        fn meta(&self) -> SourceMeta {
            SourceMeta {
                id: self.id_static.into(),
                name: format!("Test source {}", self.id_static),
                url: "https://example.test".into(),
                license: "MIT".into(),
                update_frequency: "test".into(),
                tier: 1,
            }
        }
        async fn ingest_copper(&self, _ctx: &Context) -> IngestResult<CopperSnapshot> {
            Ok(CopperSnapshot {
                source_id: self.id_static.into(),
                fetched_at: Utc::now(),
                path: PathBuf::from(format!("copper/{}", self.id_static)),
                files: vec![CopperRef {
                    source_id: self.id_static.into(),
                    manifest_path: PathBuf::from("manifest.json"),
                    file_name: "file.parquet".into(),
                    file_sha256: "a".repeat(64),
                }],
                license: "MIT".into(),
            })
        }
        async fn promote_silver(
            &self,
            snap: &CopperSnapshot,
            _ctx: &Context,
        ) -> IngestResult<Vec<SilverEntity>> {
            Ok(vec![SilverEntity {
                name: format!("{}_entity", self.id_static),
                path: PathBuf::from("silver/x.parquet"),
                schema_version: "v1".into(),
                copper_refs: snap.files.clone(),
                row_count: 100,
            }])
        }
        async fn contribute_gold(
            &self,
            silver: &[SilverEntity],
            _ctx: &Context,
        ) -> IngestResult<GoldContribution> {
            Ok(GoldContribution {
                source_id: self.id_static.into(),
                tables_touched: silver.iter().map(|e| e.name.clone()).collect(),
                notes: vec![format!("contribution de {}", self.id_static)],
            })
        }
    }

    /// Source de test : échoue à l'ingestion Copper.
    struct CopperFailSource;

    #[async_trait]
    impl DataLayer for CopperFailSource {
        fn id(&self) -> &'static str {
            "copper-fail"
        }
        fn meta(&self) -> SourceMeta {
            SourceMeta {
                id: "copper-fail".into(),
                name: "Test fail source".into(),
                url: "https://example.test".into(),
                license: "MIT".into(),
                update_frequency: "test".into(),
                tier: 2,
            }
        }
        async fn ingest_copper(&self, _ctx: &Context) -> IngestResult<CopperSnapshot> {
            Err(IngestError::Other("copper ko volontaire".into()))
        }
        async fn promote_silver(
            &self,
            _: &CopperSnapshot,
            _: &Context,
        ) -> IngestResult<Vec<SilverEntity>> {
            unreachable!("ne devrait pas être appelé")
        }
        async fn contribute_gold(
            &self,
            _: &[SilverEntity],
            _: &Context,
        ) -> IngestResult<GoldContribution> {
            unreachable!()
        }
    }

    #[tokio::test]
    async fn empty_registry_runs_cleanly() {
        let reg = LayerRegistry::new();
        let (_tmp, ctx) = temp_ctx();
        let report = reg.run_full_pipeline(&ctx).await.unwrap();
        assert!(report.copper.is_empty());
        assert!(report.silver.is_empty());
        assert_eq!(report.fully_successful_count(), 0);
        assert!(!report.gold_lineage.gold_artifacts.is_empty());
    }

    #[tokio::test]
    async fn pipeline_runs_all_steps_for_ok_source() {
        let mut reg = LayerRegistry::new();
        reg.register(Arc::new(OkSource { id_static: "src-a" }));
        let (_tmp, ctx) = temp_ctx();
        let report = reg.run_full_pipeline(&ctx).await.unwrap();

        assert_eq!(report.copper.len(), 1);
        assert_eq!(report.silver.len(), 1);
        assert_eq!(report.gold_contributions.len(), 1);
        assert_eq!(report.fully_successful_count(), 1);
        assert!(report.failed_sources().is_empty());

        let lineage_sources: Vec<&str> = report.gold_lineage.source_ids();
        assert!(lineage_sources.contains(&"src-a"));
    }

    #[tokio::test]
    async fn one_failed_source_doesnt_stop_others() {
        let mut reg = LayerRegistry::new();
        reg.register(Arc::new(OkSource { id_static: "ok-1" }));
        reg.register(Arc::new(CopperFailSource));
        reg.register(Arc::new(OkSource { id_static: "ok-2" }));

        let (_tmp, ctx) = temp_ctx();
        let report = reg.run_full_pipeline(&ctx).await.unwrap();

        assert_eq!(report.copper.len(), 3);
        let ok_count = report.copper.iter().filter(|r| r.is_ok()).count();
        assert_eq!(ok_count, 2);

        let silver_ok = report.silver.iter().filter(|r| r.is_ok()).count();
        assert_eq!(silver_ok, 2);

        let failed = report.failed_sources();
        assert!(failed.contains(&"copper-fail".to_string()));
        assert_eq!(failed.len(), 1);
    }

    #[tokio::test]
    async fn health_check_default_ok() {
        let mut reg = LayerRegistry::new();
        reg.register(Arc::new(OkSource { id_static: "src" }));
        let (_tmp, ctx) = temp_ctx();
        let reports = reg.health_check_all(&ctx).await;
        assert_eq!(reports.len(), 1);
        let r = reports[0].result.as_ref().unwrap();
        assert!(r.ok);
    }
}
