//! Jointure géographique datacenter ↔ maille IRIS — table Gold
//! `datacenter_iris_link`.
//!
//! Pour chaque datacenter européen connu de [`sobria_geoloc`], on cherche
//! la maille IRIS la plus proche (distance grand-cercle, formule
//! haversine) parmi les Features du GeoJSON `iris_geometries.geojson`
//! conservé en Copper RTE IRIS (cf. `sources::rte_iris`). Le centroïde
//! IRIS est la moyenne des sommets de son polygone — approximation
//! suffisamment précise pour un nearest-neighbor.
//!
//! Voir ADR-0009 et chantier C26.3.

use std::path::Path;

use geojson::{GeoJson, Geometry, Value as GeoValue};
use sobria_geoloc::DatacenterRecord;

use crate::error::{IngestError, IngestResult};

/// Une ligne de la table Gold `datacenter_iris_link`.
#[derive(Debug, Clone)]
pub struct DatacenterIrisLink {
    /// Identifiant stable du datacenter (depuis `sobria-geoloc`).
    pub datacenter_id: String,
    /// Code IRIS de la maille INSEE la plus proche.
    pub code_iris: String,
    /// Distance grand-cercle (km) du datacenter au centroïde IRIS.
    pub distance_km: f64,
    /// Latitude du centroïde IRIS (WGS84).
    pub iris_centroid_lat: f64,
    /// Longitude du centroïde IRIS (WGS84).
    pub iris_centroid_lon: f64,
}

/// Centroïde simple d'une géométrie GeoJSON (moyenne arithmétique des
/// sommets). Suffisant pour un nearest-neighbor à l'échelle France.
/// Renvoie `None` pour des géométries non polygonales / vides.
fn centroid_of(geom: &Geometry) -> Option<(f64, f64)> {
    let mut sum_lon = 0.0_f64;
    let mut sum_lat = 0.0_f64;
    let mut n = 0_u64;
    let mut accumulate = |coords: &[Vec<f64>]| {
        for c in coords {
            if c.len() >= 2 {
                sum_lon += c[0];
                sum_lat += c[1];
                n += 1;
            }
        }
    };
    match &geom.value {
        GeoValue::Polygon(rings) => {
            for ring in rings {
                accumulate(ring);
            }
        },
        GeoValue::MultiPolygon(polys) => {
            for poly in polys {
                for ring in poly {
                    accumulate(ring);
                }
            }
        },
        GeoValue::Point(coord) => {
            if coord.len() >= 2 {
                return Some((coord[0], coord[1]));
            }
        },
        _ => return None,
    }
    if n == 0 {
        return None;
    }
    #[allow(clippy::cast_precision_loss)] // n borné par la taille du GeoJSON, OK pour f64
    let nf = n as f64;
    Some((sum_lon / nf, sum_lat / nf))
}

/// Distance grand-cercle entre deux points WGS84, en kilomètres.
/// Formule haversine — voir <https://en.wikipedia.org/wiki/Haversine_formula>.
#[must_use]
pub fn haversine_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const R: f64 = 6371.0; // rayon moyen de la Terre, km
    let to_rad = |d: f64| d.to_radians();
    let phi1 = to_rad(lat1);
    let phi2 = to_rad(lat2);
    let dphi = to_rad(lat2 - lat1);
    let dlambda = to_rad(lon2 - lon1);
    let a = (dphi / 2.0).sin().powi(2) + phi1.cos() * phi2.cos() * (dlambda / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    R * c
}

/// Liste les centroïdes IRIS depuis un GeoJSON. Renvoie un vecteur
/// `(code_iris, lat, lon)` — vide si le GeoJSON ne contient aucune Feature
/// exploitable.
///
/// Le GeoJSON est attendu sous forme de `FeatureCollection` avec une
/// propriété `code_iris` (string) sur chaque Feature.
fn extract_iris_centroids(geojson_text: &str) -> IngestResult<Vec<(String, f64, f64)>> {
    let parsed: GeoJson = geojson_text
        .parse::<GeoJson>()
        .map_err(|e| IngestError::schema(format!("GeoJSON IRIS invalide : {e}")))?;
    let GeoJson::FeatureCollection(fc) = parsed else {
        return Ok(Vec::new());
    };
    let mut out = Vec::with_capacity(fc.features.len());
    for f in &fc.features {
        let Some(props) = &f.properties else {
            continue;
        };
        let Some(code) = props
            .get("code_iris")
            .and_then(|v| v.as_str().map(String::from))
        else {
            continue;
        };
        let Some(geom) = &f.geometry else { continue };
        let Some((lon, lat)) = centroid_of(geom) else {
            continue;
        };
        out.push((code, lat, lon));
    }
    Ok(out)
}

/// Construit la jointure datacenter → IRIS la plus proche pour la liste
/// de datacenters fournie. Si `geojson_text` ne contient aucun centroïde
/// exploitable, renvoie un Vec vide (pas d'erreur — la table Gold
/// `datacenter_iris_link` sera simplement vide).
pub fn build_links(
    datacenters: &[DatacenterRecord],
    geojson_text: &str,
) -> IngestResult<Vec<DatacenterIrisLink>> {
    let centroids = extract_iris_centroids(geojson_text)?;
    if centroids.is_empty() {
        return Ok(Vec::new());
    }
    let mut links = Vec::with_capacity(datacenters.len());
    for dc in datacenters {
        let mut best: Option<(usize, f64)> = None;
        for (i, (_code, lat, lon)) in centroids.iter().enumerate() {
            let d = haversine_km(dc.lat, dc.lon, *lat, *lon);
            if best.is_none_or(|(_, bd)| d < bd) {
                best = Some((i, d));
            }
        }
        let Some((idx, dist)) = best else { continue };
        let (code, lat, lon) = &centroids[idx];
        links.push(DatacenterIrisLink {
            datacenter_id: dc.id.clone(),
            code_iris: code.clone(),
            distance_km: dist,
            iris_centroid_lat: *lat,
            iris_centroid_lon: *lon,
        });
    }
    Ok(links)
}

/// Charge le GeoJSON depuis un dossier de snapshot Copper RTE IRIS, puis
/// calcule les liens. Renvoie `Ok(vec![])` si le fichier n'existe pas (le
/// pipeline doit pouvoir produire un Gold même sans Copper RTE).
pub async fn build_links_from_snapshot(
    snapshot_dir: &Path,
    datacenters: &[DatacenterRecord],
) -> IngestResult<Vec<DatacenterIrisLink>> {
    let geojson_path = snapshot_dir.join("iris_geometries.geojson");
    if !geojson_path.exists() {
        tracing::warn!(
            path = %geojson_path.display(),
            "iris_link: GeoJSON IRIS absent — datacenter_iris_link sera vide"
        );
        return Ok(Vec::new());
    }
    let text = tokio::fs::read_to_string(&geojson_path).await?;
    // Le parsing geojson + haversine est CPU-bound mais court (~ms à
    // ~secondes selon volume). Pas besoin de spawn_blocking ici.
    build_links(datacenters, &text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dc(id: &str, lat: f64, lon: f64) -> DatacenterRecord {
        DatacenterRecord {
            id: id.into(),
            name: format!("DC {id}"),
            operator: "test".into(),
            country_iso: "FR".into(),
            city: "Paris".into(),
            lat,
            lon,
            pue: 1.5,
            wue_l_per_kwh: None,
            if_electrical_g_per_kwh: 60.0,
            capacity_mw: None,
            sources: vec!["test".into()],
            hourly_profile_24h: vec![],
        }
    }

    #[test]
    fn haversine_paris_marseille_about_660km() {
        // Paris (48.8566, 2.3522) ↔ Marseille (43.2965, 5.3698)
        let d = haversine_km(48.8566, 2.3522, 43.2965, 5.3698);
        // Distance réelle ~661 km — on tolère ±10 km.
        assert!(
            (d - 661.0).abs() < 10.0,
            "distance attendue ~661 km, obtenue {d}"
        );
    }

    #[test]
    fn haversine_same_point_is_zero() {
        let d = haversine_km(48.0, 2.0, 48.0, 2.0);
        assert!(d < 1e-9, "même point → distance 0, obtenu {d}");
    }

    #[test]
    fn centroid_of_simple_square_polygon() {
        let geom: Geometry = Geometry::new(GeoValue::Polygon(vec![vec![
            vec![0.0, 0.0],
            vec![2.0, 0.0],
            vec![2.0, 2.0],
            vec![0.0, 2.0],
            vec![0.0, 0.0],
        ]]));
        let c = centroid_of(&geom).unwrap();
        // Moyenne des 5 sommets : (4/5, 4/5) = (0.8, 0.8)
        assert!((c.0 - 0.8).abs() < 1e-9);
        assert!((c.1 - 0.8).abs() < 1e-9);
    }

    #[test]
    fn extract_centroids_filters_features_without_code_iris() {
        let g = r#"{
            "type": "FeatureCollection",
            "features": [
                {
                    "type": "Feature",
                    "properties": { "code_iris": "751010101" },
                    "geometry": {
                        "type": "Polygon",
                        "coordinates": [[[2.34, 48.86],[2.35, 48.86],[2.35, 48.87],[2.34, 48.87],[2.34, 48.86]]]
                    }
                },
                {
                    "type": "Feature",
                    "properties": { "autre": "valeur" },
                    "geometry": {
                        "type": "Point",
                        "coordinates": [3.0, 50.0]
                    }
                }
            ]
        }"#;
        let centroids = extract_iris_centroids(g).unwrap();
        assert_eq!(centroids.len(), 1, "feature sans code_iris ignorée");
        assert_eq!(centroids[0].0, "751010101");
    }

    #[test]
    fn build_links_picks_closest_iris_for_each_datacenter() {
        let g = r#"{
            "type": "FeatureCollection",
            "features": [
                {
                    "type": "Feature",
                    "properties": { "code_iris": "iris-paris" },
                    "geometry": { "type": "Point", "coordinates": [2.35, 48.86] }
                },
                {
                    "type": "Feature",
                    "properties": { "code_iris": "iris-marseille" },
                    "geometry": { "type": "Point", "coordinates": [5.37, 43.30] }
                }
            ]
        }"#;
        let dcs = vec![
            dc("dc-paris", 48.85, 2.34),     // ≈ Paris
            dc("dc-marseille", 43.31, 5.40), // ≈ Marseille
        ];
        let links = build_links(&dcs, g).unwrap();
        assert_eq!(links.len(), 2);
        let paris = links
            .iter()
            .find(|l| l.datacenter_id == "dc-paris")
            .unwrap();
        assert_eq!(paris.code_iris, "iris-paris");
        assert!(
            paris.distance_km < 5.0,
            "DC Paris doit être très près d'iris-paris"
        );
        let marseille = links
            .iter()
            .find(|l| l.datacenter_id == "dc-marseille")
            .unwrap();
        assert_eq!(marseille.code_iris, "iris-marseille");
    }

    #[test]
    fn build_links_returns_empty_for_empty_collection() {
        let g = r#"{"type":"FeatureCollection","features":[]}"#;
        let dcs = vec![dc("x", 0.0, 0.0)];
        let links = build_links(&dcs, g).unwrap();
        assert!(links.is_empty(), "pas de centroïde → pas de lien");
    }

    #[test]
    fn build_links_rejects_invalid_geojson() {
        let dcs = vec![dc("x", 0.0, 0.0)];
        let res = build_links(&dcs, "not a valid geojson");
        assert!(res.is_err(), "GeoJSON invalide doit échouer explicitement");
    }
}
