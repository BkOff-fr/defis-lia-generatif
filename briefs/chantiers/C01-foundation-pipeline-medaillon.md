# Chantier #1 — Foundation du pipeline médaillon

> **Sprint** : S1 (fin) + S2 (début)
> **Crates concernées** : `sobria-core`, `sobria-ingest`
> **Durée cible** : 5 jours
> **Approche** : TDD (tests d'abord), types-first, zéro `unwrap()` en code de prod.
> **Pré-requis** : pack de cadrage v1.2 + bootstrap S1 mergé.

---

## 0. Pourquoi ce chantier en premier ?

Tous les autres sprints en dépendent. Sans foundation propre, chaque source future devra réinventer la roue (hash, manifest, download, validation). En implémentant la foundation **avant la première source**, on respecte le principe énoncé en ADR-0009 : *« onboarding d'une nouvelle source = un seul trait à implémenter »*.

C'est ennuyeux mais c'est ce qui fait la différence entre un projet « hackathon » et un projet « contribution publique sérieuse ».

---

## 1. Périmètre

### En périmètre

- Types fondamentaux dans `sobria-core` (Datacenter, EmissionFactor, EstimationRequest, EstimationResult).
- Module `hash` : SHA-256 streaming sur fichiers gros (≥ 5 GB).
- Module `manifest` : lecture / écriture / validation des `manifest.json` Copper.
- Module `download` : téléchargement HTTP avec vérification de hash à la volée + reprise.
- Module `lineage` : structures de traçabilité Copper → Silver → Gold.
- Trait `DataLayer` enrichi (avec `health_check`, dépendances).
- `LayerRegistry::run_full_pipeline()` réellement implémenté.
- Tests unitaires + property-based + golden files.

### Hors périmètre (autres chantiers)

- Implémentation concrète de la source ComparIA (chantier #2).
- Implémentation concrète de la source RTE IRIS (chantier #3).
- Promotion Silver → Gold avec jointures inter-sources (chantier #5).
- App Tauri, frontend, extension (chantiers ultérieurs).

---

## 2. Décomposition en modules

### 2.1 `sobria-core::types` (extension)

**Surface API ajoutée** :

```rust
pub struct Datacenter {
    pub id: String,
    pub provider: String,
    pub region: String,
    pub country_iso: String,
    pub pue: UncertaintyInterval,
    pub wue: Option<UncertaintyInterval>,
    pub sources: Vec<String>,
}

pub struct EmissionFactor {
    pub country_iso: String,
    pub year: u16,
    pub g_co2eq_per_kwh: UncertaintyInterval,
    pub source: String,
    pub valid_from: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
}

pub struct EstimationRequest {
    pub model_id: String,
    pub tokens_in: u32,
    pub tokens_out_estimated: u32,
    pub datacenter_id: Option<String>,
    pub timestamp: DateTime<Utc>,
}

pub struct EstimationResult {
    pub request: EstimationRequest,
    pub indicators: Vec<IndicatorValue>,
    pub equivalents: Vec<Equivalent>,
    pub hypotheses: Vec<Hypothesis>,
    pub computed_at: DateTime<Utc>,
    pub seed: u64,
}

pub struct Hypothesis {
    pub key: String,
    pub value: serde_json::Value,
    pub source: String,
}
```

**Tests obligatoires** :
- Round-trip serde JSON (insta snapshots).
- Property-based : un `UncertaintyInterval` valide a toujours `p5 ≤ p50 ≤ p95`.
- Validation `country_iso` à 2 caractères majuscules.

### 2.2 `sobria-ingest::hash` (nouveau)

**Objectif** : hasher des fichiers de plusieurs Go sans charger en mémoire.

```rust
/// Hashe un fichier en streaming. Retourne le hash hexadécimal sur 64 caractères.
pub async fn sha256_file(path: &Path) -> Result<String>;

/// Hashe un flux lecteur en streaming.
pub async fn sha256_reader<R: AsyncRead + Unpin>(reader: R) -> Result<String>;

/// Vérifie qu'un fichier correspond à un hash attendu.
pub async fn verify_file(path: &Path, expected_hex: &str) -> Result<bool>;
```

**Tests** :
- Vecteurs RFC 6234 (chaîne vide, "abc", million de "a").
- Property : `sha256(s1+s2) == streaming(s1) + streaming(s2)`.
- Performance : > 500 MB/s sur un disque SSD courant (bench Criterion).

### 2.3 `sobria-ingest::manifest` (nouveau)

**Format** : `manifest.json` v1, schéma Json strict.

```json
{
  "schema_version": "1",
  "source_id": "comparia",
  "fetched_at": "2026-05-12T14:32:08Z",
  "files": [
    {
      "name": "conversations.parquet",
      "url": "https://www.data.gouv.fr/api/1/datasets/r/7651fd0b-...",
      "sha256": "7a3f9b...",
      "size_bytes": 715456000,
      "http_headers": { "etag": "..." }
    }
  ],
  "license": "Etalab 2.0",
  "license_url": "https://www.etalab.gouv.fr/licence-ouverte-open-licence"
}
```

**API** :

```rust
pub struct CopperManifest { ... }
impl CopperManifest {
    pub fn new(source_id: &str) -> Self;
    pub fn add_file(&mut self, entry: ManifestFileEntry);
    pub fn save(&self, path: &Path) -> Result<()>;
    pub fn load(path: &Path) -> Result<Self>;
    pub fn validate(&self) -> Result<()>;
}
```

**Tests** :
- Serde round-trip strict.
- Validation : rejette un manifest sans `schema_version`.
- Validation : rejette un manifest avec URL non-HTTPS.

### 2.4 `sobria-ingest::download` (nouveau)

**Objectif** : télécharger un gros fichier, calculer son hash en parallèle, gérer la reprise.

```rust
pub struct Downloader { client: reqwest::Client, /* ... */ }

impl Downloader {
    pub fn new() -> Self;

    /// Télécharge `url` vers `dest`. Hashe en streaming et retourne (taille, sha256).
    /// Si `dest` existe et `expected_sha256` est fourni, vérifie et retourne sans re-télécharger.
    pub async fn fetch_to_file(
        &self,
        url: &str,
        dest: &Path,
        expected_sha256: Option<&str>,
    ) -> Result<DownloadOutcome>;
}

pub struct DownloadOutcome {
    pub bytes: u64,
    pub sha256: String,
    pub status: DownloadStatus,
}

pub enum DownloadStatus { Downloaded, CachedHit, Resumed }
```

**Edge cases obligatoirement gérés** :
- Serveur sans support `Range` → fallback re-download complet.
- Hash attendu fourni mais mismatch → erreur explicite `HashMismatch`.
- Espace disque insuffisant → erreur `Io::StorageFull` avec chemin.
- Timeout → retry exponentiel max 3 fois.

**Tests** :
- Mock server (`wiremock` crate) avec réponses contrôlées.
- Test reprise : interrompre à 50 %, relancer, vérifier complet.

### 2.5 `sobria-ingest::lineage` (nouveau)

Structures de traçabilité — propage les hashes Copper jusqu'au Gold.

```rust
pub struct CopperRef {
    pub source_id: String,
    pub manifest_path: PathBuf,
    pub file_sha256: String,
}

pub struct SilverLineage {
    pub entity: String,
    pub schema_version: String,
    pub copper_refs: Vec<CopperRef>,
    pub row_count: u64,
    pub written_at: DateTime<Utc>,
}

pub struct GoldLineage {
    pub silver_inputs: Vec<SilverLineage>,
    pub gold_artifacts: Vec<String>,
    pub assembled_at: DateTime<Utc>,
}

impl GoldLineage {
    /// Sérialise en JSON-LD inclus dans `datasheet.jsonld`.
    pub fn to_jsonld(&self) -> serde_json::Value;
}
```

**Tests** :
- Property : un `GoldLineage` ne perd jamais un hash Copper.

### 2.6 `sobria-ingest::layer` (extension)

Enrichissement du trait `DataLayer` :

```rust
#[async_trait]
pub trait DataLayer: Send + Sync {
    fn id(&self) -> &'static str;
    fn meta(&self) -> SourceMeta;

    /// Liste des sources dont celle-ci dépend (résolu par le registry).
    fn depends_on(&self) -> Vec<&'static str> { vec![] }

    /// Health check (ex: API disponible, fichier accessible).
    async fn health_check(&self, ctx: &Context) -> anyhow::Result<HealthReport> { ... }

    async fn ingest_copper(&self, ctx: &Context) -> Result<CopperSnapshot>;
    async fn promote_silver(&self, snapshot: &CopperSnapshot, ctx: &Context) -> Result<Vec<SilverEntity>>;
    async fn contribute_gold(&self, silver: &[SilverEntity], ctx: &Context) -> Result<GoldContribution>;
}

pub struct HealthReport {
    pub ok: bool,
    pub message: String,
    pub last_check: DateTime<Utc>,
}
```

### 2.7 `sobria-ingest::registry` (extension)

Vraie orchestration :

```rust
impl LayerRegistry {
    pub async fn health_check_all(&self, ctx: &Context) -> Vec<(String, HealthReport)>;

    pub async fn run_full_pipeline(&self, ctx: &Context) -> Result<PipelineReport>;
    pub async fn run_copper(&self, ctx: &Context, source_id: Option<&str>) -> Result<()>;
    pub async fn run_silver(&self, ctx: &Context, source_id: Option<&str>) -> Result<()>;
    pub async fn run_gold(&self, ctx: &Context) -> Result<GoldLineage>;

    pub async fn validate(&self, ctx: &Context) -> Result<ValidationReport>;
}

pub struct PipelineReport {
    pub copper_done: Vec<String>,
    pub silver_done: Vec<String>,
    pub gold_lineage: GoldLineage,
    pub duration_ms: u64,
}
```

---

## 3. Definition of Done — global

- [ ] `cargo build --workspace` passe sans warning.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passe.
- [ ] `cargo fmt --check` passe.
- [ ] `cargo test --workspace` passe (≥ 30 tests unitaires).
- [ ] `cargo doc --workspace --no-deps` génère sans warning.
- [ ] Aucun `unwrap()` / `expect()` en code de prod (sauf cas justifié + commentaire).
- [ ] Tracing structurés sur tous les points d'entrée publics.
- [ ] CHANGELOG.md mis à jour.
- [ ] Commits Conventional respectés.

---

## 4. Anti-patterns à éviter

- ❌ Charger un fichier entier en mémoire pour le hasher.
- ❌ Bloquer le runtime tokio avec des opérations synchrones lourdes.
- ❌ Logger des secrets (tokens, headers d'auth).
- ❌ Ignorer les erreurs HTTP transitoires (réessayer).
- ❌ Vendre un `Result<T, anyhow::Error>` comme API publique (utiliser `thiserror`).
- ❌ Tests qui dépendent du réseau Internet pour passer (utiliser des mocks).

---

## 5. Plan de réalisation jour par jour

| Jour | Sous-tâche | Tests | Validation |
|------|------------|-------|------------|
| J1 | sobria-core types + tests serde | round-trip + proptest | `cargo test -p sobria-core` |
| J2 | module hash + module manifest | vecteurs RFC + serde | `cargo test -p sobria-ingest hash` |
| J3 | module download + mocks | wiremock + reprise | `cargo test -p sobria-ingest download` |
| J4 | module lineage + DataLayer enrichi | proptest lineage | `cargo test -p sobria-ingest` |
| J5 | registry orchestration + tests intégration | golden files | `cargo test --workspace` |

---

## 6. Sorties attendues

- 9 nouveaux modules Rust documentés.
- ≥ 30 tests unitaires.
- 1 fichier `schemas/copper/manifest-v1.json` (JSON Schema).
- 1 fichier `schemas/lineage/gold-jsonld-v1.json`.
- Mise à jour du CHANGELOG.md.
- Génération `cargo doc` lisible.
