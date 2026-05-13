# Chantier #13 — M20 Territoire FR + Sankey énergétique

> **Pré-requis** : v0.2.2-datacenters mergé.
> **Crates touchées** : `sobria-geoloc` (territoire + Sankey), `sobria-app`
> (DTOs + IPC).
> **Frontend** : `web/src/routes/(modules)/m20/+page.svelte` — chantier
> Claude Code séparé.
> **Durée cible** : 1-2 jours Rust.
> **Référence CDC** : v1.3 §4 M20.

---

## 0. Objectif

Apporter l'**angle territorial français** unique du dossier candidature :
croiser les données RTE IRIS (consommation industrielle par maille
géographique) avec ComparIA (usage LLM) pour produire :

1. **Carte des 200 sites industriels** top consommateurs en France
   (cluster zoom-dépendant, drill-down par site).
2. **Agrégats par région** (13 régions ADMIN1) avec leur mix énergétique.
3. **Sankey énergétique** : flux Production (mix RTE) → Datacenter FR
   → Famille LLM (GPT / Claude / Mistral / Llama / Gemini).
4. **Scénarios régionaux** : "si on déplace X% du trafic IA vers la
   région Y, impact CO₂eq ?".

## 1. Données — origine et flux

**Politique** : aucune donnée inventée. Toutes les valeurs proviennent
de datasets ODRÉ officiels (Etalab 2.0), avec traçabilité SHA-256
+ URL source + timestamp UTC.

### 1.0 Architecture du fetch (ADR-0009)

Conformément à ADR-0009, **seul `sobria-ingest` parle aux APIs externes**.
Trois modes de téléchargement exposés :

1. **CLI** : `cargo run -p sobria-ingest -- fetch territoire-fr [--limit 200]`
   et `fetch rte-mix [--year 2023]`.
2. **Bootstrap auto** : `scripts/bootstrap.sh` exécute les deux fetch s'ils
   manquent à l'installation.
3. **In-app (v1.1)** : commande IPC `fetch_official_dataset` exposée par
   `sobria-app` pour un bouton « Télécharger les données officielles »
   dans l'onboarding / Paramètres (à câbler après validation des deux
   premiers modes).

### 1.1 Sites industriels (RTE IRIS — Etalab 2.0)

Source : ODRÉ — RTE, NaTran, Teréga « Consommation IRIS sites industriels ».

Endpoint : <https://odre.opendatasoft.com/api/explore/v2.1/catalog/datasets/consommation-electrique-par-secteur-dactivite-iris/records>

Sortie : `crates/sobria-geoloc/data/territoire_fr.json` (top 200 par
conso élec décroissante, mapping département → région ISO INSEE 2024,
DROM exclus en v1.0).

Champs par site :

```rust
pub struct IndustrialSiteRecord {
    pub code_iris: String,           // ex: "751010101" (Paris 1er, Saint-Germain)
    pub name: String,                // libellé site / commune
    pub commune: String,
    pub department_code: String,     // 01..95, 971..976
    pub region_iso: String,          // FR-IDF, FR-ARA, ... (ADMIN1)
    pub lat: f64,
    pub lon: f64,
    pub consumption_mwh_elec: f64,   // annuel
    pub consumption_mwh_gas: Option<f64>,
    pub sector: String,              // "Métallurgie", "Chimie", ...
    pub year: u16,                   // année de référence (2022 typique)
    pub source: String,              // URL ODRÉ
}
```

### 1.2 Régions FR (13 régions ADMIN1)

Liste figée :

| Code ISO | Nom | INSEE | Mix élec part nucléaire |
|---|---|---|---|
| FR-IDF | Île-de-France | 11 | importée — mix national |
| FR-ARA | Auvergne-Rhône-Alpes | 84 | 75% (parc Bugey, Cruas, etc.) |
| FR-NAQ | Nouvelle-Aquitaine | 75 | 60% (Civaux, Blayais) |
| FR-NOR | Normandie | 28 | 90% (Paluel, Penly, Flamanville) |
| FR-OCC | Occitanie | 76 | 50% (Golfech, Tricastin) |
| FR-PAC | Provence-Alpes-Côte d'Azur | 93 | 0% — solaire + hydro + gaz |
| FR-HDF | Hauts-de-France | 32 | 80% (Gravelines) |
| FR-BFC | Bourgogne-Franche-Comté | 27 | 0% — hydro + import |
| FR-GES | Grand Est | 44 | 80% (Cattenom, Fessenheim arrêté) |
| FR-CVL | Centre-Val-de-Loire | 24 | 90% (Belleville, Dampierre, Chinon, St-Laurent) |
| FR-PDL | Pays de la Loire | 52 | 0% — éolien + thermique gaz |
| FR-BRE | Bretagne | 53 | 0% — éolien + hydro + import |
| FR-COR | Corse | 94 | 0% — thermique fioul (mix peu décarboné) |

(DROM : Guadeloupe, Martinique, Guyane, Réunion, Mayotte — v1.1.)

## 2. Sankey énergétique

### 2.1 Modèle

Trois couches, flux quantifiés en GWh/an (estimés pour la France) :

```
[ Production (5) ]   →   [ Datacenter FR (4) ]   →   [ Famille LLM (5) ]
  Nucléaire 70%               OVH Roubaix              GPT (OpenAI)
  Hydro 12%                   OVH Gravelines           Claude (Anthropic)
  Eolien 10%                  Scaleway Paris           Mistral
  Solaire 4%                  AWS Paris                Llama (Meta)
  Gaz/Autre 4%                                         Gemini (Google)
```

### 2.2 Sources et hypothèses

- **Mix production** : RTE Bilan électrique 2024 (CC-BY).
- **Allocation DC** : pondérée par la capacité publique des 4 DC FR
  (cf. dataset M12). À défaut de mesure : répartition uniforme.
- **Allocation LLM family** : extrapolée depuis ComparIA usage shares
  (votes 2024). Par défaut estimée :
  - GPT : 45%, Claude : 22%, Mistral : 14%, Llama : 11%, Gemini : 8%.
- **Volume total annuel FR** : ~1.5 TWh estimé pour l'inférence LLM en
  France (calibration par ordre de grandeur — sources : Luccioni 2023
  pour intensité par token + estimations Mistral du volume FR 2024).

### 2.3 Conservation des flux

Invariants validés par les tests :
- Pour chaque couche n, `Σ(sortants_couche_n) == Σ(entrants_couche_n+1)`.
- Pas de cycle (graphe DAG strict).
- `Σ(production) == Σ(usage_par_famille_LLM) == volume_total`.

## 3. Surface IPC

```rust
#[tauri::command]
fn list_industrial_sites_fr(limit: u32, offset: u32)
    -> IpcResult<Vec<IndustrialSiteSummaryDto>>;

#[tauri::command]
fn aggregate_iris_by_region()
    -> IpcResult<Vec<RegionFrAggregateDto>>;

#[tauri::command]
fn get_region_fr_detail(region_iso: String)
    -> IpcResult<RegionFrDetailDto>;

#[tauri::command]
fn sankey_fr_data() -> IpcResult<SankeyDataDto>;
```

**`RegionFrDetailDto`** :
```rust
pub struct RegionFrDetailDto {
    pub region_iso: String,
    pub name: String,
    pub total_consumption_mwh_elec: f64,
    pub total_consumption_mwh_gas: f64,
    pub industrial_sites_count: usize,
    pub nuclear_share_pct: f64,
    pub centroid_lat: f64,
    pub centroid_lon: f64,
    pub top_sites: Vec<IndustrialSiteSummaryDto>, // top 5
}
```

**`SankeyDataDto`** :
```rust
pub struct SankeyDataDto {
    pub nodes: Vec<SankeyNodeDto>,
    pub links: Vec<SankeyLinkDto>,
    /// Volume total en GWh/an utilisé pour la calibration.
    pub total_gwh: f64,
    pub sources: Vec<String>,
}

pub struct SankeyNodeDto {
    pub id: String,        // "prod-nuclear"
    pub label: String,     // "Nucléaire"
    pub layer: u8,         // 0=prod, 1=DC, 2=LLM
    pub value_gwh: f64,
}

pub struct SankeyLinkDto {
    pub source: String,    // id node source
    pub target: String,    // id node target
    pub value_gwh: f64,
}
```

## 4. Definition of Done

### Rust
- [ ] `sobria-geoloc/data/industrial_sites_fr.json` : 200 sites.
- [ ] `sobria-geoloc/src/territoire_fr.rs` : types + helpers
      `all_industrial_sites_fr`, `find_site_by_code_iris`,
      `regions_fr`, `aggregate_by_region_fr`.
- [ ] `sobria-geoloc/src/sankey_fr.rs` : `generate_sankey_fr()`,
      `SankeyNode`, `SankeyLink`, `SankeyData`.
- [ ] DTOs + 4 commandes IPC enregistrées.
- [ ] ≥ 15 tests (chargement dataset, agrégation, Sankey conservation).
- [ ] `cargo clippy -p sobria-geoloc -p sobria-app -- -D warnings` propre.

### Doc
- [ ] `docs/sources/CATALOGUE-TERRITOIRE-FR.md` : sources RTE/ODRÉ +
      mix régionaux + méthodologie Sankey.
- [ ] Note dans CDC §4.3 ajoutant le composant Sankey à M20.

## 5. Non-objectifs (différés)

- **Tous les 50K IRIS** (pas seulement industriels) → backlog v1.1.
- **DROM** (Outre-mer) → backlog v1.1.
- **Live pull RTE eco2mix horaire** → backlog v1.1.
- **Simulation de scénario "déplacer X% du trafic"** → frontend
  (utilise `estimate_prompt` avec paramètres régionaux différents).

## 6. Risques

| Risque | Probabilité | Parade |
|---|---|---|
| Sites RTE non publics / opacité | Moyen | Sélection top 200 communément cités, sources documentées |
| Sankey trop synthétique pour être crédible | Moyen | Sources explicites + tooltip méthodologie |
| Allocation LLM family imprécise | Élevé | Bandes d'incertitude affichées par le frontend |
| Mix régional non-linéaire (échanges inter-régions) | Moyen | Documenter l'hypothèse "mix régional = production régionale" |

---

*Brief Cowork. Exécution C13.1 (territoire_fr), C13.2 (sankey_fr),
C13.3 (DTOs + IPC), C13.4 (tests + doc). Frontend M20 chantier Claude Code.*
