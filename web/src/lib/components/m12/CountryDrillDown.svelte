<script lang="ts">
  import { Globe, Server, Zap, Building2, X } from '@lucide/svelte';
  import type { CountryAggregateDto, DatacenterSummaryDto } from '$lib/api';
  import { COUNTRY_FLAG } from './DatacenterDrillDown.svelte';

  type Props = {
    country: CountryAggregateDto;
    datacenters: DatacenterSummaryDto[];
    onclose: () => void;
    onSelectDc: (dc: DatacenterSummaryDto) => void;
  };
  const { country, datacenters, onclose, onSelectDc }: Props = $props();

  function fmt(value: number, digits = 2): string {
    if (!Number.isFinite(value)) return '—';
    return new Intl.NumberFormat('fr-FR', { maximumFractionDigits: digits }).format(value);
  }

  const dcsInCountry = $derived(datacenters.filter((dc) => dc.country_iso === country.country_iso));

  const ifTone = $derived.by<'lime' | 'amber' | 'coral'>(() => {
    if (country.if_electrical_g_per_kwh < 100) return 'lime';
    if (country.if_electrical_g_per_kwh < 300) return 'amber';
    return 'coral';
  });
</script>

<article class="dd" aria-label="Détail pays">
  <header class="dh">
    <div class="dh-l">
      <span class="flag">{COUNTRY_FLAG[country.country_iso] ?? '🇪🇺'}</span>
      <div>
        <div class="eyebrow">Agrégat pays</div>
        <h3>{country.country_iso}</h3>
        <div class="sub mono">{country.datacenter_count} datacenters référencés</div>
      </div>
    </div>
    <button class="x-btn" type="button" onclick={onclose} aria-label="Fermer">
      <X size={14} strokeWidth={1.8} />
    </button>
  </header>

  <div class="grid">
    <div class="cell">
      <div class="c-l"><Zap size={11} strokeWidth={1.8} /> IF électrique</div>
      <div class="c-v" data-tone={ifTone}>
        {fmt(country.if_electrical_g_per_kwh, 0)}<span class="u">g/kWh</span>
      </div>
    </div>
    <div class="cell">
      <div class="c-l"><Server size={11} strokeWidth={1.8} /> PUE moyen</div>
      <div class="c-v">{fmt(country.avg_pue, 2)}</div>
    </div>
    <div class="cell">
      <div class="c-l"><Building2 size={11} strokeWidth={1.8} /> Capacité totale</div>
      <div class="c-v">
        {country.total_capacity_mw !== undefined && country.total_capacity_mw !== null
          ? `${fmt(country.total_capacity_mw, 0)} MW`
          : 'n.c.'}
      </div>
    </div>
    <div class="cell">
      <div class="c-l"><Globe size={11} strokeWidth={1.8} /> Centroïde</div>
      <div class="c-v mono">
        {fmt(country.centroid_lat, 2)}°, {fmt(country.centroid_lon, 2)}°
      </div>
    </div>
  </div>

  {#if dcsInCountry.length > 0}
    <section class="dc-list">
      <div class="dl-h">Datacenters dans {country.country_iso}</div>
      <ul>
        {#each dcsInCountry as dc (dc.id)}
          <li>
            <button class="dc-row" type="button" onclick={() => onSelectDc(dc)}>
              <span class="dc-name">{dc.name}</span>
              <span class="dc-op mono">{dc.operator}</span>
              <span class="dc-pue mono">PUE {fmt(dc.pue, 2)}</span>
            </button>
          </li>
        {/each}
      </ul>
    </section>
  {/if}
</article>

<style>
  .dd {
    padding: 18px 20px 16px;
    background: rgba(255, 255, 255, 0.015);
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
    animation: rise 320ms var(--ease) backwards;
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
  .dh {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 12px;
    margin-bottom: 14px;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--border);
  }
  .dh-l {
    display: flex;
    gap: 10px;
    align-items: flex-start;
  }
  .flag {
    font-size: 22px;
    line-height: 1;
    padding-top: 2px;
  }
  .eyebrow {
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--ivory-3);
    margin-bottom: 4px;
  }
  h3 {
    font: 400 24px/1.1 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.01em;
    margin: 0 0 2px;
  }
  .sub {
    font: 400 11px/1 var(--font-mono);
    color: var(--ivory-3);
  }
  .x-btn {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--ivory-2);
    cursor: pointer;
  }
  .x-btn:hover {
    border-color: var(--border-hi);
    color: var(--ivory);
  }

  .grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 6px;
    margin-bottom: 14px;
  }
  .cell {
    padding: 10px 12px;
    background: rgba(0, 0, 0, 0.22);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }
  .c-l {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--ivory-3);
    margin-bottom: 6px;
  }
  .c-v {
    font: 400 20px/1 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.01em;
  }
  .c-v[data-tone='lime'] {
    color: var(--lime);
  }
  .c-v[data-tone='amber'] {
    color: var(--amber);
  }
  .c-v[data-tone='coral'] {
    color: var(--coral);
  }
  .c-v .u {
    font: 400 10px/1 var(--font-ui);
    font-style: normal;
    color: var(--ivory-3);
    margin-left: 5px;
  }

  .dc-list {
    padding-top: 10px;
    border-top: 1px solid var(--border);
  }
  .dl-h {
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--ivory-3);
    margin-bottom: 8px;
  }
  .dc-list ul {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .dc-row {
    display: grid;
    grid-template-columns: 1fr auto auto;
    gap: 8px;
    align-items: center;
    width: 100%;
    padding: 7px 10px;
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    cursor: pointer;
    text-align: left;
    transition: all var(--dur-base) var(--ease);
  }
  .dc-row:hover {
    background: rgba(197, 240, 74, 0.05);
    border-color: rgba(197, 240, 74, 0.25);
  }
  .dc-name {
    font: 500 12px/1.2 var(--font-ui);
    color: var(--ivory);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .dc-op {
    font: 500 10px/1 var(--font-mono);
    color: var(--lime);
    letter-spacing: 0.04em;
  }
  .dc-pue {
    font: 400 10px/1 var(--font-mono);
    color: var(--ivory-3);
  }
  .mono {
    font-family: var(--font-mono);
  }
</style>
