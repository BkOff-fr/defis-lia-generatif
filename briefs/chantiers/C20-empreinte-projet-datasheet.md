# Chantier #20 — M17 Empreinte projet (datasheet Gebru)

> **Pré-requis** : v0.2.8-dashboard mergé.
> **Crates touchées** : `sobria-app` (new `project_store`, DTOs, IPCs),
> `sobria-export` (new module `datasheet` JSON-LD).
> **Frontend** : `web/src/routes/(modules)/m17/+page.svelte` — Claude Code.
> **Durée cible** : 1 jour Rust.
> **Référence CDC** : v1.3 §4 M17.

---

## 0. Objectif

Permettre à un·e **chercheur·se / journaliste / contributeur·rice
scientifique** de documenter un **projet** (une étude, un article,
un benchmark, un papier) sous forme de **datasheet JSON-LD** selon
le standard académique **Gebru et al. 2018 — Datasheets for Datasets**.

Cas d'usage type : « J'écris un papier de recherche sur l'empreinte
de Claude 3.5 Sonnet sur 10 000 prompts collectés en Q1 2026.
Je veux produire un datasheet officiel reproductible et accompagner
mon papier d'un JSON-LD + PROV-O auditable. »

**Lien dossier candidature data.gouv** : la reproductibilité scientifique
est un critère central. Gebru 2018 est le standard académique mondial
pour documenter datasets et modèles. Sobr.ia génère ce standard
**automatiquement** depuis le ledger d'audit.

## 1. Modèle de données

### 1.1 Projet

Un **projet** est une **entité persistante nommée** avec :

- `id` : auto-increment SQLite
- `name` : libellé (ex: « Étude Q1 2026 Claude Sonnet »)
- `description` : texte libre (objectif, contexte)
- `period_start` / `period_end` : bornes temporelles (RFC 3339)
- `tags` : liste libre (ex: ["recherche", "claude", "q1-2026"])
- `created_at` / `updated_at`

Une fois créé, un projet est **immutable côté audit** (les entrées du
ledger pour la période sont figées). L'utilisateur peut modifier name,
description, tags, mais pas les dates (sinon le datasheet change).

### 1.2 Table SQLite

Dans `referentiel.sqlite` (à côté de `app_preferences` et `personal_goals`) :

```sql
CREATE TABLE IF NOT EXISTS projects (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    period_start    TEXT NOT NULL,
    period_end      TEXT NOT NULL,
    tags            TEXT NOT NULL DEFAULT '[]',   -- JSON array
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_projects_period
    ON projects(period_start, period_end);
```

## 2. Datasheet Gebru — 7 sections

Format standard (Gebru et al. 2018, arXiv:1803.09010) :

| § | Section | Source dans Sobr.ia |
|---|---|---|
| 1 | **Motivation** | `project.name` + `project.description` (texte libre utilisateur) |
| 2 | **Composition** | Agrégat ledger : nb requêtes, modèles utilisés, distribution temporelle |
| 3 | **Collection process** | « Estimations Monte-Carlo produites par sobria-estimator v{X.Y} avec seed {S}, N={N}. Voir AFNOR SPEC 2314 » |
| 4 | **Preprocessing / labeling** | « Aucune transformation : chaque entrée correspond à un acte d'estimation utilisateur unique » |
| 5 | **Uses** | Texte libre (ou défaut : « Recherche scientifique, audit interne, reporting CSRD ») |
| 6 | **Distribution** | URI vers le PROV-O JSON-LD, hash SHA-256 de l'export, licences (MIT/Etalab 2.0) |
| 7 | **Maintenance** | Email de contact (préférences utilisateur), version Sobr.ia, date dernière mise à jour |

## 3. Format JSON-LD

Combinaison de vocabulaires :

```json
{
  "@context": {
    "schema": "https://schema.org/",
    "prov": "http://www.w3.org/ns/prov#",
    "dcat": "http://www.w3.org/ns/dcat#",
    "sobria": "https://sobr.ia/vocab#"
  },
  "@graph": [
    {
      "@id": "sobria:project-{id}",
      "@type": ["schema:Dataset", "prov:Entity"],
      "schema:name": "Étude Q1 2026 Claude Sonnet",
      "schema:description": "...",
      "schema:dateCreated": "...",
      "schema:license": "https://opensource.org/licenses/MIT",
      "schema:temporalCoverage": "2026-01-01/2026-04-01",
      "schema:variableMeasured": ["co2eq", "energy", "water"],
      "schema:size": 247,
      "sobria:datasheetSection": {
        "@id": "sobria:project-{id}/datasheet"
      },
      ...
    },
    {
      "@id": "sobria:project-{id}/datasheet",
      "@type": "sobria:Datasheet",
      "sobria:motivation": "...",
      "sobria:composition": { ... },
      "sobria:collectionProcess": "...",
      "sobria:preprocessing": "...",
      "sobria:uses": "...",
      "sobria:distribution": { ... },
      "sobria:maintenance": { ... }
    }
  ]
}
```

## 4. Surface IPC

```rust
#[tauri::command] fn list_projects(state: ...) -> IpcResult<Vec<ProjectDto>>;
#[tauri::command] fn create_project(req: CreateProjectDto, state: ...) -> IpcResult<ProjectDto>;
#[tauri::command] fn get_project(id: i64, state: ...) -> IpcResult<ProjectDto>;
#[tauri::command] fn update_project(id: i64, req: UpdateProjectDto, state: ...) -> IpcResult<ProjectDto>;
#[tauri::command] fn delete_project(id: i64, state: ...) -> IpcResult<()>;
#[tauri::command] fn generate_project_datasheet(id: i64, state: ...) -> IpcResult<DatasheetDto>;
```

### DTOs

```rust
pub struct ProjectDto {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub period_start: String,  // RFC 3339
    pub period_end: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub struct CreateProjectDto {
    pub name: String,
    pub description: String,
    pub period_start: String,
    pub period_end: String,
    pub tags: Vec<String>,
}

pub struct UpdateProjectDto {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    // PAS de period_start/end — voir §1.1, dates immutables.
}

pub struct DatasheetDto {
    pub project: ProjectDto,
    pub jsonld: serde_json::Value,
    pub composition: CompositionDto,
    pub sha256: String,
}

pub struct CompositionDto {
    pub total_requests: u32,
    pub unique_models: Vec<String>,
    pub total_co2eq_g_p50: f64,
    pub total_energy_wh_p50: f64,
    pub total_water_l_p50: f64,
    pub date_first_entry: Option<String>,
    pub date_last_entry: Option<String>,
}
```

## 5. Validations

- `name` non vide, longueur ≤ 200 chars.
- `description` longueur ≤ 5000 chars.
- `period_start` < `period_end`.
- `tags` ≤ 10 éléments, chaque tag ≤ 50 chars, slug-like (a-z0-9-).
- `update_project` : au moins un champ doit être présent.

## 6. Definition of Done

### Rust
- [ ] `crates/sobria-app/src/project_store.rs` : `ProjectStore` CRUD complet.
- [ ] `AppState.projects: Mutex<ProjectStore>` ajouté.
- [ ] `crates/sobria-export/src/datasheet.rs` : `build_datasheet()` qui
      assemble le JSON-LD selon Gebru.
- [ ] DTOs dans `sobria-app/src/dto.rs`.
- [ ] 6 commandes IPC enregistrées.
- [ ] ≥ 14 tests :
      - CRUD round-trip, validation name/description/tags/dates
      - update partiel (un champ à la fois)
      - delete idempotent
      - datasheet a 7 sections, jsonld @context complet, composition cohérente
      - empty_period si aucune entrée du ledger dans la période
      - SHA-256 stable pour même entrée (reproductibilité)
- [ ] `cargo clippy -p sobria-app -p sobria-export -- -D warnings` propre.

### Doc
- [ ] Note dans `docs/methodology/DATASHEET-GEBRU.md` : mapping Sobr.ia → Gebru,
      exemple complet d'un projet + son datasheet.

## 7. Non-objectifs

- **Export PDF du datasheet** → réutilise `export_csrd_report` (C14) sur
  la période du projet pour avoir PDF + PROV-O. Pas de nouvelle génération
  PDF spécifique en C20.
- **Versioning du datasheet** (snapshot) → différé v1.1.
- **Partage entre utilisateurs** (Zenodo, OSF) → différé v1.1.
- **Tags hiérarchiques** → flat tags suffisants v1.0.

## 8. Risques

| Risque | Probabilité | Parade |
|---|---|---|
| Gebru sections trop académiques pour étudiant·e·s | Moyenne | Persona-gating : M17 visible par chercheur·se seulement (cf ADR-0010) |
| JSON-LD validateur externe ne valide pas | Faible | Tests : parse @context, structure @graph |
| Performance sur grands projets (10⁵ entrées) | Faible | Idem dashboard : agrégat parsing total OK au volume v1.0 |

---

*Brief Cowork. Exécution C20.1 (project_store), C20.2 (datasheet),
C20.3 (DTOs+IPC), C20.4 (tests), C20.5 (prompt Claude Code).*
