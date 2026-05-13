# Chantier #15 — M16 Forecaster 12 mois (bande d'incertitude)

> **Pré-requis** : v0.2.4-csrd-report mergé.
> **Crates touchées** : `sobria-estimator` (extension `scenarios.rs` ou
> nouveau module), `sobria-app` (DTO + IPC).
> **Frontend** : `web/src/routes/(modules)/m16/+page.svelte` — Claude Code.
> **Durée cible** : 0.5-1 jour Rust.
> **Référence CDC** : v1.3 §4 M16.

---

## 0. Objectif

Projeter, sur 12 mois (ou 1-60), l'**empreinte cumulée IA générative**
d'un·e utilisateur·rice ou d'une organisation, avec **bande d'incertitude
P5-P50-P95** propagée mensuellement, et superposition de plusieurs
scénarios (status quo / accélération adoption / ralentissement).

Différence vs C11 `simulate_scenarios.forecast` :
- C11 fournit uniquement P50 mensuel. M16 expose **P5 et P95** aussi pour
  afficher la bande d'incertitude.
- M16 supporte **N scénarios de croissance** côte à côte (pas juste un seul
  monthly_growth_pct).
- M16 fournit la série **cumulative** (somme courante) en plus de
  mensuelle.

## 1. Modèle de calcul

**Baseline** : on lance l'estimateur **une seule fois** pour la
configuration de référence (modèle + tokens + datacenter) et on récupère
les trois quantiles `co2eq_p5_g`, `co2eq_p50_g`, `co2eq_p95_g`.

**Projection** par scénario, pour chaque mois `n` ∈ `[0, months[` :

```
monthly_co2_p{q}_n = baseline_p{q} × volume_per_day × 30 × (1 + growth)^n
```

où `growth = monthly_growth_pct / 100`. Calcul indépendant pour chaque
quantile — on respecte la propriété de monotonie `p5 ≤ p50 ≤ p95` à chaque mois
(garantie tant que `baseline_p5 ≤ baseline_p50 ≤ baseline_p95`, ce qui est
le cas par construction).

**Cumul** : `cum_n = Σ monthly_co2_n` (running sum).

## 2. API publique

```rust
// sobria-estimator/src/yearly_forecast.rs

pub struct YearlyForecastRequest {
    pub baseline: EstimationRequest,
    pub scenarios: Vec<YearlyScenario>,   // 1..=10 scénarios
    pub months: u32,                       // 1..=60
    pub base_volume_per_day: f64,          // [0, 10⁶]
}

pub struct YearlyScenario {
    pub label: String,
    pub monthly_growth_pct: f64,           // [-50, 50]
}

pub struct YearlyForecastResult {
    /// Baseline propre, exposé tel quel (pour info / sanity check).
    pub baseline_co2eq_p5_g: f64,
    pub baseline_co2eq_p50_g: f64,
    pub baseline_co2eq_p95_g: f64,
    pub baseline_audit_id: i64,    // 0 si non journalisé
    /// Un outcome par scénario, dans l'ordre de la requête.
    pub scenarios: Vec<YearlyScenarioOutcome>,
}

pub struct YearlyScenarioOutcome {
    pub label: String,
    pub monthly_growth_pct: f64,
    /// Mensuel : longueur = months.
    pub monthly_p5_g: Vec<f64>,
    pub monthly_p50_g: Vec<f64>,
    pub monthly_p95_g: Vec<f64>,
    /// Cumulatif (somme courante) : longueur = months.
    pub cumulative_p5_g: Vec<f64>,
    pub cumulative_p50_g: Vec<f64>,
    pub cumulative_p95_g: Vec<f64>,
    /// Totaux annuels = cumul à n=months-1.
    pub annual_p5_g: f64,
    pub annual_p50_g: f64,
    pub annual_p95_g: f64,
}

pub fn forecast_yearly(
    engine: &MonteCarloEngine,
    req: &YearlyForecastRequest,
) -> EstimatorResult<YearlyForecastResult>;
```

## 3. Validations (côté `forecast_yearly`)

- `scenarios` ne doit pas être vide. Cap à 10 scénarios max (anti-abus).
- `months ∈ [1, 60]`.
- `base_volume_per_day ∈ [0, 1_000_000]` (volume strictement nul autorisé
  pour visualiser le pure baseline sans agréger).
- chaque `monthly_growth_pct ∈ [-50, 50]`.
- chaque scenario.label unique.
- model_id du baseline reconnu (propagation via `EstimationParams::for_model`).

Erreurs sous `EstimatorError::Schema`.

## 4. Surface IPC (sobria-app)

```rust
#[tauri::command]
fn forecast_yearly_budget(
    req: YearlyForecastRequestDto,
    state: tauri::State<'_, AppState>,
) -> IpcResult<YearlyForecastResultDto>;
```

Le baseline **est** journalisé dans le ledger (comme `estimate_prompt`).
Les scénarios sont des projections déterministes — pas de journalisation.

## 5. Definition of Done

### Rust
- [ ] `sobria-estimator/src/yearly_forecast.rs` avec types + `forecast_yearly()`.
- [ ] Re-exports `sobria-estimator::lib.rs`.
- [ ] DTOs côté `sobria-app` (snake_case mirrorés, sans `EstimationRequest`
      timestamp côté front, ajouté server-side).
- [ ] Commande IPC `forecast_yearly_budget` enregistrée.
- [ ] 8 tests : validations (empty/too many/bad months/bad growth),
      growth 0 → constante, growth 5% → géométrique, conservation
      P5 ≤ P50 ≤ P95, cumul croissant.

### Doc
- [ ] Note dans `docs/methodology/FORECAST-12-MOIS.md` : limites du modèle
      (hypothèse géométrique simple, pas de saisonnalité).

## 6. Non-objectifs (différés)

- **Saisonnalité** (M-pic en hiver, creux été) → v1.1 si pertinent.
- **Variance Monte-Carlo intra-mois** → on suppose la même incertitude
  proportionnelle à chaque mois (ce qui est conservateur).
- **Multi-modèles** (panier de modèles) → C17 Comparer.

## 7. Risques

| Risque | Probabilité | Parade |
|---|---|---|
| Incertitude trop large rend la projection illisible | Moyenne | UX : option « masquer la bande » côté Claude Code |
| 10 scénarios × 60 mois = 1800 points peut saturer Plot | Faible | Limites côté Rust enforcées |
| Hypothèse linéaire de croissance | Élevée | Documenté explicitement dans la méthodo + tooltip UI |

---

*Brief Cowork. Exécution C15.1 (module Rust), C15.2 (DTOs + IPC),
C15.3 (tests), C15.4 (prompt Claude Code séparé).*
