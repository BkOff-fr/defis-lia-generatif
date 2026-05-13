# Chantier #11 — M13 Simulateur « Et si...? »

> **Pré-requis** : v0.2.0-estimer mergé, C10 frontend en cours.
> **Crates touchées** : `sobria-estimator` (nouveau module `scenarios.rs`),
> `sobria-app` (DTO + IPC).
> **Frontend** : `web/src/routes/(modules)/m13/+page.svelte` (chantier
> Claude Code séparé après livraison Rust).
> **Durée cible** : 1-2 jours.
> **Référence CDC** : v1.3 §4 M13.

---

## 0. Objectif

Permettre à l'utilisateur d'**explorer l'impact d'un changement de levier**
sur son empreinte d'usage IA — modèle, datacenter, mix énergétique,
longueur de réponse, fréquence d'usage — et de projeter cet impact sur
**12 mois** avec une hypothèse de croissance d'adoption.

Trois rendus attendus :

1. **Verdict CO₂ instantané** : delta absolu (gCO₂eq) et relatif (%) entre
   baseline et chaque scénario.
2. **Waterfall contribution** : si on enchaîne 7 leviers indépendants, on
   isole la contribution de chacun (attribution séquentielle, voir §3.3).
3. **Projection 12 mois** : série mensuelle de CO₂eq cumulé, baseline +
   chaque scénario, avec hypothèse géométrique de croissance.

## 1. Les 7 leviers

Sélection retenue pour v1.3 (modifiable, c'est le frontend qui décide
quels leviers exposer — le backend accepte n'importe quel sous-ensemble) :

| # | Lever | Type | Domaine | Exemple |
|---|-------|------|---------|---------|
| 1 | **Modèle** | enum | catalogue M9 | `gpt-4o-mini` → `claude-3-5-sonnet` |
| 2 | **Région datacenter** | enum | ISO 3166-1 alpha-2 | `US` → `FR` |
| 3 | **PUE datacenter** | float | [1.05, 1.6] | 1.4 → 1.1 |
| 4 | **Mix élec (g/kWh)** | float | [10, 800] | 380 (US-mix) → 56 (FR-mix) |
| 5 | **Tokens de sortie** | u32 | [1, 10⁶] | 500 → 200 (réponse courte) |
| 6 | **Embodied/req** | float | [0.0001, 1.0] | 0.02 → 0.01 (modèle plus efficient) |
| 7 | **WUE (L/kWh)** | float | [0.0, 5.0] | 1.5 → 0.5 (datacenter sec) |

Le frontend mappe ces leviers vers les `ParamOverrides` ci-dessous.
Aucun n'est obligatoire ; un scénario peut combiner 1-7 leviers.

## 2. Architecture

### 2.1 Module Rust `sobria-estimator/src/scenarios.rs`

```rust
use serde::{Deserialize, Serialize};
use crate::{params::EstimationParams, MonteCarloEngine};
use sobria_core::{EstimationRequest, EstimationResult};

/// Overrides partiels appliqués sur `EstimationParams` ou `EstimationRequest`.
/// Tous les champs sont `Option` : `None` = garde la valeur du baseline.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ParamOverrides {
    pub model_id: Option<String>,
    pub tokens_out: Option<u32>,
    pub pue: Option<f64>,
    pub if_electrical_g_per_kwh: Option<f64>,
    pub embodied_g_per_request: Option<f64>,
    pub wue_l_per_kwh: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub label: String,
    pub overrides: ParamOverrides,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastConfig {
    /// Horizon en mois (typiquement 12).
    pub months: u32,
    /// Croissance mensuelle (en pourcentage, ex: 5.0 = +5%/mois).
    pub monthly_growth_pct: f64,
    /// Volume baseline : nombre de prompts/jour au mois 0.
    pub base_volume_per_day: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationRequest {
    pub baseline: EstimationRequest,
    pub scenarios: Vec<Scenario>,
    pub forecast: Option<ForecastConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioOutcome {
    pub label: String,
    pub result: EstimationResult,
    /// Δ par rapport au baseline P50, en gCO₂eq.
    pub delta_co2eq_g: f64,
    /// Δ relatif en pourcentage du baseline P50.
    pub delta_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastResult {
    pub months: u32,
    /// Série mensuelle baseline (gCO₂eq, 12 valeurs).
    pub baseline_monthly_co2eq_g: Vec<f64>,
    /// Cumul annuel baseline (gCO₂eq).
    pub baseline_annual_co2eq_g: f64,
    /// Cumul annuel par scénario (gCO₂eq), ordre = scenarios.
    pub scenarios_annual_co2eq_g: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub baseline: EstimationResult,
    pub scenarios: Vec<ScenarioOutcome>,
    pub forecast: Option<ForecastResult>,
}

pub fn simulate(
    engine: &MonteCarloEngine,
    request: &SimulationRequest,
) -> EstimatorResult<SimulationResult>;
```

### 2.2 Algorithme

```
1. Estimer le baseline avec son model_id et ses params défaut.
2. Pour chaque scénario :
     a. Construire EstimationRequest dérivé (avec tokens_out override
        si présent, sinon = baseline).
     b. Construire EstimationParams dérivé :
          - Partir de EstimationParams::for_model(scenario.model_id
            ou baseline.model_id).
          - Appliquer chaque override comme Distribution::Point.
     c. engine.estimate(derived_req, derived_params).
     d. Calculer delta_co2eq_g et delta_pct vs baseline.
3. Si forecast est défini :
     a. Pour baseline et chaque scénario :
          - mois_0 = 30 × base_volume_per_day × co2eq_per_request_p50
          - mois_n = mois_0 × (1 + growth)^n
          - annual = somme des 12 mois
     b. Retourner ForecastResult.
4. Retourner SimulationResult.
```

### 2.3 Attribution waterfall (côté frontend)

L'attribution stricte (Shapley) coûte 2^N estimations pour N leviers,
trop cher pour 7 leviers (128 estimations). On adopte une attribution
**séquentielle** :

- Le frontend envoie 7 scénarios, chacun ajoutant un lever supplémentaire :
  - Scénario 1 : baseline + lever_1
  - Scénario 2 : baseline + lever_1 + lever_2
  - …
  - Scénario 7 : baseline + tous les leviers
- La contribution du lever_k est `scénario_k.P50 - scénario_{k-1}.P50`.

C'est documenté dans la UI ("attribution dans cet ordre — résultat
dépend de l'ordre choisi"). Cohérent avec les pratiques sectorielles
(voir Ghahramani 2024).

## 3. Surface IPC

Une commande Tauri dans `sobria-app/main.rs` :

```rust
#[tauri::command]
fn simulate_scenarios(
    req: SimulationRequestDto,
    state: tauri::State<'_, AppState>,
) -> IpcResult<SimulationResultDto>;
```

Validation côté `logic::simulate_scenarios` :
- baseline.model_id connu (sinon `unknown_model`),
- chaque scenario.overrides.model_id (si présent) connu,
- forecast.months ∈ [1, 60],
- forecast.monthly_growth_pct ∈ [-50.0, 50.0],
- forecast.base_volume_per_day ∈ [0.0, 10⁶],
- au plus 20 scénarios par requête (garde-fou anti-abus).

### Audit ledger

Décision retenue : **chaque simulation produit UNE entrée d'audit**
(le baseline) pour ne pas saturer le ledger. Les scénarios sont
journalisés dans le `EstimationResult.hypotheses` du baseline sous
forme structurée (une hypothèse par scénario nommé).

## 4. Definition of Done

### Rust
- [ ] `scenarios.rs` avec 7 types publics + `simulate()`.
- [ ] Re-exports dans `sobria-estimator/src/lib.rs`.
- [ ] 6 tests : baseline-only, model override change P50, PUE override
      change P50, unknown_model rejeté, forecast 0% = somme constante,
      forecast 5% = série géométrique cohérente.
- [ ] DTOs dans `sobria-app/src/dto.rs` (mirror 1-pour-1 des types Rust).
- [ ] `logic::simulate_scenarios` avec validation + journalisation audit.
- [ ] Commande Tauri `simulate_scenarios` enregistrée.
- [ ] 4 tests `logic::tests` (happy path, unknown model, doublons label,
      forecast invalide).
- [ ] `cargo clippy -p sobria-estimator -p sobria-app -- -D warnings` propre.

### Doc
- [ ] Note rapide dans `docs/methodology/SIMULATEUR-WATERFALL.md`
      expliquant l'attribution séquentielle et ses limites.

## 4bis. Insight méthodologique à afficher dans l'UI

Le simulateur révèle un phénomène contre-intuitif que l'UI **doit** mettre
en avant : selon la taille du modèle et la longueur du prompt, le levier
dominant change radicalement.

| Profil prompt | Modèle | Levier dominant | Part du total |
|---|---|---|---|
| 100/500 tokens | gpt-4o-mini (~10B) | **Embodied carbon** | ≈ 99% |
| 1000/5000 tokens | claude-3-5-sonnet (~250B) | **Compute × mix élec** | ≈ 80% |
| Tokens longs / petits modèles | mistral-medium-3 | mixte | équilibré |

Conséquence UX : la carte « verdict CO₂ » doit afficher en clair :

> « Sur ce profil, votre principal levier est **[embodied / mix élec / tokens / PUE]**
> (≈ X% du total). Modifier les autres aura un impact marginal. »

Le frontend identifie le levier dominant en simulant 4 scénarios "isoler
chaque composante à 0" (embodied=0, compute_factor=0, etc.) et compare
les deltas. Le levier qui produit le delta absolu le plus grand est le
dominant.

Cette mise en perspective évite que l'utilisateur passe 10 min à
optimiser PUE alors qu'il devrait changer de modèle.

## 5. Non-objectifs (différés)

- **Attribution Shapley** (2^N) → backlog v1.1.
- **Sensitivity analysis** (variance par lever) → backlog v1.1.
- **Optimisation automatique** ("quel set de leviers minimise CO₂?") →
  hors scope MVP, reviendrait à exposer un solveur côté UI.
- **Persistence des scénarios sauvegardés** → utiliser le journal
  existant côté front (localStorage) pour v1.0.

## 6. Risques

| Risque | Probabilité | Parade |
|---|---|---|
| Latence excessive si 20 scénarios × N=10⁴ | Moyenne | Bornes max scénarios + tests perf, fallback N=5000 si > 100ms |
| Attribution waterfall mal comprise par l'utilisateur | Moyenne | Tooltip explicite "ordre des leviers compte" |
| Forecast trop linéaire → ne capte pas saisonnalité | Faible (MVP) | Documenter l'hypothèse géométrique, raffinement v1.1 |

---

*Brief rédigé par Cowork. Exécution séparée en C11.1 (Rust types + simulate),
C11.2 (DTOs), C11.3 (logic + IPC), C11.4 (tests). Frontend M13 = chantier
séparé Claude Code après livraison Rust.*
