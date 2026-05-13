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
    DistributionBins, EstimationRequest, EstimationResult, Equivalent, Hypothesis, Indicator,
    IndicatorValue, ModuleId, Persona,
};
use sobria_estimator::{
    CalibrationStatus, ForecastConfig, ForecastResult, ModelPreset, Openness, ParamOverrides,
    Scenario, ScenarioOutcome, SimulationRequest, SimulationResult,
};
use sobria_geoloc::{CountryAggregate, DatacenterRecord};

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
// list_models
// ─────────────────────────────────────────────────────────────────────────────

/// Preset modèle envoyé au frontend (déclinaison statique → owned strings).
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
}

impl From<&ModelPreset> for ModelPresetDto {
    fn from(p: &ModelPreset) -> Self {
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
    pub request: EstimationRequestEchoDto,
    pub indicators: Vec<IndicatorDto>,
    pub equivalents: Vec<EquivalentDto>,
    pub hypotheses: Vec<HypothesisDto>,
    pub computed_at: String,
    pub seed: u64,
    /// ID de l'entrée du ledger d'audit qui journalise ce résultat.
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
}

impl AuditEntrySummaryDto {
    /// Construit le résumé en extrayant `model_id` + `co2eq_p50` du payload.
    /// Si le payload est purgé ou mal formé, on remplit avec des sentinelles.
    #[must_use]
    pub fn from_entry(e: &AuditEntry) -> Self {
        let (model_id, co2eq_p50) = parse_payload(&e.estimation_result_json);
        let sig_short = e.sig.chars().take(16).collect();
        Self {
            id: e.id,
            timestamp: e.timestamp.to_rfc3339(),
            model_id,
            co2eq_p50,
            sig_short,
            purged: e.is_purged(),
        }
    }
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
        }
    }
}

fn parse_payload(payload: &str) -> (String, f64) {
    if payload == sobria_audit::PURGED_SENTINEL {
        return ("(purgé)".into(), f64::NAN);
    }
    let parsed: Result<EstimationResult, _> = serde_json::from_str(payload);
    match parsed {
        Ok(r) => {
            let model_id = r.request.model_id.clone();
            let co2eq = r
                .indicators
                .iter()
                .find(|i| matches!(i.indicator, Indicator::Co2Eq))
                .map_or(f64::NAN, |i| i.interval.p50);
            (model_id, co2eq)
        }
        Err(_) => ("(invalide)".into(), f64::NAN),
    }
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
    fn parse_payload_valid_extracts_model_and_co2() {
        let result = EstimationResult {
            request: EstimationRequest {
                model_id: "claude-3-5-sonnet".into(),
                tokens_in: 50,
                tokens_out_estimated: 200,
                datacenter_id: None,
                timestamp: Utc::now(),
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
