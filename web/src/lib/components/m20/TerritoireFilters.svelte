<script lang="ts">
  import { Filter, Globe, RotateCcw } from '@lucide/svelte';
  import type { IndustrialSiteSummaryDto, RegionFrAggregateDto } from '$lib/api';

  export type TerritoireFilterState = {
    enabledRegions: Set<string>;
    minConsumptionGwh: number;
  };

  type Props = {
    regions: RegionFrAggregateDto[];
    sites: IndustrialSiteSummaryDto[];
    state: TerritoireFilterState;
    onreset: () => void;
  };

  let { regions, sites, state = $bindable(), onreset }: Props = $props();

  function toggleRegion(iso: string) {
    const next = new Set(state.enabledRegions);
    if (next.has(iso)) next.delete(iso);
    else next.add(iso);
    state.enabledRegions = next;
  }

  function fmt(value: number, digits = 0): string {
    if (!Number.isFinite(value)) return '—';
    return new Intl.NumberFormat('fr-FR', { maximumFractionDigits: digits }).format(value);
  }

  // Plage de consommation totale présente dans les sites — pour informer le slider.
  const maxConsumptionGwh = $derived.by(() => {
    let m = 0;
    for (const s of sites) {
      const g = s.consumption_total_mwh / 1000;
      if (g > m) m = g;
    }
    return m;
  });

  const anyActive = $derived(state.enabledRegions.size > 0 || state.minConsumptionGwh > 0);
</script>

<aside class="filters" aria-label="Filtres territoire FR">
  <header class="fh">
    <div>
      <div class="eyebrow">
        <Filter size={11} strokeWidth={1.8} /> Filtres
      </div>
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
      <Globe size={11} strokeWidth={1.8} /> Régions
      <span class="counter mono">
        {state.enabledRegions.size === 0 ? 'toutes' : state.enabledRegions.size}
      </span>
    </div>
    <ul class="region-list">
      {#each regions as r (r.region_iso)}
        <li>
          <label class="region-row" class:on={state.enabledRegions.has(r.region_iso)}>
            <input
              type="checkbox"
              checked={state.enabledRegions.has(r.region_iso)}
              onchange={() => toggleRegion(r.region_iso)}
            />
            <span class="iso mono">{r.region_iso.replace('FR-', '')}</span>
            <span class="rname">{r.region_name}</span>
            <span class="rcount mono">{r.site_count}</span>
          </label>
        </li>
      {/each}
    </ul>
  </section>

  <section class="block">
    <label for="min-cons" class="block-h">
      Conso. ≥
      <span class="counter mono">{fmt(state.minConsumptionGwh, 1)} GWh</span>
    </label>
    <input
      id="min-cons"
      type="range"
      min="0"
      max={Math.max(1, Math.ceil(maxConsumptionGwh))}
      step="0.5"
      bind:value={state.minConsumptionGwh}
      aria-valuemin="0"
      aria-valuemax={Math.max(1, Math.ceil(maxConsumptionGwh))}
      aria-valuenow={state.minConsumptionGwh}
      class="slider"
    />
    <div class="slider-axis mono">
      <span>0</span>
      <span>{fmt(Math.ceil(maxConsumptionGwh), 0)} GWh</span>
    </div>
  </section>
</aside>

<style>
  .filters {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 18px;
    background: linear-gradient(180deg, rgba(255, 255, 255, 0.025), rgba(255, 255, 255, 0.005));
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
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

  .region-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
    max-height: 320px;
    overflow-y: auto;
  }
  .region-row {
    display: grid;
    grid-template-columns: 14px 32px 1fr auto;
    gap: 8px;
    align-items: center;
    padding: 6px 8px;
    background: rgba(0, 0, 0, 0.18);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
  }
  .region-row:hover {
    border-color: var(--border-hi);
  }
  .region-row:focus-within {
    border-color: var(--lime);
  }
  .region-row.on {
    background: var(--lime-soft);
    border-color: rgba(197, 240, 74, 0.4);
  }
  .region-row input {
    accent-color: var(--lime);
    cursor: pointer;
  }
  .iso {
    font: 600 10px/1 var(--font-mono);
    color: var(--ivory-3);
  }
  .region-row.on .iso {
    color: var(--lime);
  }
  .rname {
    font: 400 12px/1.2 var(--font-ui);
    color: var(--ivory-2);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .region-row.on .rname {
    color: var(--ivory);
  }
  .rcount {
    font: 500 10px/1 var(--font-mono);
    color: var(--ivory-3);
  }

  .slider {
    -webkit-appearance: none;
    appearance: none;
    width: 100%;
    height: 4px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.08);
    margin: 6px 0 0;
    cursor: pointer;
  }
  .slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--lime);
    border: 2px solid var(--ink);
    cursor: pointer;
  }
  .slider::-moz-range-thumb {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--lime);
    border: 2px solid var(--ink);
    cursor: pointer;
  }
  .slider:focus {
    outline: 2px solid var(--lime);
    outline-offset: 4px;
  }
  .slider-axis {
    display: flex;
    justify-content: space-between;
    font: 400 9px/1 var(--font-mono);
    color: var(--ivory-4);
    letter-spacing: 0.04em;
    margin-top: 4px;
  }

  .mono {
    font-family: var(--font-mono);
  }
</style>
