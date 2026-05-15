# C25 — Datacenter selection + immersive /datacenters — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a reusable `DatacenterPicker` that lets users choose a datacenter on /estimate, /comparer and /simuler, has the backend override PUE/IF/WUE on the resulting estimation, persists the last pick across sessions, and turns /datacenters into a full-bleed immersive map with floating glass UI.

**Architecture:** Two phases. **A — picker bundle** : add `default_datacenter_id` to the prefs store (SQLite k/v) → new helper `apply_datacenter_override` plugged into the 4 estimation flows + batch → new Svelte component shared across 3 routes. **B — immersive map** : pure CSS+structural refactor of `/datacenters` (no IPC changes). Phase A must ship as a cohesive unit (frontend depends on backend); Phase B is independent and could ship alone.

**Tech Stack:** Rust (sobria-app, sobria-core, sobria-geoloc, sobria-estimator), Tauri 2 IPC, SvelteKit 2 + Svelte 5 runes, Playwright, rusqlite.

**Spec:** [briefs/chantiers/C25-datacenter-selection.md](C25-datacenter-selection.md)

---

## Pre-flight checks (run once before starting)

- [ ] Confirm tauri dev is stopped (no `sobria-app.exe` running). Reason: file edits to Rust crates trigger rebuilds that block on the target lock if a debug binary is alive. Run `Get-Process sobria-app, cargo -ErrorAction SilentlyContinue | Stop-Process -Force` (PowerShell) if needed.
- [ ] Confirm working tree is clean enough that we can commit per task: `git status` — review modified files; commit any work-in-progress that's unrelated to C25 separately.

---

## File Structure

### New files
- `web/src/lib/components/DatacenterPicker.svelte` — custom dropdown component (Phase A)
- `web/tests/datacenter-picker.spec.ts` — Playwright e2e (Phase A)

### Modified files (Phase A)
- `crates/sobria-app/src/preferences_store.rs` — `StoredPreferences.default_datacenter_id`, KEY_DEFAULT_DATACENTER, read/write
- `crates/sobria-app/src/dto.rs` — `AppPreferencesDto.default_datacenter_id`
- `crates/sobria-app/src/logic.rs` — `get_app_preferences` / `set_app_preferences` map the new field; `apply_datacenter_override` helper; wired into `estimate_prompt`, `estimate_for_comparison`, `benchmark_models`, `simulate`, and the batch handler
- `web/src/lib/api.ts` — `AppPreferencesDto.default_datacenter_id?: string`
- `web/src/lib/preferences.ts` — `INITIAL` adds the new field; 6 `savePreferences({...})` sites updated
- `web/src/lib/components/Composer.svelte` — replace hardcoded "Datacenter (auto)" card with `<DatacenterPicker/>`, accept `datacenters` + `selectedDatacenter` props
- `web/src/routes/+page.svelte` (M1) — bootstrap `listDatacenters()`, pre-fill from `$preferences.default_datacenter_id`, pass to Composer, include `datacenter_id` in `estimatePrompt(...)`
- `web/src/routes/comparer/+page.svelte` (M3) — same wiring
- `web/src/routes/simuler/+page.svelte` (M13) — same wiring

### Modified files (Phase B)
- `web/src/routes/datacenters/+page.svelte` — absolute layout, remove constraint container
- `web/src/lib/components/m12/DatacenterFilters.svelte` — glass surface styles
- `web/src/lib/components/m12/DatacenterDrillDown.svelte` — glass styles + slide-in position
- `web/src/lib/components/m12/CountryDrillDown.svelte` — same as DatacenterDrillDown

---

# Phase A — Picker bundle

---

### Task A1: Add `default_datacenter_id` key to the preferences store (Rust)

**Files:**
- Modify: `crates/sobria-app/src/preferences_store.rs`
- Test: same file, `#[cfg(test)] mod tests` at bottom

- [ ] **Step 1: Write the failing test**

Append to `crates/sobria-app/src/preferences_store.rs` `mod tests` (find an existing test like `default_method_round_trip` and put it next to it):

```rust
    #[test]
    fn default_datacenter_id_round_trip() {
        let (_tmp, mut store) = open_temp();
        assert!(matches!(store.read_all().unwrap().default_datacenter_id, None));

        store
            .set_default_datacenter_id(Some("aws-us-east-1"))
            .unwrap();
        let back = store.read_all().unwrap().default_datacenter_id;
        assert_eq!(back.as_deref(), Some("aws-us-east-1"));

        // Reverting to None must also persist.
        store.set_default_datacenter_id(None).unwrap();
        assert!(store.read_all().unwrap().default_datacenter_id.is_none());
    }
```

- [ ] **Step 2: Run test to verify it fails**

```powershell
cargo test -p sobria-app default_datacenter_id_round_trip 2>&1 | Select-Object -Last 20
```

Expected: FAIL — `set_default_datacenter_id` doesn't exist, `StoredPreferences` has no `default_datacenter_id` field.

- [ ] **Step 3: Add the field + key + read/write**

Edit `StoredPreferences` (around line 47) — add at the bottom of the struct, after `also_show_methods`:

```rust
    /// Dernier datacenter sélectionné par l'utilisateur dans /estimate,
    /// /comparer ou /simuler (C25). `None` = aucun choisi → l'estimation
    /// utilise les `EstimationParams` par défaut.
    pub default_datacenter_id: Option<String>,
```

Find the block of `const KEY_*` constants near the top of the file and add (alphabetical order with the existing keys):

```rust
const KEY_DEFAULT_DATACENTER: &str = "default_datacenter_id";
```

In `read_all`, after the `also_show_methods` block (around line 111-115), add:

```rust
        let default_datacenter_id = self.read_raw(KEY_DEFAULT_DATACENTER)?;
```

Add `default_datacenter_id` to the `StoredPreferences { ... }` literal that `read_all` returns at the end.

After the existing `set_default_method` method, add the new setter:

```rust
    /// Persiste l'id du datacenter par défaut (C25). `None` efface la clé.
    pub fn set_default_datacenter_id(&mut self, id: Option<&str>) -> Result<(), AppError> {
        match id {
            Some(v) => self.write_raw(KEY_DEFAULT_DATACENTER, v),
            None => self.delete_raw(KEY_DEFAULT_DATACENTER),
        }
    }
```

If `delete_raw` doesn't exist, add it just below `write_raw`:

```rust
    fn delete_raw(&mut self, key: &str) -> Result<(), AppError> {
        self.conn
            .execute("DELETE FROM app_preferences WHERE key = ?1", params![key])?;
        Ok(())
    }
```

- [ ] **Step 4: Run test to verify it passes**

```powershell
cargo test -p sobria-app default_datacenter_id_round_trip 2>&1 | Select-Object -Last 10
```

Expected: PASS.

- [ ] **Step 5: Confirm no other test broke**

```powershell
cargo test -p sobria-app --lib preferences_store 2>&1 | Select-Object -Last 10
```

Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/sobria-app/src/preferences_store.rs
git commit -m "feat(app): add default_datacenter_id to preferences store (C25)"
```

---

### Task A2: Expose `default_datacenter_id` in the IPC `AppPreferencesDto`

**Files:**
- Modify: `crates/sobria-app/src/dto.rs`
- Modify: `crates/sobria-app/src/logic.rs` (`get_app_preferences`, `set_app_preferences`)
- Test: `crates/sobria-app/src/dto.rs` `mod tests`

- [ ] **Step 1: Write the failing test**

Add to `dto.rs` `mod tests`:

```rust
    #[test]
    fn app_preferences_dto_round_trip_with_default_datacenter_id() {
        let dto = AppPreferencesDto {
            persona: None,
            enabled_modules: vec![],
            onboarded: false,
            lang: "fr".into(),
            default_method: sobria_core::EmpreinteMethod::AfnorSobria,
            also_show_methods: vec![],
            default_datacenter_id: Some("ovh-gra-gravelines".into()),
        };
        let json = serde_json::to_string(&dto).unwrap();
        assert!(json.contains("default_datacenter_id"));
        let back: AppPreferencesDto = serde_json::from_str(&json).unwrap();
        assert_eq!(back.default_datacenter_id.as_deref(), Some("ovh-gra-gravelines"));

        // Backward-compat: a JSON without the field must deserialize with None.
        let legacy = serde_json::json!({
            "persona": null,
            "enabled_modules": [],
            "onboarded": false,
            "lang": "fr",
            "default_method": "afnor_sobria",
            "also_show_methods": []
        });
        let parsed: AppPreferencesDto = serde_json::from_value(legacy).unwrap();
        assert!(parsed.default_datacenter_id.is_none());
    }
```

- [ ] **Step 2: Run test to verify it fails**

```powershell
cargo test -p sobria-app app_preferences_dto_round_trip_with_default_datacenter_id 2>&1 | Select-Object -Last 10
```

Expected: FAIL — field doesn't exist on `AppPreferencesDto`.

- [ ] **Step 3: Add the field to `AppPreferencesDto`**

Find `pub struct AppPreferencesDto` in `dto.rs` and add as the last field:

```rust
    /// Dernier datacenter sélectionné, pré-rempli au prochain chargement
    /// des routes /estimate, /comparer, /simuler (C25). `None` = pas de
    /// préfill, l'utilisateur part d'un picker vide.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_datacenter_id: Option<String>,
```

- [ ] **Step 4: Map field in `get_app_preferences` / `set_app_preferences`**

In `crates/sobria-app/src/logic.rs`, find `get_app_preferences` — at the `AppPreferencesDto { ... }` literal, add the field reading from `stored.default_datacenter_id`. Then in `set_app_preferences`, find where it calls `set_default_method(...)` and add a parallel `store.set_default_datacenter_id(prefs.default_datacenter_id.as_deref())?;`.

- [ ] **Step 5: Run round-trip test**

```powershell
cargo test -p sobria-app app_preferences_dto_round_trip_with_default_datacenter_id 2>&1 | Select-Object -Last 10
```

Expected: PASS.

- [ ] **Step 6: Verify all `AppPreferencesDto` literals in tests still compile**

```powershell
cargo build -p sobria-app --tests 2>&1 | Select-Object -Last 20
```

Expected: success. If any test literal of `AppPreferencesDto { ... }` is now missing `default_datacenter_id`, add `default_datacenter_id: None,` (the `#[serde(default)]` is only for serde, not for Rust struct literals).

- [ ] **Step 7: Commit**

```bash
git add crates/sobria-app/src/dto.rs crates/sobria-app/src/logic.rs
git commit -m "feat(app): expose default_datacenter_id in AppPreferencesDto (C25)"
```

---

### Task A3: Mirror `default_datacenter_id` in the TypeScript layer

**Files:**
- Modify: `web/src/lib/api.ts`
- Modify: `web/src/lib/preferences.ts` (`INITIAL`)
- Modify: `web/src/routes/onboarding/+page.svelte` (1 `savePreferences` call)
- Modify: `web/src/routes/parametres/+page.svelte` (5 `savePreferences` calls)

- [ ] **Step 1: Add field to `AppPreferencesDto` in api.ts**

Find `export interface AppPreferencesDto` (around line 752) and add as the last property:

```ts
  /** Dernier datacenter choisi pour pré-remplir le picker (C25). */
  default_datacenter_id?: string;
```

- [ ] **Step 2: Update `INITIAL` in `preferences.ts`**

Find the `INITIAL: PreferencesState` literal and add (preserve trailing comma style of file):

```ts
  default_datacenter_id: undefined,
```

just below `also_show_methods: [],`.

- [ ] **Step 3: Update the 6 `savePreferences({...})` call sites**

Run this grep to find them: `grep -n "savePreferences({" web/src/routes/onboarding/+page.svelte web/src/routes/parametres/+page.svelte`

For each `savePreferences({...})` block, add as the last field of the object literal:

```ts
        default_datacenter_id: $preferences.default_datacenter_id
```

(For the onboarding call, where `$preferences` isn't imported, use `default_datacenter_id: undefined` since onboarding always starts from blank state.)

- [ ] **Step 4: Verify check + lint**

```powershell
cd web; npm run check 2>&1 | Select-Object -Last 10
```

Expected: 0 errors (the existing tsconfig `node` warning is preexisting).

```powershell
npm run lint 2>&1 | Select-Object -Last 10
```

Expected: clean. If prettier flags formatting, run `npx prettier --write src/lib/api.ts src/lib/preferences.ts src/routes/onboarding/+page.svelte src/routes/parametres/+page.svelte`.

- [ ] **Step 5: Commit**

```bash
git add web/src/lib/api.ts web/src/lib/preferences.ts web/src/routes/onboarding/+page.svelte web/src/routes/parametres/+page.svelte
git commit -m "feat(web): mirror default_datacenter_id in AppPreferencesDto (C25)"
```

---

### Task A4: `apply_datacenter_override` helper + unit tests

**Files:**
- Modify: `crates/sobria-app/src/logic.rs`
- Test: same file, `mod tests`

- [ ] **Step 1: Write the failing tests**

Add to `logic.rs` `mod tests` (find an existing `estimate_prompt_*` test for placement):

```rust
    #[test]
    fn apply_datacenter_override_replaces_pue_if_and_wue() {
        use sobria_estimator::distributions::Distribution;
        let mut params = sobria_estimator::params::EstimationParams::for_model("gpt-4o-mini").unwrap();
        // Choose a known DC id from `crates/sobria-geoloc/data/datacenters.json`.
        // Pick one with a WUE so all 3 indicators are exercised.
        apply_datacenter_override(&mut params, Some("ovh-gra-gravelines")).unwrap();
        assert!(matches!(params.pue, Distribution::Point { .. }));
        assert!(matches!(params.if_electrical_g_per_kwh, Distribution::Point { .. }));
        // WUE override is conditional on the DC record having Some(wue).
        // If `ovh-gra-gravelines` has WUE in the data file, this assertion holds;
        // if not, swap the id for one that does (run `listDatacenters` in /datacenters to see which).
    }

    #[test]
    fn apply_datacenter_override_unknown_id_returns_invalid_request() {
        let mut params = sobria_estimator::params::EstimationParams::for_model("gpt-4o-mini").unwrap();
        let err = apply_datacenter_override(&mut params, Some("does-not-exist")).unwrap_err();
        assert_eq!(err.code(), "invalid_request");
    }

    #[test]
    fn apply_datacenter_override_none_is_noop() {
        use sobria_estimator::distributions::Distribution;
        let mut params = sobria_estimator::params::EstimationParams::for_model("gpt-4o-mini").unwrap();
        let original_pue = params.pue;
        apply_datacenter_override(&mut params, None).unwrap();
        // PartialEq isn't derived on Distribution; compare by Debug repr as a cheap proxy.
        assert_eq!(format!("{:?}", params.pue), format!("{:?}", original_pue));
    }
```

- [ ] **Step 2: Run tests to verify they fail**

```powershell
cargo test -p sobria-app apply_datacenter_override 2>&1 | Select-Object -Last 15
```

Expected: FAIL — `apply_datacenter_override` doesn't exist.

- [ ] **Step 3: Implement the helper**

Add to `logic.rs`, anywhere near the top of the file after the `use` block and before the first `pub fn`:

```rust
/// Quand l'utilisateur a sélectionné un datacenter, surcharge `params.pue`,
/// `params.if_electrical_g_per_kwh` et — si la fiche DC en a un —
/// `params.wue_l_per_kwh` par des `Distribution::Point` dérivés du record.
///
/// Erreurs :
/// - `InvalidRequest` si l'id est inconnu (la liste UI vient toujours de
///   `list_datacenters` donc un id orphelin est forcément un bug).
fn apply_datacenter_override(
    params: &mut sobria_estimator::params::EstimationParams,
    datacenter_id: Option<&str>,
) -> IpcResult<()> {
    let Some(id) = datacenter_id else {
        return Ok(());
    };
    let dc = sobria_geoloc::find_datacenter(id).ok_or_else(|| {
        IpcError::from(AppError::InvalidRequest(format!(
            "datacenter inconnu : {id}"
        )))
    })?;
    params.pue = sobria_estimator::distributions::Distribution::Point { value: dc.pue };
    params.if_electrical_g_per_kwh = sobria_estimator::distributions::Distribution::Point {
        value: dc.if_electrical_g_per_kwh,
    };
    if let Some(wue) = dc.wue_l_per_kwh {
        params.wue_l_per_kwh = sobria_estimator::distributions::Distribution::Point { value: wue };
    }
    Ok(())
}
```

- [ ] **Step 4: Run tests**

```powershell
cargo test -p sobria-app apply_datacenter_override 2>&1 | Select-Object -Last 10
```

Expected: all 3 pass. If the WUE test fails for `ovh-gra-gravelines`, open `crates/sobria-geoloc/data/datacenters.json`, find a DC with `wue_l_per_kwh` set, and swap the id in the test.

- [ ] **Step 5: Commit**

```bash
git add crates/sobria-app/src/logic.rs
git commit -m "feat(app): add apply_datacenter_override helper (C25)"
```

---

### Task A5: Wire `apply_datacenter_override` into `estimate_prompt` + persist last pick

**Files:**
- Modify: `crates/sobria-app/src/logic.rs::estimate_prompt`
- Test: same file, `mod tests`

- [ ] **Step 1: Write the failing tests**

```rust
    #[test]
    fn estimate_prompt_with_datacenter_overrides_indicators() {
        let (_tmp, state) = fresh_state();
        let baseline = estimate_prompt(
            EstimationRequestDto {
                model_id: "gpt-4o-mini".into(),
                tokens_in: 100,
                tokens_out_estimated: 500,
                datacenter_id: None,
                method: None,
            },
            &state,
        )
        .unwrap();
        let with_dc = estimate_prompt(
            EstimationRequestDto {
                model_id: "gpt-4o-mini".into(),
                tokens_in: 100,
                tokens_out_estimated: 500,
                datacenter_id: Some("ovh-gra-gravelines".into()),
                method: None,
            },
            &state,
        )
        .unwrap();
        let baseline_co2 = baseline
            .indicators
            .iter()
            .find(|i| i.indicator == "co2eq")
            .unwrap()
            .p50;
        let with_dc_co2 = with_dc
            .indicators
            .iter()
            .find(|i| i.indicator == "co2eq")
            .unwrap()
            .p50;
        assert!(
            (baseline_co2 - with_dc_co2).abs() > 1e-9,
            "DC override should move CO2 P50 ({baseline_co2} vs {with_dc_co2})"
        );
    }

    #[test]
    fn estimate_prompt_with_unknown_datacenter_returns_invalid_request() {
        let (_tmp, state) = fresh_state();
        let err = estimate_prompt(
            EstimationRequestDto {
                model_id: "gpt-4o-mini".into(),
                tokens_in: 100,
                tokens_out_estimated: 500,
                datacenter_id: Some("does-not-exist".into()),
                method: None,
            },
            &state,
        )
        .unwrap_err();
        assert_eq!(err.code(), "invalid_request");
    }

    #[test]
    fn estimate_prompt_persists_default_datacenter_id() {
        let (_tmp, state) = fresh_state();
        let _ = estimate_prompt(
            EstimationRequestDto {
                model_id: "gpt-4o-mini".into(),
                tokens_in: 100,
                tokens_out_estimated: 500,
                datacenter_id: Some("ovh-gra-gravelines".into()),
                method: None,
            },
            &state,
        )
        .unwrap();
        let stored = state.preferences.lock().unwrap().read_all().unwrap();
        assert_eq!(stored.default_datacenter_id.as_deref(), Some("ovh-gra-gravelines"));

        // Reverting to None must also persist.
        let _ = estimate_prompt(
            EstimationRequestDto {
                model_id: "gpt-4o-mini".into(),
                tokens_in: 100,
                tokens_out_estimated: 500,
                datacenter_id: None,
                method: None,
            },
            &state,
        )
        .unwrap();
        let stored2 = state.preferences.lock().unwrap().read_all().unwrap();
        assert!(stored2.default_datacenter_id.is_none());
    }
```

- [ ] **Step 2: Run tests to verify they fail**

```powershell
cargo test -p sobria-app estimate_prompt_with_datacenter estimate_prompt_with_unknown_datacenter estimate_prompt_persists_default 2>&1 | Select-Object -Last 20
```

Expected: failures (CO2 unchanged, unknown id silently passes, prefs not persisted).

- [ ] **Step 3: Wire helper + persistence into `estimate_prompt`**

In `logic.rs::estimate_prompt`, after `let params = EstimationParams::for_model(&model_id).map_err(AppError::from)?;`, add:

```rust
    let mut params = params; // make mutable
    let datacenter_id = req.datacenter_id.clone();
    apply_datacenter_override(&mut params, datacenter_id.as_deref())?;
```

After the successful `ledger.append(...)` block and `drop(ledger);`, add the persistence side-effect:

```rust
    // Persistance "last used" (C25). Best-effort : un échec ne casse pas
    // l'estimation, on log et on continue.
    if let Ok(mut store) = state.preferences.lock() {
        if let Err(e) = store.set_default_datacenter_id(datacenter_id.as_deref()) {
            tracing::warn!(error = ?e, "set_default_datacenter_id échoué (non-bloquant)");
        }
    }
```

- [ ] **Step 4: Run tests**

```powershell
cargo test -p sobria-app estimate_prompt 2>&1 | Select-Object -Last 15
```

Expected: all `estimate_prompt_*` tests pass (existing ones + new ones).

- [ ] **Step 5: Commit**

```bash
git add crates/sobria-app/src/logic.rs
git commit -m "feat(app): wire DC override + persist last pick in estimate_prompt (C25)"
```

---

### Task A6: Wire into `estimate_for_comparison`

**Files:**
- Modify: `crates/sobria-app/src/logic.rs::estimate_for_comparison`
- Test: same file

- [ ] **Step 1: Write the failing test**

```rust
    #[test]
    fn estimate_for_comparison_honors_datacenter_id() {
        let baseline = estimate_for_comparison(EstimationRequestDto {
            model_id: "gpt-4o-mini".into(),
            tokens_in: 100,
            tokens_out_estimated: 500,
            datacenter_id: None,
            method: Some(sobria_core::EmpreinteMethod::AfnorSobria),
        })
        .unwrap();
        let with_dc = estimate_for_comparison(EstimationRequestDto {
            model_id: "gpt-4o-mini".into(),
            tokens_in: 100,
            tokens_out_estimated: 500,
            datacenter_id: Some("ovh-gra-gravelines".into()),
            method: Some(sobria_core::EmpreinteMethod::AfnorSobria),
        })
        .unwrap();
        let b = baseline.indicators.iter().find(|i| i.indicator == "co2eq").unwrap().p50;
        let d = with_dc.indicators.iter().find(|i| i.indicator == "co2eq").unwrap().p50;
        assert!((b - d).abs() > 1e-9);
    }
```

- [ ] **Step 2: Run test to verify it fails**

```powershell
cargo test -p sobria-app estimate_for_comparison_honors_datacenter_id 2>&1 | Select-Object -Last 10
```

Expected: FAIL — CO2 unchanged.

- [ ] **Step 3: Wire helper**

In `logic.rs::estimate_for_comparison`, after `let params = EstimationParams::for_model(...)?;`, mirror Task A5:

```rust
    let mut params = params;
    apply_datacenter_override(&mut params, req.datacenter_id.as_deref())?;
```

No persistence in this path — `estimate_for_comparison` is stateless (already documented in its rustdoc).

- [ ] **Step 4: Run test**

```powershell
cargo test -p sobria-app estimate_for_comparison 2>&1 | Select-Object -Last 10
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/sobria-app/src/logic.rs
git commit -m "feat(app): honor datacenter_id in estimate_for_comparison (C25)"
```

---

### Task A7: Wire into `benchmark_models` (M3 /comparer)

**Files:**
- Modify: `crates/sobria-app/src/logic.rs::benchmark_models`
- Test: same file

- [ ] **Step 1: Write the failing test**

```rust
    #[test]
    fn benchmark_with_datacenter_applies_to_every_model() {
        let (_tmp, state) = fresh_state();
        let baseline = benchmark_models(
            BenchmarkRequestDto {
                model_ids: vec!["gpt-4o-mini".into(), "claude-3-5-sonnet".into()],
                tokens_in: 100,
                tokens_out_estimated: 500,
                datacenter_id: None,
            },
            &state,
        )
        .unwrap();
        let with_dc = benchmark_models(
            BenchmarkRequestDto {
                model_ids: vec!["gpt-4o-mini".into(), "claude-3-5-sonnet".into()],
                tokens_in: 100,
                tokens_out_estimated: 500,
                datacenter_id: Some("ovh-gra-gravelines".into()),
            },
            &state,
        )
        .unwrap();
        // Every outcome's CO2 must shift consistently when the DC changes.
        for (b, d) in baseline.outcomes.iter().zip(with_dc.outcomes.iter()) {
            assert!((b.co2eq_p50_g - d.co2eq_p50_g).abs() > 1e-9, "model {} unchanged", b.model_id);
        }
    }
```

- [ ] **Step 2: Run test to verify it fails**

```powershell
cargo test -p sobria-app benchmark_with_datacenter 2>&1 | Select-Object -Last 10
```

Expected: FAIL.

- [ ] **Step 3: Wire helper inside the per-model loop**

In `logic.rs::benchmark_models`, the inner loop currently builds an `EstimationRequestDto { ..., datacenter_id: req.datacenter_id.clone(), ..., method: None }` per model and calls `estimate_prompt(est_req, state)`. That call already does the override (Task A5). **But** the persistence side-effect of A5 will fire once per model — which is fine semantically (it converges to the same id) but wasteful.

If `benchmark_models` is calling `estimate_prompt` (verify by reading lines around 1244 in `logic.rs`), the helper is already applied transitively. **In that case skip step 3**, the test should pass simply because A5's wiring covers it.

If `benchmark_models` does NOT call `estimate_prompt` (it might call the engine directly), apply the helper after each `EstimationParams::for_model(...)`.

- [ ] **Step 4: Run test**

```powershell
cargo test -p sobria-app benchmark 2>&1 | Select-Object -Last 10
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/sobria-app/src/logic.rs
git commit -m "feat(app): verify benchmark_models honors datacenter_id (C25)"
```

---

### Task A8: Wire into `simulate` (M13 /simuler) baseline

**Files:**
- Modify: `crates/sobria-app/src/logic.rs` — the IPC handler that takes a `SimulationRequestDto` containing a `baseline: EstimationRequestDto`
- Test: same file

- [ ] **Step 1: Write the failing test**

```rust
    #[test]
    fn simulate_baseline_with_datacenter_applies_override() {
        let (_tmp, state) = fresh_state();
        let baseline = simulate(
            SimulationRequestDto {
                baseline: EstimationRequestDto {
                    model_id: "gpt-4o-mini".into(),
                    tokens_in: 100,
                    tokens_out_estimated: 500,
                    datacenter_id: None,
                    method: None,
                },
                scenarios: vec![],
                forecast: None,
            },
            &state,
        )
        .unwrap();
        let with_dc = simulate(
            SimulationRequestDto {
                baseline: EstimationRequestDto {
                    model_id: "gpt-4o-mini".into(),
                    tokens_in: 100,
                    tokens_out_estimated: 500,
                    datacenter_id: Some("ovh-gra-gravelines".into()),
                    method: None,
                },
                scenarios: vec![],
                forecast: None,
            },
            &state,
        )
        .unwrap();
        let b = baseline.baseline.indicators.iter().find(|i| matches!(i.indicator, sobria_core::Indicator::Co2Eq)).unwrap().interval.p50;
        let d = with_dc.baseline.indicators.iter().find(|i| matches!(i.indicator, sobria_core::Indicator::Co2Eq)).unwrap().interval.p50;
        assert!((b - d).abs() > 1e-9);
    }
```

- [ ] **Step 2: Run test to verify it fails**

```powershell
cargo test -p sobria-app simulate_baseline_with_datacenter 2>&1 | Select-Object -Last 10
```

Expected: FAIL.

- [ ] **Step 3: Wire helper into the simulation IPC handler**

In `logic.rs`, find the `simulate` IPC function (it builds a `core_baseline_req` from `req.baseline.into_core(...)` and then calls into `sobria_estimator::simulate`). Before calling the estimator, fetch `EstimationParams::for_model(&req.baseline.model_id)?`, apply the override using `req.baseline.datacenter_id.as_deref()`, then pass the mutated `params` down to `sobria_estimator::simulate`. **Inspect the existing code path** to confirm whether the estimator's `simulate` already takes `params` as an input or constructs them internally. If the latter, you'll need to refactor the estimator's `simulate` signature to accept a `&EstimationParams` (already done if you look — it does, since C24's polish G). If not, apply the override here.

- [ ] **Step 4: Run test**

```powershell
cargo test -p sobria-app simulate_baseline 2>&1 | Select-Object -Last 10
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/sobria-app/src/logic.rs
git commit -m "feat(app): honor datacenter_id in simulate baseline (C25)"
```

---

### Task A9: Wire into the batch CSV handler

**Files:**
- Modify: `crates/sobria-app/src/logic.rs` — batch row-processing loop (around line 815 where `est_req` is built per row)
- Test: same file (the existing batch tests use no `datacenter_id`; add one that does)

- [ ] **Step 1: Write the failing test**

```rust
    #[test]
    fn batch_row_with_datacenter_id_applies_override() {
        let (_tmp, state) = fresh_state();
        // Two-row batch: first without DC, second with. CO2 should differ.
        let rows = vec![
            batch::BatchInputRow {
                model_id: "gpt-4o-mini".into(),
                tokens_in: 100,
                tokens_out: 500,
                datacenter_id: None,
            },
            batch::BatchInputRow {
                model_id: "gpt-4o-mini".into(),
                tokens_in: 100,
                tokens_out: 500,
                datacenter_id: Some("ovh-gra-gravelines".into()),
            },
        ];
        let result = run_batch_rows(rows, &state).unwrap();
        assert!((result.output_rows[0].co2eq_p50_g - result.output_rows[1].co2eq_p50_g).abs() > 1e-9);
    }
```

Adjust constructor names (`BatchInputRow`, `run_batch_rows`) to match the actual API in `logic.rs` and `batch.rs` — read the existing batch tests to verify.

- [ ] **Step 2: Run test to verify it fails**

```powershell
cargo test -p sobria-app batch_row_with_datacenter 2>&1 | Select-Object -Last 10
```

Expected: FAIL.

- [ ] **Step 3: Verify the row loop already calls `estimate_prompt`**

Read `logic.rs` around line 815. The current loop builds an `EstimationRequestDto` per row, including `datacenter_id: row.datacenter_id.clone()`, then calls `estimate_prompt(est_req, state)`. Since Task A5 wired the override into `estimate_prompt`, **no change needed in batch**. The test should now pass without further code edits — go to step 4.

- [ ] **Step 4: Run test**

```powershell
cargo test -p sobria-app batch_row_with_datacenter 2>&1 | Select-Object -Last 10
```

Expected: PASS. If still failing, inspect the loop — `estimate_prompt`'s call signature may have changed since A5.

- [ ] **Step 5: Commit**

```bash
git add crates/sobria-app/src/logic.rs
git commit -m "test(app): cover batch CSV per-row datacenter_id override (C25)"
```

---

### Task A10: Create `DatacenterPicker.svelte` component

**Files:**
- Create: `web/src/lib/components/DatacenterPicker.svelte`

- [ ] **Step 1: Write the component**

```svelte
<script lang="ts">
  import { Server, Search, ChevronDown, X } from '@lucide/svelte';
  import type { DatacenterSummaryDto } from '$lib/api';

  interface Props {
    datacenters: DatacenterSummaryDto[];
    selected: DatacenterSummaryDto | null;
  }

  let { datacenters, selected = $bindable() }: Props = $props();

  let open = $state(false);
  let query = $state('');
  let activeIndex = $state(0);

  const flagFor = (iso: string): string => {
    // ISO-2 letters → regional indicator emoji.
    if (iso.length !== 2) return '🏳️';
    const base = 0x1f1e6;
    const a = iso.toUpperCase().charCodeAt(0) - 65;
    const b = iso.toUpperCase().charCodeAt(1) - 65;
    return String.fromCodePoint(base + a, base + b);
  };

  const countryName = (iso: string): string => {
    try {
      return new Intl.DisplayNames(['fr'], { type: 'region' }).of(iso) ?? iso;
    } catch {
      return iso;
    }
  };

  const filtered = $derived(
    query.trim() === ''
      ? datacenters
      : datacenters.filter((dc) => {
          const q = query.toLowerCase();
          return (
            dc.name.toLowerCase().includes(q) ||
            dc.city.toLowerCase().includes(q) ||
            dc.operator.toLowerCase().includes(q) ||
            dc.country_iso.toLowerCase().includes(q) ||
            countryName(dc.country_iso).toLowerCase().includes(q)
          );
        })
  );

  const grouped = $derived.by(() => {
    const map = new Map<string, DatacenterSummaryDto[]>();
    for (const dc of filtered) {
      const arr = map.get(dc.country_iso) ?? [];
      arr.push(dc);
      map.set(dc.country_iso, arr);
    }
    return Array.from(map.entries()).sort(([a], [b]) => countryName(a).localeCompare(countryName(b)));
  });

  function toggle() {
    open = !open;
    if (open) {
      query = '';
      activeIndex = 0;
    }
  }

  function pick(dc: DatacenterSummaryDto | null) {
    selected = dc;
    open = false;
  }

  function onKey(e: KeyboardEvent) {
    if (!open) return;
    if (e.key === 'Escape') {
      e.preventDefault();
      open = false;
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      activeIndex = Math.min(activeIndex + 1, filtered.length); // +1 to allow "Aucun" sentinel at index 0
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      activeIndex = Math.max(activeIndex - 1, 0);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (activeIndex === 0) pick(null);
      else pick(filtered[activeIndex - 1] ?? null);
    }
  }

  function onClickOutside(e: MouseEvent) {
    if (!open) return;
    const root = e.currentTarget as HTMLElement;
    if (!root.contains(e.target as Node)) open = false;
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="picker" role="combobox" aria-expanded={open} aria-haspopup="listbox" onclickcapture={onClickOutside}>
  <button type="button" class="trigger" onclick={toggle} aria-label="Choisir un datacenter">
    <span class="ico"><Server size={18} strokeWidth={1.6} /></span>
    {#if selected}
      <span class="col">
        <span class="ll">Datacenter</span>
        <span class="vv">{flagFor(selected.country_iso)} {selected.name} · {selected.city}</span>
        <span class="vm">{selected.operator} · {selected.if_electrical_g_per_kwh.toFixed(0)} g/kWh · PUE {selected.pue.toFixed(2)}</span>
      </span>
    {:else}
      <span class="col">
        <span class="ll">Datacenter</span>
        <span class="vv">Aucun choisi</span>
        <span class="vm">L'estimation utilise vos PUE/IF par défaut</span>
      </span>
    {/if}
    <span class="chev" aria-hidden="true"><ChevronDown size={14} strokeWidth={1.8} /></span>
  </button>

  {#if open}
    <div class="panel" role="listbox" tabindex="-1">
      <div class="search">
        <Search size={14} strokeWidth={1.8} />
        <input
          type="text"
          bind:value={query}
          placeholder="Rechercher (nom, ville, opérateur, pays)…"
          autocomplete="off"
          data-autofocus
        />
        {#if query}
          <button type="button" class="clear" aria-label="Effacer la recherche" onclick={() => (query = '')}>
            <X size={12} strokeWidth={2} />
          </button>
        {/if}
      </div>

      <ul class="options">
        <li
          class="option none"
          class:active={activeIndex === 0}
          role="option"
          aria-selected={selected === null}
          onclick={() => pick(null)}
        >
          <span class="opt-label">Aucun choisi</span>
          <span class="opt-meta">Utilise les paramètres par défaut</span>
        </li>
        {#each grouped as [iso, dcs]}
          <li class="group" role="separator">
            <span>{flagFor(iso)} {countryName(iso)}</span>
          </li>
          {#each dcs as dc}
            {@const flatIndex = filtered.indexOf(dc) + 1}
            <li
              class="option"
              class:active={activeIndex === flatIndex}
              role="option"
              aria-selected={selected?.id === dc.id}
              onclick={() => pick(dc)}
            >
              <span class="opt-label">{dc.name} · {dc.city}</span>
              <span class="opt-meta">{dc.operator} · {dc.if_electrical_g_per_kwh.toFixed(0)} g/kWh · PUE {dc.pue.toFixed(2)}</span>
            </li>
          {/each}
        {/each}
        {#if filtered.length === 0}
          <li class="empty">Aucun datacenter ne correspond.</li>
        {/if}
      </ul>
    </div>
  {/if}
</div>

<style>
  .picker {
    position: relative;
    width: 100%;
  }
  .trigger {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: 12px;
    width: 100%;
    padding: 12px 14px;
    background: var(--surface);
    border: 1px solid color-mix(in oklab, var(--ink-mute) 14%, transparent);
    border-radius: 12px;
    cursor: pointer;
    text-align: left;
  }
  .trigger:hover {
    border-color: color-mix(in oklab, var(--ink-mute) 28%, transparent);
  }
  .ico {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    background: color-mix(in oklab, var(--accent) 12%, transparent);
    border-radius: 8px;
    color: var(--accent);
  }
  .col {
    display: grid;
    gap: 2px;
    min-width: 0;
  }
  .ll {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--ink-mute);
  }
  .vv {
    font-size: 14px;
    color: var(--ink);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .vm {
    font-size: 12px;
    color: var(--ink-mute);
    font-feature-settings: 'tnum';
  }
  .chev {
    color: var(--ink-mute);
  }
  .panel {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    right: 0;
    z-index: 30;
    background: var(--surface);
    border: 1px solid color-mix(in oklab, var(--ink-mute) 16%, transparent);
    border-radius: 12px;
    box-shadow: 0 12px 32px color-mix(in oklab, black 12%, transparent);
    max-height: 360px;
    display: grid;
    grid-template-rows: auto 1fr;
    overflow: hidden;
  }
  .search {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: 8px;
    padding: 10px 12px;
    border-bottom: 1px solid color-mix(in oklab, var(--ink-mute) 10%, transparent);
    color: var(--ink-mute);
  }
  .search input {
    border: none;
    outline: none;
    background: transparent;
    color: var(--ink);
    font-size: 13px;
    width: 100%;
  }
  .clear {
    background: transparent;
    border: none;
    color: var(--ink-mute);
    cursor: pointer;
  }
  .options {
    list-style: none;
    padding: 6px 0;
    margin: 0;
    overflow-y: auto;
  }
  .group {
    padding: 8px 14px 4px;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--ink-mute);
  }
  .option {
    display: grid;
    gap: 1px;
    padding: 8px 14px;
    cursor: pointer;
  }
  .option:hover,
  .option.active {
    background: color-mix(in oklab, var(--accent) 8%, transparent);
  }
  .option .opt-label {
    font-size: 13px;
    color: var(--ink);
  }
  .option .opt-meta {
    font-size: 11px;
    color: var(--ink-mute);
    font-feature-settings: 'tnum';
  }
  .option.none {
    border-bottom: 1px solid color-mix(in oklab, var(--ink-mute) 8%, transparent);
  }
  .empty {
    padding: 14px;
    text-align: center;
    color: var(--ink-mute);
    font-size: 12px;
  }
</style>
```

- [ ] **Step 2: Verify svelte-check passes**

```powershell
cd web; npm run check 2>&1 | Select-Object -Last 10
```

Expected: 0 errors (the `tsconfig.json` `node` types warning is preexisting and unrelated).

- [ ] **Step 3: Run prettier**

```powershell
npx prettier --write src/lib/components/DatacenterPicker.svelte
```

- [ ] **Step 4: Commit**

```bash
git add web/src/lib/components/DatacenterPicker.svelte
git commit -m "feat(web): add DatacenterPicker component (C25)"
```

---

### Task A11: Wire `DatacenterPicker` into Composer + /estimate route

**Files:**
- Modify: `web/src/lib/components/Composer.svelte` (replace the hardcoded card around lines 200-213)
- Modify: `web/src/routes/+page.svelte` (M1) (bootstrap `listDatacenters`, pre-fill, pass to Composer, include in `estimatePrompt`)

- [ ] **Step 1: Composer accepts new props**

In `Composer.svelte` `<script>`:

```ts
import DatacenterPicker from './DatacenterPicker.svelte';
import type { DatacenterSummaryDto } from '$lib/api';

interface Props {
  // ... existing props
  datacenters: DatacenterSummaryDto[];
  selectedDatacenter: DatacenterSummaryDto | null;
}

let { /* existing */, datacenters, selectedDatacenter = $bindable() }: Props = $props();
```

- [ ] **Step 2: Replace the hardcoded card markup**

In `Composer.svelte` template, find the block (around lines 200-213) starting with `<div class="context-card">` containing `Datacenter (auto)` and ending with the closing `</div>`. Replace with:

```svelte
<div class="context-row">
  <DatacenterPicker {datacenters} bind:selected={selectedDatacenter} />
  <div class="context-card">
    <!-- … existing "Mix électrique · LIVE" card stays unchanged … -->
```

(Keep the second card — only the first one is replaced.)

- [ ] **Step 3: Bootstrap in `/+page.svelte`**

In `web/src/routes/+page.svelte` `<script>`, after the existing model loading:

```ts
import { listDatacenters, type DatacenterSummaryDto } from '$lib/api';

let datacenters = $state<DatacenterSummaryDto[]>([]);
let selectedDatacenter = $state<DatacenterSummaryDto | null>(null);

$effect(() => {
  void (async () => {
    if (!isTauriContext()) return;
    try {
      datacenters = await listDatacenters();
      // Pre-fill from prefs once both list and prefs are loaded.
      const def = $preferences.default_datacenter_id;
      if (def && !selectedDatacenter) {
        selectedDatacenter = datacenters.find((d) => d.id === def) ?? null;
      }
    } catch (e) {
      // Non-blocking: picker stays disabled with empty list.
      console.warn('listDatacenters failed', e);
    }
  })();
});
```

Pass to Composer:

```svelte
<Composer
  ...existing props...
  {datacenters}
  bind:selectedDatacenter
/>
```

In the estimate submission handler, include `datacenter_id`:

```ts
const dto = await estimatePrompt({
  model_id: selectedModelId,
  tokens_in: tokensIn,
  tokens_out_estimated: tokensOut,
  datacenter_id: selectedDatacenter?.id ?? undefined,
  method
});
```

- [ ] **Step 4: Run check + lint**

```powershell
cd web; npm run check 2>&1 | Select-Object -Last 8
```

Expected: 0 errors.

```powershell
npm run lint 2>&1 | Select-Object -Last 6
```

Expected: clean.

- [ ] **Step 5: Manual smoke (Tauri dev)**

In a separate terminal: `cargo tauri dev` from `crates/sobria-app/`. Open the app → /estimate → click the Datacenter card → verify the dropdown opens with a search field and country-grouped list. Pick one. Run an estimation. Close the app.

- [ ] **Step 6: Commit**

```bash
git add web/src/lib/components/Composer.svelte web/src/routes/+page.svelte
git commit -m "feat(web): wire DatacenterPicker into Composer + /estimate (C25)"
```

---

### Task A12: Wire picker into /comparer (M3)

**Files:**
- Modify: `web/src/routes/comparer/+page.svelte`

- [ ] **Step 1: Bootstrap + render**

Mirror Task A11's bootstrap pattern. Add `<DatacenterPicker {datacenters} bind:selected={selectedDatacenter} />` above the model selection grid. In the submit handler, include `datacenter_id: selectedDatacenter?.id ?? undefined` in the `BenchmarkRequestDto` payload.

If `BenchmarkRequestDto` doesn't yet have `datacenter_id` in `api.ts`, check — it should already (since the Rust `BenchmarkRequestDto` has it). If not, add `datacenter_id?: string;` to the TS interface in `api.ts`.

- [ ] **Step 2: Run check + lint**

```powershell
cd web; npm run check 2>&1 | Select-Object -Last 6
npm run lint 2>&1 | Select-Object -Last 6
```

Expected: clean.

- [ ] **Step 3: Manual smoke**

In the running tauri dev, open /comparer → pick 2 models + a DC → compare → verify both outcomes differ vs. no-DC baseline.

- [ ] **Step 4: Commit**

```bash
git add web/src/routes/comparer/+page.svelte web/src/lib/api.ts
git commit -m "feat(web): wire DatacenterPicker into /comparer (C25)"
```

---

### Task A13: Wire picker into /simuler (M13)

**Files:**
- Modify: `web/src/routes/simuler/+page.svelte`

- [ ] **Step 1: Bootstrap + render**

Same pattern as A11/A12, applied to the baseline section of /simuler. The picker controls the baseline `EstimationRequestDto`; scenarios with their own `ParamOverrides` still override on top (intentional — that's how /simuler is designed).

- [ ] **Step 2: Check + lint**

```powershell
cd web; npm run check 2>&1 | Select-Object -Last 6
npm run lint 2>&1 | Select-Object -Last 6
```

Expected: clean.

- [ ] **Step 3: Commit**

```bash
git add web/src/routes/simuler/+page.svelte
git commit -m "feat(web): wire DatacenterPicker into /simuler baseline (C25)"
```

---

### Task A14: Playwright e2e — picker behavior + persistence

**Files:**
- Create: `web/tests/datacenter-picker.spec.ts`

- [ ] **Step 1: Write the test**

```ts
import { test, expect } from '@playwright/test';

test.describe('C25 DatacenterPicker', () => {
  test('open, search, pick, estimate, refresh, picker pre-filled', async ({ page }) => {
    await page.goto('/');
    // The picker trigger lives inside the Composer; find by its accessible name.
    const trigger = page.getByRole('button', { name: /Choisir un datacenter/i });
    await expect(trigger).toBeVisible();
    await trigger.click();

    const search = page.getByPlaceholder(/Rechercher/);
    await search.fill('ovh');
    const firstOvh = page.getByRole('option').filter({ hasText: /OVH/i }).first();
    await firstOvh.click();

    // Picker closes and shows the chosen DC.
    await expect(trigger).toContainText(/OVH/i);

    // Run an estimation (whatever the page provides — submit button).
    const submit = page.getByRole('button', { name: /Estimer/i });
    if (await submit.isEnabled()) {
      await submit.click();
      // Result shows a numeric P50.
      await expect(page.getByText(/CO₂eq|gCO/)).toBeVisible();
    }

    // Reload page — picker should be pre-filled with the same DC.
    await page.reload();
    await expect(page.getByRole('button', { name: /Choisir un datacenter/i })).toContainText(/OVH/i);
  });

  test('explicitly clearing to "Aucun choisi" persists', async ({ page }) => {
    await page.goto('/');
    const trigger = page.getByRole('button', { name: /Choisir un datacenter/i });
    await trigger.click();
    await page.getByRole('option', { name: /Aucun choisi/i }).click();
    await expect(trigger).toContainText(/Aucun choisi/i);
    // Trigger an estimate to flush persistence.
    const submit = page.getByRole('button', { name: /Estimer/i });
    if (await submit.isEnabled()) await submit.click();
    await page.reload();
    await expect(page.getByRole('button', { name: /Choisir un datacenter/i })).toContainText(/Aucun choisi/i);
  });
});
```

- [ ] **Step 2: Run the spec against a live tauri dev**

The project's Playwright config probably points at `http://localhost:5173` (vite). Start tauri dev in one terminal (so the IPC bridge is alive), then in another:

```powershell
cd web; npm run e2e -- datacenter-picker
```

Expected: both tests pass. If a selector is too generic and the page has multiple buttons matching, narrow with `.locator('.composer ...')` or similar — read `tests/m17.spec.ts` for existing patterns.

- [ ] **Step 3: Commit**

```bash
git add web/tests/datacenter-picker.spec.ts
git commit -m "test(web): e2e datacenter picker behavior + persistence (C25)"
```

---

### Phase A — Done check

Before moving to Phase B, verify the full picker bundle:

```powershell
cargo test -p sobria-app 2>&1 | Select-Object -Last 5
cargo clippy -p sobria-app -p sobria-core -p sobria-geoloc -p sobria-estimator -- -D warnings 2>&1 | Select-Object -Last 5
cd web; npm run check 2>&1 | Select-Object -Last 5
npm run lint 2>&1 | Select-Object -Last 5
```

All four must be clean before starting Phase B.

---

# Phase B — Immersive /datacenters

---

### Task B1: Restructure /datacenters layout to absolute-positioned

**Files:**
- Modify: `web/src/routes/datacenters/+page.svelte`

- [ ] **Step 1: Read current layout structure**

Open `web/src/routes/datacenters/+page.svelte` and find the template section. Note the existing grid/flex layout for `DatacenterMap`, `DatacenterFilters`, `DatacenterDrillDown`, `CountryDrillDown`.

- [ ] **Step 2: Rewrite the layout block**

Replace the existing wrapper container with an absolute-positioned scheme. Pseudo-structure:

```svelte
<div class="dc-route">
  <!-- Map fills 100% of the route content area -->
  <DatacenterMap
    {datacenters}
    {filters}
    bind:selectedDc
    bind:selectedCountry
    class="dc-map-fill"
  />

  <!-- Filters: top-left glass card -->
  <div class="dc-filters-overlay">
    <DatacenterFilters bind:state={filters} {datacenters} {countries} />
  </div>

  <!-- Drill-down: right glass panel, only when something is selected -->
  {#if selectedDc}
    <div class="dc-drill-overlay">
      <DatacenterDrillDown
        dc={selectedDc}
        detail={dcDetail}
        loading={dcDetailLoading}
        error={dcDetailError}
        on:close={() => (selectedDc = null)}
      />
    </div>
  {:else if selectedCountry}
    <div class="dc-drill-overlay">
      <CountryDrillDown
        country={selectedCountry}
        on:close={() => (selectedCountry = null)}
      />
    </div>
  {/if}
</div>
```

- [ ] **Step 3: CSS scoped to this route**

In the `<style>` block of `+page.svelte`:

```css
.dc-route {
  position: relative;
  width: 100%;
  height: 100%;
  min-height: calc(100vh - var(--app-header-h, 64px));
  overflow: hidden;
}
:global(.dc-map-fill) {
  position: absolute !important;
  inset: 0 !important;
  width: 100% !important;
  height: 100% !important;
}
.dc-filters-overlay {
  position: absolute;
  top: 16px;
  left: 16px;
  z-index: 5;
  max-width: 280px;
}
.dc-drill-overlay {
  position: absolute;
  top: 16px;
  right: 16px;
  bottom: 16px;
  width: 340px;
  z-index: 5;
}
```

If `DatacenterMap` already sets its own position/size internally, the `:global` selector above forces the override.

- [ ] **Step 4: Run check + lint**

```powershell
cd web; npm run check 2>&1 | Select-Object -Last 6
npm run lint 2>&1 | Select-Object -Last 6
```

Expected: clean.

- [ ] **Step 5: Manual smoke**

In tauri dev, open /datacenters → map fills the route → filters card overlaps top-left → click a pin → drill-down panel appears top-right and only when selected.

- [ ] **Step 6: Commit**

```bash
git add web/src/routes/datacenters/+page.svelte
git commit -m "feat(web): immersive layout for /datacenters (C25)"
```

---

### Task B2: Glass styling for `DatacenterFilters`

**Files:**
- Modify: `web/src/lib/components/m12/DatacenterFilters.svelte`

- [ ] **Step 1: Add glass styles to the root element**

Find the root `<aside>` / `<div>` of the component and add a class (e.g., `glass`). Append in the component's `<style>`:

```css
.glass {
  background: color-mix(in oklab, var(--surface) 70%, transparent);
  backdrop-filter: blur(14px) saturate(1.2);
  -webkit-backdrop-filter: blur(14px) saturate(1.2);
  border: 1px solid color-mix(in oklab, var(--ink-mute) 12%, transparent);
  border-radius: 14px;
  box-shadow: 0 8px 24px color-mix(in oklab, black 12%, transparent);
  padding: 14px;
}
```

If the component is consumed elsewhere outside /datacenters and you don't want the glass look there, gate via a prop `floating?: boolean` and conditionally add the class.

- [ ] **Step 2: Run check + lint**

```powershell
cd web; npm run check 2>&1 | Select-Object -Last 6
npm run lint 2>&1 | Select-Object -Last 6
```

Expected: clean.

- [ ] **Step 3: Commit**

```bash
git add web/src/lib/components/m12/DatacenterFilters.svelte
git commit -m "style(web): glass surface on DatacenterFilters (C25)"
```

---

### Task B3: Glass styling for `DatacenterDrillDown` + `CountryDrillDown`

**Files:**
- Modify: `web/src/lib/components/m12/DatacenterDrillDown.svelte`
- Modify: `web/src/lib/components/m12/CountryDrillDown.svelte`

- [ ] **Step 1: Add glass class to both root containers**

Same glass class as B2 on the root `<aside>` / panel element. Add a close button (×) if it doesn't already exist; emit `close` event so the route can null out `selectedDc` / `selectedCountry`.

- [ ] **Step 2: Run check + lint**

```powershell
cd web; npm run check 2>&1 | Select-Object -Last 6
npm run lint 2>&1 | Select-Object -Last 6
```

Expected: clean.

- [ ] **Step 3: Commit**

```bash
git add web/src/lib/components/m12/DatacenterDrillDown.svelte web/src/lib/components/m12/CountryDrillDown.svelte
git commit -m "style(web): glass surface on drill-down panels (C25)"
```

---

### Task B4: Playwright — immersive layout regression test

**Files:**
- Create: `web/tests/datacenters-immersive.spec.ts`

- [ ] **Step 1: Write the test**

```ts
import { test, expect } from '@playwright/test';

test('C25 /datacenters immersive layout', async ({ page }) => {
  await page.goto('/datacenters');

  // Map fills route: absolute, inset 0.
  const mapEl = page.locator('.dc-map-fill').first();
  await expect(mapEl).toBeVisible();
  const mapBox = await mapEl.boundingBox();
  const viewport = page.viewportSize();
  expect(mapBox).not.toBeNull();
  expect(viewport).not.toBeNull();
  // Map should be at least 80% of the viewport height (loose check
  // to accommodate the app shell header).
  expect(mapBox!.height).toBeGreaterThan(viewport!.height * 0.6);

  // Filters overlay is positioned absolute.
  const filters = page.locator('.dc-filters-overlay');
  await expect(filters).toBeVisible();
  const filtersPos = await filters.evaluate((el) => getComputedStyle(el).position);
  expect(filtersPos).toBe('absolute');

  // Drill-down is NOT in the DOM until something is selected.
  await expect(page.locator('.dc-drill-overlay')).toHaveCount(0);
});
```

- [ ] **Step 2: Run the spec**

```powershell
cd web; npm run e2e -- datacenters-immersive
```

Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add web/tests/datacenters-immersive.spec.ts
git commit -m "test(web): regression test for immersive /datacenters layout (C25)"
```

---

### Phase B — Done check

```powershell
cd web; npm run check 2>&1 | Select-Object -Last 5
npm run lint 2>&1 | Select-Object -Last 5
npm run e2e -- datacenter 2>&1 | Select-Object -Last 8
```

All clean.

---

# Wrap-up

### Final verification

- [ ] Run the full test suite:

```powershell
cargo test --workspace 2>&1 | Select-Object -Last 10
cargo clippy --workspace -- -D warnings 2>&1 | Select-Object -Last 5
cd web; npm run check 2>&1 | Select-Object -Last 5
npm run lint 2>&1 | Select-Object -Last 5
npm run e2e 2>&1 | Select-Object -Last 10
```

All four must be clean.

- [ ] Update `CHANGELOG.md` with a `## v0.5.0 — C25 (en cours)` block listing the picker, the override semantics, persistence, and the immersive map.

- [ ] Add a screenshot of `/estimate` with the picker open and `/datacenters` with the map immersive, attach to the PR description.

- [ ] Reference the spec (`briefs/chantiers/C25-datacenter-selection.md`) in the PR description.

---

*Plan généré lors du brainstorming approuvé 2026-05-14. Exécutable task-par-task ou en bloc via `superpowers:executing-plans`.*
