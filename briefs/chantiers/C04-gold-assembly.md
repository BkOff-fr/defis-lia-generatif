# Chantier #4 — Assemblage de la couche Gold

> **Pré-requis** : chantiers #1, #2, #3 mergés.
> **Crates touchées** : `sobria-ingest` uniquement.
> **Durée cible** : 1-2 jours.

---

## 0. Objectif

Le pipeline médaillon produit, en sortie, **quatre artefacts** dans `data/gold/` :

| Artefact | Format | Rôle |
|----------|--------|------|
| `referentiel.sqlite` | SQLite WAL | Lecture rapide par l'app Tauri (ADR-0003) |
| `analytics.parquet` | Parquet | Lecture DuckDB pour scénarios (ADR-0003) |
| `datasheet.jsonld` | JSON-LD | Datasheet (Gebru et al. 2018) — PROV-O |
| `MANIFEST.sha256` | texte | Hashes SHA-256 de tous les artefacts |

Le chantier consiste à écrire ces artefacts à partir des `SilverEntity` produites par les sources, en propageant le lineage et les métadonnées.

---

## 1. Périmètre

### En périmètre

- Module `gold` dans `sobria-ingest` avec les 4 fonctions d'écriture.
- Orchestration depuis `LayerRegistry::run_full_pipeline`.
- `PipelineReport` enrichi avec les chemins des 4 artefacts.
- Schéma `referentiel.sqlite` minimal mais utile (sources, silver_entities, lineage).
- Tests d'intégration de bout en bout avec MockSource.

### Hors périmètre (v2)

- **Jointures inter-sources métier** : par exemple croiser un modèle ComparIA avec son datacenter ADEME. Sera fait dans un chantier suivant quand on aura le mapping métier précis.
- **Vues matérialisées DuckDB analytiques**.
- **Signature GPG** du `MANIFEST.sha256`.
- **Tables typées** par entité (en v1, on garde une approche méta : on liste les entités, on ne dénormalise pas le contenu).

---

## 2. Conception

### 2.1 `referentiel.sqlite` v1

Trois tables minimales :

```sql
CREATE TABLE sources (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    license TEXT NOT NULL,
    update_frequency TEXT NOT NULL,
    tier INTEGER NOT NULL
);

CREATE TABLE silver_entities (
    entity_name TEXT NOT NULL,
    source_id TEXT NOT NULL,
    schema_version TEXT NOT NULL,
    parquet_path TEXT NOT NULL,
    row_count INTEGER NOT NULL,
    PRIMARY KEY (entity_name, source_id),
    FOREIGN KEY (source_id) REFERENCES sources(id)
);

CREATE TABLE lineage (
    copper_sha256 TEXT NOT NULL,
    silver_entity TEXT NOT NULL,
    source_id TEXT NOT NULL,
    file_name TEXT NOT NULL,
    PRIMARY KEY (copper_sha256, silver_entity)
);
```

**Mode WAL** activé pour permettre des lectures concurrentes pendant les écritures (utile à l'app Tauri qui pourrait lire pendant qu'un re-pipeline tourne).

L'app pourra ainsi répondre à des questions comme :
- « Quelles sources composent le Gold ? » → `SELECT * FROM sources`.
- « Combien de lignes ComparIA ? » → `SELECT row_count FROM silver_entities WHERE source_id = 'comparia'`.
- « Quels fichiers Copper composent le Silver `comparia_votes` ? » → `SELECT * FROM lineage WHERE silver_entity = 'comparia_votes'`.

### 2.2 `analytics.parquet` v1

Format : **catalogue tabulaire** des entités Silver.

| entity_name | source_id | schema_version | row_count | copper_sha256_list |
|-------------|-----------|----------------|-----------|---------------------|
| `comparia_conversations` | `comparia` | `v1` | 250000 | `7a3f...` |
| `rte_iris_consommation` | `rte-iris` | `v1` | 49872 | `9d3e...` |

C'est une vue méta exploitable en DuckDB :
```sql
SELECT * FROM 'analytics.parquet' WHERE source_id = 'comparia';
```

La v2 ajoutera de vraies tables analytiques (sommes par mois, par modèle…).

### 2.3 `datasheet.jsonld`

On réutilise `GoldLineage::to_jsonld()` déjà implémenté en chantier #1. Sortie : JSON-LD compatible PROV-O + schema.org/Dataset, indenté lisible.

### 2.4 `MANIFEST.sha256`

Format `sha256sum` standard, une ligne par fichier :

```
7a3f9b...  referentiel.sqlite
2c8e0a...  analytics.parquet
b91d44...  datasheet.jsonld
```

Permet la vérification d'intégrité via `sha256sum --check MANIFEST.sha256`.

---

## 3. Definition of Done

- [ ] `cargo build --workspace` passe.
- [ ] `cargo clippy --workspace -- -D warnings` passe.
- [ ] `cargo test --workspace` passe.
- [ ] `LayerRegistry::run_full_pipeline` produit les 4 fichiers Gold.
- [ ] `referentiel.sqlite` contient 3 tables avec données cohérentes.
- [ ] `analytics.parquet` lisible par polars/DuckDB.
- [ ] `datasheet.jsonld` parsable JSON-LD avec contexte PROV-O.
- [ ] `MANIFEST.sha256` au format `sha256sum`.

---

## 4. Risques

| Risque | Mitigation |
|--------|-----------|
| rusqlite + Tokio mix (rusqlite est blocking) | `spawn_blocking` pour les writes SQLite |
| Polars `df!` macro échoue sur vec vide | Garantir au moins 1 entité Silver pour écrire, sinon skip + log |
| Concurrence accès SQLite | Mode WAL + transactions courtes |
