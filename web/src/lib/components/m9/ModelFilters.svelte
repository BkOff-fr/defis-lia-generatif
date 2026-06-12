<script lang="ts" module>
  import type { ModelPresetDto } from '$lib/api';

  export type SortKey = 'name' | 'params_asc' | 'params_desc' | 'co2eq_asc';

  export type FilterState = {
    enabledProviders: Set<string>;
    enabledCalibrations: Set<ModelPresetDto['calibration']>;
    /** `null` → tous · sinon une seule valeur autorisée. */
    openness: ModelPresetDto['openness'] | null;
    sort: SortKey;
  };

  export function emptyFilters(): FilterState {
    return {
      enabledProviders: new Set(),
      enabledCalibrations: new Set(),
      openness: null,
      sort: 'name'
    };
  }
</script>

<script lang="ts">
  import { Filter, RotateCcw } from '@lucide/svelte';
  import { CALIB_LABEL, OPENNESS_LABEL } from './ModelCard.svelte';

  type Props = {
    models: ModelPresetDto[];
    state: FilterState;
    onreset: () => void;
  };
  let { models, state = $bindable(), onreset }: Props = $props();

  // Univers des providers présents dans le dataset.
  const providers = $derived.by(() => {
    const set = new Set<string>();
    for (const m of models) set.add(m.provider);
    return [...set].sort((a, b) => a.localeCompare(b, 'fr'));
  });

  const calibrations: ModelPresetDto['calibration'][] = ['validated', 'indicative', 'extrapolated'];

  const SORT_LABEL: Record<SortKey, string> = {
    name: 'Nom (A→Z)',
    params_asc: 'Paramètres ↑',
    params_desc: 'Paramètres ↓',
    co2eq_asc: 'Frugalité (CO₂eq ↑)'
  };

  const OPENNESS_OPTS: { v: string; label: string }[] = [
    { v: 'all', label: 'Tous' },
    { v: 'open', label: OPENNESS_LABEL.open },
    { v: 'open_weights', label: OPENNESS_LABEL.open_weights },
    { v: 'closed', label: OPENNESS_LABEL.closed }
  ];

  function toggleProvider(p: string) {
    const next = new Set(state.enabledProviders);
    if (next.has(p)) next.delete(p);
    else next.add(p);
    state = { ...state, enabledProviders: next };
  }
  function toggleCalibration(c: ModelPresetDto['calibration']) {
    const next = new Set(state.enabledCalibrations);
    if (next.has(c)) next.delete(c);
    else next.add(c);
    state = { ...state, enabledCalibrations: next };
  }
  function setOpenness(v: string) {
    state = { ...state, openness: v === 'all' ? null : (v as ModelPresetDto['openness']) };
  }
  function setSort(v: string) {
    state = { ...state, sort: v as SortKey };
  }

  const activeCount = $derived(
    state.enabledProviders.size + state.enabledCalibrations.size + (state.openness !== null ? 1 : 0)
  );
</script>

<aside class="filters" aria-label="Filtres référentiel">
  <header class="filters-h">
    <span class="ico"><Filter size={14} strokeWidth={1.8} /></span>
    <h3>Filtres</h3>
    {#if activeCount > 0}
      <span class="badge-count mono" aria-label="{activeCount} filtres actifs">{activeCount}</span>
      <button class="reset" type="button" onclick={onreset} aria-label="Réinitialiser les filtres">
        <RotateCcw size={11} strokeWidth={2} /> Reset
      </button>
    {/if}
  </header>

  <!-- Tri -->
  <fieldset class="block">
    <legend>Trier par</legend>
    <div class="select-wrap">
      <select
        aria-label="Critère de tri"
        value={state.sort}
        onchange={(e) => setSort((e.currentTarget as HTMLSelectElement).value)}
      >
        {#each Object.entries(SORT_LABEL) as [k, label] (k)}
          <option value={k}>{label}</option>
        {/each}
      </select>
    </div>
  </fieldset>

  <!-- Providers -->
  <fieldset class="block">
    <legend>Provider ({providers.length})</legend>
    <ul class="chips">
      {#each providers as p (p)}
        {@const on = state.enabledProviders.has(p)}
        <li>
          <button
            type="button"
            class="chip"
            class:on
            aria-pressed={on}
            onclick={() => toggleProvider(p)}
          >
            {p}
          </button>
        </li>
      {/each}
    </ul>
  </fieldset>

  <!-- Calibration -->
  <fieldset class="block">
    <legend>Calibration</legend>
    <ul class="chips">
      {#each calibrations as c (c)}
        {@const on = state.enabledCalibrations.has(c)}
        <li>
          <button
            type="button"
            class="chip"
            class:on
            data-tone={c}
            aria-pressed={on}
            onclick={() => toggleCalibration(c)}
          >
            {CALIB_LABEL[c]}
          </button>
        </li>
      {/each}
    </ul>
  </fieldset>

  <!-- Openness -->
  <fieldset class="block">
    <legend>Ouverture</legend>
    <div class="radios" role="radiogroup" aria-label="Filtre ouverture">
      {#each OPENNESS_OPTS as o (o.v)}
        {@const on = o.v === 'all' ? state.openness === null : state.openness === o.v}
        <label class="radio" class:on>
          <input
            type="radio"
            name="openness"
            value={o.v}
            checked={on}
            onchange={() => setOpenness(o.v)}
          />
          {o.label}
        </label>
      {/each}
    </div>
  </fieldset>
</aside>

<style>
  .filters {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 16px 16px 18px;
    background: rgba(255, 255, 255, 0.015);
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
    position: sticky;
    top: 16px;
  }
  .filters-h {
    display: flex;
    align-items: center;
    gap: 8px;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--border);
  }
  .filters-h .ico {
    display: inline-grid;
    place-items: center;
    color: var(--ivory-3);
  }
  .filters-h h3 {
    font: 500 13px/1 var(--font-ui);
    color: var(--ivory);
    margin: 0;
    letter-spacing: 0.02em;
  }
  .badge-count {
    margin-left: auto;
    display: inline-grid;
    place-items: center;
    min-width: 20px;
    height: 20px;
    padding: 0 6px;
    border-radius: 999px;
    background: var(--lime-soft);
    border: 1px solid rgba(197, 240, 74, 0.3);
    color: var(--lime);
    font: 600 12px/1 var(--font-mono);
  }
  .reset {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    margin-left: 6px;
    padding: 5px 9px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--ivory-2);
    font: 500 12px/1 var(--font-ui);
    cursor: pointer;
  }
  .reset:hover {
    background: var(--surface-hi);
    border-color: var(--border-hi);
    color: var(--ivory);
  }

  .block {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 0;
    margin: 0;
    border: none;
  }
  legend {
    font: 500 12px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--ivory-3);
    padding: 0;
    margin-bottom: 2px;
  }

  .select-wrap {
    position: relative;
  }
  select {
    width: 100%;
    height: 32px;
    padding: 0 28px 0 10px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--ivory);
    font: 400 12px/1 var(--font-ui);
    appearance: none;
    cursor: pointer;
  }
  select:focus-visible {
    border-color: var(--lime);
    box-shadow: 0 0 0 3px rgba(197, 240, 74, 0.18);
    outline: none;
  }

  .chips {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
  }
  .chip {
    height: 26px;
    padding: 0 10px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 999px;
    color: var(--ivory-2);
    font: 500 12px/1 var(--font-ui);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
  }
  .chip:hover {
    background: var(--surface-hi);
    border-color: var(--border-hi);
    color: var(--ivory);
  }
  .chip.on {
    background: var(--lime-soft);
    border-color: rgba(197, 240, 74, 0.4);
    color: var(--lime);
  }
  .chip[data-tone='validated'].on {
    background: var(--lime-soft);
    border-color: rgba(197, 240, 74, 0.4);
    color: var(--lime);
  }
  .chip[data-tone='indicative'].on {
    background: rgba(245, 183, 105, 0.1);
    border-color: rgba(245, 183, 105, 0.4);
    color: var(--amber);
  }
  .chip[data-tone='extrapolated'].on {
    background: rgba(240, 108, 90, 0.1);
    border-color: rgba(240, 108, 90, 0.4);
    color: var(--coral);
  }

  .radios {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .radio {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--ivory-2);
    font: 400 12px/1 var(--font-ui);
    cursor: pointer;
  }
  .radio:hover {
    border-color: var(--border-hi);
    color: var(--ivory);
  }
  .radio:focus-within {
    border-color: var(--lime);
    color: var(--ivory);
  }
  .radio.on {
    border-color: rgba(197, 240, 74, 0.4);
    color: var(--lime);
    background: var(--lime-soft);
  }
  .radio input {
    accent-color: var(--lime);
  }

  .mono {
    font-family: var(--font-mono);
  }
</style>
