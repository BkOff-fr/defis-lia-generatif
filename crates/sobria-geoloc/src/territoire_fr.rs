//! Chargement du dataset Territoire FR (sites industriels par IRIS) produit
//! par `sobria-ingest fetch territoire-fr`.
//!
//! Voir `briefs/chantiers/C13-territoire-fr-sankey.md` et ADR-0009.
//!
//! **Contrat de données** : ce module lit strictement le JSON produit par
//! l'ingest. Le schéma est figé sur la réponse réelle de l'API ODRÉ
//! (dataset `consommation-annuelle-par-iris`, publisher RTE/NaTran/Teréga).
//!
//! **Aucune donnée n'est inventée ici.** Les types sont des miroirs locaux
//! des structs définies dans `sobria-ingest::sources::territoire_fr` —
//! duplication justifiée pour éviter une dépendance entre `sobria-geoloc`
//! (léger) et `sobria-ingest` (lourd, polars + duckdb).

use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Erreurs publiques de chargement.
#[derive(Debug, Error)]
pub enum TerritoireFrError {
    #[error("fichier introuvable : {0}")]
    NotFound(std::path::PathBuf),
    #[error("io : {0}")]
    Io(#[from] std::io::Error),
    #[error("json : {0}")]
    Json(#[from] serde_json::Error),
    #[error("contrat violé : {0}")]
    Schema(String),
}

pub type TerritoireFrResult<T> = Result<T, TerritoireFrError>;

// ─────────────────────────────────────────────────────────────────────────────
// Types miroirs du JSON produit par sobria-ingest::sources::territoire_fr.
// ─────────────────────────────────────────────────────────────────────────────

/// Métadonnées d'un artefact ODRÉ (URL, SHA-256, timestamp).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ArtifactMeta {
    pub version: String,
    pub fetched_at: String,
    pub source_url: String,
    pub source_sha256: String,
    pub license: String,
    pub notes: Vec<String>,
}

/// Métadonnées d'une région (centroïde + part nucléaire mix régional).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct RegionMeta {
    pub region_iso: String,
    pub name: String,
    pub insee_code: String,
    pub centroid_lat: f64,
    pub centroid_lon: f64,
    pub nuclear_share_pct: f64,
}

/// Fiche d'un site industriel par IRIS.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct IndustrialSite {
    pub code_iris: String,
    pub commune: String,
    pub commune_code: String,
    pub department_code: String,
    pub department_label: String,
    pub region_insee_code: String,
    pub region_iso: String,
    pub lat: f64,
    pub lon: f64,
    pub consumption_mwh_elec: f64,
    pub consumption_mwh_gas_grtgaz: f64,
    pub consumption_mwh_gas_terega: f64,
    pub consumption_total_mwh: f64,
    pub pdl_count_elec: u32,
    pub pdl_count_gas: u32,
    pub pdl_total: u32,
    pub year: u32,
}

/// Artefact complet déposé par `sobria-ingest fetch territoire-fr`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct TerritoireFrArtifact {
    #[serde(rename = "_meta")]
    pub meta: ArtifactMeta,
    pub regions: Vec<RegionMeta>,
    pub industrial_sites: Vec<IndustrialSite>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Loader runtime
// ─────────────────────────────────────────────────────────────────────────────

/// Charge `territoire_fr.json` depuis le disque. Aucune valeur synthétisée.
///
/// **Errors** :
/// - `NotFound` si le fichier n'existe pas (l'utilisateur doit lancer
///   `cargo run -p sobria-ingest -- fetch territoire-fr`).
/// - `Json` si le fichier est corrompu.
/// - `Schema` si la structure ne correspond pas au contrat (par ex. `regions`
///   vide ou liste de sites sans donnée).
pub fn load_territoire_fr(path: &Path) -> TerritoireFrResult<TerritoireFrArtifact> {
    if !path.exists() {
        return Err(TerritoireFrError::NotFound(path.to_path_buf()));
    }
    let text = std::fs::read_to_string(path)?;
    let artifact: TerritoireFrArtifact = serde_json::from_str(&text)?;
    validate(&artifact)?;
    Ok(artifact)
}

fn validate(a: &TerritoireFrArtifact) -> TerritoireFrResult<()> {
    if a.regions.is_empty() {
        return Err(TerritoireFrError::Schema(
            "regions est vide — contrat ODRÉ violé".into(),
        ));
    }
    if a.industrial_sites.is_empty() {
        return Err(TerritoireFrError::Schema(
            "industrial_sites est vide — vérifier l'extraction ODRÉ".into(),
        ));
    }
    if a.meta.source_sha256.is_empty() {
        return Err(TerritoireFrError::Schema(
            "_meta.source_sha256 absent — la traçabilité ODRÉ est requise".into(),
        ));
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Agrégation par région
// ─────────────────────────────────────────────────────────────────────────────

/// Agrégat de tous les sites industriels d'une région.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RegionFrAggregate {
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
    /// Top 5 sites par consommation totale (ordre décroissant).
    pub top_sites: Vec<IndustrialSiteSummary>,
}

/// Vue tronquée d'un site (pour les top-N régionaux).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IndustrialSiteSummary {
    pub code_iris: String,
    pub commune: String,
    pub consumption_total_mwh: f64,
}

/// Agrège les sites par région ISO. Trie par `region_iso` ASC.
#[must_use]
pub fn aggregate_by_region(artifact: &TerritoireFrArtifact) -> Vec<RegionFrAggregate> {
    use std::collections::HashMap;

    // Index région ISO → métadonnées.
    let region_index: HashMap<&str, &RegionMeta> = artifact
        .regions
        .iter()
        .map(|r| (r.region_iso.as_str(), r))
        .collect();

    let mut by_region: HashMap<String, Vec<&IndustrialSite>> = HashMap::new();
    for s in &artifact.industrial_sites {
        by_region.entry(s.region_iso.clone()).or_default().push(s);
    }

    let mut out: Vec<RegionFrAggregate> = by_region
        .into_iter()
        .filter_map(|(iso, sites)| {
            let meta = region_index.get(iso.as_str())?;
            let total_elec: f64 = sites.iter().map(|s| s.consumption_mwh_elec).sum();
            let total_gas: f64 = sites
                .iter()
                .map(|s| s.consumption_mwh_gas_grtgaz + s.consumption_mwh_gas_terega)
                .sum();
            let total = sites.iter().map(|s| s.consumption_total_mwh).sum();
            // Top 5 sites par conso totale décroissante.
            let mut sorted = sites.clone();
            sorted.sort_by(|a, b| {
                b.consumption_total_mwh
                    .partial_cmp(&a.consumption_total_mwh)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let top_sites = sorted
                .into_iter()
                .take(5)
                .map(|s| IndustrialSiteSummary {
                    code_iris: s.code_iris.clone(),
                    commune: s.commune.clone(),
                    consumption_total_mwh: s.consumption_total_mwh,
                })
                .collect();
            Some(RegionFrAggregate {
                region_iso: iso,
                region_name: meta.name.clone(),
                insee_code: meta.insee_code.clone(),
                site_count: sites.len(),
                total_consumption_mwh_elec: total_elec,
                total_consumption_mwh_gas: total_gas,
                total_consumption_mwh: total,
                centroid_lat: meta.centroid_lat,
                centroid_lon: meta.centroid_lon,
                nuclear_share_pct: meta.nuclear_share_pct,
                top_sites,
            })
        })
        .collect();
    out.sort_by(|a, b| a.region_iso.cmp(&b.region_iso));
    out
}

/// Cherche un site IRIS par son code (recherche linéaire — peu volumineux).
#[must_use]
pub fn find_site_by_code_iris<'a>(
    artifact: &'a TerritoireFrArtifact,
    code_iris: &str,
) -> Option<&'a IndustrialSite> {
    artifact
        .industrial_sites
        .iter()
        .find(|s| s.code_iris == code_iris)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// JSON minimal embarqué — pas une "vraie donnée" produite mais une
    /// fixture pour valider le loader. Reflète strictement le schéma ODRÉ.
    const FIXTURE_JSON: &str = r#"{
        "_meta": {
            "version": "1.0.0",
            "fetched_at": "2026-05-13T12:00:00+00:00",
            "source_url": "https://odre.opendatasoft.com/api/explore/v2.1/catalog/datasets/consommation-annuelle-par-iris/records",
            "source_sha256": "deadbeef00000000000000000000000000000000000000000000000000000000",
            "license": "Etalab 2.0",
            "notes": ["fixture de test"]
        },
        "regions": [
            {"region_iso": "FR-IDF", "name": "Île-de-France", "insee_code": "11",
             "centroid_lat": 48.7, "centroid_lon": 2.6, "nuclear_share_pct": 0.0},
            {"region_iso": "FR-HDF", "name": "Hauts-de-France", "insee_code": "32",
             "centroid_lat": 50.0, "centroid_lon": 2.7, "nuclear_share_pct": 78.4}
        ],
        "industrial_sites": [
            {"code_iris": "751010101", "commune": "Paris 1er", "commune_code": "75101",
             "department_code": "75", "department_label": "Paris",
             "region_insee_code": "11", "region_iso": "FR-IDF",
             "lat": 48.86, "lon": 2.34,
             "consumption_mwh_elec": 12000.0, "consumption_mwh_gas_grtgaz": 200.0,
             "consumption_mwh_gas_terega": 0.0, "consumption_total_mwh": 12200.0,
             "pdl_count_elec": 3, "pdl_count_gas": 1, "pdl_total": 4, "year": 2022},
            {"code_iris": "591830001", "commune": "Dunkerque", "commune_code": "59183",
             "department_code": "59", "department_label": "Nord",
             "region_insee_code": "32", "region_iso": "FR-HDF",
             "lat": 51.04, "lon": 2.38,
             "consumption_mwh_elec": 800000.0, "consumption_mwh_gas_grtgaz": 0.0,
             "consumption_mwh_gas_terega": 0.0, "consumption_total_mwh": 800000.0,
             "pdl_count_elec": 12, "pdl_count_gas": 0, "pdl_total": 12, "year": 2022}
        ]
    }"#;

    fn fixture() -> TerritoireFrArtifact {
        serde_json::from_str(FIXTURE_JSON).expect("fixture JSON valide")
    }

    #[test]
    fn fixture_parses_with_strict_schema() {
        let a = fixture();
        assert_eq!(a.regions.len(), 2);
        assert_eq!(a.industrial_sites.len(), 2);
        assert!(!a.meta.source_sha256.is_empty());
    }

    #[test]
    fn validate_accepts_well_formed_artifact() {
        assert!(validate(&fixture()).is_ok());
    }

    #[test]
    fn validate_rejects_empty_regions() {
        let mut a = fixture();
        a.regions.clear();
        assert!(matches!(validate(&a), Err(TerritoireFrError::Schema(_))));
    }

    #[test]
    fn validate_rejects_empty_sites() {
        let mut a = fixture();
        a.industrial_sites.clear();
        assert!(matches!(validate(&a), Err(TerritoireFrError::Schema(_))));
    }

    #[test]
    fn validate_rejects_missing_sha() {
        let mut a = fixture();
        a.meta.source_sha256.clear();
        assert!(matches!(validate(&a), Err(TerritoireFrError::Schema(_))));
    }

    #[test]
    fn load_returns_not_found_for_missing_path() {
        let err = load_territoire_fr(Path::new("/nonexistent/path.json")).unwrap_err();
        assert!(matches!(err, TerritoireFrError::NotFound(_)));
    }

    #[test]
    fn aggregate_by_region_groups_sites() {
        let a = fixture();
        let agg = aggregate_by_region(&a);
        assert_eq!(agg.len(), 2);
        // Tri alpha
        assert_eq!(agg[0].region_iso, "FR-HDF");
        assert_eq!(agg[1].region_iso, "FR-IDF");
        // HDF : 1 site, conso totale 800000
        let hdf = &agg[0];
        assert_eq!(hdf.site_count, 1);
        assert!((hdf.total_consumption_mwh - 800000.0).abs() < 1e-9);
        assert!((hdf.nuclear_share_pct - 78.4).abs() < 1e-9);
        assert_eq!(hdf.top_sites.len(), 1);
        assert_eq!(hdf.top_sites[0].commune, "Dunkerque");
    }

    #[test]
    fn aggregate_top_sites_limited_to_5() {
        let mut a = fixture();
        // Ajoute 6 sites supplémentaires en IDF
        for i in 0..6 {
            a.industrial_sites.push(IndustrialSite {
                code_iris: format!("751010{}", 200 + i),
                commune: format!("Paris commune {i}"),
                commune_code: format!("75{i:03}"),
                department_code: "75".into(),
                department_label: "Paris".into(),
                region_insee_code: "11".into(),
                region_iso: "FR-IDF".into(),
                lat: 48.86,
                lon: 2.34,
                consumption_mwh_elec: (1000 + i * 100) as f64,
                consumption_mwh_gas_grtgaz: 0.0,
                consumption_mwh_gas_terega: 0.0,
                consumption_total_mwh: (1000 + i * 100) as f64,
                pdl_count_elec: 1,
                pdl_count_gas: 0,
                pdl_total: 1,
                year: 2022,
            });
        }
        let agg = aggregate_by_region(&a);
        let idf = agg.iter().find(|r| r.region_iso == "FR-IDF").unwrap();
        assert_eq!(idf.site_count, 7);
        assert_eq!(idf.top_sites.len(), 5, "top_sites doit être borné à 5");
        // Le top 1 est Paris 1er (12200 > tous les autres)
        assert_eq!(idf.top_sites[0].commune, "Paris 1er");
    }

    #[test]
    fn find_site_by_known_code() {
        let a = fixture();
        let site = find_site_by_code_iris(&a, "751010101").unwrap();
        assert_eq!(site.commune, "Paris 1er");
    }

    #[test]
    fn find_site_unknown_returns_none() {
        let a = fixture();
        assert!(find_site_by_code_iris(&a, "999999999").is_none());
    }
}
