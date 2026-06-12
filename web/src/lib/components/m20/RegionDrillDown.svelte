<script lang="ts">
  import { Globe, Zap, Flame, Atom, X, MapPin } from '@lucide/svelte';
  import type { RegionFrAggregateDto } from '$lib/api';

  type Props = {
    region: RegionFrAggregateDto;
    onclose: () => void;
  };
  const { region, onclose }: Props = $props();

  function fmt(value: number, digits = 2): string {
    if (!Number.isFinite(value)) return '—';
    return new Intl.NumberFormat('fr-FR', { maximumFractionDigits: digits }).format(value);
  }

  // Couleur de la pastille nucléaire selon la part — lime si >50%, amber si 0-50%, coral 0%.
  const nucleartone = $derived.by<'lime' | 'amber' | 'coral'>(() => {
    if (region.nuclear_share_pct >= 50) return 'lime';
    if (region.nuclear_share_pct > 0) return 'amber';
    return 'coral';
  });
</script>

<article class="dd" aria-label="Détail région">
  <header class="dh">
    <div class="dh-l">
      <span class="ico"><Globe size={14} strokeWidth={1.8} /></span>
      <div>
        <div class="eyebrow">Région · ADMIN1</div>
        <h3>{region.region_name}</h3>
        <div class="sub mono">{region.region_iso} · INSEE {region.insee_code}</div>
      </div>
    </div>
    <button class="x-btn" type="button" onclick={onclose} aria-label="Fermer le détail">
      <X size={14} strokeWidth={1.8} />
    </button>
  </header>

  <div class="metrics">
    <div class="metric">
      <div class="m-l"><Zap size={11} strokeWidth={1.8} /> Élec annuel</div>
      <div class="m-v">
        {fmt(region.total_consumption_mwh_elec / 1000, 1)}<span class="u">GWh</span>
      </div>
    </div>
    <div class="metric">
      <div class="m-l"><Flame size={11} strokeWidth={1.8} /> Gaz annuel</div>
      <div class="m-v">
        {fmt(region.total_consumption_mwh_gas / 1000, 1)}<span class="u">GWh</span>
      </div>
    </div>
    <div class="metric total">
      <div class="m-l"><MapPin size={11} strokeWidth={1.8} /> {region.site_count} sites</div>
      <div class="m-v">{fmt(region.total_consumption_mwh / 1000, 1)}<span class="u">GWh</span></div>
    </div>
  </div>

  <div class="nuke">
    <div class="nuke-h">
      <Atom size={12} strokeWidth={1.8} /> Mix électrique régional · nucléaire
    </div>
    <div class="nuke-bar" aria-hidden="true">
      <span
        class="nuke-fill"
        data-tone={nucleartone}
        style="width: {Math.max(2, region.nuclear_share_pct).toFixed(1)}%"
      ></span>
    </div>
    <div class="nuke-val mono" data-tone={nucleartone}>
      {fmt(region.nuclear_share_pct, 0)}%
    </div>
  </div>

  {#if region.top_sites.length > 0}
    <section class="top">
      <div class="top-h">Top {region.top_sites.length} sites de la région</div>
      <ul>
        {#each region.top_sites as s, i (s.code_iris)}
          <li>
            <span class="rank mono">{i + 1}</span>
            <span class="rn">{s.commune}</span>
            <span class="rs mono">{s.code_iris}</span>
            <span class="rv mono">{fmt(s.consumption_total_mwh / 1000, 2)} GWh</span>
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
    min-width: 0;
  }
  .dh-l .ico {
    display: inline-grid;
    place-items: center;
    width: 28px;
    height: 28px;
    background: rgba(126, 182, 255, 0.12);
    border: 1px solid rgba(126, 182, 255, 0.3);
    border-radius: 8px;
    color: var(--blue);
    flex-shrink: 0;
  }
  .eyebrow {
    font: 500 12px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--ivory-3);
    margin-bottom: 4px;
  }
  h3 {
    font: 400 22px/1.1 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.01em;
    margin: 0 0 2px;
  }
  .sub {
    font: 400 12px/1 var(--font-mono);
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
    background: var(--surface-hi);
    border-color: var(--border-hi);
    color: var(--ivory);
  }

  .metrics {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: 14px;
  }
  .metric {
    padding: 10px 12px;
    background: rgba(0, 0, 0, 0.22);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }
  .metric.total {
    background: rgba(126, 182, 255, 0.04);
    border-color: rgba(126, 182, 255, 0.25);
  }
  .m-l {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font: 500 12px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--ivory-3);
    margin-bottom: 5px;
  }
  .m-v {
    font: 400 22px/1 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.01em;
  }
  .metric.total .m-v {
    color: var(--blue);
  }
  .m-v .u {
    font: 400 12px/1 var(--font-ui);
    font-style: normal;
    color: var(--ivory-3);
    margin-left: 5px;
  }

  .nuke {
    padding: 10px 12px;
    background: rgba(0, 0, 0, 0.22);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    margin-bottom: 14px;
  }
  .nuke-h {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font: 500 12px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--ivory-3);
    margin-bottom: 8px;
  }
  .nuke-bar {
    height: 6px;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 999px;
    margin-bottom: 5px;
    overflow: hidden;
  }
  .nuke-fill {
    display: block;
    height: 100%;
    border-radius: 999px;
    transition: width 350ms var(--ease);
  }
  .nuke-fill[data-tone='lime'] {
    background: linear-gradient(90deg, rgba(197, 240, 74, 0.5), var(--lime));
  }
  .nuke-fill[data-tone='amber'] {
    background: linear-gradient(90deg, rgba(245, 183, 105, 0.5), var(--amber));
  }
  .nuke-fill[data-tone='coral'] {
    background: linear-gradient(90deg, rgba(240, 108, 90, 0.5), var(--coral));
  }
  .nuke-val {
    font: 600 13px/1 var(--font-mono);
    text-align: right;
  }
  .nuke-val[data-tone='lime'] {
    color: var(--lime);
  }
  .nuke-val[data-tone='amber'] {
    color: var(--amber);
  }
  .nuke-val[data-tone='coral'] {
    color: var(--coral);
  }

  .top {
    padding-top: 10px;
    border-top: 1px solid var(--border);
  }
  .top-h {
    font: 500 12px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--ivory-3);
    margin-bottom: 8px;
  }
  .top ul {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .top li {
    display: grid;
    grid-template-columns: 22px 1fr auto;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    background: rgba(0, 0, 0, 0.18);
    border-radius: var(--radius-sm);
    font: 400 12px/1.3 var(--font-ui);
  }
  .rank {
    text-align: center;
    font: 600 12px/1 var(--font-mono);
    color: var(--lime);
  }
  .rn {
    color: var(--ivory);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .rs {
    grid-column: 2 / 3;
    grid-row: 2 / 3;
    font: 400 12px/1 var(--font-mono);
    color: var(--ivory-3);
    margin-top: 2px;
  }
  .rv {
    grid-column: 3 / 4;
    grid-row: 1 / 3;
    font: 600 12px/1 var(--font-mono);
    color: var(--ivory-2);
    align-self: center;
  }
  .mono {
    font-family: var(--font-mono);
  }
</style>
