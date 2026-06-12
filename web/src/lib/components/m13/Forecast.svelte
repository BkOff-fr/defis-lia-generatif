<script lang="ts">
  import { TrendingUp, Calendar, Activity } from '@lucide/svelte';
  import type { ForecastResultDto } from '$lib/api';

  type Props = {
    forecast: ForecastResultDto | null;
    volumePerDay: number;
    growthPct: number;
    onchange: (volumePerDay: number, growthPct: number) => void;
  };

  let { forecast, volumePerDay = $bindable(), growthPct = $bindable(), onchange }: Props = $props();

  function handleVol(e: Event) {
    const v = (e.target as HTMLInputElement).valueAsNumber;
    if (Number.isFinite(v) && v >= 0) {
      volumePerDay = v;
      onchange(v, growthPct);
    }
  }
  function handleGrowth(e: Event) {
    const v = (e.target as HTMLInputElement).valueAsNumber;
    if (Number.isFinite(v) && v >= -50 && v <= 50) {
      growthPct = v;
      onchange(volumePerDay, v);
    }
  }

  type Scale = { mult: number; unit: string };
  function pickScale(value: number): Scale {
    if (!Number.isFinite(value) || value === 0) return { mult: 1e-3, unit: 'kg CO₂eq' };
    const v = Math.abs(value);
    if (v >= 1e9) return { mult: 1e-9, unit: 't CO₂eq' };
    if (v >= 1e6) return { mult: 1e-6, unit: 't CO₂eq' };
    if (v >= 1e3) return { mult: 1e-3, unit: 'kg CO₂eq' };
    return { mult: 1, unit: 'g CO₂eq' };
  }

  function fmt(value: number, digits = 2): string {
    if (!Number.isFinite(value)) return '—';
    return new Intl.NumberFormat('fr-FR', { maximumFractionDigits: digits }).format(value);
  }

  // Échelle de la courbe : décidée sur le max mensuel pour cohérence visuelle.
  const curveScale = $derived.by<Scale>(() => {
    const max = forecast?.baseline_monthly_co2eq_g.reduce((m, v) => (v > m ? v : m), 0) ?? 0;
    return pickScale(max);
  });

  // Annual scale séparée — l'annuel est ~12× le pic mensuel donc parfois
  // change d'unité.
  const annualScale = $derived(
    forecast ? pickScale(forecast.baseline_annual_co2eq_g) : pickScale(0)
  );

  /** Points SVG normalisés pour la courbe 12 mois (viewBox 0 0 600 140). */
  const path = $derived.by<string>(() => {
    if (!forecast || forecast.baseline_monthly_co2eq_g.length === 0) {
      return 'M 0 130 L 600 130';
    }
    const series = forecast.baseline_monthly_co2eq_g;
    const max = series.reduce((m, v) => (v > m ? v : m), 0);
    if (max === 0) return 'M 0 130 L 600 130';
    const n = series.length;
    const segs: string[] = [];
    for (let i = 0; i < n; i++) {
      const x = (i / Math.max(1, n - 1)) * 600;
      const y = 130 - ((series[i] ?? 0) / max) * 110;
      segs.push(`${i === 0 ? 'M' : 'L'} ${x.toFixed(2)} ${y.toFixed(2)}`);
    }
    return segs.join(' ');
  });

  /** Path pour la zone sous la courbe. */
  const areaPath = $derived.by<string>(() => {
    if (!forecast || forecast.baseline_monthly_co2eq_g.length === 0) {
      return 'M 0 130 L 600 130 Z';
    }
    return `${path} L 600 130 L 0 130 Z`;
  });

  const MONTH_LABELS = ['M0', 'M1', 'M2', 'M3', 'M4', 'M5', 'M6', 'M7', 'M8', 'M9', 'M10', 'M11'];
</script>

<article class="fc">
  <header class="fh">
    <div class="fh-l">
      <span class="ico"><TrendingUp size={13} strokeWidth={1.8} /></span>
      <div>
        <div class="eyebrow">Projection 12 mois</div>
        <h3>Forecast géométrique</h3>
      </div>
    </div>
  </header>

  <div class="fc-inputs">
    <label class="fc-field">
      <span><Activity size={11} strokeWidth={1.8} /> Volume / jour</span>
      <input
        type="number"
        min="0"
        max="1000000"
        step="1"
        class="fc-input mono"
        value={volumePerDay}
        oninput={handleVol}
        aria-label="Nombre de prompts par jour"
      />
      <span class="suffix">prompts</span>
    </label>
    <label class="fc-field">
      <span><Calendar size={11} strokeWidth={1.8} /> Croissance mensuelle</span>
      <input
        type="number"
        min="-50"
        max="50"
        step="0.5"
        class="fc-input mono"
        value={growthPct}
        oninput={handleGrowth}
        aria-label="Croissance mensuelle en pourcent"
      />
      <span class="suffix">% / mois</span>
    </label>
  </div>

  {#if forecast}
    <div class="fc-curve" aria-label="Courbe d'émissions mensuelles projetées">
      <svg viewBox="0 0 600 140" preserveAspectRatio="none">
        <defs>
          <linearGradient id="fc-area" x1="0" x2="0" y1="0" y2="1">
            <stop offset="0%" stop-color="#c5f04a" stop-opacity="0.35" />
            <stop offset="100%" stop-color="#c5f04a" stop-opacity="0" />
          </linearGradient>
        </defs>
        <!-- Grille -->
        {#each [0, 0.25, 0.5, 0.75, 1] as g (g)}
          <line
            x1="0"
            y1={130 - g * 110}
            x2="600"
            y2={130 - g * 110}
            stroke="rgba(255,255,255,0.04)"
            stroke-width="1"
            stroke-dasharray="2 4"
          />
        {/each}
        <!-- Zone -->
        <path d={areaPath} fill="url(#fc-area)" />
        <!-- Courbe -->
        <path d={path} fill="none" stroke="#c5f04a" stroke-width="2" stroke-linejoin="round" />
        <!-- Points -->
        {#each forecast.baseline_monthly_co2eq_g as v, i (i)}
          {@const maxVal = forecast.baseline_monthly_co2eq_g.reduce((m, x) => (x > m ? x : m), 0)}
          {@const x = (i / Math.max(1, forecast.baseline_monthly_co2eq_g.length - 1)) * 600}
          {@const y = 130 - (maxVal > 0 ? (v / maxVal) * 110 : 0)}
          <circle cx={x} cy={y} r="2.5" fill="#c5f04a" />
        {/each}
      </svg>
      <div class="fc-axis">
        {#each MONTH_LABELS as m (m)}
          <span>{m}</span>
        {/each}
      </div>
    </div>

    <div class="fc-stats">
      <div class="stat">
        <div class="stat-l">Pic mensuel (M11)</div>
        <div class="stat-v">
          {fmt(
            (forecast.baseline_monthly_co2eq_g[forecast.baseline_monthly_co2eq_g.length - 1] ?? 0) *
              curveScale.mult,
            2
          )}
          <span class="u">{curveScale.unit}</span>
        </div>
      </div>
      <div class="stat">
        <div class="stat-l">Total annuel baseline</div>
        <div class="stat-v big">
          {fmt(forecast.baseline_annual_co2eq_g * annualScale.mult, 2)}
          <span class="u">{annualScale.unit}</span>
        </div>
      </div>
      {#if forecast.scenarios_annual_co2eq_g.length > 0}
        {@const scenarioAnnual = forecast.scenarios_annual_co2eq_g[0] ?? 0}
        {@const deltaG = scenarioAnnual - forecast.baseline_annual_co2eq_g}
        <div class="stat">
          <div class="stat-l">Total annuel scénario</div>
          <div class="stat-v">
            {fmt(scenarioAnnual * annualScale.mult, 2)}
            <span class="u">{annualScale.unit}</span>
          </div>
          <div class="stat-delta" class:lime={deltaG < 0} class:coral={deltaG > 0}>
            {deltaG > 0 ? '+' : ''}{fmt(deltaG * annualScale.mult, 2)}
            {annualScale.unit}
          </div>
        </div>
      {/if}
    </div>
  {:else}
    <p class="fc-empty">Renseigne un volume positif pour lancer la projection.</p>
  {/if}
</article>

<style>
  .fc {
    padding: 22px 24px 22px;
    background: rgba(255, 255, 255, 0.015);
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
  }
  .fh {
    margin-bottom: 14px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border);
  }
  .fh-l {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .fh-l .ico {
    display: inline-grid;
    place-items: center;
    width: 28px;
    height: 28px;
    background: var(--lime-soft);
    border: 1px solid rgba(197, 240, 74, 0.3);
    border-radius: 8px;
    color: var(--lime);
  }
  .fh .eyebrow {
    font: 500 12px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--ivory-3);
    margin-bottom: 4px;
  }
  .fh h3 {
    font: 400 22px/1.15 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.01em;
    margin: 0;
  }

  .fc-inputs {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
    margin-bottom: 18px;
  }
  .fc-field {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
  }
  .fc-field > span:first-child {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font: 500 12px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--ivory-3);
  }
  .fc-input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    color: var(--ivory);
    font: 600 14px/1 var(--font-mono);
    text-align: right;
    padding: 0 4px;
  }
  .fc-input:focus {
    outline: none;
  }
  .fc-field .suffix {
    font: 400 12px/1 var(--font-mono);
    color: var(--ivory-3);
  }

  .fc-curve {
    margin-bottom: 14px;
  }
  .fc-curve svg {
    width: 100%;
    height: 140px;
    display: block;
  }
  .fc-axis {
    display: flex;
    justify-content: space-between;
    margin-top: 4px;
    font: 400 12px/1 var(--font-mono);
    color: var(--ivory-4);
    letter-spacing: 0.04em;
  }

  .fc-stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(170px, 1fr));
    gap: 10px;
    padding-top: 12px;
    border-top: 1px solid var(--border);
  }
  .stat {
    padding: 10px 14px;
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }
  .stat-l {
    font: 500 12px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--ivory-3);
    margin-bottom: 6px;
  }
  .stat-v {
    font: 400 22px/1 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.01em;
  }
  .stat-v.big {
    font-size: 28px;
    color: var(--lime);
  }
  .stat-v .u {
    font: 400 12px/1 var(--font-ui);
    font-style: normal;
    color: var(--ivory-3);
    margin-left: 4px;
  }
  .stat-delta {
    margin-top: 5px;
    font: 600 12px/1 var(--font-mono);
    color: var(--ivory-3);
  }
  .stat-delta.lime {
    color: var(--lime);
  }
  .stat-delta.coral {
    color: var(--coral);
  }

  .fc-empty {
    margin: 0;
    font: 400 13px/1.5 var(--font-ui);
    color: var(--ivory-3);
    font-style: italic;
  }
  .mono {
    font-family: var(--font-mono);
  }
</style>
