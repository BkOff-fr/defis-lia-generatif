# Chantier #18 — M9 Référentiel modèles (catalogue browsable)

> **Pré-requis** : v0.2.6-benchmark mergé.
> **Crates touchées** : `sobria-app` (DTO + IPC). `sobria-estimator` est
> déjà la source de vérité (`MODEL_REGISTRY` + helpers).
> **Frontend** : `web/src/routes/(modules)/m9/+page.svelte` — Claude Code.
> **Durée cible** : 2-3 heures Rust.
> **Référence CDC** : v1.3 §4 M9.

---

## 0. Objectif

Donner à l'utilisateur un **catalogue browsable** des 8 modèles de
référence (gpt-4o, gpt-4o-mini, claude-3-5-sonnet, mistral-large-2,
mistral-medium-3, llama-3-1-70b, llama-3-1-8b, gemini-2-0-flash) avec :

- **Vue grille** : cards avec badge calibration + openness + provider
- **Fiche détaillée** : params distributionnels epsilon prefill/decode et
  embodied (P5/P50/P95) avec sources, baseline estimation pour contexte
- **Comparaison** : "ce modèle vs un modèle de référence (gpt-4o-mini)"

C'est l'écran de **transparence méthodologique** : un chercheur ou
journaliste peut consulter exactement quelles valeurs Sobr.ia utilise.

## 1. Architecture

`list_models()` existe déjà (C09, returns `ModelPresetDto`). On ajoute :

- **`get_model_detail(id) -> ModelDetailDto`** : version enrichie qui
  expose les vraies plages distributionnelles P5/P50/P95 utilisées par
  l'estimateur, plus une estimation baseline (1 prompt référence) en
  contexte.

## 2. Surface IPC

```rust
#[tauri::command]
fn get_model_detail(
    id: String,
    state: tauri::State<'_, AppState>,
) -> IpcResult<ModelDetailDto>;
```

### `ModelDetailDto`

```rust
pub struct ModelDetailDto {
    // Tout ModelPresetDto + :
    pub id: String,
    pub display_name: String,
    pub provider: String,
    pub family: String,
    pub approx_params_billions: f64,
    pub openness: String,
    pub calibration: String,
    pub sources: Vec<String>,
    // Champs neufs (params distributionnels) :
    pub epsilon_prefill_mj_per_token: TripletDto,
    pub epsilon_decode_mj_per_token: TripletDto,
    pub embodied_g_per_request: TripletDto,
    /// Estimation baseline pour contexte (gpt-4o-mini 100/500 tokens
    /// avec PUE/IF par défaut). Donne à l'UI un point de comparaison
    /// concret. **Pas journalisée** dans l'audit ledger (fiche statique).
    pub baseline_co2eq_p5_g: f64,
    pub baseline_co2eq_p50_g: f64,
    pub baseline_co2eq_p95_g: f64,
    pub baseline_energy_wh_p50: f64,
    pub baseline_water_l_p50: f64,
}

pub struct TripletDto {
    pub p5: f64,
    pub p50: f64,
    pub p95: f64,
}
```

## 3. Validations

- `id` doit être un model_id connu (`find_preset(id).is_some()`), sinon
  `not_found`.

## 4. Définition of Done

### Rust
- [ ] `ModelDetailDto` + `TripletDto` ajoutés dans `dto.rs`.
- [ ] `logic::get_model_detail()` qui :
      1. Cherche le preset (sinon `not_found`).
      2. Construit les params depuis le preset.
      3. Calcule un baseline 100/500 tokens **sans journaliser**.
      4. Assemble le DTO.
- [ ] Commande Tauri `get_model_detail` enregistrée.
- [ ] 4 tests :
      - id inconnu → `not_found`
      - id connu → DTO complet avec sources non vides + triplets ordonnés
      - 8 modèles tous queryables individuellement
      - baseline non journalisé (ledger len inchangée)

## 5. Non-objectifs

- **Édition / ajout de modèles** par l'utilisateur → v1.1.
- **Comparaison N modèles** → c'est M3 (C17), pas M9.
- **Recommandation contextuelle** → v1.1.

## 6. Prompt Claude Code séparé

Voir `C18-PROMPT-CLAUDE-CODE-M9.md` — UI grille + fiche détaillée
avec graphes des distributions et sources cliquables.

---

*Brief Cowork. Exécution C18.1 (DTO + logic + tests), C18.2 (prompt).*
