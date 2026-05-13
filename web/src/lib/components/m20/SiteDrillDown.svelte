<script lang="ts">
  import { MapPin, Zap, Flame, Hash, Calendar, ArrowUpRight, X } from '@lucide/svelte';
  import type { IndustrialSiteSummaryDto } from '$lib/api';

  type Props = {
    site: IndustrialSiteSummaryDto;
    onclose: () => void;
  };
  const { site, onclose }: Props = $props();

  function fmt(value: number, digits = 2): string {
    if (!Number.isFinite(value)) return '—';
    return new Intl.NumberFormat('fr-FR', { maximumFractionDigits: digits }).format(value);
  }

  // Mapping ISO → nom court FR (cf. brief C13 §1.2).
  const REGION_NAMES: Record<string, string> = {
    'FR-IDF': 'Île-de-France',
    'FR-ARA': 'Auvergne-Rhône-Alpes',
    'FR-NAQ': 'Nouvelle-Aquitaine',
    'FR-NOR': 'Normandie',
    'FR-OCC': 'Occitanie',
    'FR-PAC': "Provence-Alpes-Côte d'Azur",
    'FR-HDF': 'Hauts-de-France',
    'FR-BFC': 'Bourgogne-Franche-Comté',
    'FR-GES': 'Grand Est',
    'FR-CVL': 'Centre-Val-de-Loire',
    'FR-PDL': 'Pays de la Loire',
    'FR-BRE': 'Bretagne',
    'FR-COR': 'Corse'
  };

  const regionLabel = $derived(REGION_NAMES[site.region_iso] ?? site.region_iso);
  const odreUrl = $derived(
    `https://odre.opendatasoft.com/explore/dataset/consommation-electrique-par-secteur-dactivite-iris/?q=${encodeURIComponent(site.code_iris)}`
  );
</script>

<article class="dd" aria-label="Détail site industriel">
  <header class="dh">
    <div class="dh-l">
      <span class="ico"><MapPin size={14} strokeWidth={1.8} /></span>
      <div>
        <div class="eyebrow">Site industriel · IRIS</div>
        <h3>{site.commune}</h3>
        <div class="sub mono">{site.code_iris}</div>
      </div>
    </div>
    <button class="x-btn" type="button" onclick={onclose} aria-label="Fermer le détail">
      <X size={14} strokeWidth={1.8} />
    </button>
  </header>

  <dl class="grid">
    <dt>Commune</dt>
    <dd>{site.commune}</dd>

    <dt>Département</dt>
    <dd class="mono">{site.department_code}</dd>

    <dt>Région</dt>
    <dd>{regionLabel} <span class="iso mono">({site.region_iso})</span></dd>

    <dt>Coordonnées</dt>
    <dd class="mono">{fmt(site.lat, 4)}°N, {fmt(site.lon, 4)}°E</dd>
  </dl>

  <div class="metrics">
    <div class="metric">
      <div class="m-l"><Zap size={11} strokeWidth={1.8} /> Élec annuel</div>
      <div class="m-v">{fmt(site.consumption_mwh_elec / 1000, 2)}<span class="u">GWh</span></div>
      <div class="m-r mono">{fmt(site.consumption_mwh_elec, 0)} MWh</div>
    </div>
    <div class="metric">
      <div class="m-l"><Flame size={11} strokeWidth={1.8} /> Gaz annuel</div>
      <div class="m-v">{fmt(site.consumption_mwh_gas / 1000, 2)}<span class="u">GWh</span></div>
      <div class="m-r mono">{fmt(site.consumption_mwh_gas, 0)} MWh</div>
    </div>
    <div class="metric total">
      <div class="m-l"><Hash size={11} strokeWidth={1.8} /> Total énergie</div>
      <div class="m-v">{fmt(site.consumption_total_mwh / 1000, 2)}<span class="u">GWh</span></div>
      <div class="m-r mono">{site.pdl_total} PDL</div>
    </div>
  </div>

  <footer class="dfoot">
    <div class="year mono">
      <Calendar size={11} strokeWidth={1.8} /> Année source : {site.year}
    </div>
    <a class="src-link" href={odreUrl} target="_blank" rel="noopener noreferrer">
      Voir sur ODRÉ <ArrowUpRight size={11} strokeWidth={2} />
    </a>
  </footer>
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
    background: var(--lime-soft);
    border: 1px solid rgba(197, 240, 74, 0.3);
    border-radius: 8px;
    color: var(--lime);
    flex-shrink: 0;
  }
  .eyebrow {
    font: 500 10px/1 var(--font-ui);
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
    transition: all var(--dur-base) var(--ease);
  }
  .x-btn:hover {
    background: var(--surface-hi);
    border-color: var(--border-hi);
    color: var(--ivory);
  }

  .grid {
    display: grid;
    grid-template-columns: 110px 1fr;
    gap: 6px 12px;
    margin: 0 0 14px;
  }
  .grid dt {
    font: 500 10px/1.4 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--ivory-3);
    padding-top: 2px;
  }
  .grid dd {
    font: 400 12px/1.4 var(--font-ui);
    color: var(--ivory);
    margin: 0;
  }
  .grid dd .iso {
    font-family: var(--font-mono);
    color: var(--ivory-3);
    font-size: 10px;
  }

  .metrics {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: 12px;
  }
  .metric {
    padding: 10px 12px;
    background: rgba(0, 0, 0, 0.22);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }
  .metric.total {
    background: rgba(197, 240, 74, 0.04);
    border-color: rgba(197, 240, 74, 0.25);
  }
  .m-l {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--ivory-3);
    margin-bottom: 5px;
  }
  .m-v {
    font: 400 24px/1 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.01em;
  }
  .metric.total .m-v {
    color: var(--lime);
  }
  .m-v .u {
    font: 400 12px/1 var(--font-ui);
    font-style: normal;
    color: var(--ivory-3);
    margin-left: 5px;
  }
  .m-r {
    font: 400 10px/1 var(--font-mono);
    color: var(--ivory-3);
    margin-top: 3px;
    letter-spacing: 0.04em;
  }

  .dfoot {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 10px;
    padding-top: 10px;
    border-top: 1px solid var(--border);
    font: 400 11px/1 var(--font-mono);
    color: var(--ivory-3);
  }
  .year {
    display: inline-flex;
    align-items: center;
    gap: 5px;
  }
  .src-link {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: var(--lime);
    text-decoration: none;
    border-bottom: 1px dashed rgba(197, 240, 74, 0.3);
    padding-bottom: 1px;
  }
  .src-link:hover {
    color: var(--ivory);
    border-bottom-color: var(--ivory);
  }
  .mono {
    font-family: var(--font-mono);
  }
</style>
