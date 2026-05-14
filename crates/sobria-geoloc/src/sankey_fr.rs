//! Sankey énergétique national FR — alimenté par les données réelles RTE
//! eco2mix produites par `sobria-ingest fetch rte-mix`.
//!
//! **Méthodologie v1.0** : on expose **uniquement** ce que RTE publie
//! officiellement — production par source + échanges nets. On ne fait
//! aucune allocation arbitraire vers les datacenters ou les familles LLM
//! tant que ces données ne sont pas disponibles publiquement (ComparIA
//! usage shares + bilan datacenters FR).
//!
//! Couches du Sankey v1.0 :
//! - **Layer 0** : sources de production (Nucléaire, Hydro, Éolien, Solaire,
//!   Gaz, Charbon, Fioul, Bioénergies, Pompage).
//! - **Layer 1** : devenir de la production (Consommation intérieure +
//!   Export net si > 0, sinon Import comptabilisé séparément).
//!
//! Le frontend M20 pourra superposer ce Sankey factuel avec un overlay
//! "scénario IA" plus tard (v1.1) — mais le squelette de base reste
//! 100% sourcé.

use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SankeyFrError {
    #[error("fichier introuvable : {0}")]
    NotFound(std::path::PathBuf),
    #[error("io : {0}")]
    Io(#[from] std::io::Error),
    #[error("json : {0}")]
    Json(#[from] serde_json::Error),
    #[error("contrat violé : {0}")]
    Schema(String),
}

pub type SankeyFrResult<T> = Result<T, SankeyFrError>;

// ─────────────────────────────────────────────────────────────────────────────
// Types miroirs du JSON produit par sobria-ingest::sources::territoire_fr
// (variante rte_mix).
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct RteMixMeta {
    pub version: String,
    pub fetched_at: String,
    pub source_url: String,
    pub source_sha256: String,
    pub license: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct RteMixSourceTotals {
    pub nuclear_twh: f64,
    pub hydro_twh: f64,
    pub wind_twh: f64,
    pub solar_twh: f64,
    pub gas_twh: f64,
    pub coal_twh: f64,
    pub oil_twh: f64,
    pub bioenergies_twh: f64,
    pub pumped_twh: f64,
    pub exchange_net_twh: f64,
    pub total_production_twh: f64,
    pub records_processed: u32,
    pub year: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct RteMixArtifact {
    #[serde(rename = "_meta")]
    pub meta: RteMixMeta,
    pub mix: RteMixSourceTotals,
}

// ─────────────────────────────────────────────────────────────────────────────
// Loader runtime
// ─────────────────────────────────────────────────────────────────────────────

/// Charge `rte_mix_fr.json` depuis le disque.
pub fn load_rte_mix(path: &Path) -> SankeyFrResult<RteMixArtifact> {
    if !path.exists() {
        return Err(SankeyFrError::NotFound(path.to_path_buf()));
    }
    let text = std::fs::read_to_string(path)?;
    let artifact: RteMixArtifact = serde_json::from_str(&text)?;
    validate(&artifact)?;
    Ok(artifact)
}

fn validate(a: &RteMixArtifact) -> SankeyFrResult<()> {
    if a.meta.source_sha256.is_empty() {
        return Err(SankeyFrError::Schema(
            "_meta.source_sha256 absent — traçabilité RTE requise".into(),
        ));
    }
    if a.mix.total_production_twh <= 0.0 {
        return Err(SankeyFrError::Schema(format!(
            "total_production_twh non strictement positif : {}",
            a.mix.total_production_twh
        )));
    }
    if a.mix.records_processed == 0 {
        return Err(SankeyFrError::Schema(
            "records_processed = 0 — l'agrégation eco2mix n'a rien traité".into(),
        ));
    }
    // Sanity check vs Bilan RTE annuel.
    // - Production totale FR ~ 460-540 TWh selon l'année (Bilan RTE 2019-2024).
    // - Nucléaire FR ~ 280-380 TWh.
    // Toute valeur < 400 ou > 600 TWh est suspecte (probablement un bug
    // de granularité 15-min vs 30-min comme corrigé en 2026-05).
    let total = a.mix.total_production_twh;
    if !(400.0..=600.0).contains(&total) {
        return Err(SankeyFrError::Schema(format!(
            "total_production_twh = {total:.1} TWh hors plage plausible \
             [400, 600] TWh (Bilan RTE annuel). Vérifier le facteur de \
             conversion pas-30-min dans sobria-ingest."
        )));
    }
    let nuclear = a.mix.nuclear_twh;
    if !(200.0..=400.0).contains(&nuclear) {
        return Err(SankeyFrError::Schema(format!(
            "nuclear_twh = {nuclear:.1} TWh hors plage plausible \
             [200, 400] TWh (Bilan RTE annuel)."
        )));
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Génération du Sankey
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct SankeyNode {
    pub id: String,
    pub label: String,
    /// 0 = source de production, 1 = devenir (consommation/export).
    pub layer: u8,
    pub value_twh: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct SankeyLink {
    pub source: String,
    pub target: String,
    pub value_twh: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SankeyData {
    pub nodes: Vec<SankeyNode>,
    pub links: Vec<SankeyLink>,
    pub total_production_twh: f64,
    pub year: u32,
    pub source_url: String,
    pub source_sha256: String,
}

/// Construit le Sankey à partir du mix RTE chargé.
///
/// Chaque source de production qui a une contribution > 0 devient un
/// nœud layer 0. Toute la production est ensuite agrégée vers un nœud
/// "Consommation intérieure" (layer 1) — si l'échange net est positif
/// (export), on déduit cette part en un nœud "Export net" séparé.
#[must_use]
pub fn generate_sankey_fr(art: &RteMixArtifact) -> SankeyData {
    let m = &art.mix;
    // Couples (id, label, valeur).
    let sources: Vec<(&str, &str, f64)> = vec![
        ("prod-nuclear", "Nucléaire", m.nuclear_twh),
        ("prod-hydro", "Hydraulique", m.hydro_twh),
        ("prod-wind", "Éolien", m.wind_twh),
        ("prod-solar", "Solaire", m.solar_twh),
        ("prod-gas", "Gaz", m.gas_twh),
        ("prod-coal", "Charbon", m.coal_twh),
        ("prod-oil", "Fioul", m.oil_twh),
        ("prod-bio", "Bioénergies", m.bioenergies_twh),
        ("prod-pumped", "Pompage", m.pumped_twh),
    ];

    let mut nodes: Vec<SankeyNode> = Vec::new();
    let mut links: Vec<SankeyLink> = Vec::new();

    // Layer 0 — sources de production strictement positives.
    let positive: Vec<&(&str, &str, f64)> = sources.iter().filter(|(_, _, v)| *v > 0.0).collect();
    for (id, label, value) in &positive {
        nodes.push(SankeyNode {
            id: (*id).into(),
            label: (*label).into(),
            layer: 0,
            value_twh: *value,
        });
    }

    let total_prod: f64 = positive.iter().map(|(_, _, v)| *v).sum();

    // Layer 1 — devenir : si exchange_net > 0 (export FR), on l'isole.
    let export_net = m.exchange_net_twh.max(0.0);
    let import_net = (-m.exchange_net_twh).max(0.0);
    let domestic = (total_prod - export_net).max(0.0);

    if domestic > 0.0 {
        nodes.push(SankeyNode {
            id: "use-domestic".into(),
            label: "Consommation intérieure".into(),
            layer: 1,
            value_twh: domestic,
        });
    }
    if export_net > 0.0 {
        nodes.push(SankeyNode {
            id: "use-export-net".into(),
            label: "Export net".into(),
            layer: 1,
            value_twh: export_net,
        });
    }
    if import_net > 0.0 {
        // Si la France importe, on l'expose comme un nœud source (layer 0).
        nodes.push(SankeyNode {
            id: "prod-import-net".into(),
            label: "Import net".into(),
            layer: 0,
            value_twh: import_net,
        });
    }

    // Liens : chaque source positive distribue sa part proportionnellement
    // entre `domestic` et `export_net`. Si l'export représente E/T du total,
    // chaque source envoie E/T de sa contribution vers Export net.
    let denom = if total_prod > 0.0 { total_prod } else { 1.0 };
    let share_export = if total_prod > 0.0 {
        export_net / total_prod
    } else {
        0.0
    };
    for (id, _label, value) in &positive {
        let to_export = value * share_export;
        let to_domestic = value - to_export;
        if to_domestic > 0.0 {
            links.push(SankeyLink {
                source: (*id).into(),
                target: "use-domestic".into(),
                value_twh: to_domestic,
            });
        }
        if to_export > 0.0 {
            links.push(SankeyLink {
                source: (*id).into(),
                target: "use-export-net".into(),
                value_twh: to_export,
            });
        }
    }
    // Import net : flux unique vers la consommation intérieure.
    if import_net > 0.0 {
        links.push(SankeyLink {
            source: "prod-import-net".into(),
            target: "use-domestic".into(),
            value_twh: import_net,
        });
    }
    // Garantie : Σ liens = total production + import_net (conservation).
    debug_assert!(
        (links.iter().map(|l| l.value_twh).sum::<f64>() - (total_prod + import_net)).abs() < 1e-6,
        "non-conservation des flux : Σlinks ≠ total + import"
    );
    let _ = denom; // garde-fou : éviter division par zéro silencieuse

    SankeyData {
        nodes,
        links,
        total_production_twh: total_prod,
        year: m.year,
        source_url: art.meta.source_url.clone(),
        source_sha256: art.meta.source_sha256.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_artifact() -> RteMixArtifact {
        RteMixArtifact {
            meta: RteMixMeta {
                version: "1.0.0".into(),
                fetched_at: "2026-05-13T12:00:00+00:00".into(),
                source_url: "https://odre.opendatasoft.com/.../eco2mix-national-cons-def".into(),
                source_sha256: "abc123".into(),
                license: "Etalab 2.0".into(),
                notes: vec!["fixture test".into()],
            },
            mix: RteMixSourceTotals {
                nuclear_twh: 320.0,
                hydro_twh: 50.0,
                wind_twh: 45.0,
                solar_twh: 20.0,
                gas_twh: 30.0,
                coal_twh: 1.0,
                oil_twh: 1.0,
                bioenergies_twh: 5.0,
                pumped_twh: 3.0,
                exchange_net_twh: 50.0, // export net
                total_production_twh: 475.0,
                records_processed: 35040,
                year: 2023,
            },
        }
    }

    #[test]
    fn validate_accepts_sample() {
        assert!(validate(&sample_artifact()).is_ok());
    }

    #[test]
    fn validate_rejects_zero_total() {
        let mut a = sample_artifact();
        a.mix.total_production_twh = 0.0;
        assert!(matches!(validate(&a), Err(SankeyFrError::Schema(_))));
    }

    #[test]
    fn validate_rejects_zero_records() {
        let mut a = sample_artifact();
        a.mix.records_processed = 0;
        assert!(matches!(validate(&a), Err(SankeyFrError::Schema(_))));
    }

    #[test]
    fn sankey_nodes_count_matches_active_sources() {
        let sankey = generate_sankey_fr(&sample_artifact());
        // 9 sources avec valeur > 0 + 1 consommation + 1 export = 11
        assert_eq!(sankey.nodes.len(), 11);
    }

    #[test]
    fn sankey_layers_are_consistent() {
        let sankey = generate_sankey_fr(&sample_artifact());
        let l0_count = sankey.nodes.iter().filter(|n| n.layer == 0).count();
        let l1_count = sankey.nodes.iter().filter(|n| n.layer == 1).count();
        assert_eq!(l0_count, 9, "9 sources actives");
        assert_eq!(l1_count, 2, "consommation + export");
    }

    #[test]
    fn sankey_conservation_of_flows() {
        let sankey = generate_sankey_fr(&sample_artifact());
        let sum_links: f64 = sankey.links.iter().map(|l| l.value_twh).sum();
        // Pas d'import dans la fixture → total flux == total prod.
        assert!(
            (sum_links - sankey.total_production_twh).abs() < 1e-6,
            "Σ liens ({sum_links}) ≠ total prod ({})",
            sankey.total_production_twh
        );
    }

    #[test]
    fn sankey_export_is_isolated_when_positive() {
        let sankey = generate_sankey_fr(&sample_artifact());
        let export_node = sankey
            .nodes
            .iter()
            .find(|n| n.id == "use-export-net")
            .expect("export net node attendu");
        assert!((export_node.value_twh - 50.0).abs() < 1e-9);
    }

    #[test]
    fn sankey_import_becomes_source_when_negative_exchange() {
        let mut a = sample_artifact();
        a.mix.exchange_net_twh = -30.0; // import net 30 TWh
        let sankey = generate_sankey_fr(&a);
        let import_node = sankey
            .nodes
            .iter()
            .find(|n| n.id == "prod-import-net")
            .expect("import net node attendu");
        assert!((import_node.value_twh - 30.0).abs() < 1e-9);
        assert_eq!(import_node.layer, 0);
        // Pas d'export
        assert!(sankey.nodes.iter().all(|n| n.id != "use-export-net"));
    }

    #[test]
    fn sankey_skips_zero_value_sources() {
        let mut a = sample_artifact();
        a.mix.coal_twh = 0.0;
        a.mix.oil_twh = 0.0;
        let sankey = generate_sankey_fr(&a);
        assert!(sankey.nodes.iter().all(|n| n.id != "prod-coal"));
        assert!(sankey.nodes.iter().all(|n| n.id != "prod-oil"));
    }

    #[test]
    fn sankey_links_split_proportionally_to_export_share() {
        // Avec 50 TWh export sur 475 TWh total, chaque source envoie
        // 50/475 ≈ 10.5% vers export net.
        let sankey = generate_sankey_fr(&sample_artifact());
        let nuclear_to_export = sankey
            .links
            .iter()
            .find(|l| l.source == "prod-nuclear" && l.target == "use-export-net")
            .unwrap();
        let expected = 320.0 * (50.0 / 475.0);
        assert!(
            (nuclear_to_export.value_twh - expected).abs() < 1e-6,
            "nuclear→export {} ≠ {}",
            nuclear_to_export.value_twh,
            expected
        );
    }

    #[test]
    fn load_returns_not_found_for_missing_path() {
        let err = load_rte_mix(Path::new("/nonexistent/rte_mix.json")).unwrap_err();
        assert!(matches!(err, SankeyFrError::NotFound(_)));
    }

    #[test]
    fn validate_rejects_half_value_regression() {
        // Garde-fou contre le bug 2026-05 (FACTOR 0.25h alors que le pas
        // réalisé est 30-min). Si une future ingestion régresse, total
        // ressort à ~243 TWh au lieu de ~494 TWh, et la validation alerte.
        let mut a = sample_artifact();
        a.mix.nuclear_twh = 160.0;
        a.mix.total_production_twh = 243.0;
        let err = validate(&a).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("plausible") || msg.contains("hors plage"),
            "message d'erreur attendu (plage plausible), reçu : {msg}"
        );
    }

    #[test]
    fn shipped_rte_mix_data_passes_validation() {
        // Vérifie que le fichier rte_mix_fr.json embarqué dans le crate
        // passe la validation. Couvre la régression silencieuse où on
        // pourrait recharger un mauvais artefact.
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data")
            .join("rte_mix_fr.json");
        let art = load_rte_mix(&path).expect("rte_mix_fr.json charge OK");
        // Sanity vs Bilan RTE 2023 (production totale ~494 TWh, nucl ~320 TWh).
        assert!(
            (450.0..=540.0).contains(&art.mix.total_production_twh),
            "total {} TWh hors plage Bilan RTE 2023 ±10 %",
            art.mix.total_production_twh
        );
        assert!(
            (280.0..=380.0).contains(&art.mix.nuclear_twh),
            "nucléaire {} TWh hors plage Bilan RTE 2023 ±15 %",
            art.mix.nuclear_twh
        );
    }
}
