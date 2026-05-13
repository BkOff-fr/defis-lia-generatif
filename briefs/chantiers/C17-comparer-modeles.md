# Chantier #17 — M3 Comparer modèles (benchmark côte-à-côte)

> **Pré-requis** : v0.2.5-forecaster mergé.
> **Crates touchées** : `sobria-app` (DTOs + logic + IPC). Pas de
> nouveau module Rust dans `sobria-estimator` — réutilise `estimate_prompt`.
> **Frontend** : `web/src/routes/(modules)/m3/+page.svelte` — Claude Code.
> **Durée cible** : 0.5 jour Rust.
> **Référence CDC** : v1.3 §4 M3.

---

## 0. Objectif

Permettre à l'utilisateur de **benchmarker N modèles** (1-20) sur **un même
prompt** et visualiser :

- Le classement par CO₂eq (gagnant en lime, dernier en coral).
- Une fiche calibration par modèle (validated / indicative / extrapolated)
  pour transparence méthodologique.
- Une comparaison côte-à-côte des 3 indicateurs (CO₂eq / énergie / eau)
  avec P5-P50-P95.

Use case typique : « Je veux écrire un email professionnel. Quel modèle est
le moins polluant entre gpt-4o-mini, claude-3-5-sonnet, mistral-medium-3,
llama-3-1-70b ? »

## 1. Architecture

Implémentation **minimaliste** : pas de nouveau module dans `sobria-estimator`.
On boucle simplement sur `MonteCarloEngine::estimate()` pour chaque
`model_id` et on assemble les résultats. Tout vit dans `sobria-app::logic`.

## 2. Surface IPC

```rust
#[tauri::command]
fn benchmark_models(
    req: BenchmarkRequestDto,
    state: tauri::State<'_, AppState>,
) -> IpcResult<BenchmarkResultDto>;
```

### `BenchmarkRequestDto`

```rust
pub struct BenchmarkRequestDto {
    /// 1..=20 modèles à comparer.
    pub model_ids: Vec<String>,
    pub tokens_in: u32,
    pub tokens_out_estimated: u32,
    #[serde(default)]
    pub datacenter_id: Option<String>,
}
```

### `BenchmarkResultDto`

```rust
pub struct BenchmarkResultDto {
    /// Un outcome par modèle (ordre = ordre de la requête).
    pub outcomes: Vec<BenchmarkOutcomeDto>,
    /// model_ids classés du moins au plus émetteur (CO2eq P50).
    pub ranking_by_co2eq_p50: Vec<String>,
    /// model_ids classés du moins au plus énergivore (P50).
    pub ranking_by_energy_p50: Vec<String>,
    /// model_ids classés du moins au plus consommateur d'eau (P50).
    pub ranking_by_water_p50: Vec<String>,
    /// Échantillon de prompt utilisé (echo).
    pub tokens_in: u32,
    pub tokens_out_estimated: u32,
}

pub struct BenchmarkOutcomeDto {
    pub model_id: String,
    /// Métadonnées du preset (depuis ModelPreset).
    pub display_name: String,
    pub provider: String,
    pub family: String,
    pub openness: String,
    pub calibration: String,
    /// Estimation complète (avec bins, audit_id).
    pub result: EstimationResultDto,
    /// Rang dans le classement CO2eq (1 = meilleur).
    pub rank_co2eq: u32,
    pub rank_energy: u32,
    pub rank_water: u32,
}
```

## 3. Validations

- `model_ids.len()` ∈ [1, 20].
- Tous les `model_id` doivent être connus (rejet `unknown_model` avec
  la liste des inconnus).
- Pas de doublons dans `model_ids`.
- `tokens_in` + `tokens_out_estimated` validés par
  `EstimationRequest::validate()`.

## 4. Audit ledger

**Chaque modèle benchmarké crée 1 entrée d'audit.** Donc 20 modèles =
20 nouvelles entrées. C'est cohérent avec `estimate_prompt` (1 acte
analytique = 1 entrée). Pour ne pas saturer le ledger en exploration,
l'UI peut proposer un mode « brouillon » (à voir en v1.1) qui n'appelle
pas l'IPC tant que l'utilisateur n'a pas validé sa configuration.

## 5. Definition of Done

### Rust
- [ ] DTOs ajoutés dans `sobria-app/src/dto.rs`.
- [ ] `logic::benchmark_models()` avec validation stricte + classements.
- [ ] Commande Tauri `benchmark_models` enregistrée dans `generate_handler!`.
- [ ] 6 tests :
  - validation : empty list, > 20 models, duplicates, unknown model.
  - happy path : 4 modèles, classement CO2eq cohérent, fiches calibration.
  - audit : N appels créent N entrées dans le ledger.
- [ ] `cargo clippy -p sobria-app -- -D warnings` propre.

## 6. Non-objectifs

- **Comparaison sur N prompts** (matrice modèle × prompt) → C17b si demande.
- **Recommandation automatique** "le meilleur pour ton cas" — l'UI laisse
  l'utilisateur arbitrer entre CO2eq / energy / water / coût.
- **Coût € par modèle** → C18 si une source de prix publique est trouvée.

## 7. Risques

| Risque | Probabilité | Parade |
|---|---|---|
| 20 × Monte-Carlo 10⁴ = ~200 ms total | Faible | OK |
| Saturation ledger en exploration | Moyenne | Mode brouillon à venir v1.1 |
| Ranking trompeur si différences < bruit | Élevée (cf. C12 insight) | UI affiche les overlaps P5-P95 pour montrer si écart significatif |

---

*Brief Cowork. Exécution C17.1 (DTOs + logic), C17.2 (IPC + tests),
C17.3 (prompt Claude Code).*
