# Chantier #21 — M18 Batch CSV → rapport agrégé

> **Pré-requis** : v0.2.9-datasheet mergé.
> **Crates touchées** : `sobria-app` (new module `batch`, DTOs, IPC).
> **Frontend** : `web/src/routes/(modules)/m18/+page.svelte` — Claude Code.
> **Durée cible** : 0.5-1 jour Rust.
> **Référence CDC** : v1.3 §4 M18.

---

## 0. Objectif

Permettre à l'utilisateur d'**uploader un CSV contenant N prompts** et
d'obtenir en sortie :

1. Une **estimation par ligne** (boucle sur `estimate_prompt`) avec
   chaque ligne journalisée dans le ledger.
2. Un **rapport agrégé** :
   - Statistiques globales (totaux, moyennes).
   - Classement par modèle (count + CO₂eq total).
   - Distribution des consommations (P5/P50/P95 cumulés).
3. Un **export CSV des résultats** (1 ligne par estimation : entrée + sortie).

Use case typique :
- **Chercheur·se** : « J'ai 500 prompts dans un papier, je veux leur empreinte
  totale pour citer dans la section méthodologie. »
- **Pro tech** : « Je veux benchmarker 200 prompts internes Slack/Notion sur
  3 modèles, pour décider quel modèle utiliser. »

## 1. Format CSV d'entrée

```csv
model_id,tokens_in,tokens_out,datacenter_id
gpt-4o-mini,100,500,
claude-3-5-sonnet,200,1000,aws-eu-west-3-paris
mistral-medium-3,80,300,
```

### Validations

- **Header obligatoire** : `model_id,tokens_in,tokens_out,datacenter_id`
  (ordre strict v1.0).
- **Encoding** : UTF-8 (BOM toléré).
- **Min 1 ligne** de données, **max `MAX_BATCH_ROWS = 1000`** par batch.
- Chaque `model_id` doit exister dans `MODEL_REGISTRY`.
- `tokens_in` et `tokens_out` ∈ [1, 1 000 000].
- `datacenter_id` optionnel (vide = pas de DC spécifié).

## 2. Format CSV de sortie

```csv
row_index,model_id,tokens_in,tokens_out,datacenter_id,co2eq_p5_g,co2eq_p50_g,co2eq_p95_g,energy_wh_p50,water_l_p50,audit_id
1,gpt-4o-mini,100,500,,1.5,2.1,2.8,0.4,0.0012,1
2,claude-3-5-sonnet,200,1000,aws-eu-west-3-paris,8.2,11.4,15.1,4.2,0.013,2
...
```

## 3. Surface IPC

```rust
#[tauri::command]
fn run_batch_from_csv(
    req: BatchRequestDto,
    state: tauri::State<'_, AppState>,
) -> IpcResult<BatchResultDto>;
```

### `BatchRequestDto`

```rust
pub struct BatchRequestDto {
    /// Chemin absolu vers le CSV d'entrée.
    pub input_csv_path: String,
    /// Chemin de sortie pour le CSV des résultats. Optionnel : si None,
    /// pas d'export disque, juste agrégat retourné.
    pub output_csv_path: Option<String>,
}
```

### `BatchResultDto`

```rust
pub struct BatchResultDto {
    pub rows_processed: u32,
    pub rows_rejected: u32,
    pub aggregate: BatchAggregateDto,
    pub by_model: Vec<BatchModelAggregateDto>,
    /// Si `output_csv_path` fourni, chemin du fichier écrit.
    pub output_csv_path: Option<String>,
    /// Premier et dernier `audit_id` couverts par ce batch.
    pub first_audit_id: i64,
    pub last_audit_id: i64,
}

pub struct BatchAggregateDto {
    pub total_co2eq_g_p50: f64,
    pub total_energy_wh_p50: f64,
    pub total_water_l_p50: f64,
    pub avg_co2eq_g_p50: f64,
    pub min_co2eq_g_p50: f64,
    pub max_co2eq_g_p50: f64,
}

pub struct BatchModelAggregateDto {
    pub model_id: String,
    pub count: u32,
    pub total_co2eq_g_p50: f64,
    pub avg_co2eq_g_p50: f64,
}
```

## 4. Comportement

1. **Parse CSV** depuis `input_csv_path`. Si parsing échoue (fichier absent,
   format invalide, header manquant) → `IpcError { code: "invalid_request" }`.
2. **Valide les lignes** une par une. Une ligne invalide (modèle inconnu,
   tokens hors bornes) est **comptée comme rejetée** mais n'arrête pas le batch
   (sauf si > 50% des lignes sont rejetées → `IpcError`).
3. **Boucle estimate_prompt** sur les lignes valides. Chaque appel
   **journalise dans le ledger** (1 entrée par ligne).
4. **Agrégation** : totaux, moyennes, min/max P50, groupby model_id.
5. **Export CSV** si `output_csv_path` fourni. Une ligne par estimation.
6. **Retour** : `BatchResultDto` avec stats + chemin export.

## 5. Definition of Done

### Rust
- [ ] Module `sobria-app/src/batch.rs` : `parse_csv()`, `run_batch()`,
      `export_results_csv()`, types `BatchRow`, `BatchEstimation`.
- [ ] DTOs dans `dto.rs`.
- [ ] `logic::run_batch_from_csv()` orchestre tout.
- [ ] Commande IPC `run_batch_from_csv` enregistrée.
- [ ] ≥ 12 tests :
      - parse OK (CSV minimal valide)
      - parse rejette header absent / mal formé
      - parse rejette > 1000 lignes
      - lignes invalides comptées comme rejetées (pas d'erreur globale)
      - >50% rejets → erreur globale
      - happy path : 5 lignes → 5 entries audit, agrégat cohérent
      - by_model groupé correctement (3 modèles)
      - export CSV produit fichier avec 5 lignes + header
- [ ] `cargo clippy -p sobria-app -- -D warnings` propre.

### Doc
- [ ] Note dans `docs/methodology/BATCH-CSV-FORMAT.md` : spec format CSV
      + exemple complet input/output.

## 6. Non-objectifs

- **Format Parquet en entrée** → différé v1.1.
- **Streaming pour gros CSV** → différé v1.1 (cap à 1000 lignes v1.0).
- **Reprise sur erreur partielle** (resume) → différé v1.1.
- **Estimation parallèle multi-thread** → différé v1.1 (Monte-Carlo déjà
  rapide, 1000 lignes × 10ms = 10s acceptable).
- **CSV avec colonnes optionnelles supplémentaires** (tags, notes) →
  différé v1.1.

## 7. Risques

| Risque | Probabilité | Parade |
|---|---|---|
| Fichier > 1000 lignes saturé ledger | Moyenne | Cap strict + message clair |
| CSV mal encodé (UTF-16, Latin-1) | Moyenne | Documenter "UTF-8 attendu" |
| Lignes invalides confondent l'utilisateur | Élevée | Retour structuré `rows_rejected` avec compte explicite |
| Performance 1000 lignes × Monte-Carlo | Faible | ~10s, acceptable. UI : progress bar |

---

*Brief Cowork. Exécution C21.1 (module batch), C21.2 (DTOs + IPC),
C21.3 (tests), C21.4 (prompt Claude Code).*
