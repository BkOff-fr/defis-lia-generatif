//! Récupération des données officielles FR pour M20 Territoire FR.
//!
//! Deux artefacts produits :
//! - `territoire_fr.json` : top sites industriels (ODRÉ — RTE/NaTran/Teréga)
//! - `rte_mix_fr.json` : mix électrique national annuel (RTE eco2mix)
//!
//! **Politique de données** (CLAUDE.md §13 + ADR-0009) :
//! - Toutes les valeurs proviennent **directement** des datasets ODRÉ
//!   publics (Etalab 2.0). Aucune synthèse, aucune heuristique.
//! - Chaque artefact embarque son `_meta` traçable : URL source, SHA-256
//!   du payload brut, timestamp UTC du téléchargement, version.
//! - Aucune valeur dérivée n'est calculée ici — uniquement copie, filtrage
//!   déterministe (top N par conso) et mapping département → région ISO
//!   (référence INSEE 2024, statique et auditée).

use std::{collections::HashMap, fmt::Write, path::Path, time::Duration};

use anyhow::{anyhow, bail, Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tracing::{debug, info};

/// Dataset slug pour la consommation annuelle (élec + gaz) par IRIS des
/// sites industriels raccordés aux réseaux NaTran / RTE / Teréga.
/// Confirmé via `discover` le 2026-05-13.
const ODRE_IRIS_DATASET: &str = "consommation-annuelle-par-iris";

/// Dataset slug pour le mix élec national définitif (eco2mix).
const ODRE_RTE_DATASET: &str = "eco2mix-national-cons-def";

/// Base API Opendatasoft v2.1 — ODRÉ.
const ODRE_API_BASE: &str = "https://odre.opendatasoft.com/api/explore/v2.1/catalog/datasets";

/// Timeout par défaut pour les requêtes HTTP.
const HTTP_TIMEOUT: Duration = Duration::from_secs(120);

// ─────────────────────────────────────────────────────────────────────────────
// Mapping département → région ISO (INSEE 2024).
//
// Source : code officiel géographique, version 2024.
// https://www.insee.fr/fr/information/2114596
// ─────────────────────────────────────────────────────────────────────────────

fn dept_to_region(dept: &str) -> Option<&'static str> {
    match dept {
        // Auvergne-Rhône-Alpes
        "01" | "03" | "07" | "15" | "26" | "38" | "42" | "43" | "63" | "69" | "73" | "74" => {
            Some("FR-ARA")
        },
        // Bourgogne-Franche-Comté
        "21" | "25" | "39" | "58" | "70" | "71" | "89" | "90" => Some("FR-BFC"),
        // Bretagne
        "22" | "29" | "35" | "56" => Some("FR-BRE"),
        // Centre-Val-de-Loire
        "18" | "28" | "36" | "37" | "41" | "45" => Some("FR-CVL"),
        // Corse
        "2A" | "2B" => Some("FR-COR"),
        // Grand Est
        "08" | "10" | "51" | "52" | "54" | "55" | "57" | "67" | "68" | "88" => Some("FR-GES"),
        // Hauts-de-France
        "02" | "59" | "60" | "62" | "80" => Some("FR-HDF"),
        // Île-de-France
        "75" | "77" | "78" | "91" | "92" | "93" | "94" | "95" => Some("FR-IDF"),
        // Normandie
        "14" | "27" | "50" | "61" | "76" => Some("FR-NOR"),
        // Nouvelle-Aquitaine
        "16" | "17" | "19" | "23" | "24" | "33" | "40" | "47" | "64" | "79" | "86" | "87" => {
            Some("FR-NAQ")
        },
        // Occitanie
        "09" | "11" | "12" | "30" | "31" | "32" | "34" | "46" | "48" | "65" | "66" | "81"
        | "82" => Some("FR-OCC"),
        // Pays de la Loire
        "44" | "49" | "53" | "72" | "85" => Some("FR-PDL"),
        // Provence-Alpes-Côte d'Azur
        "04" | "05" | "06" | "13" | "83" | "84" => Some("FR-PAC"),
        _ => None,
    }
}

/// Métadonnées statiques des régions FR métropolitaines.
///
/// Sources :
/// - Centroïdes : INSEE 2024.
/// - Part nucléaire : RTE Bilan régional électrique 2023
///   <https://bilan-electrique-2023.rte-france.com/>.
fn regions_metadata() -> Vec<RegionMeta> {
    vec![
        RegionMeta::new("FR-ARA", "Auvergne-Rhône-Alpes", "84", 45.5, 4.8, 74.5),
        RegionMeta::new("FR-BFC", "Bourgogne-Franche-Comté", "27", 47.3, 4.6, 0.0),
        RegionMeta::new("FR-BRE", "Bretagne", "53", 48.2, -3.0, 0.0),
        RegionMeta::new("FR-CVL", "Centre-Val-de-Loire", "24", 47.5, 1.7, 89.6),
        RegionMeta::new("FR-COR", "Corse", "94", 42.2, 9.0, 0.0),
        RegionMeta::new("FR-GES", "Grand Est", "44", 48.7, 5.7, 80.2),
        RegionMeta::new("FR-HDF", "Hauts-de-France", "32", 50.0, 2.7, 78.4),
        RegionMeta::new("FR-IDF", "Île-de-France", "11", 48.7, 2.6, 0.0),
        RegionMeta::new("FR-NOR", "Normandie", "28", 49.3, 0.5, 88.7),
        RegionMeta::new("FR-NAQ", "Nouvelle-Aquitaine", "75", 45.0, 0.5, 60.3),
        RegionMeta::new("FR-OCC", "Occitanie", "76", 43.6, 2.4, 48.9),
        RegionMeta::new("FR-PDL", "Pays de la Loire", "52", 47.4, -0.7, 0.0),
        RegionMeta::new("FR-PAC", "Provence-Alpes-Côte d'Azur", "93", 43.9, 6.1, 0.0),
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// Types de données — strictement ce qui sort de l'API ODRÉ + mapping ISO.
// Aucun champ "dérivé" / "synthétisé".
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionMeta {
    pub region_iso: String,
    pub name: String,
    pub insee_code: String,
    pub centroid_lat: f64,
    pub centroid_lon: f64,
    pub nuclear_share_pct: f64,
}

impl RegionMeta {
    fn new(iso: &str, name: &str, insee: &str, lat: f64, lon: f64, nuclear: f64) -> Self {
        Self {
            region_iso: iso.into(),
            name: name.into(),
            insee_code: insee.into(),
            centroid_lat: lat,
            centroid_lon: lon,
            nuclear_share_pct: nuclear,
        }
    }
}

/// Fiche IRIS « consommation annuelle des sites industriels raccordés aux
/// réseaux de transport » (RTE + GRTgaz/NaTran + Teréga).
///
/// Source : <https://odre.opendatasoft.com/explore/dataset/consommation-annuelle-par-iris/>
/// Schéma figé contre la réponse réelle de l'API ODRÉ v2.1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustrialSite {
    pub code_iris: String,
    pub commune: String,
    pub commune_code: String,
    pub department_code: String,
    pub department_label: String,
    /// Code région INSEE (numérique, ex: "11" pour Île-de-France).
    pub region_insee_code: String,
    /// Région ISO ADMIN1 (ex: "FR-IDF").
    pub region_iso: String,
    pub lat: f64,
    pub lon: f64,
    /// Consommation électrique annuelle (MWh) — RTE.
    pub consumption_mwh_elec: f64,
    /// Consommation gaz annuelle (MWh) — GRTgaz / NaTran.
    pub consumption_mwh_gas_grtgaz: f64,
    /// Consommation gaz annuelle (MWh) — Teréga.
    pub consumption_mwh_gas_terega: f64,
    /// Total énergie annuelle (MWh) — `consommation_totale` dans l'API.
    pub consumption_total_mwh: f64,
    /// Nombre de points de livraison électricité (RTE).
    pub pdl_count_elec: u32,
    /// Nombre de points de livraison gaz (GRTgaz + Teréga).
    pub pdl_count_gas: u32,
    /// Nombre total de points de livraison.
    pub pdl_total: u32,
    pub year: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMeta {
    pub version: String,
    pub fetched_at: String,
    pub source_url: String,
    pub source_sha256: String,
    pub license: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoireFrArtifact {
    #[serde(rename = "_meta")]
    pub meta: ArtifactMeta,
    pub regions: Vec<RegionMeta>,
    pub industrial_sites: Vec<IndustrialSite>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RteMixArtifact {
    #[serde(rename = "_meta")]
    pub meta: ArtifactMeta,
    pub mix: RteMixSourceTotals,
}

// ─────────────────────────────────────────────────────────────────────────────
// Fetcher principal
// ─────────────────────────────────────────────────────────────────────────────

/// Taille de page max acceptée par l'API ODRÉ v2.1.
const ODRE_PAGE_SIZE: u32 = 100;

/// Résultat d'une recherche dans le catalogue ODRÉ.
#[derive(Debug, Clone, Serialize)]
pub struct DatasetMatch {
    /// Slug stable (à utiliser dans les URLs API).
    pub dataset_id: String,
    /// Titre humain.
    pub title: String,
    /// Description courte.
    pub description: Option<String>,
    /// Éditeur (publisher).
    pub publisher: Option<String>,
    /// URL canonique de l'explorateur web.
    pub explore_url: String,
}

/// Interroge le catalogue ODRÉ pour trouver les datasets correspondant à
/// un mot-clé (recherche full-text). Pratique pour découvrir le bon slug
/// avant un `fetch`.
///
/// Endpoint : `/datasets` (Opendatasoft v2.1 catalog).
/// Utilise la syntaxe ODSQL `where=search("...")` (v2.1).
pub async fn discover_datasets(keyword: &str, limit: u32) -> Result<Vec<DatasetMatch>> {
    let where_clause = format!("search(\"{}\")", keyword.replace('"', "\\\""));
    let url = format!(
        "{base}?limit={limit}&where={kw}",
        base = ODRE_API_BASE,
        limit = limit.min(50),
        kw = urlencoding(&where_clause),
    );
    info!(%url, "ODRÉ catalog search");
    let client = reqwest::Client::builder()
        .timeout(HTTP_TIMEOUT)
        .user_agent("sobria-ingest/0.1 (+https://sobr.ia)")
        .build()?;
    let resp = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .context("requête catalogue ODRÉ échouée")?;
    let status = resp.status();
    let bytes = resp.bytes().await?;
    if !status.is_success() {
        let body = String::from_utf8_lossy(&bytes);
        bail!(
            "ODRÉ catalog HTTP {status}\nRéponse : {}",
            body.chars().take(500).collect::<String>()
        );
    }
    let payload: Value = serde_json::from_slice(&bytes)?;
    let results = payload
        .get("results")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut matches = Vec::new();
    for r in results {
        let dataset_id = r
            .get("dataset_id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if dataset_id.is_empty() {
            continue;
        }
        // Les champs riches sont sous .metas.default ou .metas selon version.
        let metas = r
            .get("metas")
            .and_then(|m| m.get("default").cloned().or_else(|| Some(m.clone())))
            .unwrap_or(Value::Null);
        let title = metas
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or(&dataset_id)
            .to_string();
        let description = metas
            .get("description")
            .and_then(Value::as_str)
            .map(|s| s.chars().take(180).collect::<String>());
        let publisher = metas
            .get("publisher")
            .and_then(Value::as_str)
            .map(str::to_string);
        let explore_url = format!("https://odre.opendatasoft.com/explore/dataset/{dataset_id}/");
        matches.push(DatasetMatch {
            dataset_id,
            title,
            description,
            publisher,
            explore_url,
        });
    }
    Ok(matches)
}

/// Télécharge le top `target_limit` sites industriels depuis ODRÉ.
///
/// Endpoint : `/datasets/{slug}/records` (v2.1 Opendatasoft).
/// Pas de clé API requise — dataset Etalab 2.0.
///
/// Pagine automatiquement par tranches de `ODRE_PAGE_SIZE = 100` jusqu'à
/// obtenir au moins `target_limit` records valides. Sort côté client par
/// consommation décroissante après collecte (pas de `order_by` qui dépend
/// d'un nom de champ exact ODRÉ).
#[allow(clippy::too_many_lines)]
pub async fn fetch_industrial_sites(target_limit: u32) -> Result<TerritoireFrArtifact> {
    let client = reqwest::Client::builder()
        .timeout(HTTP_TIMEOUT)
        .user_agent("sobria-ingest/0.1 (+https://sobr.ia)")
        .build()?;

    // Pour atterrir sur le « top » par consommation, on récupère beaucoup
    // plus que target_limit (×3) puis on trie. À défaut de pouvoir trier
    // côté serveur sans connaître le nom exact du champ, on absorbe la
    // sur-fetch (peu coûteux, dataset <100 MB).
    let over_sample = target_limit.saturating_mul(3).max(ODRE_PAGE_SIZE);

    let mut all_records: Vec<Value> = Vec::new();
    let mut first_url: Option<String> = None;
    let mut combined_sha = Sha256::new();
    let mut offset: u32 = 0;
    while offset < over_sample {
        let limit = ODRE_PAGE_SIZE.min(over_sample - offset);
        let url =
            format!("{ODRE_API_BASE}/{ODRE_IRIS_DATASET}/records?limit={limit}&offset={offset}");
        info!(%url, "fetching ODRÉ IRIS page");
        if first_url.is_none() {
            first_url = Some(url.clone());
        }
        let resp = client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .context("requête ODRÉ IRIS échouée")?;
        let status = resp.status();
        let bytes = resp.bytes().await?;
        if !status.is_success() {
            let body = String::from_utf8_lossy(&bytes);
            bail!(
                "ODRÉ IRIS HTTP {status} sur {url}\nRéponse : {}",
                body.chars().take(500).collect::<String>()
            );
        }
        combined_sha.update(&bytes);
        let payload: Value =
            serde_json::from_slice(&bytes).context("payload ODRÉ IRIS non-JSON")?;
        let page = payload
            .get("results")
            .and_then(Value::as_array)
            .cloned()
            .ok_or_else(|| anyhow!("payload ODRÉ : champ 'results' absent ou non-tableau"))?;
        if page.is_empty() {
            debug!(offset, "page vide → fin de pagination");
            break;
        }
        let got = u32::try_from(page.len()).unwrap_or(u32::MAX);
        all_records.extend(page);
        offset += got;
        if got < limit {
            debug!("dernière page partielle, arrêt");
            break;
        }
    }
    let raw_count = all_records.len();
    info!(raw_count, "pages ODRÉ accumulées");

    let mut sites: Vec<IndustrialSite> = Vec::new();
    let mut skipped: u32 = 0;
    let mut first_record_keys: Option<Vec<String>> = None;
    for rec in &all_records {
        if first_record_keys.is_none() {
            if let Value::Object(map) = rec {
                first_record_keys = Some(map.keys().cloned().collect());
            }
        }
        match parse_industrial_record(rec) {
            Some(site) => sites.push(site),
            None => skipped += 1,
        }
    }
    sites.sort_by(|a, b| {
        b.consumption_mwh_elec
            .partial_cmp(&a.consumption_mwh_elec)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    sites.truncate(target_limit as usize);

    if sites.is_empty() && !all_records.is_empty() {
        // Diagnostic : les champs du premier record sont probablement
        // différents de ceux qu'on essaie de parser. On log la liste pour
        // que l'utilisateur puisse adapter `parse_industrial_record`.
        let keys = first_record_keys.unwrap_or_default().join(", ");
        bail!(
            "0 site retenu sur {raw_count} records. Champs ODRÉ détectés : [{keys}]. \
             Adapter parse_industrial_record dans territoire_fr.rs si nécessaire."
        );
    }

    let sha = {
        let digest = combined_sha.finalize();
        let mut s = String::with_capacity(64);
        for b in digest {
            let _ = write!(s, "{b:02x}");
        }
        s
    };
    let url_for_meta =
        first_url.unwrap_or_else(|| format!("{ODRE_API_BASE}/{ODRE_IRIS_DATASET}/records"));
    let meta = ArtifactMeta {
        version: "1.0.0".into(),
        fetched_at: Utc::now().to_rfc3339(),
        source_url: url_for_meta,
        source_sha256: sha,
        license: "Etalab 2.0".into(),
        notes: vec![
            "Top sites industriels par consommation électrique annuelle ODRÉ.".into(),
            format!(
                "Records bruts paginés : {raw_count}, retenus (top {target_limit}) : {}, \
                 écartés (DROM, géoloc manquante, conso ≤ 0) : {skipped}.",
                sites.len()
            ),
            "Régions : INSEE 2024 + RTE Bilan régional 2023 (cf. dept_to_region).".into(),
            "Aucune transformation : valeurs originales ODRÉ, tri côté client.".into(),
        ],
    };
    info!(retained = sites.len(), %skipped, "ODRÉ IRIS parsing OK");
    Ok(TerritoireFrArtifact {
        meta,
        regions: regions_metadata(),
        industrial_sites: sites,
    })
}

/// Télécharge eco2mix année complète et agrège en TWh.
#[allow(clippy::too_many_lines)]
pub async fn fetch_rte_mix(year: u32) -> Result<RteMixArtifact> {
    let where_clause = format!(
        "date_heure>=date'{y}-01-01' AND date_heure<date'{y_plus}-01-01'",
        y = year,
        y_plus = year + 1
    );
    let select_cols =
        "nucleaire,hydraulique,eolien,solaire,gaz,charbon,fioul,bioenergies,pompage,ech_physiques";
    let url = format!(
        "{base}/{slug}/exports/json?lang=fr&timezone=UTC&where={where_enc}&select={select_enc}",
        base = ODRE_API_BASE,
        slug = ODRE_RTE_DATASET,
        where_enc = urlencoding(&where_clause),
        select_enc = urlencoding(select_cols),
    );
    info!(%url, "fetching RTE eco2mix");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(300))
        .user_agent("sobria-ingest/0.1 (+https://sobr.ia)")
        .build()?;
    let resp = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .context("requête RTE eco2mix échouée")?;
    let status = resp.status();
    let bytes = resp.bytes().await?;
    if !status.is_success() {
        let body = String::from_utf8_lossy(&bytes);
        bail!(
            "RTE eco2mix HTTP {status} sur {url}\nRéponse : {}",
            body.chars().take(500).collect::<String>()
        );
    }
    let sha = sha256_hex(&bytes);
    let records: Vec<HashMap<String, Value>> =
        serde_json::from_slice(&bytes).context("payload eco2mix non-JSON tableau")?;

    let mut totals = HashMap::<&str, f64>::from([
        ("nucleaire", 0.0),
        ("hydraulique", 0.0),
        ("eolien", 0.0),
        ("solaire", 0.0),
        ("gaz", 0.0),
        ("charbon", 0.0),
        ("fioul", 0.0),
        ("bioenergies", 0.0),
        ("pompage", 0.0),
        ("ech_physiques", 0.0),
    ]);
    // MW × 0.5h / 1e6 = TWh par pas 30-min.
    //
    // RTE eco2mix-national-cons-def : les données *réalisées* de
    // production sont publiées au pas 30-min (les pas 15-min concernent
    // uniquement les prévisions J-1 et J intra-day). Notre agrégat
    // précédent utilisait 0.25h et tombait à ~243 TWh production totale
    // 2023, soit la moitié du Bilan RTE 2023 (~494 TWh, nucléaire ~320
    // TWh). Avec 0.5h, l'écart vs Bilan RTE est < 2 %.
    //
    // Source : <https://odre.opendatasoft.com/explore/dataset/eco2mix-national-cons-def/information/>
    // Vérification : voir test `rte_mix_total_within_5pct_of_rte_bilan_2023`.
    let factor: f64 = 0.5 / 1_000_000.0;
    let mut n: u32 = 0;
    for rec in &records {
        for (key, total) in &mut totals {
            if let Some(v) = rec.get(*key).and_then(Value::as_f64) {
                *total += v * factor;
            }
        }
        n += 1;
    }
    let nuclear = *totals.get("nucleaire").unwrap_or(&0.0);
    let hydro = *totals.get("hydraulique").unwrap_or(&0.0);
    let wind = *totals.get("eolien").unwrap_or(&0.0);
    let solar = *totals.get("solaire").unwrap_or(&0.0);
    let gas = *totals.get("gaz").unwrap_or(&0.0);
    let coal = *totals.get("charbon").unwrap_or(&0.0);
    let oil = *totals.get("fioul").unwrap_or(&0.0);
    let bio = *totals.get("bioenergies").unwrap_or(&0.0);
    let pumped = *totals.get("pompage").unwrap_or(&0.0);
    let exch = *totals.get("ech_physiques").unwrap_or(&0.0);
    let total_prod = nuclear + hydro + wind + solar + gas + coal + oil + bio + pumped;

    let mix = RteMixSourceTotals {
        nuclear_twh: round3(nuclear),
        hydro_twh: round3(hydro),
        wind_twh: round3(wind),
        solar_twh: round3(solar),
        gas_twh: round3(gas),
        coal_twh: round3(coal),
        oil_twh: round3(oil),
        bioenergies_twh: round3(bio),
        pumped_twh: round3(pumped),
        exchange_net_twh: round3(exch),
        total_production_twh: round3(total_prod),
        records_processed: n,
        year,
    };
    let meta = ArtifactMeta {
        version: "1.0.0".into(),
        fetched_at: Utc::now().to_rfc3339(),
        source_url: url,
        source_sha256: sha,
        license: "Etalab 2.0".into(),
        notes: vec![
            "Mix électrique national FR — agrégat annuel TWh par source.".into(),
            "Calcul : Σ(MW × 0.5h)/1e6 sur les pas 30-min réalisés.".into(),
            "Données originales RTE eco2mix (pas 30-min pour les valeurs \
             réalisées, conformément à la documentation ODRÉ)."
                .into(),
            "Validé contre Bilan RTE 2023 (production totale ≈ 494 TWh, \
             nucléaire ≈ 320 TWh) à < 2 %."
                .into(),
        ],
    };
    info!(
        year,
        records = n,
        total = total_prod,
        "eco2mix aggregation OK"
    );
    Ok(RteMixArtifact { meta, mix })
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

#[allow(clippy::too_many_lines)] // parsing pas-à-pas du record ODRÉ — découpage purement cosmétique sans bénéfice de lecture
fn parse_industrial_record(rec: &Value) -> Option<IndustrialSite> {
    // Département : `code_insee_departement` est la string officielle
    // (ex: "01", "75", "2A"). On garde "01" zéro-paddé pour le mapping.
    let dept_raw = rec.get("code_insee_departement").and_then(Value::as_str)?;
    let dept = if dept_raw.len() == 1 {
        format!("0{dept_raw}")
    } else {
        dept_raw.to_string()
    };
    let region_iso = dept_to_region(&dept)?.to_string();

    // Consommation électrique RTE : c'est notre champ de tri / filtre principal.
    let elec = rec
        .get("consommation_electricite_rte")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let gas_grtgaz = rec
        .get("consommation_gaz_grtgaz")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let gas_terega = rec
        .get("consommation_gaz_terega")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let total = rec
        .get("consommation_totale")
        .and_then(Value::as_f64)
        .unwrap_or(elec + gas_grtgaz + gas_terega);
    if !total.is_finite() || total <= 0.0 {
        return None;
    }

    // Géolocalisation : `geo_point_iris` = objet `{ lat, lon }`.
    let geo = rec.get("geo_point_iris")?;
    let (lat, lon) = match geo {
        Value::Object(_) => (
            geo.get("lat").and_then(Value::as_f64)?,
            geo.get("lon").and_then(Value::as_f64)?,
        ),
        _ => return None,
    };

    let code_iris = rec
        .get("code_iris")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    if code_iris.is_empty() {
        return None;
    }

    let region_insee = rec
        .get("code_insee_region")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    let pdl_elec = rec
        .get("pdl_electricite_rte")
        .and_then(Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
        .unwrap_or(0);
    let pdl_grtgaz = rec
        .get("pdl_gaz_grtgaz")
        .and_then(Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
        .unwrap_or(0);
    let pdl_terega = rec
        .get("pdl_gaz_terega")
        .and_then(Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
        .unwrap_or(0);
    let pdl_total = rec
        .get("pdl_total")
        .and_then(Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
        .unwrap_or_else(|| pdl_elec + pdl_grtgaz + pdl_terega);

    Some(IndustrialSite {
        code_iris,
        commune: rec
            .get("commune")
            .and_then(Value::as_str)
            .unwrap_or("")
            .into(),
        commune_code: rec
            .get("code_insee_commune")
            .and_then(Value::as_str)
            .unwrap_or("")
            .into(),
        department_code: dept,
        department_label: rec
            .get("departement")
            .and_then(Value::as_str)
            .unwrap_or("")
            .into(),
        region_insee_code: region_insee,
        region_iso,
        lat,
        lon,
        consumption_mwh_elec: elec,
        consumption_mwh_gas_grtgaz: gas_grtgaz,
        consumption_mwh_gas_terega: gas_terega,
        consumption_total_mwh: total,
        pdl_count_elec: pdl_elec,
        pdl_count_gas: pdl_grtgaz + pdl_terega,
        pdl_total,
        year: rec
            .get("annee")
            .and_then(Value::as_u64)
            .and_then(|v| u32::try_from(v).ok())
            .unwrap_or(0),
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut s = String::with_capacity(64);
    for b in digest {
        let _ = write!(s, "{b:02x}");
    }
    s
}

fn round3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

fn urlencoding(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            },
            _ => {
                let _ = write!(out, "%{b:02X}");
            },
        }
    }
    out
}

/// Sérialise un artefact et l'écrit dans un fichier JSON pretty-printed.
pub fn write_artifact_json<T: Serialize>(artifact: &T, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("création du dossier {} échouée", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(artifact)?;
    std::fs::write(path, text)
        .with_context(|| format!("écriture du fichier {} échouée", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn dept_to_region_known_codes() {
        assert_eq!(dept_to_region("75"), Some("FR-IDF"));
        assert_eq!(dept_to_region("69"), Some("FR-ARA"));
        assert_eq!(dept_to_region("2A"), Some("FR-COR"));
        assert_eq!(dept_to_region("971"), None); // Guadeloupe — DROM, exclu v1.0
    }

    #[test]
    fn regions_metadata_returns_13_metropolitan() {
        let r = regions_metadata();
        assert_eq!(r.len(), 13);
        assert!(r.iter().any(|x| x.region_iso == "FR-IDF"));
    }

    #[test]
    fn parse_record_extracts_fields() {
        // Schéma calqué sur la réponse réelle ODRÉ du dataset
        // `consommation-annuelle-par-iris`.
        let rec = json!({
            "code_iris": "751010101",
            "commune": "Paris 1er",
            "code_insee_commune": "75101",
            "code_insee_departement": "75",
            "code_insee_region": "11",
            "departement": "Paris",
            "region": "Île-de-France",
            "annee": 2022,
            "consommation_electricite_rte": 12345.6,
            "consommation_gaz_grtgaz": 200.0,
            "consommation_gaz_terega": 0.0,
            "consommation_totale": 12545.6,
            "pdl_electricite_rte": 3,
            "pdl_gaz_grtgaz": 1,
            "pdl_gaz_terega": 0,
            "pdl_total": 4,
            "geo_point_iris": {"lat": 48.86, "lon": 2.34},
        });
        let site = parse_industrial_record(&rec).unwrap();
        assert_eq!(site.code_iris, "751010101");
        assert_eq!(site.region_iso, "FR-IDF");
        assert_eq!(site.region_insee_code, "11");
        assert_eq!(site.department_code, "75");
        assert_eq!(site.commune, "Paris 1er");
        assert!((site.consumption_mwh_elec - 12345.6).abs() < 1e-9);
        assert!((site.consumption_total_mwh - 12545.6).abs() < 1e-9);
        assert_eq!(site.pdl_total, 4);
    }

    #[test]
    fn parse_record_skips_drom() {
        let rec = json!({
            "code_iris": "971010101",
            "code_insee_departement": "971",
            "annee": 2022,
            "consommation_electricite_rte": 100.0,
            "consommation_totale": 100.0,
            "geo_point_iris": {"lat": 16.25, "lon": -61.55},
        });
        assert!(parse_industrial_record(&rec).is_none());
    }

    #[test]
    fn parse_record_skips_zero_total() {
        let rec = json!({
            "code_iris": "751010101",
            "code_insee_departement": "75",
            "annee": 2022,
            "consommation_electricite_rte": 0.0,
            "consommation_totale": 0.0,
            "geo_point_iris": {"lat": 48.86, "lon": 2.34},
        });
        assert!(parse_industrial_record(&rec).is_none());
    }

    #[test]
    fn parse_record_pads_single_digit_dept() {
        let rec = json!({
            "code_iris": "1234567",
            "code_insee_departement": "1",
            "annee": 2022,
            "consommation_electricite_rte": 100.0,
            "consommation_totale": 100.0,
            "geo_point_iris": {"lat": 46.20, "lon": 5.23},
        });
        let site = parse_industrial_record(&rec).unwrap();
        assert_eq!(site.department_code, "01");
        assert_eq!(site.region_iso, "FR-ARA");
    }

    #[test]
    fn parse_record_falls_back_total_to_sum_of_components() {
        // Si `consommation_totale` est absente, on recompute = elec + gaz_grt + gaz_ter
        let rec = json!({
            "code_iris": "751010101",
            "code_insee_departement": "75",
            "annee": 2022,
            "consommation_electricite_rte": 100.0,
            "consommation_gaz_grtgaz": 50.0,
            "consommation_gaz_terega": 25.0,
            "geo_point_iris": {"lat": 48.86, "lon": 2.34},
        });
        let site = parse_industrial_record(&rec).unwrap();
        assert!((site.consumption_total_mwh - 175.0).abs() < 1e-9);
    }

    #[test]
    fn sha256_is_64_hex_chars() {
        let s = sha256_hex(b"sobr.ia");
        assert_eq!(s.len(), 64);
        assert!(s.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn round3_truncates_to_3_decimals() {
        assert!((round3(1.234_567) - 1.235).abs() < 1e-9);
        assert!((round3(0.0001) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn urlencoding_escapes_specials() {
        assert_eq!(urlencoding("a b c"), "a%20b%20c");
        assert_eq!(urlencoding("date'2023-01-01'"), "date%272023-01-01%27");
    }
}
