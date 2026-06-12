<script lang="ts">
  import { Leaf, TrendingDown, TrendingUp } from '@lucide/svelte';
  import type { DistributionBins, ScenarioOutcomeDto, EstimationResultDto } from '$lib/api';

  type Props = {
    /** Outcome de la « Configuration actuelle » — null si pas d'override actif. */
    scenario: ScenarioOutcomeDto | null;
    /** Baseline (toujours présent). */
    baseline: EstimationResultDto;
  };
  const { scenario, baseline }: Props = $props();

  type Scale = { mult: number; unit: string };

  const CO2_CHAIN: readonly Scale[] = [
    { mult: 1e-3, unit: 'kg CO₂eq' },
    { mult: 1, unit: 'g CO₂eq' },
    { mult: 1e3, unit: 'mg CO₂eq' },
    { mult: 1e6, unit: 'µg CO₂eq' }
  ];

  function pickScale(p50: number): Scale {
    if (!Number.isFinite(p50) || p50 === 0) return CO2_CHAIN[1] ?? { mult: 1, unit: 'g CO₂eq' };
    for (const s of CO2_CHAIN) {
      const v = Math.abs(p50 * s.mult);
      if (v >= 1 && v < 1000) return s;
    }
    return CO2_CHAIN[CO2_CHAIN.length - 1] ?? { mult: 1, unit: 'g CO₂eq' };
  }

  function fmt(value: number, sig = 3): string {
    if (!Number.isFinite(value)) return '—';
    if (value === 0) return '0';
    return new Intl.NumberFormat('fr-FR', {
      maximumSignificantDigits: sig,
      minimumSignificantDigits: 1
    }).format(value);
  }

  const activeResult = $derived(scenario ? scenario.result : baseline);
  const co2 = $derived(activeResult.indicators.find((i) => i.indicator === 'co2eq'));
  const baselineCo2 = $derived(baseline.indicators.find((i) => i.indicator === 'co2eq'));

  const scale = $derived(co2 ? pickScale(co2.p50) : { mult: 1, unit: 'g CO₂eq' });

  // Le delta vient du backend (canonique). On gère le cas "pas de scénario" où
  // on affiche juste le baseline sans pill comparative.
  const deltaPct = $derived(scenario ? scenario.delta_pct : 0);
  const hasDelta = $derived(scenario !== null);
  const deltaImproved = $derived(deltaPct < 0);

  // Histogramme distributionnel (bins Monte-Carlo).
  function binsPath(b: DistributionBins): string {
    if (b.counts.length === 0 || b.max <= b.min) return 'M 0 64 L 600 64 Z';
    const maxCount = b.counts.reduce((acc, c) => (c > acc ? c : acc), 0);
    if (maxCount === 0) return 'M 0 64 L 600 64 Z';
    const n = b.counts.length;
    const segs: string[] = ['M 0 64'];
    for (let i = 0; i < n; i++) {
      const x = ((i + 0.5) / n) * 600;
      const y = 64 - ((b.counts[i] ?? 0) / maxCount) * 54;
      segs.push(`L ${x.toFixed(2)} ${y.toFixed(2)}`);
    }
    segs.push('L 600 64 Z');
    return segs.join(' ');
  }

  function gaussianPath(p5: number, p50: number, p95: number): string {
    if (!isFinite(p5) || !isFinite(p95) || p95 <= p5) return 'M 0 64 L 600 64 Z';
    const n = 40;
    const span = p95 - p5;
    const xMin = p5 - span * 0.2;
    const xMax = p95 + span * 0.2;
    const sigma = span / 3.3;
    const pts: Array<[number, number]> = [];
    for (let i = 0; i < n; i++) {
      const x = xMin + ((xMax - xMin) * i) / (n - 1);
      const yGauss = Math.exp(-Math.pow(x - p50, 2) / (2 * sigma * sigma));
      pts.push([(i / (n - 1)) * 600, 64 - yGauss * 54]);
    }
    const first = pts[0];
    if (!first) return 'M 0 64 L 600 64 Z';
    return `M ${first[0]} 64 ${pts.map(([x, y]) => `L ${x} ${y}`).join(' ')} L 600 64 Z`;
  }

  const distPath = $derived(
    co2
      ? co2.bins
        ? binsPath(co2.bins)
        : gaussianPath(co2.p5, co2.p50, co2.p95)
      : 'M 0 64 L 600 64 Z'
  );
  const isMc = $derived(!!co2?.bins);
</script>

<article class="verdict" aria-live="polite">
  <header class="vh">
    <div class="vh-label">
      <Leaf size={13} strokeWidth={1.8} />
      Verdict CO₂eq · médiane
    </div>
    {#if hasDelta && baselineCo2}
      <span class="pill" class:lime={deltaImproved} class:coral={!deltaImproved}>
        {#if deltaImproved}
          <TrendingDown size={12} strokeWidth={2} />
        {:else}
          <TrendingUp size={12} strokeWidth={2} />
        {/if}
        {deltaPct > 0 ? '+' : ''}{fmt(deltaPct, 3)}% vs baseline
      </span>
    {/if}
  </header>

  {#if co2}
    <div class="v-value">
      <span class="num">{fmt(co2.p50 * scale.mult)}</span>
      <span class="unit">{scale.unit}</span>
    </div>

    <div class="v-range">
      <span class="key">P5–P95</span>
      <span class="lime mono">{fmt(co2.p5 * scale.mult)}</span>
      <span class="key">→</span>
      <span class="lime mono">{fmt(co2.p95 * scale.mult)}</span>
    </div>

    <div class="dist" aria-hidden="true">
      <svg viewBox="0 0 600 70" preserveAspectRatio="none">
        <defs>
          <linearGradient id="vd-band" x1="0" x2="1">
            <stop offset="0%" stop-color="#c5f04a" stop-opacity="0.1" />
            <stop offset="50%" stop-color="#c5f04a" stop-opacity="0.5" />
            <stop offset="100%" stop-color="#c5f04a" stop-opacity="0.1" />
          </linearGradient>
        </defs>
        <line x1="0" y1="64" x2="600" y2="64" stroke="rgba(255,255,255,0.08)" stroke-width="1" />
        <path d={distPath} fill="url(#vd-band)" stroke="#c5f04a" stroke-width="1.4" />
      </svg>
    </div>
    <div class="dist-meta mono">
      {isMc ? 'distribution monte-carlo · 10⁴ tirages' : 'approximation gaussienne (p5/p50/p95)'}
    </div>

    {#if hasDelta && baselineCo2}
      <div class="v-bvar">
        Baseline P50 : <span class="mono">{fmt(baselineCo2.p50 * scale.mult)} {scale.unit}</span>
      </div>
    {:else}
      <div class="v-bvar v-bvar-hint">Actionne un levier pour comparer à ce baseline.</div>
    {/if}
  {/if}
</article>

<style>
  .verdict {
    position: relative;
    padding: 28px 28px 22px;
    background: linear-gradient(
      160deg,
      rgba(197, 240, 74, 0.05),
      rgba(197, 240, 74, 0.01) 60%,
      rgba(255, 255, 255, 0.015)
    );
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
    overflow: hidden;
  }
  .verdict::after {
    content: '';
    position: absolute;
    top: -50%;
    right: -20%;
    width: 80%;
    height: 200%;
    background: radial-gradient(ellipse at center, rgba(197, 240, 74, 0.16), transparent 60%);
    filter: blur(40px);
    pointer-events: none;
  }

  .vh {
    position: relative;
    z-index: 1;
    display: flex;
    align-items: center;
    gap: 14px;
    margin-bottom: 14px;
  }
  .vh-label {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font: 500 12px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--ivory-3);
  }
  .vh-label :global(svg) {
    color: var(--lime);
  }

  .pill {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 5px 11px;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--border);
    border-radius: 999px;
    font: 600 12px/1 var(--font-mono);
    letter-spacing: 0.04em;
    margin-left: auto;
  }
  .pill.lime {
    background: rgba(197, 240, 74, 0.1);
    border-color: rgba(197, 240, 74, 0.3);
    color: var(--lime);
  }
  .pill.coral {
    background: rgba(240, 108, 90, 0.1);
    border-color: rgba(240, 108, 90, 0.3);
    color: var(--coral);
  }

  .v-value {
    position: relative;
    z-index: 1;
    display: flex;
    align-items: baseline;
    gap: 14px;
    margin-bottom: 6px;
  }
  .v-value .num {
    font: 400 88px/0.95 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.03em;
    text-shadow: 0 0 60px rgba(197, 240, 74, 0.12);
    animation: pop 600ms var(--ease-spring) backwards;
  }
  .v-value .unit {
    font: 400 20px/1 var(--font-ui);
    font-style: normal;
    color: var(--ivory-2);
  }
  @keyframes pop {
    from {
      opacity: 0;
      transform: translateY(10px) scale(0.94);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .v-range {
    position: relative;
    z-index: 1;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 5px 11px;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--border);
    border-radius: 999px;
    font: 400 12px/1 var(--font-mono);
    color: var(--ivory-3);
    margin-bottom: 18px;
  }
  .v-range .key {
    color: var(--ivory-4);
  }
  .v-range .lime {
    color: var(--lime);
  }

  .dist {
    position: relative;
    z-index: 1;
    height: 70px;
    margin-bottom: 6px;
  }
  .dist svg {
    width: 100%;
    height: 100%;
    display: block;
  }

  .dist-meta {
    font: 400 12px/1 var(--font-mono);
    color: var(--ivory-3);
    letter-spacing: 0.04em;
    text-transform: lowercase;
    margin-bottom: 12px;
  }

  .v-bvar {
    font: 400 12px/1.5 var(--font-ui);
    color: var(--ivory-3);
  }
  .v-bvar-hint {
    font-style: italic;
  }
  .mono {
    font-family: var(--font-mono);
  }
</style>
