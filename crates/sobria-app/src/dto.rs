//! Data Transfer Objects entre Rust (Tauri) et le frontend SvelteKit.
//!
//! Conventions :
//! - Tous les DTO sont `serde::Serialize` (sortie IPC).
//! - Les DTO d'entrée (`*Dto` côté requête) sont aussi `Deserialize`.
//! - Pas de `Option<T>` côté entrée sauf vrai cas optionnel UI (ex: `datacenter_id`).
//! - Les timestamps sont des `String` RFC 3339 (interopérable JavaScript).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sobria_audit::{AuditEntry, IntegrityReport};
use sobria_core::{
    DistributionBins, Equivalent, EstimationRequest, EstimationResult, Hypothesis, Indicator,
    IndicatorValue, ModuleId, Persona,
};
use sobria_estimator::{
    CalibrationStatus, ForecastConfig, ForecastResult, ModelPreset, Openness, ParamOverrides,
    Scenario, ScenarioOutcome, SimulationRequest, SimulationResult, VendorDisclosure, VendorScope,
    VendorUnit, VisionPricing, YearlyForecastRequest, YearlyForecastResult, YearlyScenario,
    YearlyScenarioOutcome,
};
use sobria_geoloc::{
    CountryAggregate, DatacenterRecord, IndustrialSite, IndustrialSiteSummary, RegionFrAggregate,
    SankeyData, SankeyLink, SankeyNode,
};

// ─────────────────────────────────────────────────────────────────────────────
// meta_info
// ─────────────────────────────────────────────────────────────────────────────

/// Informations runtime de l'app (footer + diagnostic).
#[derive(Debug, Clone, Serialize)]
pub struct MetaInfo {
    /// Version sémantique du package `sobria-app`.
    pub app_version: String,
    /// Seed Monte-Carlo configuré (déterminisme).
    pub estimator_seed: u64,
    /// N tirages Monte-Carlo (défaut 10⁴).
    pub estimator_n: u32,
    /// Chemin du ledger SQLite (info seule — pas de FS access).
    pub audit_path: String,
    /// Racine de données utilisateur.
    pub data_root: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// referentiel (C26.5 — accès au Gold)
// ─────────────────────────────────────────────────────────────────────────────

/// Statut compact du référentiel Gold pour la page Paramètres.
/// Mirroir TS dans `web/src/lib/api.ts::ReferentielStatusDto`.
#[derive(Debug, Clone, Serialize)]
pub struct ReferentielStatusDto {
    /// `true` si le référentiel a pu être ouvert ; sinon les autres champs
    /// peuvent être vides.
    pub available: bool,
    /// Message court lisible humain (raison de l'indisponibilité, ou OK).
    pub message: String,
    /// Version sémantique du référentiel (héritée de `sobria-ingest`).
    pub version: String,
    /// Date de dernière modification du fichier (RFC 3339).
    pub snapshot_date: String,
    /// SHA-256 hexadécimal du SQLite (intégrité).
    pub sha256: String,
    /// Nombre de sources contributrices.
    pub source_count: u64,
    /// Nombre de modèles distincts dans `model_overview`.
    pub model_count: u64,
    /// Chemin du SQLite consulté (info seule).
    pub path: String,
}

/// Résultat d'une demande explicite de rechargement (`reload_referentiel`).
/// Mirroir TS dans `web/src/lib/api.ts::ReferentielReloadResultDto`.
#[derive(Debug, Clone, Serialize)]
pub struct ReferentielReloadResultDto {
    /// `true` si `dvc pull` a réussi ET le référentiel est ouvert.
    pub success: bool,
    /// Message d'aide / d'erreur lisible humain.
    pub message: String,
    /// Stdout/stderr concaténés de `dvc pull` (debug, max ~4 ko).
    pub dvc_output: String,
    /// Statut résultant (None si reload a échoué avant ouverture).
    pub status: Option<ReferentielStatusDto>,
}

// ─────────────────────────────────────────────────────────────────────────────
// list_models
// ─────────────────────────────────────────────────────────────────────────────

/// Preset modèle envoyé au frontend (déclinaison statique → owned strings).
///
/// **C34.4** — 3 bools de capabilities (vision/audio/reasoning) + `deprecated` :
/// allow `struct_excessive_bools` car chaque flag a une sémantique distincte
/// et indépendante (cf. doc équivalente sur `ModelPreset`).
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize)]
pub struct ModelPresetDto {
    pub id: String,
    pub display_name: String,
    pub provider: String,
    pub family: String,
    pub approx_params_billions: f64,
    pub openness: String,
    pub calibration: String,
    pub sources: Vec<String>,
    /// **C34.4** — Date de sortie publique (ISO `YYYY-MM-DD`).
    pub release_date: String,
    /// **C34.4** — Paramètres actifs (en milliards). `= approx_params_billions`
    /// pour dense, < pour MoE.
    pub active_params_b: f64,
    /// **C34.4** — Famille typée du fabricant (snake_case).
    pub model_family: String,
    /// **C34.4** — Architecture (`dense_transformer`, `moe`, `mamba`, `hybrid`).
    pub architecture: String,
    /// **C34.4** — Accepte des images en entrée.
    pub vision_capable: bool,
    /// **C34.4** — Accepte de l'audio en entrée.
    pub audio_capable: bool,
    /// **C34.4** — Reasoning model (extended thinking / chain-of-thought intégré).
    pub reasoning_capable: bool,
    /// **C34.4** — `(P5, P95)` du ratio thinking/output tokens. `None` si
    /// pas reasoning_capable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_token_multiplier: Option<(f64, f64)>,
    /// **C34.4** — Overhead système typique (interface app vendor).
    pub default_context_overhead_tokens: u32,
    /// **C34.4** — `true` pour les modèles obsolètes (à filtrer par défaut UI).
    pub deprecated: bool,
    /// **C34.4** — URL canonique de la source vendor (model card).
    pub source_url: String,
}

impl From<&ModelPreset> for ModelPresetDto {
    fn from(p: &ModelPreset) -> Self {
        use sobria_estimator::{ArchitectureKind, ModelFamily};
        Self {
            id: p.id.to_string(),
            display_name: p.display_name.to_string(),
            provider: p.provider.to_string(),
            family: p.family.to_string(),
            approx_params_billions: p.approx_params_billions,
            openness: match p.openness {
                Openness::Open => "open",
                Openness::OpenWeights => "open_weights",
                Openness::Closed => "closed",
            }
            .into(),
            calibration: match p.calibration {
                CalibrationStatus::Validated => "validated",
                CalibrationStatus::Indicative => "indicative",
                CalibrationStatus::Extrapolated => "extrapolated",
            }
            .into(),
            sources: p.sources.iter().map(|s| (*s).to_string()).collect(),
            release_date: p.release_date.to_string(),
            active_params_b: p.active_params_b,
            model_family: match p.model_family {
                ModelFamily::Anthropic => "anthropic",
                ModelFamily::OpenAi => "open_ai",
                ModelFamily::GoogleDeepMind => "google_deep_mind",
                ModelFamily::MetaAi => "meta_ai",
                ModelFamily::MistralAi => "mistral_ai",
                ModelFamily::DeepSeek => "deep_seek",
                ModelFamily::Xai => "xai",
                ModelFamily::Alibaba => "alibaba",
                ModelFamily::Microsoft => "microsoft",
                ModelFamily::Other => "other",
            }
            .into(),
            architecture: match p.architecture {
                ArchitectureKind::DenseTransformer => "dense_transformer",
                ArchitectureKind::Moe { .. } => "moe",
                ArchitectureKind::Mamba => "mamba",
                ArchitectureKind::Hybrid => "hybrid",
            }
            .into(),
            vision_capable: p.vision_capable,
            audio_capable: p.audio_capable,
            reasoning_capable: p.reasoning_capable,
            thinking_token_multiplier: p.thinking_token_multiplier,
            default_context_overhead_tokens: p.default_context_overhead_tokens,
            deprecated: p.deprecated,
            source_url: p.source_url.to_string(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// estimate_prompt
// ─────────────────────────────────────────────────────────────────────────────

/// Payload de demande d'estimation envoyé par le frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstimationRequestDto {
    pub model_id: String,
    pub tokens_in: u32,
    pub tokens_out_estimated: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub datacenter_id: Option<String>,
    /// Méthodologie utilisée pour ce calcul (C24).
    ///
    /// `None` → fallback sur la méthodologie par défaut de l'utilisateur
    /// (`AppPreferencesDto::default_method`), ou `EmpreinteMethod::default_method()`
    /// si aucune préférence n'est encore enregistrée.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<sobria_core::EmpreinteMethod>,
    /// **C34.3** — Modalités d'input du prompt (texte, vision, document,
    /// audio). `None`/absent → uniquement Text (compatible v0.8.x).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modalities: Option<Vec<sobria_core::InputModality>>,
    /// **C34.3** — Overhead système (system prompt + tools + memory +
    /// thinking). `None`/absent → zéros (compatible v0.8.x).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overhead: Option<sobria_core::ContextOverhead>,
}

impl EstimationRequestDto {
    /// Convertit en `sobria_core::EstimationRequest` (ajoute le timestamp).
    #[must_use]
    pub fn into_core(self, timestamp: DateTime<Utc>) -> EstimationRequest {
        EstimationRequest {
            model_id: self.model_id,
            tokens_in: self.tokens_in,
            tokens_out_estimated: self.tokens_out_estimated,
            datacenter_id: self.datacenter_id,
            timestamp,
            modalities: self.modalities.unwrap_or_default(),
            overhead: self.overhead.unwrap_or_default(),
        }
    }
}

/// Intervalle d'un indicateur (P5-P50-P95) + histogramme distributionnel
/// optionnel.
#[derive(Debug, Clone, Serialize)]
pub struct IndicatorDto {
    /// Nom de l'indicateur : `co2eq`, `energy`, `water`.
    pub indicator: String,
    pub p5: f64,
    pub p50: f64,
    pub p95: f64,
    pub unit: String,
    /// Histogramme Monte-Carlo (équi-width). `None` pour les entrées
    /// d'audit antérieures à v0.2 ou les estimations à N trop petit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bins: Option<DistributionBins>,
}

impl From<&IndicatorValue> for IndicatorDto {
    fn from(v: &IndicatorValue) -> Self {
        Self {
            indicator: match v.indicator {
                Indicator::Co2Eq => "co2eq",
                Indicator::Energy => "energy",
                Indicator::Water => "water",
                Indicator::CriticalMetals => "critical_metals",
                Indicator::Cost => "cost",
            }
            .into(),
            p5: v.interval.p5,
            p50: v.interval.p50,
            p95: v.interval.p95,
            unit: v.unit.clone(),
            bins: v.bins.clone(),
        }
    }
}

/// Équivalent parlant (UI).
#[derive(Debug, Clone, Serialize)]
pub struct EquivalentDto {
    pub label: String,
    pub value: f64,
    pub source: String,
}

impl From<&Equivalent> for EquivalentDto {
    fn from(e: &Equivalent) -> Self {
        Self {
            label: e.label.clone(),
            value: e.value,
            source: e.source.clone(),
        }
    }
}

/// Hypothèse cliquable (UI).
#[derive(Debug, Clone, Serialize)]
pub struct HypothesisDto {
    pub key: String,
    pub value: serde_json::Value,
    pub source: String,
}

impl From<&Hypothesis> for HypothesisDto {
    fn from(h: &Hypothesis) -> Self {
        Self {
            key: h.key.clone(),
            value: h.value.clone(),
            source: h.source.clone(),
        }
    }
}

/// Résultat d'estimation complet renvoyé au frontend.
#[derive(Debug, Clone, Serialize)]
pub struct EstimationResultDto {
    /// Méthodologie utilisée pour produire ce résultat (C24).
    /// Affiché en badge dans `ResultBlock.svelte` + dans le journal d'audit.
    pub method: sobria_core::EmpreinteMethod,
    pub request: EstimationRequestEchoDto,
    pub indicators: Vec<IndicatorDto>,
    pub equivalents: Vec<EquivalentDto>,
    pub hypotheses: Vec<HypothesisDto>,
    pub computed_at: String,
    pub seed: u64,
    /// ID de l'entrée du ledger d'audit qui journalise ce résultat.
    /// `0` = estimation éphémère (non journalisée, cf. estimate_for_comparison).
    pub audit_id: i64,
}

/// Écho de la requête (avec timestamp ajouté côté serveur).
#[derive(Debug, Clone, Serialize)]
pub struct EstimationRequestEchoDto {
    pub model_id: String,
    pub tokens_in: u32,
    pub tokens_out_estimated: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datacenter_id: Option<String>,
    pub timestamp: String,
}

impl From<&EstimationRequest> for EstimationRequestEchoDto {
    fn from(r: &EstimationRequest) -> Self {
        Self {
            model_id: r.model_id.clone(),
            tokens_in: r.tokens_in,
            tokens_out_estimated: r.tokens_out_estimated,
            datacenter_id: r.datacenter_id.clone(),
            timestamp: r.timestamp.to_rfc3339(),
        }
    }
}

impl EstimationResultDto {
    /// Construit le DTO depuis un `EstimationResult` + l'`audit_id` du ledger.
    #[must_use]
    pub fn from_result(r: &EstimationResult, audit_id: i64) -> Self {
        Self {
            method: r.method,
            request: (&r.request).into(),
            indicators: r.indicators.iter().map(Into::into).collect(),
            equivalents: r.equivalents.iter().map(Into::into).collect(),
            hypotheses: r.hypotheses.iter().map(Into::into).collect(),
            computed_at: r.computed_at.to_rfc3339(),
            seed: r.seed,
            audit_id,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// audit
// ─────────────────────────────────────────────────────────────────────────────

/// Rapport d'intégrité de la chaîne d'audit.
#[derive(Debug, Clone, Serialize)]
pub struct IntegrityReportDto {
    pub total_entries: usize,
    pub valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_invalid_id: Option<i64>,
    pub message: String,
}

impl From<&IntegrityReport> for IntegrityReportDto {
    fn from(r: &IntegrityReport) -> Self {
        Self {
            total_entries: r.total_entries,
            valid: r.valid,
            first_invalid_id: r.first_invalid_id,
            message: r.message.clone(),
        }
    }
}

/// Résumé d'une entrée du ledger (pour la liste paginée).
#[derive(Debug, Clone, Serialize)]
pub struct AuditEntrySummaryDto {
    pub id: i64,
    pub timestamp: String,
    pub model_id: String,
    pub co2eq_p50: f64,
    pub sig_short: String,
    pub purged: bool,
    /// Méthodologie qui a produit cette entrée (C24).
    /// Extraite du payload JSON. Pour les entrées historiques pré-C24,
    /// vaut `AfnorSobria` (seul moteur disponible à l'époque) via le
    /// `#[serde(default)]` sur `EstimationResult.method`.
    pub method: sobria_core::EmpreinteMethod,
}

impl AuditEntrySummaryDto {
    /// Construit le résumé en extrayant `model_id` + `co2eq_p50` + `method`
    /// du payload. Si le payload est purgé ou mal formé, on remplit avec
    /// des sentinelles (méthode → AfnorSobria par défaut, cohérent avec
    /// les ledgers historiques).
    #[must_use]
    pub fn from_entry(e: &AuditEntry) -> Self {
        let parsed = parse_payload_full(&e.estimation_result_json);
        let sig_short = e.sig.chars().take(16).collect();
        Self {
            id: e.id,
            timestamp: e.timestamp.to_rfc3339(),
            model_id: parsed.model_id,
            co2eq_p50: parsed.co2eq_p50,
            sig_short,
            purged: e.is_purged(),
            method: parsed.method,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// territoire_fr (C13 — M20 Territoire FR)
// ─────────────────────────────────────────────────────────────────────────────

/// Résumé d'un site industriel pour la carte M20.
#[derive(Debug, Clone, Serialize)]
pub struct IndustrialSiteSummaryDto {
    pub code_iris: String,
    pub commune: String,
    pub department_code: String,
    pub region_iso: String,
    pub lat: f64,
    pub lon: f64,
    pub consumption_mwh_elec: f64,
    pub consumption_mwh_gas: f64,
    pub consumption_total_mwh: f64,
    pub pdl_total: u32,
    pub year: u32,
}

impl From<&IndustrialSite> for IndustrialSiteSummaryDto {
    fn from(s: &IndustrialSite) -> Self {
        Self {
            code_iris: s.code_iris.clone(),
            commune: s.commune.clone(),
            department_code: s.department_code.clone(),
            region_iso: s.region_iso.clone(),
            lat: s.lat,
            lon: s.lon,
            consumption_mwh_elec: s.consumption_mwh_elec,
            consumption_mwh_gas: s.consumption_mwh_gas_grtgaz + s.consumption_mwh_gas_terega,
            consumption_total_mwh: s.consumption_total_mwh,
            pdl_total: s.pdl_total,
            year: s.year,
        }
    }
}

/// Agrégat régional FR pour le drill-down.
#[derive(Debug, Clone, Serialize)]
pub struct RegionFrAggregateDto {
    pub region_iso: String,
    pub region_name: String,
    pub insee_code: String,
    pub site_count: usize,
    pub total_consumption_mwh_elec: f64,
    pub total_consumption_mwh_gas: f64,
    pub total_consumption_mwh: f64,
    pub centroid_lat: f64,
    pub centroid_lon: f64,
    pub nuclear_share_pct: f64,
    pub top_sites: Vec<TopSiteDto>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TopSiteDto {
    pub code_iris: String,
    pub commune: String,
    pub consumption_total_mwh: f64,
}

impl From<&IndustrialSiteSummary> for TopSiteDto {
    fn from(s: &IndustrialSiteSummary) -> Self {
        Self {
            code_iris: s.code_iris.clone(),
            commune: s.commune.clone(),
            consumption_total_mwh: s.consumption_total_mwh,
        }
    }
}

impl From<&RegionFrAggregate> for RegionFrAggregateDto {
    fn from(r: &RegionFrAggregate) -> Self {
        Self {
            region_iso: r.region_iso.clone(),
            region_name: r.region_name.clone(),
            insee_code: r.insee_code.clone(),
            site_count: r.site_count,
            total_consumption_mwh_elec: r.total_consumption_mwh_elec,
            total_consumption_mwh_gas: r.total_consumption_mwh_gas,
            total_consumption_mwh: r.total_consumption_mwh,
            centroid_lat: r.centroid_lat,
            centroid_lon: r.centroid_lon,
            nuclear_share_pct: r.nuclear_share_pct,
            top_sites: r.top_sites.iter().map(Into::into).collect(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// sankey_fr (C13)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct SankeyNodeDto {
    pub id: String,
    pub label: String,
    pub layer: u8,
    pub value_twh: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SankeyLinkDto {
    pub source: String,
    pub target: String,
    pub value_twh: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SankeyDataDto {
    pub nodes: Vec<SankeyNodeDto>,
    pub links: Vec<SankeyLinkDto>,
    pub total_production_twh: f64,
    pub year: u32,
    pub source_url: String,
    pub source_sha256: String,
}

impl From<&SankeyNode> for SankeyNodeDto {
    fn from(n: &SankeyNode) -> Self {
        Self {
            id: n.id.clone(),
            label: n.label.clone(),
            layer: n.layer,
            value_twh: n.value_twh,
        }
    }
}

impl From<&SankeyLink> for SankeyLinkDto {
    fn from(l: &SankeyLink) -> Self {
        Self {
            source: l.source.clone(),
            target: l.target.clone(),
            value_twh: l.value_twh,
        }
    }
}

impl From<&SankeyData> for SankeyDataDto {
    fn from(s: &SankeyData) -> Self {
        Self {
            nodes: s.nodes.iter().map(Into::into).collect(),
            links: s.links.iter().map(Into::into).collect(),
            total_production_twh: s.total_production_twh,
            year: s.year,
            source_url: s.source_url.clone(),
            source_sha256: s.source_sha256.clone(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// batch CSV (C21 — M18)
// ─────────────────────────────────────────────────────────────────────────────

/// Requête de traitement batch CSV.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequestDto {
    /// Chemin absolu vers le CSV d'entrée.
    pub input_csv_path: String,
    /// Si fourni : écrit un CSV de résultats à ce chemin.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_csv_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchAggregateDto {
    pub total_co2eq_g_p50: f64,
    pub total_energy_wh_p50: f64,
    pub total_water_l_p50: f64,
    pub avg_co2eq_g_p50: f64,
    pub min_co2eq_g_p50: f64,
    pub max_co2eq_g_p50: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchModelAggregateDto {
    pub model_id: String,
    pub count: u32,
    pub total_co2eq_g_p50: f64,
    pub avg_co2eq_g_p50: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchResultDto {
    pub rows_processed: u32,
    pub rows_rejected: u32,
    pub aggregate: BatchAggregateDto,
    pub by_model: Vec<BatchModelAggregateDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_csv_path: Option<String>,
    pub first_audit_id: i64,
    pub last_audit_id: i64,
}

// ─────────────────────────────────────────────────────────────────────────────
// projects + datasheet (C20 — M17 Empreinte projet)
// ─────────────────────────────────────────────────────────────────────────────

/// Représentation d'un projet vers le frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDto {
    pub id: i64,
    pub name: String,
    pub description: String,
    /// RFC 3339 UTC.
    pub period_start: String,
    pub period_end: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Payload de création.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectDto {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub period_start: String,
    pub period_end: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Payload de mise à jour partielle (dates non modifiables — cf. brief §1.1).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateProjectDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// Composition agrégée renvoyée avec le datasheet.
#[derive(Debug, Clone, Serialize)]
pub struct CompositionDto {
    pub total_requests: u32,
    pub unique_models: Vec<String>,
    pub total_co2eq_g_p50: f64,
    pub total_energy_wh_p50: f64,
    pub total_water_l_p50: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_first_entry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_last_entry: Option<String>,
    /// Polish G (C24) — Méthodologies utilisées pour les entrées de la
    /// période. Surfacé par l'UI M17 pour info chercheur.
    pub methodologies_used: Vec<sobria_core::EmpreinteMethod>,
}

impl From<&sobria_export::Composition> for CompositionDto {
    fn from(c: &sobria_export::Composition) -> Self {
        Self {
            total_requests: c.total_requests,
            unique_models: c.unique_models.clone(),
            total_co2eq_g_p50: c.total_co2eq_g_p50,
            total_energy_wh_p50: c.total_energy_wh_p50,
            total_water_l_p50: c.total_water_l_p50,
            date_first_entry: c.date_first_entry.map(|d| d.to_rfc3339()),
            date_last_entry: c.date_last_entry.map(|d| d.to_rfc3339()),
            methodologies_used: c.methodologies_used.clone(),
        }
    }
}

/// Datasheet Gebru produit pour un projet.
#[derive(Debug, Clone, Serialize)]
pub struct DatasheetDto {
    pub project: ProjectDto,
    pub jsonld: serde_json::Value,
    pub composition: CompositionDto,
    pub sha256: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// dashboard + eco-budget (C19 — M15 + M25)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct DashboardComparisonDto {
    pub previous_total_co2eq_g_p50: f64,
    pub delta_co2eq_pct: f64,
    pub previous_total_requests: u32,
    pub delta_requests_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TopModelDto {
    pub model_id: String,
    pub request_count: u32,
    pub total_co2eq_g_p50: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DailySeriesPointDto {
    pub date: String,
    pub request_count: u32,
    pub co2eq_g_p50: f64,
    pub energy_wh_p50: f64,
    pub water_l_p50: f64,
}

/// Total agrégé pour une méthodologie unique sur la période (Polish E, C24).
#[derive(Debug, Clone, Serialize)]
pub struct MethodTotalDto {
    pub method: sobria_core::EmpreinteMethod,
    pub request_count: u32,
    pub total_co2eq_g_p50: f64,
    pub total_energy_wh_p50: f64,
    pub total_water_l_p50: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardSummaryDto {
    pub period_label: String,
    pub period_start: String,
    pub period_end: String,
    pub total_requests: u32,
    pub total_co2eq_g_p50: f64,
    pub total_energy_wh_p50: f64,
    pub total_water_l_p50: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vs_previous: Option<DashboardComparisonDto>,
    pub top_models: Vec<TopModelDto>,
    pub daily_series: Vec<DailySeriesPointDto>,
    /// Polish E (C24) — Breakdown par méthodologie présente dans la période.
    pub method_breakdown: Vec<MethodTotalDto>,
    /// `true` si la période contient + d'une méthodologie. Le frontend
    /// affiche alors un warning : sommer 2 méthodos n'est pas scientifique.
    pub warning_multi_method: bool,
}

impl From<&crate::dashboard::DashboardSummary> for DashboardSummaryDto {
    fn from(s: &crate::dashboard::DashboardSummary) -> Self {
        Self {
            period_label: s.period_label.clone(),
            period_start: s.period_start.to_rfc3339(),
            period_end: s.period_end.to_rfc3339(),
            total_requests: s.total_requests,
            total_co2eq_g_p50: s.total_co2eq_g_p50,
            total_energy_wh_p50: s.total_energy_wh_p50,
            total_water_l_p50: s.total_water_l_p50,
            vs_previous: s.vs_previous.as_ref().map(|v| DashboardComparisonDto {
                previous_total_co2eq_g_p50: v.previous_total_co2eq_g_p50,
                delta_co2eq_pct: v.delta_co2eq_pct,
                previous_total_requests: v.previous_total_requests,
                delta_requests_pct: v.delta_requests_pct,
            }),
            top_models: s
                .top_models
                .iter()
                .map(|m| TopModelDto {
                    model_id: m.model_id.clone(),
                    request_count: m.request_count,
                    total_co2eq_g_p50: m.total_co2eq_g_p50,
                })
                .collect(),
            daily_series: s
                .daily_series
                .iter()
                .map(|p| DailySeriesPointDto {
                    date: p.date.clone(),
                    request_count: p.request_count,
                    co2eq_g_p50: p.co2eq_g_p50,
                    energy_wh_p50: p.energy_wh_p50,
                    water_l_p50: p.water_l_p50,
                })
                .collect(),
            method_breakdown: s
                .method_breakdown
                .iter()
                .map(|m| MethodTotalDto {
                    method: m.method,
                    request_count: m.request_count,
                    total_co2eq_g_p50: m.total_co2eq_g_p50,
                    total_energy_wh_p50: m.total_energy_wh_p50,
                    total_water_l_p50: m.total_water_l_p50,
                })
                .collect(),
            warning_multi_method: s.warning_multi_method,
        }
    }
}

/// Objectif personnel échangé avec le frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalGoalDto {
    /// "co2eq" | "energy" | "water"
    pub indicator: String,
    /// "daily" | "weekly" | "monthly"
    pub period: String,
    pub value_max: f64,
    pub unit: String,
}

/// Statut de consommation d'un objectif.
#[derive(Debug, Clone, Serialize)]
pub struct BudgetStatusDto {
    pub goal: PersonalGoalDto,
    pub current_value: f64,
    pub period_start: String,
    pub period_end: String,
    /// 0..100+ (peut dépasser).
    pub consumed_pct: f64,
    /// "ok" (<70%), "warning" (70-100%), "exceeded" (>100%)
    pub status: String,
    /// value_max - current_value (peut être < 0).
    pub remaining: f64,
}

// ─────────────────────────────────────────────────────────────────────────────
// model detail (C18 — M9 Référentiel modèles)
// ─────────────────────────────────────────────────────────────────────────────

/// Triplet P5/P50/P95 (équivalent à `UncertaintyInterval` sans la
/// validation côté DTO).
#[derive(Debug, Clone, Serialize)]
pub struct TripletDto {
    pub p5: f64,
    pub p50: f64,
    pub p95: f64,
}

/// Fiche détaillée d'un modèle exposant ses params distributionnels et
/// un baseline contextuel (gpt-4o-mini 100/500 tokens, paramètres par
/// défaut). **Pas journalisée** dans l'audit ledger.
///
/// **C34.5** — étendu avec les capabilities du modèle (vision, audio,
/// reasoning, MoE) pour alimenter la fiche M9.
///
/// 4 bools indépendants : `vision_capable`, `audio_capable`,
/// `reasoning_capable`, `deprecated` (cf. doc équivalente sur `ModelPreset`).
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize)]
pub struct ModelDetailDto {
    pub id: String,
    pub display_name: String,
    pub provider: String,
    pub family: String,
    pub approx_params_billions: f64,
    pub openness: String,
    pub calibration: String,
    pub sources: Vec<String>,
    /// Plage P5/P50/P95 de l'énergie prefill (mJ par token d'entrée).
    pub epsilon_prefill_mj_per_token: TripletDto,
    /// Plage P5/P50/P95 de l'énergie decode (mJ par token de sortie).
    pub epsilon_decode_mj_per_token: TripletDto,
    /// Plage P5/P50/P95 de l'embodied carbon amorti (gCO₂eq par requête).
    pub embodied_g_per_request: TripletDto,
    /// CO₂eq P5/P50/P95 pour le prompt de référence (100 in / 500 out).
    pub baseline_co2eq_p5_g: f64,
    pub baseline_co2eq_p50_g: f64,
    pub baseline_co2eq_p95_g: f64,
    pub baseline_energy_wh_p50: f64,
    pub baseline_water_l_p50: f64,
    /// **C32.4** — Disclosures officielles publiées par le fabricant.
    /// Vide pour les modèles dont le fabricant n'a pas publié (Anthropic,
    /// OpenAI au 2026-05).
    pub vendor_disclosures: Vec<VendorDisclosureDto>,
    /// **C34.5** — Date de sortie publique (ISO `YYYY-MM-DD`).
    pub release_date: String,
    /// **C34.5** — Paramètres actifs (= total pour dense, < pour MoE).
    pub active_params_b: f64,
    /// **C34.5** — Famille typée (snake_case).
    pub model_family: String,
    /// **C34.5** — Architecture (`dense_transformer` / `moe` / `mamba` /
    /// `hybrid`).
    pub architecture: String,
    /// **C34.5** — Si MoE, nombre d'experts (sinon `None`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moe_experts: Option<u32>,
    /// **C34.5** — Si MoE, nombre d'experts actifs par token (sinon `None`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moe_active_experts: Option<u32>,
    /// **C34.5** — Accepte des images en entrée.
    pub vision_capable: bool,
    /// **C34.5** — Tarification tokens vision (formule vendor). `None` si
    /// `vision_capable = false`. Sérialisé en JSON tagged (`{"kind": …}`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vision_pricing: Option<VisionPricing>,
    /// **C34.5** — Accepte de l'audio en entrée.
    pub audio_capable: bool,
    /// **C34.5** — Reasoning model intégré.
    pub reasoning_capable: bool,
    /// **C34.5** — `(P5, P95)` ratio thinking/output tokens. `None` si
    /// pas reasoning.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_token_multiplier: Option<(f64, f64)>,
    /// **C34.5** — Overhead système typique de l'interface app vendor.
    pub default_context_overhead_tokens: u32,
    /// **C34.5** — Modèle obsolète (filtrer par défaut UI).
    pub deprecated: bool,
    /// **C34.5** — URL canonique de la source vendor.
    pub source_url: String,
}

/// **C32.4** — Chiffre officiel publié par un fabricant (Mistral × ADEME,
/// Google Gemini, Meta Llama). Voir `sobria_estimator::VendorDisclosure`.
#[derive(Debug, Clone, Serialize)]
pub struct VendorDisclosureDto {
    pub vendor: String,
    /// `"training"` | `"inference_per_prompt"`.
    pub scope: String,
    pub value: f64,
    /// `"t_co2eq"` | `"g_co2eq"` | `"wh"` | `"ml_water"` | `"m3_water"`.
    pub unit: String,
    pub source_url: String,
    pub published_at: String,
    pub methodology_note: String,
}

/// **C32.4** — Ligne de la table comparaison vendor disclosure (M9 page
/// principale). Agrège les disclosures de tous les modèles d'un fabricant
/// donné.
#[derive(Debug, Clone, Serialize)]
pub struct VendorComparisonRowDto {
    /// Nom du fabricant (ex : `"Mistral AI"`, `"OpenAI"`).
    pub vendor: String,
    /// `true` si au moins un modèle du vendor publie une disclosure
    /// prompt-level (inference_per_prompt).
    pub has_prompt_level: bool,
    /// `true` si au moins un modèle du vendor publie une disclosure
    /// training.
    pub has_training: bool,
    /// Première source URL trouvée (si au moins une disclosure existe).
    /// `None` si aucun modèle du vendor n'a de disclosure.
    pub primary_source_url: Option<String>,
}

impl From<&VendorDisclosure> for VendorDisclosureDto {
    fn from(d: &VendorDisclosure) -> Self {
        Self {
            vendor: d.vendor.to_string(),
            scope: match d.scope {
                VendorScope::Training => "training",
                VendorScope::InferencePerPrompt => "inference_per_prompt",
            }
            .into(),
            value: d.value,
            unit: match d.unit {
                VendorUnit::TCo2Eq => "t_co2eq",
                VendorUnit::GCo2Eq => "g_co2eq",
                VendorUnit::Wh => "wh",
                VendorUnit::MlWater => "ml_water",
                VendorUnit::M3Water => "m3_water",
            }
            .into(),
            source_url: d.source_url.to_string(),
            published_at: d.published_at.to_string(),
            methodology_note: d.methodology_note.to_string(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// benchmark (C17 — M3 Comparer modèles)
// ─────────────────────────────────────────────────────────────────────────────

/// Requête de benchmark N modèles sur un même prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkRequestDto {
    /// 1..=20 model_ids à comparer.
    pub model_ids: Vec<String>,
    pub tokens_in: u32,
    pub tokens_out_estimated: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub datacenter_id: Option<String>,
}

/// Outcome d'un modèle benchmarké.
#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkOutcomeDto {
    pub model_id: String,
    pub display_name: String,
    pub provider: String,
    pub family: String,
    pub openness: String,
    pub calibration: String,
    pub result: EstimationResultDto,
    pub rank_co2eq: u32,
    pub rank_energy: u32,
    pub rank_water: u32,
}

/// Résultat global d'un benchmark.
#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkResultDto {
    pub outcomes: Vec<BenchmarkOutcomeDto>,
    pub ranking_by_co2eq_p50: Vec<String>,
    pub ranking_by_energy_p50: Vec<String>,
    pub ranking_by_water_p50: Vec<String>,
    pub tokens_in: u32,
    pub tokens_out_estimated: u32,
}

// ─────────────────────────────────────────────────────────────────────────────
// yearly forecast (C15 — M16 Forecaster 12 mois)
// ─────────────────────────────────────────────────────────────────────────────

/// Payload du forecast envoyé par le frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearlyForecastRequestDto {
    pub baseline: EstimationRequestDto,
    pub scenarios: Vec<YearlyScenarioDto>,
    pub months: u32,
    pub base_volume_per_day: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearlyScenarioDto {
    pub label: String,
    pub monthly_growth_pct: f64,
}

impl From<YearlyScenarioDto> for YearlyScenario {
    fn from(d: YearlyScenarioDto) -> Self {
        Self {
            label: d.label,
            monthly_growth_pct: d.monthly_growth_pct,
        }
    }
}

impl YearlyForecastRequestDto {
    /// Convertit en types internes (ajoute le timestamp baseline).
    #[must_use]
    pub fn into_core(self, baseline_timestamp: DateTime<Utc>) -> YearlyForecastRequest {
        YearlyForecastRequest {
            baseline: self.baseline.into_core(baseline_timestamp),
            scenarios: self.scenarios.into_iter().map(Into::into).collect(),
            months: self.months,
            base_volume_per_day: self.base_volume_per_day,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct YearlyScenarioOutcomeDto {
    pub label: String,
    pub monthly_growth_pct: f64,
    pub monthly_p5_g: Vec<f64>,
    pub monthly_p50_g: Vec<f64>,
    pub monthly_p95_g: Vec<f64>,
    pub cumulative_p5_g: Vec<f64>,
    pub cumulative_p50_g: Vec<f64>,
    pub cumulative_p95_g: Vec<f64>,
    pub annual_p5_g: f64,
    pub annual_p50_g: f64,
    pub annual_p95_g: f64,
}

impl From<&YearlyScenarioOutcome> for YearlyScenarioOutcomeDto {
    fn from(o: &YearlyScenarioOutcome) -> Self {
        Self {
            label: o.label.clone(),
            monthly_growth_pct: o.monthly_growth_pct,
            monthly_p5_g: o.monthly_p5_g.clone(),
            monthly_p50_g: o.monthly_p50_g.clone(),
            monthly_p95_g: o.monthly_p95_g.clone(),
            cumulative_p5_g: o.cumulative_p5_g.clone(),
            cumulative_p50_g: o.cumulative_p50_g.clone(),
            cumulative_p95_g: o.cumulative_p95_g.clone(),
            annual_p5_g: o.annual_p5_g,
            annual_p50_g: o.annual_p50_g,
            annual_p95_g: o.annual_p95_g,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct YearlyForecastResultDto {
    pub baseline_co2eq_p5_g: f64,
    pub baseline_co2eq_p50_g: f64,
    pub baseline_co2eq_p95_g: f64,
    /// `audit_id` de l'entrée journalisée pour la baseline (0 si non journalisé).
    pub baseline_audit_id: i64,
    pub scenarios: Vec<YearlyScenarioOutcomeDto>,
}

impl YearlyForecastResultDto {
    #[must_use]
    pub fn from_result(r: &YearlyForecastResult, baseline_audit_id: i64) -> Self {
        Self {
            baseline_co2eq_p5_g: r.baseline_co2eq_p5_g,
            baseline_co2eq_p50_g: r.baseline_co2eq_p50_g,
            baseline_co2eq_p95_g: r.baseline_co2eq_p95_g,
            baseline_audit_id,
            scenarios: r.scenarios.iter().map(Into::into).collect(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CSRD report (C14 — M22)
// ─────────────────────────────────────────────────────────────────────────────

/// Requête de génération de rapport CSRD/AGEC envoyée par le frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsrdReportRequestDto {
    /// ISO 8601 (`2026-01-01T00:00:00Z`).
    pub period_start: String,
    pub period_end: String,
    pub organization_name: String,
    /// Locale UI — v1.0 : `"fr"`.
    pub locale: String,
}

/// Réponse renvoyée après génération.
#[derive(Debug, Clone, Serialize)]
pub struct CsrdReportResultDto {
    pub pdf_path: String,
    pub provo_path: String,
    pub pdf_sha256: String,
    pub audit_entries_count: usize,
    pub total_requests: u32,
    pub total_co2eq_g_p50: f64,
    pub total_energy_wh_p50: f64,
    pub total_water_l_p50: f64,
}

// ─────────────────────────────────────────────────────────────────────────────
// simulation (C11 — M13 Simulateur « Et si...? »)
// ─────────────────────────────────────────────────────────────────────────────

/// Payload de simulation envoyé par le frontend.
///
/// Voir `briefs/chantiers/C11-simulateur-et-si.md`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationRequestDto {
    pub baseline: EstimationRequestDto,
    pub scenarios: Vec<ScenarioDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forecast: Option<ForecastConfigDto>,
}

/// Scénario envoyé par le frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioDto {
    pub label: String,
    #[serde(default)]
    pub overrides: ParamOverridesDto,
}

/// Overrides paramétriques optionnels. Tous les champs sont facultatifs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParamOverridesDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens_out: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pue: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub if_electrical_g_per_kwh: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embodied_g_per_request: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wue_l_per_kwh: Option<f64>,
}

impl From<ParamOverridesDto> for ParamOverrides {
    fn from(d: ParamOverridesDto) -> Self {
        Self {
            model_id: d.model_id,
            tokens_out: d.tokens_out,
            pue: d.pue,
            if_electrical_g_per_kwh: d.if_electrical_g_per_kwh,
            embodied_g_per_request: d.embodied_g_per_request,
            wue_l_per_kwh: d.wue_l_per_kwh,
        }
    }
}

impl From<ScenarioDto> for Scenario {
    fn from(d: ScenarioDto) -> Self {
        Self {
            label: d.label,
            overrides: d.overrides.into(),
        }
    }
}

/// Config de projection 12 mois.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastConfigDto {
    pub months: u32,
    pub monthly_growth_pct: f64,
    pub base_volume_per_day: f64,
}

impl From<ForecastConfigDto> for ForecastConfig {
    fn from(d: ForecastConfigDto) -> Self {
        Self {
            months: d.months,
            monthly_growth_pct: d.monthly_growth_pct,
            base_volume_per_day: d.base_volume_per_day,
        }
    }
}

impl SimulationRequestDto {
    /// Convertit en `SimulationRequest` interne en ajoutant le timestamp baseline.
    #[must_use]
    pub fn into_core(self, baseline_timestamp: DateTime<Utc>) -> SimulationRequest {
        SimulationRequest {
            baseline: self.baseline.into_core(baseline_timestamp),
            scenarios: self.scenarios.into_iter().map(Into::into).collect(),
            forecast: self.forecast.map(Into::into),
        }
    }
}

// ── Sortie ───────────────────────────────────────────────────────────────────

/// Résultat d'un scénario, prêt à afficher.
#[derive(Debug, Clone, Serialize)]
pub struct ScenarioOutcomeDto {
    pub label: String,
    pub result: EstimationResultDto,
    /// Δ par rapport au baseline P50, en gCO₂eq (peut être négatif).
    pub delta_co2eq_g: f64,
    /// Δ relatif en pourcentage du baseline P50.
    pub delta_pct: f64,
}

/// Résultat d'une projection 12 mois.
#[derive(Debug, Clone, Serialize)]
pub struct ForecastResultDto {
    pub months: u32,
    pub base_volume_per_day: f64,
    pub monthly_growth_pct: f64,
    pub baseline_monthly_co2eq_g: Vec<f64>,
    pub baseline_annual_co2eq_g: f64,
    pub scenarios_annual_co2eq_g: Vec<f64>,
}

impl From<ForecastResult> for ForecastResultDto {
    fn from(r: ForecastResult) -> Self {
        Self {
            months: r.months,
            base_volume_per_day: r.base_volume_per_day,
            monthly_growth_pct: r.monthly_growth_pct,
            baseline_monthly_co2eq_g: r.baseline_monthly_co2eq_g,
            baseline_annual_co2eq_g: r.baseline_annual_co2eq_g,
            scenarios_annual_co2eq_g: r.scenarios_annual_co2eq_g,
        }
    }
}

/// Résultat global d'une simulation.
#[derive(Debug, Clone, Serialize)]
pub struct SimulationResultDto {
    /// Estimation baseline (avec `audit_id` réel — journalisée).
    pub baseline: EstimationResultDto,
    /// Outcomes par scénario. `result.audit_id == 0` (non journalisé).
    pub scenarios: Vec<ScenarioOutcomeDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forecast: Option<ForecastResultDto>,
}

impl SimulationResultDto {
    /// Construit le DTO de sortie en passant l'`audit_id` de l'entrée baseline
    /// dans le ledger. Les scénarios reçoivent un `audit_id = 0` (non journalisés
    /// — voir brief C11 §3.audit).
    #[must_use]
    pub fn from_result(r: &SimulationResult, baseline_audit_id: i64) -> Self {
        let baseline = EstimationResultDto::from_result(&r.baseline, baseline_audit_id);
        let scenarios = r
            .scenarios
            .iter()
            .map(|o: &ScenarioOutcome| ScenarioOutcomeDto {
                label: o.label.clone(),
                result: EstimationResultDto::from_result(&o.result, 0),
                delta_co2eq_g: o.delta_co2eq_g,
                delta_pct: o.delta_pct,
            })
            .collect();
        Self {
            baseline,
            scenarios,
            forecast: r.forecast.clone().map(Into::into),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// datacenters (C12 — M12)
// ─────────────────────────────────────────────────────────────────────────────

/// Résumé d'un datacenter, suffisant pour placer un marker sur la carte.
#[derive(Debug, Clone, Serialize)]
pub struct DatacenterSummaryDto {
    pub id: String,
    pub name: String,
    pub operator: String,
    pub country_iso: String,
    pub city: String,
    pub lat: f64,
    pub lon: f64,
    pub pue: f64,
    pub if_electrical_g_per_kwh: f64,
}

impl From<&DatacenterRecord> for DatacenterSummaryDto {
    fn from(d: &DatacenterRecord) -> Self {
        Self {
            id: d.id.clone(),
            name: d.name.clone(),
            operator: d.operator.clone(),
            country_iso: d.country_iso.clone(),
            city: d.city.clone(),
            lat: d.lat,
            lon: d.lon,
            pue: d.pue,
            if_electrical_g_per_kwh: d.if_electrical_g_per_kwh,
        }
    }
}

/// Détail complet pour le drill-down (donut + barres + 24h).
#[derive(Debug, Clone, Serialize)]
pub struct DatacenterDetailDto {
    pub id: String,
    pub name: String,
    pub operator: String,
    pub country_iso: String,
    pub city: String,
    pub lat: f64,
    pub lon: f64,
    pub pue: f64,
    pub if_electrical_g_per_kwh: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wue_l_per_kwh: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity_mw: Option<f64>,
    pub sources: Vec<String>,
    pub hourly_profile_24h: Vec<f64>,
    /// CO₂eq P50 (gCO₂eq) pour un prompt de référence (gpt-4o-mini 100/500 tokens)
    /// avec les paramètres PUE/IF/WUE du DC. Permet à l'UI de remplir les
    /// "barres" sans nouvelle commande IPC.
    pub baseline_co2eq_p50_g: f64,
    /// Idem pour l'énergie (Wh) — médiane.
    pub baseline_energy_wh_p50: f64,
    /// Idem pour l'eau (L) — médiane.
    pub baseline_water_l_p50: f64,
}

/// Agrégat par pays pour la vue dézoomée Europe.
#[derive(Debug, Clone, Serialize)]
pub struct CountryAggregateDto {
    pub country_iso: String,
    pub datacenter_count: usize,
    pub avg_pue: f64,
    pub if_electrical_g_per_kwh: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_capacity_mw: Option<f64>,
    pub centroid_lat: f64,
    pub centroid_lon: f64,
}

impl From<&CountryAggregate> for CountryAggregateDto {
    fn from(c: &CountryAggregate) -> Self {
        Self {
            country_iso: c.country_iso.clone(),
            datacenter_count: c.datacenter_count,
            avg_pue: c.avg_pue,
            if_electrical_g_per_kwh: c.if_electrical_g_per_kwh,
            total_capacity_mw: c.total_capacity_mw,
            centroid_lat: c.centroid_lat,
            centroid_lon: c.centroid_lon,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// preferences (C10 — ADR-0010)
// ─────────────────────────────────────────────────────────────────────────────

/// Préférences utilisateur partagées entre Rust et le frontend SvelteKit.
///
/// Voir [ADR-0010](../../docs/adr/ADR-0010-personas-and-module-gating.md) et
/// `briefs/chantiers/C10-onboarding-personas.md` §2.2.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppPreferencesDto {
    /// Persona courant (`null` tant que l'utilisateur n'a pas validé l'onboarding).
    #[serde(default)]
    pub persona: Option<Persona>,
    /// Modules visibles dans le rail UI. Set fermé v1.3 (24 IDs possibles, M4 réservé).
    pub enabled_modules: Vec<ModuleId>,
    /// `true` une fois l'onboarding complété au moins une fois.
    pub onboarded: bool,
    /// Langue UI : `"fr"` ou `"en"`.
    pub lang: String,
    /// Méthodologie utilisée par défaut pour les calculs (C24).
    /// AFNOR SPEC 2314 (Sobr.ia) au premier lancement — choix souverain.
    #[serde(default)]
    pub default_method: sobria_core::EmpreinteMethod,
    /// Méthodologies additionnelles à afficher en référence dans le
    /// panneau "Voir aussi" (C24). Liste vide par défaut.
    #[serde(default)]
    pub also_show_methods: Vec<sobria_core::EmpreinteMethod>,
    /// Dernier datacenter sélectionné, pré-rempli au prochain chargement
    /// des routes /estimate, /comparer, /simuler (C25). `None` = pas de
    /// préfill, l'utilisateur part d'un picker vide.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_datacenter_id: Option<String>,
}

impl AppPreferencesDto {
    /// Valeurs par défaut renvoyées quand `app_preferences` est vide
    /// (premier lancement). Utilise le bundle `pro_tech` qui est le plus
    /// équilibré (cf. ADR-0010 §"Onboarding non-bloquant").
    #[must_use]
    pub fn defaults() -> Self {
        Self {
            persona: None,
            enabled_modules: Persona::ProTech.default_modules(),
            onboarded: false,
            lang: "fr".into(),
            default_method: sobria_core::EmpreinteMethod::default_method(),
            also_show_methods: Vec::new(),
            default_datacenter_id: None,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Catalogue de méthodologies (C24)
// ─────────────────────────────────────────────────────────────────────────────

/// Métadonnées d'une méthodologie d'empreinte LLM, exposées au frontend
/// pour affichage dans `Settings → Méthodologies` (page `/methodologies`).
///
/// Voir [`sobria_estimator::MethodologyInfo`] côté backend pour la source
/// de vérité ; ce DTO en est une projection 1:1 sérialisable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodologyInfoDto {
    /// Identifiant stable (`afnor_sobria` / `ecologits`).
    pub method: sobria_core::EmpreinteMethod,
    /// Nom affiché en UI (langue FR).
    pub display_name: String,
    /// Courte description 1-2 phrases (langue FR).
    pub short_description: String,
    /// URL de référence (DOI ou doc officielle).
    pub reference_url: String,
    /// DOI normalisé si dispo.
    pub doi: Option<String>,
    /// Licence de la méthodologie publiée.
    pub license: String,
    /// Statut de calibration ("peer_reviewed_reproduced" / "public_method_calibration_pending" / "indicative").
    pub calibration: String,
    /// Année de publication de la méthodologie de référence.
    pub year_published: u16,
    /// Organisation qui maintient la méthodologie de référence.
    pub maintained_by: String,
}

impl From<&sobria_estimator::MethodologyInfo> for MethodologyInfoDto {
    fn from(info: &sobria_estimator::MethodologyInfo) -> Self {
        let calibration = match info.calibration {
            sobria_estimator::MethodologyCalibration::PeerReviewedReproduced => {
                "peer_reviewed_reproduced"
            },
            sobria_estimator::MethodologyCalibration::PublicMethodCalibrationPending => {
                "public_method_calibration_pending"
            },
            sobria_estimator::MethodologyCalibration::Indicative => "indicative",
        };
        Self {
            method: info.method,
            display_name: info.display_name.into(),
            short_description: info.short_description.into(),
            reference_url: info.reference_url.into(),
            doi: info.doi.map(Into::into),
            license: info.license.into(),
            calibration: calibration.into(),
            year_published: info.year_published,
            maintained_by: info.maintained_by.into(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// extension navigateur — pairing (C27.5.c/d)
// ─────────────────────────────────────────────────────────────────────────────

/// Code 6 chiffres affiché à l'utilisateur côté UI Sobr.ia pour appairer
/// l'extension navigateur. TTL 5 min.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingCodeDto {
    /// Les 6 chiffres à recopier dans l'extension.
    pub code: String,
    /// Instant d'expiration en RFC 3339.
    pub expires_at: String,
    /// Secondes restantes (calculé serveur, indicatif pour l'UI).
    pub seconds_remaining: i64,
}

/// Secret produit après validation d'un code de pairing — c'est ce que
/// l'extension va stocker en `chrome.storage.local` et présenter à chaque
/// requête.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingSecretDto {
    /// `id` du pairing créé (ULID).
    pub pairing_id: String,
    /// Secret 32 octets encodé hex (64 chars). À transmettre tel quel à
    /// l'extension via le bridge, puis jeté côté app desktop.
    pub secret_hex: String,
}

/// Représentation UI d'un pairing actif (liste dans /parametres).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingDto {
    pub id: String,
    pub fingerprint: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<String>,
}

impl From<&crate::extension_store::PairingRow> for PairingDto {
    fn from(r: &crate::extension_store::PairingRow) -> Self {
        Self {
            id: r.id.clone(),
            fingerprint: r.fingerprint.clone(),
            created_at: r.created_at.to_rfc3339(),
            last_seen_at: r.last_seen_at.map(|d| d.to_rfc3339()),
            revoked_at: r.revoked_at.map(|d| d.to_rfc3339()),
        }
    }
}

/// Représentation UI d'un événement ingéré depuis l'extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionEventDto {
    pub id: String,
    pub pairing_id: String,
    pub ts: String,
    pub method: String,
    pub model_id: String,
    pub tokens_in: u32,
    pub tokens_out: u32,
    pub gco2eq_p50: f64,
    pub water_ml: f64,
    pub energy_wh: f64,
    pub ingested_at: String,
}

impl From<&crate::extension_store::ExtensionEventRow> for ExtensionEventDto {
    fn from(r: &crate::extension_store::ExtensionEventRow) -> Self {
        Self {
            id: r.id.clone(),
            pairing_id: r.pairing_id.clone(),
            ts: r.ts.to_rfc3339(),
            method: r.method.clone(),
            model_id: r.model_id.clone(),
            tokens_in: r.tokens_in,
            tokens_out: r.tokens_out,
            gco2eq_p50: r.gco2eq_p50,
            water_ml: r.water_ml,
            energy_wh: r.energy_wh,
            ingested_at: r.ingested_at.to_rfc3339(),
        }
    }
}

/// Champs extraits d'un payload `estimation_result_json` du ledger.
/// Utilisé par [`AuditEntrySummaryDto::from_entry`].
struct ParsedPayload {
    model_id: String,
    co2eq_p50: f64,
    method: sobria_core::EmpreinteMethod,
}

fn parse_payload_full(payload: &str) -> ParsedPayload {
    if payload == sobria_audit::PURGED_SENTINEL {
        return ParsedPayload {
            model_id: "(purgé)".into(),
            co2eq_p50: f64::NAN,
            // Une entrée purgée ne révèle plus sa méthode ; on défaut sur
            // AfnorSobria (sentinel cohérent avec ledger v1 historique).
            method: sobria_core::EmpreinteMethod::AfnorSobria,
        };
    }
    let parsed: Result<EstimationResult, _> = serde_json::from_str(payload);
    match parsed {
        Ok(r) => {
            let model_id = r.request.model_id.clone();
            let co2eq_p50 = r
                .indicators
                .iter()
                .find(|i| matches!(i.indicator, Indicator::Co2Eq))
                .map_or(f64::NAN, |i| i.interval.p50);
            ParsedPayload {
                model_id,
                co2eq_p50,
                method: r.method,
            }
        },
        Err(_) => ParsedPayload {
            model_id: "(invalide)".into(),
            co2eq_p50: f64::NAN,
            method: sobria_core::EmpreinteMethod::AfnorSobria,
        },
    }
}

/// Wrapper rétro-compat de [`parse_payload_full`] pour le code legacy qui
/// ne veut que `(model_id, co2eq_p50)`. À supprimer quand tous les
/// call-sites auront migré.
///
/// Aujourd'hui seul utilisé par les tests — gardé sous `#[cfg(test)]` en
/// attendant la migration / suppression définitive.
#[cfg(test)]
fn parse_payload(payload: &str) -> (String, f64) {
    let p = parse_payload_full(payload);
    (p.model_id, p.co2eq_p50)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sobria_core::{EstimationRequest, Indicator, IndicatorValue, UncertaintyInterval};

    #[test]
    fn estimation_request_dto_round_trip_json() {
        let dto = EstimationRequestDto {
            model_id: "gpt-4o-mini".into(),
            tokens_in: 100,
            tokens_out_estimated: 500,
            datacenter_id: None,
            method: None,
            modalities: None,
            overhead: None,
        };
        let json = serde_json::to_string(&dto).unwrap();
        let back: EstimationRequestDto = serde_json::from_str(&json).unwrap();
        assert_eq!(back.model_id, "gpt-4o-mini");
        assert_eq!(back.tokens_in, 100);
    }

    #[test]
    fn parse_payload_purged_returns_sentinel() {
        let (id, co2) = parse_payload(sobria_audit::PURGED_SENTINEL);
        assert_eq!(id, "(purgé)");
        assert!(co2.is_nan());
    }

    #[test]
    fn parse_payload_invalid_returns_invalid_marker() {
        let (id, _co2) = parse_payload("{not valid json");
        assert_eq!(id, "(invalide)");
    }

    #[test]
    fn app_preferences_dto_round_trip_with_default_datacenter_id() {
        let dto = AppPreferencesDto {
            persona: None,
            enabled_modules: vec![],
            onboarded: false,
            lang: "fr".into(),
            default_method: sobria_core::EmpreinteMethod::AfnorSobria,
            also_show_methods: vec![],
            default_datacenter_id: Some("ovh-gra-gravelines".into()),
        };
        let json = serde_json::to_string(&dto).unwrap();
        assert!(json.contains("default_datacenter_id"));
        let back: AppPreferencesDto = serde_json::from_str(&json).unwrap();
        assert_eq!(
            back.default_datacenter_id.as_deref(),
            Some("ovh-gra-gravelines")
        );

        // Backward-compat: a JSON without the field must deserialize with None.
        let legacy = serde_json::json!({
            "persona": null,
            "enabled_modules": [],
            "onboarded": false,
            "lang": "fr",
            "default_method": "afnor_sobria",
            "also_show_methods": []
        });
        let parsed: AppPreferencesDto = serde_json::from_value(legacy).unwrap();
        assert!(parsed.default_datacenter_id.is_none());
    }

    #[test]
    fn parse_payload_valid_extracts_model_and_co2() {
        let result = EstimationResult {
            method: sobria_core::EmpreinteMethod::AfnorSobria,
            request: EstimationRequest {
                model_id: "claude-3-5-sonnet".into(),
                tokens_in: 50,
                tokens_out_estimated: 200,
                datacenter_id: None,
                timestamp: Utc::now(),
                modalities: Vec::new(),
                overhead: sobria_core::ContextOverhead::default(),
            },
            indicators: vec![IndicatorValue {
                indicator: Indicator::Co2Eq,
                interval: UncertaintyInterval::new(1.0, 2.5, 4.0).unwrap(),
                unit: "gCO2eq".into(),
                bins: None,
            }],
            equivalents: vec![],
            hypotheses: vec![],
            computed_at: Utc::now(),
            seed: 42,
        };
        let payload = serde_json::to_string(&result).unwrap();
        let (id, co2) = parse_payload(&payload);
        assert_eq!(id, "claude-3-5-sonnet");
        assert!((co2 - 2.5).abs() < 1e-9);
    }
}
