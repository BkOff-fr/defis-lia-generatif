//! Dataset embarqué des 28 datacenters européens de référence.
//!
//! Voir `briefs/chantiers/C12-datacenters-europe.md` et
//! `docs/sources/CATALOGUE-DATACENTERS.md`.
//!
//! Le dataset est chargé une seule fois via [`OnceLock`] depuis le fichier
//! JSON inclus à la compilation (`include_str!`). Pas d'I/O runtime.

use std::{collections::HashMap, sync::OnceLock};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Bytes du fichier `data/datacenters.json` inclus à la compilation.
const DATA_JSON: &str = include_str!("../data/datacenters.json");

/// Fiche d'un datacenter.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct DatacenterRecord {
    /// Identifiant stable, snake_case (ex: `"aws-eu-west-3-paris"`).
    pub id: String,
    /// Nom commercial (ex: `"AWS Europe (Paris)"`).
    pub name: String,
    /// Opérateur (AWS, GCP, Azure, OVH, Scaleway, Equinix, ...).
    pub operator: String,
    /// Code pays ISO 3166-1 alpha-2 (ex: `"FR"`).
    pub country_iso: String,
    /// Ville (libellé localisé).
    pub city: String,
    /// Latitude WGS84.
    pub lat: f64,
    /// Longitude WGS84.
    pub lon: f64,
    /// PUE annuel.
    pub pue: f64,
    /// WUE (L/kWh IT), si publié.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wue_l_per_kwh: Option<f64>,
    /// IF élec local (gCO₂eq/kWh) — moyenne annuelle pays.
    pub if_electrical_g_per_kwh: f64,
    /// Capacité IT (MW), si publiée.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capacity_mw: Option<f64>,
    /// URLs ou références sources (≥ 1).
    pub sources: Vec<String>,
    /// Profil horaire 24h injecté par jointure depuis `hourly_profiles_by_country`
    /// au moment du chargement. Ne fait pas partie du JSON brut datacenter.
    #[serde(default)]
    pub hourly_profile_24h: Vec<f64>,
}

/// Agrégat par pays — utilisé pour la vue dézoomée Europe.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CountryAggregate {
    pub country_iso: String,
    pub datacenter_count: usize,
    /// Moyenne pondérée par capacité (à défaut : moyenne arithmétique).
    pub avg_pue: f64,
    pub if_electrical_g_per_kwh: f64,
    pub total_capacity_mw: Option<f64>,
    pub centroid_lat: f64,
    pub centroid_lon: f64,
}

#[derive(Debug, Clone, Deserialize)]
struct RawDataset {
    hourly_profiles_by_country: HashMap<String, Vec<f64>>,
    datacenters: Vec<DatacenterRecord>,
}

static DATA: OnceLock<Vec<DatacenterRecord>> = OnceLock::new();
static AGGREGATES: OnceLock<Vec<CountryAggregate>> = OnceLock::new();

fn load() -> &'static Vec<DatacenterRecord> {
    DATA.get_or_init(|| {
        let raw: RawDataset =
            serde_json::from_str(DATA_JSON).expect("datacenters.json invalide à la compilation");
        // Joindre le profil horaire de chaque DC en fonction de son country_iso.
        raw.datacenters
            .into_iter()
            .map(|mut dc| {
                dc.hourly_profile_24h = raw
                    .hourly_profiles_by_country
                    .get(&dc.country_iso)
                    .cloned()
                    .unwrap_or_default();
                dc
            })
            .collect()
    })
}

/// Retourne la liste complète des datacenters (référence statique).
#[must_use]
pub fn all_datacenters() -> &'static [DatacenterRecord] {
    load()
}

/// Cherche un datacenter par ID exact.
#[must_use]
pub fn find_datacenter(id: &str) -> Option<&'static DatacenterRecord> {
    load().iter().find(|d| d.id == id)
}

/// Agrège les datacenters par pays.
#[must_use]
pub fn aggregate_by_country() -> &'static [CountryAggregate] {
    AGGREGATES
        .get_or_init(|| {
            let mut by_country: HashMap<&str, Vec<&DatacenterRecord>> = HashMap::new();
            for dc in load() {
                by_country.entry(&dc.country_iso).or_default().push(dc);
            }
            let mut out: Vec<CountryAggregate> = by_country
                .into_iter()
                .map(|(country_iso, dcs)| {
                    let count = dcs.len();
                    let (sum_pue, sum_lat, sum_lon, sum_cap) = dcs.iter().fold(
                        (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64),
                        |acc, dc| {
                            (
                                acc.0 + dc.pue,
                                acc.1 + dc.lat,
                                acc.2 + dc.lon,
                                acc.3 + dc.capacity_mw.unwrap_or(0.0),
                            )
                        },
                    );
                    let n = count as f64;
                    let total_capacity_mw = if sum_cap > 0.0 { Some(sum_cap) } else { None };
                    // IF élec : tous les DC d'un pays partagent la même valeur,
                    // on prend la première (sécurisé par invariant dataset).
                    let if_g = dcs[0].if_electrical_g_per_kwh;
                    CountryAggregate {
                        country_iso: country_iso.to_string(),
                        datacenter_count: count,
                        avg_pue: sum_pue / n,
                        if_electrical_g_per_kwh: if_g,
                        total_capacity_mw,
                        centroid_lat: sum_lat / n,
                        centroid_lon: sum_lon / n,
                    }
                })
                .collect();
            // Tri stable par country_iso pour reproducibilité.
            out.sort_by(|a, b| a.country_iso.cmp(&b.country_iso));
            out
        })
        .as_slice()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn dataset_loads_and_has_28_datacenters() {
        let all = all_datacenters();
        assert_eq!(all.len(), 28, "attendu 28 DC, obtenu {}", all.len());
    }

    #[test]
    fn dataset_has_no_duplicate_ids() {
        let all = all_datacenters();
        let ids: HashSet<&str> = all.iter().map(|d| d.id.as_str()).collect();
        assert_eq!(ids.len(), all.len(), "doublons d'ID détectés");
    }

    #[test]
    fn all_coordinates_within_europe_bbox() {
        // Bbox Europe élargi : lat ∈ [35, 71], lon ∈ [-10, 35].
        for dc in all_datacenters() {
            assert!(
                (35.0..=71.0).contains(&dc.lat),
                "{} latitude {} hors Europe",
                dc.id,
                dc.lat
            );
            assert!(
                (-10.0..=35.0).contains(&dc.lon),
                "{} longitude {} hors Europe",
                dc.id,
                dc.lon
            );
        }
    }

    #[test]
    fn all_country_iso_are_valid_alpha2() {
        let valid: HashSet<&str> = [
            "FR", "DE", "IE", "NL", "GB", "SE", "FI", "ES", "IT", "PL", "CH", "AT", "DK",
        ]
        .into_iter()
        .collect();
        for dc in all_datacenters() {
            assert!(
                valid.contains(dc.country_iso.as_str()),
                "country_iso inconnu : {}",
                dc.country_iso
            );
        }
    }

    #[test]
    fn all_pue_in_realistic_range() {
        for dc in all_datacenters() {
            assert!(
                (1.0..=2.0).contains(&dc.pue),
                "{} PUE {} hors range [1.0, 2.0]",
                dc.id,
                dc.pue
            );
        }
    }

    #[test]
    fn all_if_in_realistic_range() {
        for dc in all_datacenters() {
            assert!(
                (10.0..=800.0).contains(&dc.if_electrical_g_per_kwh),
                "{} IF {} hors range [10, 800]",
                dc.id,
                dc.if_electrical_g_per_kwh
            );
        }
    }

    #[test]
    fn all_datacenters_have_sources() {
        for dc in all_datacenters() {
            assert!(
                !dc.sources.is_empty(),
                "{} n'a aucune source documentée",
                dc.id
            );
        }
    }

    #[test]
    fn all_hourly_profiles_have_24_normalized_values() {
        for dc in all_datacenters() {
            assert_eq!(
                dc.hourly_profile_24h.len(),
                24,
                "{} profil 24h longueur {} ≠ 24",
                dc.id,
                dc.hourly_profile_24h.len()
            );
            for (h, v) in dc.hourly_profile_24h.iter().enumerate() {
                assert!(
                    (0.0..=1.0).contains(v),
                    "{} profil h{} = {} hors [0,1]",
                    dc.id,
                    h,
                    v
                );
            }
        }
    }

    #[test]
    fn find_known_id_returns_some() {
        assert!(find_datacenter("ovh-rbx-roubaix").is_some());
    }

    #[test]
    fn find_unknown_id_returns_none() {
        assert!(find_datacenter("does-not-exist").is_none());
    }

    #[test]
    fn aggregate_by_country_returns_13_countries() {
        let agg = aggregate_by_country();
        assert_eq!(agg.len(), 13);
        // Le tri est stable alphabétique.
        let iso: Vec<&str> = agg.iter().map(|c| c.country_iso.as_str()).collect();
        assert_eq!(
            iso,
            vec!["AT", "CH", "DE", "DK", "ES", "FI", "FR", "GB", "IE", "IT", "NL", "PL", "SE"]
        );
    }

    #[test]
    fn aggregate_france_has_4_datacenters() {
        let fr = aggregate_by_country()
            .iter()
            .find(|c| c.country_iso == "FR")
            .expect("FR doit être présent");
        assert_eq!(fr.datacenter_count, 4);
        assert!((fr.if_electrical_g_per_kwh - 56.0).abs() < 1e-9);
    }

    #[test]
    fn aggregate_centroid_within_country_bounds() {
        let agg = aggregate_by_country();
        for c in agg {
            assert!(
                (35.0..=71.0).contains(&c.centroid_lat),
                "{} centroid lat {} hors Europe",
                c.country_iso,
                c.centroid_lat
            );
            assert!(
                (-10.0..=35.0).contains(&c.centroid_lon),
                "{} centroid lon {} hors Europe",
                c.country_iso,
                c.centroid_lon
            );
        }
    }
}
