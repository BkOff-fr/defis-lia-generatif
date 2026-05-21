<script lang="ts">
  import { Filter, RotateCcw, Building2 } from '@lucide/svelte';
  import type { DatacenterSummaryDto } from '$lib/api';
  import { COUNTRY_FLAG } from './DatacenterDrillDown.svelte';
  import { OPERATOR_COLORS } from './DatacenterMap.svelte';

  export type DatacenterFilterState = {
    enabledOperators: Set<string>;
    enabledCountries: Set<string>;
  };

  type Props = {
    datacenters: DatacenterSummaryDto[];
    state: DatacenterFilterState;
    onreset: () => void;
  };

  let { datacenters, state = $bindable(), onreset }: Props = $props();

  // Dérivés : opérateurs distincts (triés) + comptes
  const operatorCounts = $derived.by(() => {
    const map = new Map<string, number>();
    for (const dc of datacenters) {
      map.set(dc.operator, (map.get(dc.operator) ?? 0) + 1);
    }
    return [...map.entries()].sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]));
  });

  const countryCounts = $derived.by(() => {
    const map = new Map<string, number>();
    for (const dc of datacenters) {
      map.set(dc.country_iso, (map.get(dc.country_iso) ?? 0) + 1);
    }
    return [...map.entries()].sort((a, b) => a[0].localeCompare(b[0]));
  });

  function toggleOperator(op: string) {
    const next = new Set(state.enabledOperators);
    if (next.has(op)) next.delete(op);
    else next.add(op);
    state.enabledOperators = next;
  }

  function toggleCountry(iso: string) {
    const next = new Set(state.enabledCountries);
    if (next.has(iso)) next.delete(iso);
    else next.add(iso);
    state.enabledCountries = next;
  }

  const anyActive = $derived(state.enabledOperators.size > 0 || state.enabledCountries.size > 0);
</script>

<aside class="filters glass" aria-label="Filtres datacenters">
  <header class="fh">
    <div>
      <div class="eyebrow"><Filter size={11} strokeWidth={1.8} /> Filtres</div>
      <h2>Restreindre la carte</h2>
    </div>
    {#if anyActive}
      <button class="reset" type="button" onclick={onreset}>
        <RotateCcw size={12} strokeWidth={1.8} /> Reset
      </button>
    {/if}
  </header>

  <section class="block">
    <div class="block-h">
      <Building2 size={11} strokeWidth={1.8} /> Opérateurs
      <span class="counter mono">
        {state.enabledOperators.size === 0 ? 'tous' : state.enabledOperators.size}
      </span>
    </div>
    <ul class="op-list">
      {#each operatorCounts as [op, count] (op)}
        <li>
          <label class="op-row" class:on={state.enabledOperators.has(op)}>
            <input
              type="checkbox"
              checked={state.enabledOperators.has(op)}
              onchange={() => toggleOperator(op)}
            />
            <span
              class="op-swatch"
              style="background: {OPERATOR_COLORS[op] ?? '#f0ece3'}"
              aria-hidden="true"
            ></span>
            <span class="op-name">{op}</span>
            <span class="op-count mono">{count}</span>
          </label>
        </li>
      {/each}
    </ul>
  </section>

  <section class="block">
    <div class="block-h">
      Pays
      <span class="counter mono">
        {state.enabledCountries.size === 0 ? 'tous' : state.enabledCountries.size}
      </span>
    </div>
    <ul class="country-grid">
      {#each countryCounts as [iso, count] (iso)}
        <li>
          <label class="country-row" class:on={state.enabledCountries.has(iso)}>
            <input
              type="checkbox"
              checked={state.enabledCountries.has(iso)}
              onchange={() => toggleCountry(iso)}
            />
            <span class="flag" aria-hidden="true">{COUNTRY_FLAG[iso] ?? '🇪🇺'}</span>
            <span class="c-iso mono">{iso}</span>
            <span class="c-count mono">{count}</span>
          </label>
        </li>
      {/each}
    </ul>
  </section>
</aside>

<style>
  .filters {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  /* C25 B2 — surface verre dépoli pour l'overlay /datacenters */
  .glass {
    background: color-mix(in oklab, var(--surface) 70%, transparent);
    backdrop-filter: blur(14px) saturate(1.2);
    -webkit-backdrop-filter: blur(14px) saturate(1.2);
    border: 1px solid color-mix(in oklab, var(--ivory-3) 12%, transparent);
    border-radius: 14px;
    box-shadow: 0 8px 24px color-mix(in oklab, black 12%, transparent);
    padding: 14px;
  }
  .fh {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--border);
  }
  .eyebrow {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--ivory-3);
    margin-bottom: 5px;
  }
  h2 {
    font: 400 18px/1.15 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.01em;
    margin: 0;
  }
  .reset {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 5px 9px;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 999px;
    font: 500 10px/1 var(--font-ui);
    color: var(--ivory-2);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
  }
  .reset:hover {
    border-color: var(--border-hi);
    color: var(--ivory);
  }

  .block {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .block-h {
    display: flex;
    align-items: center;
    gap: 5px;
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--ivory-2);
  }
  .counter {
    margin-left: auto;
    font: 600 10px/1 var(--font-mono);
    color: var(--lime);
  }

  /* Operators list */
  .op-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .op-row {
    display: grid;
    grid-template-columns: 14px 10px 1fr auto;
    gap: 8px;
    align-items: center;
    padding: 6px 8px;
    background: rgba(0, 0, 0, 0.18);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
  }
  .op-row:hover {
    border-color: var(--border-hi);
  }
  .op-row:focus-within {
    border-color: var(--lime);
  }
  .op-row.on {
    background: var(--lime-soft);
    border-color: rgba(197, 240, 74, 0.4);
  }
  .op-row input {
    accent-color: var(--lime);
    cursor: pointer;
  }
  .op-swatch {
    width: 10px;
    height: 10px;
    border-radius: 2px;
  }
  .op-name {
    font: 500 12px/1.2 var(--font-ui);
    color: var(--ivory-2);
  }
  .op-row.on .op-name {
    color: var(--ivory);
  }
  .op-count {
    font: 500 10px/1 var(--font-mono);
    color: var(--ivory-3);
  }

  /* Country grid */
  .country-grid {
    list-style: none;
    padding: 0;
    margin: 0;
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 3px;
  }
  .country-row {
    display: grid;
    grid-template-columns: 14px 14px 1fr auto;
    gap: 6px;
    align-items: center;
    padding: 5px 7px;
    background: rgba(0, 0, 0, 0.18);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    cursor: pointer;
  }
  .country-row:hover {
    border-color: var(--border-hi);
  }
  .country-row.on {
    background: var(--lime-soft);
    border-color: rgba(197, 240, 74, 0.4);
  }
  .country-row input {
    accent-color: var(--lime);
    cursor: pointer;
  }
  .flag {
    font-size: 13px;
    line-height: 1;
  }
  .c-iso {
    font: 600 10px/1 var(--font-mono);
    color: var(--ivory-2);
  }
  .country-row.on .c-iso {
    color: var(--lime);
  }
  .c-count {
    font: 500 9px/1 var(--font-mono);
    color: var(--ivory-3);
  }
  .mono {
    font-family: var(--font-mono);
  }
</style>
