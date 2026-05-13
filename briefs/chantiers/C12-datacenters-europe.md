# Chantier #12 — M12 Datacenters Europe (carte + drill-down)

> **Pré-requis** : v0.2.0-estimer mergé, C10 frontend en cours, C11 Rust mergé.
> **Crates touchées** : `sobria-geoloc` (dataset + helpers), `sobria-app`
> (DTOs + IPC).
> **Frontend** : `web/src/routes/(modules)/m12/+page.svelte` — Leaflet
> (chantier Claude Code séparé).
> **Durée cible** : 1 jour (Rust) + 1-2 jours (frontend).
> **Référence CDC** : v1.3 §4 M12.

---

## 0. Objectif

Cartographier les **28 principaux datacenters européens** servant
l'inférence LLM (hyperscalers + opérateurs souverains), avec :

- vue agrégée par pays au zoom Europe,
- markers individuels au zoom pays/ville,
- drill-down par site : **donut** (composition mix élec) + **barres**
  (PUE, WUE, IF), + **profil 24h** (charge typique).

L'objectif n'est pas l'exhaustivité (il y a >800 DC en Europe) mais la
**représentativité** : les 28 sélectionnés couvrent ≥ 90% de la capacité
LLM accessible publiquement aux utilisateurs européens.

## 1. Sélection des 28 datacenters

Critères : (a) hébergent au moins une famille LLM majeure (OpenAI/AWS,
Anthropic, Google, Microsoft, Mistral, Meta) ou (b) sont représentatifs
d'un mix énergétique national pour comparaison.

Répartition cible :

| Pays | DC | Justification |
|---|---|---|
| 🇫🇷 FR | 4 | Souveraineté + mix décarboné (56 g/kWh) |
| 🇩🇪 DE | 4 | Hub européen, mix charbon dégressif (386 g/kWh) |
| 🇮🇪 IE | 3 | Hub hyperscalers EU, mix mixte (296 g/kWh) |
| 🇳🇱 NL | 3 | Gaz dominant, GCP europe-west4 (314 g/kWh) |
| 🇬🇧 GB | 3 | Eolien offshore + gaz (245 g/kWh) |
| 🇸🇪 SE | 2 | Hydro+nuclear, ultra-décarboné (41 g/kWh) |
| 🇫🇮 FI | 2 | Hub froid + nucléaire (132 g/kWh) |
| 🇪🇸 ES | 2 | Solaire montant (143 g/kWh) |
| 🇮🇹 IT | 1 | Gaz dominant (354 g/kWh) |
| 🇵🇱 PL | 1 | Charbon dominant (633 g/kWh) — pire cas |
| 🇨🇭 CH | 1 | Hydro+nucléaire (47 g/kWh) |
| 🇦🇹 AT | 1 | Hydro dominant (89 g/kWh) |
| 🇩🇰 DK | 1 | Eolien dominant (151 g/kWh) |

**Total : 28.**

L'IF (gCO₂eq/kWh) est la moyenne annuelle pays 2023 selon Electricity
Maps + AIB European Residual Mix. Source citée dans chaque record.

## 2. Structure de données

### 2.1 `DatacenterRecord` (sobria-geoloc)

```rust
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DatacenterRecord {
    /// Identifiant stable (snake_case, ex: "aws-eu-west-3-paris").
    pub id: String,
    /// Nom commercial (ex: "AWS Europe (Paris)").
    pub name: String,
    /// Opérateur (AWS, GCP, Azure, OVH, Scaleway, Equinix, …).
    pub operator: String,
    /// Code ISO 3166-1 alpha-2 (ex: "FR").
    pub country_iso: String,
    /// Ville (libellé localisé en FR).
    pub city: String,
    /// Latitude WGS84.
    pub lat: f64,
    /// Longitude WGS84.
    pub lon: f64,
    /// PUE annuel déclaré ou estimé (1.0 = parfait, typique 1.05-1.6).
    pub pue: f64,
    /// WUE en L/kWh IT, si publié.
    pub wue_l_per_kwh: Option<f64>,
    /// IF élec local en gCO₂eq/kWh (moyenne annuelle pays).
    pub if_electrical_g_per_kwh: f64,
    /// Capacité IT en MW, si publiée.
    pub capacity_mw: Option<f64>,
    /// URLs ou références sources (≥ 1 obligatoire).
    pub sources: Vec<String>,
    /// Profil horaire normalisé sur 24h (0.0..=1.0, somme libre).
    /// Source : ENTSO-E load average annuel pays, ou pattern générique
    /// si pas dispo. Utilisé par le drill-down "24h".
    pub hourly_profile_24h: [f64; 24],
}
```

### 2.2 Dataset embarqué

Le dataset est un fichier JSON statique `crates/sobria-geoloc/data/datacenters.json`
inclus via `include_str!()` au build. Cela permet :

- pas de fichier externe à charger au runtime,
- versionnage Git du dataset,
- lisibilité pour les contributions/PR,
- déserialisation paresseuse via `OnceLock` au premier appel.

Aucune dépendance HTTP runtime — c'est conforme à la promesse
"0 clé API bloquante" du CDC §3bis.

## 3. API publique

### 3.1 Module `sobria-geoloc/src/datacenters.rs`

```rust
/// Retourne le dataset complet (référence statique, lazy-init).
pub fn all_datacenters() -> &'static [DatacenterRecord];

/// Cherche un datacenter par ID (insensible à la casse).
pub fn find_datacenter(id: &str) -> Option<&'static DatacenterRecord>;

/// Aggrège par pays. Retourne une map country_iso → CountryAggregate.
pub fn aggregate_by_country() -> Vec<CountryAggregate>;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CountryAggregate {
    pub country_iso: String,
    pub datacenter_count: usize,
    pub avg_pue: f64,                      // moyenne pondérée capacité
    pub if_electrical_g_per_kwh: f64,      // mix élec pays
    pub total_capacity_mw: Option<f64>,
    /// Centroïde géographique des DC du pays (lat, lon).
    pub centroid: (f64, f64),
}
```

### 3.2 Commandes IPC (sobria-app)

```rust
#[tauri::command]
fn list_datacenters() -> IpcResult<Vec<DatacenterSummaryDto>>;

#[tauri::command]
fn get_datacenter_detail(id: String) -> IpcResult<DatacenterDetailDto>;

#[tauri::command]
fn aggregate_datacenters_by_country() -> IpcResult<Vec<CountryAggregateDto>>;
```

**DatacenterSummaryDto** (pour markers sur la carte) :
```rust
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
```

**DatacenterDetailDto** (pour drill-down) :
```rust
pub struct DatacenterDetailDto {
    // Tout DatacenterSummaryDto plus :
    pub wue_l_per_kwh: Option<f64>,
    pub capacity_mw: Option<f64>,
    pub sources: Vec<String>,
    pub hourly_profile_24h: Vec<f64>,    // 24 valeurs
    /// Indicateurs simulés sur 1 prompt standard pour ce DC :
    /// gpt-4o-mini 100/500 tokens, PUE et IF du DC, embodied moyen.
    /// Permet au frontend de remplir les "barres" sans rappeler estimate_prompt.
    pub baseline_co2eq_p50_g: f64,
    pub baseline_energy_wh_p50: f64,
    pub baseline_water_l_p50: f64,
}
```

`baseline_co2eq_p50_g` est calculé côté Rust en appelant l'estimateur
avec les params du DC. Cela évite que le frontend doive ré-orchestrer
N appels `estimate_prompt` pour remplir les barres.

## 4. Profil 24h — méthodologie

Source primaire : **ENTSO-E Transparency Platform** (open data, no key)
profils de charge horaire annuel moyen par pays.

Pour MVP v1.0 : on embarque les profils normalisés (0.0-1.0, indexés
sur le max horaire annuel) sous forme de 24 valeurs flottantes, une
ligne par pays dans le JSON. Source citée.

Refactor futur (v1.1) : pull live ENTSO-E si la connectivité est OK,
fallback statique sinon.

## 5. Definition of Done

### Rust
- [ ] `sobria-geoloc/data/datacenters.json` avec 28 DC + 13 profils
      horaires (un par pays).
- [ ] `sobria-geoloc/src/datacenters.rs` : `DatacenterRecord`,
      `CountryAggregate`, `all_datacenters`, `find_datacenter`,
      `aggregate_by_country`.
- [ ] Tests : dataset chargeable, 28 DC exactement, aucun ID en
      doublon, toutes coordonnées dans bbox Europe (lat [35, 71],
      lon [-10, 35]), tous country_iso valides, tous profils horaires
      ont exactement 24 valeurs ∈ [0.0, 1.0].
- [ ] DTOs dans `sobria-app/src/dto.rs`.
- [ ] 3 commandes IPC enregistrées.
- [ ] Tests `logic::tests` : list non vide, get unknown_id rejeté.
- [ ] `cargo clippy -p sobria-geoloc -p sobria-app -- -D warnings` propre.

### Doc
- [ ] `docs/sources/CATALOGUE-DATACENTERS.md` : tableau des 28 DC avec
      sources URL (Electricity Maps, AIB, ENTSO-E, rapports DC opérateurs).

## 6. Non-objectifs (différés)

- **DC > 28** → v1.1, sélection communautaire.
- **Pull ENTSO-E live** → v1.1.
- **GeoJSON polygones pays** → côté frontend (Leaflet + naturalearth-data).
- **Cross-référence ComparIA** (quel DC pour quelle conversation) →
  M20 Territoire FR.

## 7. Risques

| Risque | Probabilité | Parade |
|---|---|---|
| Données PUE/WUE volatiles (rapports annuels) | Moyenne | Date de la valeur dans `sources`, refresh manuel chaque année |
| 28 DC = arbitraire, contestable | Moyenne | Doc claire des critères de sélection (§1), liste ouverte aux PR |
| Profils 24h synthétiques non représentatifs | Faible | Source ENTSO-E, fallback documenté |
| Bundle binaire grossit (~10 KB JSON) | Très faible | Acceptable au regard de la valeur produit |

---

*Brief rédigé par Cowork. Exécution C12.1 (sobria-geoloc data+code),
C12.2 (DTOs+IPC), C12.3 (tests + doc sources). Frontend M12 = chantier
séparé Claude Code après livraison Rust.*
