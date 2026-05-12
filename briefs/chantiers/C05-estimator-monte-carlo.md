# Chantier #5 — sobria-estimator : moteur Monte-Carlo

> **Pré-requis** : pipeline médaillon mergé (C01-C04).
> **Crates touchées** : `sobria-estimator` (nouvelle implémentation).
> **Durée cible** : 2-3 jours.
> **Référence ADR** : ADR-0004 (Monte-Carlo N=10⁴, seed déterministe).

---

## 0. Objectif

Implémenter le **cœur scientifique** de Sobr.ia : prendre une requête
d'estimation (`EstimationRequest`) et restituer un résultat complet
(`EstimationResult`) avec intervalles d'incertitude P5-P95 issus de
**Monte-Carlo N=10 000**, conformément au référentiel AFNOR SPEC 2314.

## 1. Formule de référence

Voir CDC §9.1 et ADR-0004 :

```
Pour chaque tirage k ∈ [1, N=10⁴] :
  E_compute_k  = T_in × ε_prefill_k + T_out × ε_decode_k         (mJ)
  E_total_k    = (E_compute_k / 1000) × PUE_k                    (Wh)
  CO2eq_k      = (E_total_k / 1000) × IF_k + embodied_k           (g CO₂eq)
  Water_k      = (E_total_k / 1000) × WUE_k                       (L)

Agrégation finale :
  P5  = quantile(values, 0.05)
  P50 = quantile(values, 0.50)
  P95 = quantile(values, 0.95)
```

où chaque variable indicée `_k` est un **tirage d'une distribution**
spécifique à ce paramètre.

## 2. Architecture

```
crates/sobria-estimator/src/
├── lib.rs                 ← exports publics
├── error.rs               ← EstimatorError, EstimatorResult
├── distributions.rs       ← Distribution enum + sample + calibrage
├── params.rs              ← EstimationParams (struct distributionnelle)
├── engine.rs              ← MonteCarloEngine + estimate()
└── equivalents.rs         ← gCO₂eq → équivalents parlants
```

## 3. Distributions supportées

Type `Distribution` (énuméré, sérialisable) :

- **`Point { value }`** — déterministe (paramètre connu sans incertitude).
- **`Uniform { low, high }`** — uniforme bornée.
- **`Normal { mean, std }`** — gaussienne (tronquée à 0 pour valeurs positives).
- **`LogNormal { mu, sigma }`** — log-normale (typique pour ε_decode et embodied).

Helper `Distribution::log_normal_from_interval(p5, p50, p95)` :
- `mu = ln(p50)`
- `sigma = ln(p95 / p5) / (2 × z_{0.95})` où `z_{0.95} ≈ 1.6449`

Cette construction permet à un utilisateur de fournir un intervalle
qu'il connaît (issu de la littérature) et de l'utiliser comme distribution
plausible.

## 4. Paramètres distributionnels (`EstimationParams`)

| Paramètre | Distribution typique | Source par défaut |
|-----------|----------------------|-------------------|
| `epsilon_prefill_mj_per_token` | LogNormal | HF AI Energy Score |
| `epsilon_decode_mj_per_token` | LogNormal | HF AI Energy Score |
| `pue` | Uniform [1.05, 1.6] | Datacenter publié (default 1.3) |
| `if_electrical_g_per_kwh` | Point (mix horaire) | RTE / ADEME / Electricity Maps |
| `embodied_g_per_request` | LogNormal | Gupta 2022 + amortissement |
| `wue_l_per_kwh` | Uniform [0.5, 2.5] | Mytton 2021 |

Builder fluide :
```rust
let params = EstimationParams::default_for_model("gpt-4o-mini")
    .with_pue(Distribution::Point { value: 1.3 })
    .with_if_electrical(Distribution::Point { value: 56.0 }); // mix FR
```

## 5. RNG et reproductibilité

- `rand_xoshiro::Xoshiro256PlusPlus` — RNG rapide, déterministe, qualité statistique excellente.
- Seed = `SOBRIA_SEED` env var (défaut 42).
- Même seed + même params → même résultat à la nanoseconde près.

## 6. MonteCarloEngine API

```rust
pub struct MonteCarloEngine {
    n: u32,    // défaut 10_000
    seed: u64, // défaut 42
}

impl MonteCarloEngine {
    pub fn new(seed: u64) -> Self;
    pub fn with_n(mut self, n: u32) -> Self;
    
    pub fn estimate(
        &self,
        request: &EstimationRequest,
        params: &EstimationParams,
    ) -> EstimatorResult<EstimationResult>;
}
```

Le `EstimationResult` (de `sobria-core`) contient déjà tous les champs
nécessaires (`indicators`, `equivalents`, `hypotheses`, `seed`, etc.).

## 7. Tests requis

### 7.1 Sanity

- **Reproductibilité** : même seed → même résultat bit-à-bit.
- **Ordre des quantiles** : P5 ≤ P50 ≤ P95 toujours.
- **Monotonie** : doubler `tokens_out` double approximativement le P50
  (à PUE/IF constants, écart < 5 %).
- **Quasi-déterminisme** : si tous les paramètres sont Point, la
  distribution résultante est dégénérée (P5 = P50 = P95).

### 7.2 Property tests (proptest)

- Pour tout `EstimationRequest` et `EstimationParams` valides, le
  résultat est valide (intervalles ordonnés, valeurs positives).

### 7.3 Validation croisée (à compléter en v2 — chantier suivant)

- Reproduction Luccioni 2023 BLOOM (training) à ±15 %.
- Reproduction EcoLogits cas standard à ±15 %.

## 8. Définition de Done

- [ ] `cargo build --workspace` passe.
- [ ] `cargo clippy --workspace -- -D warnings` passe.
- [ ] `cargo test -p sobria-estimator --all-features` ≥ 15 tests verts.
- [ ] `MonteCarloEngine::estimate` produit un `EstimationResult` valide
  pour un cas de référence GPT-4o-mini @ FR mix.
- [ ] Documentation `///` complète sur tous les items publics.
- [ ] Aucun `unwrap`/`expect` non documenté.

## 9. Non-objectifs (v2)

- Validation croisée chiffrée Luccioni / Patterson / EcoLogits.
- Presets distributionnels par modèle (table calibrée depuis HF AI Energy Score).
- Mode batch optimisé pour 10⁶ estimations.
- Parallélisation Rayon.

## 10. Risques

| Risque | Mitigation |
|--------|-----------|
| LogNormal mal paramétrée → P5/P95 absurdes | Tests de sanity + log_normal_from_interval calibré |
| Lent (>200 ms) sur tirage N=10⁴ | Profiling avec criterion en v2 |
| Sample > 0 non garanti pour Normal | Truncate à 0 ou rejet sample |
