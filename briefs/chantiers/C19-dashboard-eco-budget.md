# Chantier #19 — M15 Dashboard personnel + M25 Eco-budget

> **Pré-requis** : v0.2.7-referentiel mergé.
> **Crates touchées** : `sobria-app` (new modules `dashboard` + `goals_store`,
> DTOs, IPCs).
> **Frontend** : `web/src/routes/(modules)/m15/+page.svelte` et `m25/+page.svelte`
> — Claude Code.
> **Durée cible** : 1 jour Rust.
> **Référence CDC** : v1.3 §4 M15 + M25.

---

## 0. Objectif

**M15 Dashboard personnel** : vue récapitulative de l'usage IA d'un·e
utilisateur·rice sur une période donnée (aujourd'hui / 7 derniers jours /
ce mois / ce mois précédent / cette année), avec :

- Métriques cumulées (CO₂eq, énergie, eau) + nb requêtes.
- Comparaison vs période précédente (% +/-).
- Top 5 modèles utilisés.
- Time series jour par jour (graphe).

**M25 Eco-budget personnel** : permet de **définir des objectifs**
mensuels/hebdomadaires/journaliers par indicateur et de **suivre la
consommation vs le budget** en temps réel, avec niveau d'alerte
(ok / warning / exceeded).

Les deux écrans partagent la même source de vérité : le ledger d'audit.
M25 ajoute une table SQLite `personal_goals` dans `referentiel.sqlite`.

## 1. M15 Dashboard

### 1.1 Périodes supportées

```rust
pub enum DashboardPeriod {
    Today,           // [00:00 aujourd'hui, maintenant]
    Last7Days,       // [now - 7d, now]
    ThisMonth,       // [début du mois courant, maintenant]
    LastMonth,       // [début mois -1, début mois courant]
    ThisYear,        // [début année courante, maintenant]
}
```

### 1.2 Surface IPC

```rust
#[tauri::command]
fn get_dashboard_summary(
    period: String,           // "today" | "last_7_days" | "this_month" | "last_month" | "this_year"
    state: tauri::State<'_, AppState>,
) -> IpcResult<DashboardSummaryDto>;
```

### 1.3 `DashboardSummaryDto`

```rust
pub struct DashboardSummaryDto {
    pub period_label: String,           // "Aujourd'hui", "7 derniers jours", ...
    pub period_start: String,           // RFC 3339
    pub period_end: String,
    pub total_requests: u32,
    pub total_co2eq_g_p50: f64,
    pub total_energy_wh_p50: f64,
    pub total_water_l_p50: f64,
    /// Comparaison vs période précédente — `None` si pas de données précédentes.
    pub vs_previous: Option<DashboardComparisonDto>,
    /// Top N modèles par CO2eq (N=5 par défaut).
    pub top_models: Vec<TopModelDto>,
    /// Série journalière — longueur dépend de la période.
    pub daily_series: Vec<DailySeriesPointDto>,
}

pub struct DashboardComparisonDto {
    pub previous_total_co2eq_g_p50: f64,
    pub delta_co2eq_pct: f64,           // +12.0% ou -23.0%
    pub previous_total_requests: u32,
    pub delta_requests_pct: f64,
}

pub struct TopModelDto {
    pub model_id: String,
    pub request_count: u32,
    pub total_co2eq_g_p50: f64,
}

pub struct DailySeriesPointDto {
    pub date: String,                   // "YYYY-MM-DD"
    pub request_count: u32,
    pub co2eq_g_p50: f64,
    pub energy_wh_p50: f64,
    pub water_l_p50: f64,
}
```

### 1.4 Comportement

- Parse le ledger d'audit pour les 2 périodes (courante + précédente).
- Pour chaque période : somme P50 par jour, par modèle, total.
- Top modèles : groupby model_id, tri par sum(CO2eq_p50) desc, top 5.
- Time series : 1 point par jour (vide si pas d'entrée ce jour-là).
- **Exclut les entrées purgées RGPD** (sentinel) des agrégats numériques
  mais les compte dans `total_requests`.

## 2. M25 Eco-budget

### 2.1 Table SQLite

Dans `referentiel.sqlite` (à côté de `app_preferences`) :

```sql
CREATE TABLE IF NOT EXISTS personal_goals (
    indicator   TEXT NOT NULL,    -- 'co2eq' | 'energy' | 'water'
    period      TEXT NOT NULL,    -- 'daily' | 'weekly' | 'monthly'
    value_max   REAL NOT NULL,    -- limit (e.g., 1000 gCO2eq/month)
    unit        TEXT NOT NULL,    -- 'gCO2eq' | 'Wh' | 'L'
    updated_at  TEXT NOT NULL,
    PRIMARY KEY (indicator, period)
);
```

Une seule contrainte unique : `(indicator, period)`. L'utilisateur peut
définir au plus 3 indicateurs × 3 périodes = 9 objectifs.

### 2.2 Surface IPC

```rust
#[tauri::command]
fn list_personal_goals(state: ...) -> IpcResult<Vec<PersonalGoalDto>>;

#[tauri::command]
fn set_personal_goal(
    goal: PersonalGoalDto,
    state: ...,
) -> IpcResult<()>;

#[tauri::command]
fn delete_personal_goal(
    indicator: String,
    period: String,
    state: ...,
) -> IpcResult<()>;

#[tauri::command]
fn get_budget_status(state: ...) -> IpcResult<Vec<BudgetStatusDto>>;
```

### 2.3 DTOs

```rust
pub struct PersonalGoalDto {
    pub indicator: String,    // "co2eq" | "energy" | "water"
    pub period: String,       // "daily" | "weekly" | "monthly"
    pub value_max: f64,
    pub unit: String,
}

pub struct BudgetStatusDto {
    pub goal: PersonalGoalDto,
    pub current_value: f64,
    pub period_start: String,
    pub period_end: String,
    pub consumed_pct: f64,         // 0..100+ (peut dépasser)
    pub status: String,            // "ok" (<70%), "warning" (70-100%),
                                   //  "exceeded" (>100%)
    pub remaining: f64,            // value_max - current_value (peut être <0)
}
```

### 2.4 Validations

- `indicator` ∈ `{"co2eq", "energy", "water"}`.
- `period` ∈ `{"daily", "weekly", "monthly"}`.
- `value_max > 0` et fini.
- `unit` doit être cohérent avec `indicator` :
  - co2eq → "gCO2eq"
  - energy → "Wh"
  - water → "L"

## 3. Definition of Done

### Rust
- [ ] `sobria-app/src/dashboard.rs` : `DashboardPeriod` enum, `aggregate`
      fn qui retourne `DashboardSummaryDto`.
- [ ] `sobria-app/src/goals_store.rs` : `PersonalGoalsStore` adossé à
      `referentiel.sqlite`, CRUD complet.
- [ ] `AppState` embarque `Mutex<PersonalGoalsStore>` (similaire à
      `preferences`).
- [ ] DTOs ajoutés dans `dto.rs`.
- [ ] 5 commandes IPC :
      `get_dashboard_summary`, `list_personal_goals`, `set_personal_goal`,
      `delete_personal_goal`, `get_budget_status`.
- [ ] ≥ 12 tests :
      - dashboard : période vide → DashboardSummary avec total_requests=0,
        agrégation correcte, top_models triés par CO2eq, daily_series longueur OK,
        vs_previous calculé si entrées précédentes, exclusion purgées.
      - goals : CRUD round-trip, validation indicateur/période/unit,
        update sur PK existante, suppression idempotente.
      - budget : status ok/warning/exceeded selon ratio, période daily
        regarde 24h, weekly = 7 jours, monthly = mois calendaire.
- [ ] `cargo clippy -p sobria-app -- -D warnings` propre.

### Frontend (prompts séparés)
- [ ] Prompt M15 : dashboard avec cards + time series + top modèles + comparaison.
- [ ] Prompt M25 : formulaire goals + barres progression + alertes visuelles.

## 4. Non-objectifs

- **Notifications push** (système OS) → différé v1.1 — sur M21 Alertes.
- **Périodes custom** (du jj/mm au jj/mm) → différé v1.1.
- **Partage de stats** → différé v1.1.
- **Objectifs partagés équipe** → c'est M19, pas M25.

## 5. Risques

| Risque | Probabilité | Parade |
|---|---|---|
| Performances sur grand ledger (10⁵ entrées) | Faible | Pour MVP : parse total ; refactor v1.1 vers SQL aggregate |
| Time series sparse (semaine sans usage) | Moyenne | Le frontend insère les jours manquants avec count=0 |
| Objectifs : périodes "weekly" peu intuitive | Moyenne | Tooltip : "depuis lundi 00:00" (semaine ISO 8601) |

---

*Brief Cowork. Exécution C19.1 (dashboard), C19.2 (goals_store), C19.3 (DTOs+IPC),
C19.4 (tests), C19.5 (prompts Claude Code).*
