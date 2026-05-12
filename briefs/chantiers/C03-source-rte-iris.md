# Chantier #3 — Source RTE / NaTran / Teréga IRIS

> **Pré-requis** : chantier #1 (foundation) + chantier #2 (ComparIA) mergés sur main.
> **Crates touchées** : `sobria-ingest` uniquement (+ schémas JSON).
> **Durée cible** : 1-2 jours.
> **Approche** : ré-exploitation directe de la foundation, ajustements mineurs vs ComparIA (CSV au lieu de Parquet).

---

## 0. Pourquoi RTE IRIS en deuxième

Deuxième dataset officiel du défi data.gouv.fr (voir `docs/sources/CATALOGUE-SOURCES.md` S02). Il apporte la **dimension territoriale française** au projet :

- Consommation industrielle annuelle d'électricité et de gaz (MWh) par maille IRIS.
- Référentiel IRIS 2023 de l'INSEE — ~50 000 mailles couvrant tout le territoire français.
- Permet le module M12 (Territoire français) avec carte choroplèthe.

Producteurs : NaTran, RTE, Teréga (via ODRÉ — Open Data Réseaux Énergies).
Licence : Licence Ouverte Etalab 2.0.

---

## 1. Données à ingérer

Voir catalogue S02. Quatre formats publiés, on en retient deux :

| Fichier | Format | Taille | Usage |
|---------|--------|--------|-------|
| `consommation_iris.csv` | CSV | 90 MB | **Silver** : table tabulaire principale |
| `iris_geometries.geojson` | GeoJSON | 91 MB | **Copper uniquement** : ressource cartographique pour M12 |

On ignore le JSON tabulaire (redondant avec CSV) et le Shapefile (redondant avec GeoJSON).

**URLs data.gouv.fr** :
- CSV : `r/631d6ec4-74c5-4f0f-9187-442cf9d1f0bc`
- GeoJSON : `r/2b584d52-f4e0-4232-87df-c456e715f334`

---

## 2. Conception

### 2.1 Stratégie Silver — passthrough CSV

Comme pour ComparIA v1, le schéma Silver est **permissif** :

- Lecture CSV via `polars::LazyCsvReader`.
- Ajout des deux colonnes systématiques : `_copper_sha256` + `_ingested_at`.
- Toutes les colonnes RTE/NaTran/Teréga d'origine sont conservées.
- Une seule entité Silver : `rte_iris_consommation`.

### 2.2 Le GeoJSON reste en Copper

Le GeoJSON est une **ressource cartographique statique** consommée par le module M12 (rendu carto). Il ne contient pas de données analytiques au sens médaillon — il définit des géométries. On le garde donc en Copper (téléchargé, hashé, manifeste), accessible directement par chemin pour le module M12 plus tard.

Cette décision est défendable :
- Le GeoJSON pèse 91 MB, le re-promouvoir en Silver serait redondant.
- Polars 0.46 n'a pas de parseur GeoJSON natif — l'effort d'intégration n'apporterait rien tant qu'on n'en exploite pas les géométries en analyse.
- Le manifest Copper contient déjà le hash GeoJSON, donc la traçabilité est préservée.

Quand le module M12 sera implémenté, il lira directement le GeoJSON depuis `data/copper/rte-iris/<date>/iris_geometries.geojson`.

### 2.3 Architecture

```
crates/sobria-ingest/src/sources/
├── mod.rs              ← ajoute pub mod rte_iris;
├── comparia.rs         ← existant
└── rte_iris.rs         ← nouveau
```

### 2.4 Flux

```
ingest_copper
  ├─ télécharger consommation_iris.csv → Copper
  ├─ télécharger iris_geometries.geojson → Copper
  ├─ écrire manifest.json avec les 2 fichiers
  └─ retourner CopperSnapshot (2 CopperRef)

promote_silver
  ├─ lire le CSV via polars LazyCsvReader (spawn_blocking)
  ├─ enrichir avec _copper_sha256 + _ingested_at
  ├─ écrire rte_iris_consommation-v1.parquet
  └─ retourner Vec<SilverEntity> (1 entité — le GeoJSON n'est pas promu)

contribute_gold
  └─ déclarer la table touchée + notes méthodologiques (origine ODRÉ, IRIS 2023)
```

### 2.5 Polars CSV en contexte async

API polars 0.46 :
```rust
use polars::prelude::*;
let lf = LazyCsvReader::new(path).with_has_header(true).finish()?;
```

Comme pour Parquet, on enveloppe dans `spawn_blocking`.

---

## 3. Schéma Silver v1

`schemas/silver/rte_iris_consommation-v1.json` — même modèle laxe que ComparIA :

- `_copper_sha256` (string, 64 hex) — lineage.
- `_ingested_at` (datetime ISO 8601).
- Autres colonnes : `additionalProperties: true`.

Le bump v1 → v2 (avec typage strict des colonnes attendues : `code_iris`, `conso_elec_mwh`, etc.) interviendra quand on aura confirmé le schéma RTE par observation directe.

---

## 4. Definition of Done

- [ ] `cargo build --workspace` passe.
- [ ] `cargo clippy --workspace -- -D warnings` passe.
- [ ] `cargo test --workspace --all-features` passe.
- [ ] `RteIrisSource` enregistrable dans le registry.
- [ ] 1 entité Silver produite (`rte_iris_consommation`) avec lineage propagé.
- [ ] GeoJSON présent en Copper, accessible par chemin.
- [ ] Tests : ingest_copper (wiremock servant CSV + GeoJSON), promote_silver (CSV synthétique), bout en bout.

---

## 5. Non-objectifs (reportés)

- Parsing du GeoJSON et écriture en Parquet Silver (sera fait au chantier M12).
- Validation cross-fichier code_iris (cohérence CSV ↔ GeoJSON).
- Téléchargement parallèle des deux fichiers.

---

## 6. Risques

| Risque | Mitigation |
|--------|-----------|
| CSV RTE encodé en latin-1 ou avec séparateur `;` | `LazyCsvReader` configure encoding + separator |
| GeoJSON trop gros pour wiremock en test | Test avec mini GeoJSON valide (1 feature) |
| Schéma CSV évolue d'une année à l'autre | Silver v1 permissif absorbe |
