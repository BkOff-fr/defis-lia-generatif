<script lang="ts">
  import { Leaf, Droplet, Hexagon } from '@lucide/svelte';
  import type { EstimationResultDto, IndicatorDto } from '$lib/api';

  type Props = { result: EstimationResultDto };
  const { result }: Props = $props();

  // Format FR : virgule décimale + précision adaptée à la magnitude.
  function fmt(value: number, maxFrac = 2): string {
    return new Intl.NumberFormat('fr-FR', {
      maximumFractionDigits: maxFrac,
      minimumFractionDigits: value < 1 ? maxFrac : 0
    }).format(value);
  }

  function pick(name: IndicatorDto['indicator']): IndicatorDto | undefined {
    return result.indicators.find((i) => i.indicator === name);
  }

  const co2 = $derived(pick('co2eq'));
  const energy = $derived(pick('energy'));
  const water = $derived(pick('water'));
  const metals = $derived(pick('critical_metals'));

  // Synthèse de la distribution Monte-Carlo à partir des percentiles P5/P50/P95.
  // Approximation Gaussienne sur l'axe linéaire (suffisante pour la UI ; la
  // vraie distribution log-normale est dans le ledger d'audit).
  function distributionPath(p5: number, p50: number, p95: number): string {
    if (!isFinite(p5) || !isFinite(p95) || p95 <= p5) {
      return 'M 0 74 L 600 74 Z';
    }
    const n = 40;
    const span = p95 - p5;
    const xMin = p5 - span * 0.2;
    const xMax = p95 + span * 0.2;
    const sigma = span / 3.3;
    const points: Array<[number, number]> = [];
    for (let i = 0; i < n; i++) {
      const x = xMin + ((xMax - xMin) * i) / (n - 1);
      const yGauss = Math.exp(-Math.pow(x - p50, 2) / (2 * sigma * sigma));
      points.push([(i / (n - 1)) * 600, 74 - yGauss * 64]);
    }
    const first = points[0];
    if (!first) return 'M 0 74 L 600 74 Z';
    const head = `M ${first[0]} 74`;
    const body = points.map(([x, y]) => `L ${x} ${y}`).join(' ');
    const tail = `L 600 74 Z`;
    return `${head} ${body} ${tail}`;
  }

  const distPath = $derived(co2 ? distributionPath(co2.p5, co2.p50, co2.p95) : 'M 0 74 L 600 74 Z');
</script>

<section class="result-block" aria-label="Résultat de l'estimation">
  <article class="hero-metric">
    <h2 class="hm-label">
      <Leaf size={14} strokeWidth={1.8} />
      Émission CO₂ équivalent · médiane
    </h2>

    {#if co2}
      <div class="hm-value">
        {fmt(co2.p50)}<span class="unit">{co2.unit}</span>
      </div>
      <div class="hm-range" aria-label="Intervalle d'incertitude P5 à P95">
        <span class="key">P5–P95</span>
        <span class="lime">{fmt(co2.p5)}</span>
        <span class="key">→</span>
        <span class="lime">{fmt(co2.p95)}</span>
        <span class="key">{co2.unit}</span>
      </div>

      <div class="dist" aria-hidden="true">
        <svg viewBox="0 0 600 80" preserveAspectRatio="none">
          <defs>
            <linearGradient id="band" x1="0" x2="1">
              <stop offset="0%" stop-color="#c5f04a" stop-opacity="0.1" />
              <stop offset="50%" stop-color="#c5f04a" stop-opacity="0.5" />
              <stop offset="100%" stop-color="#c5f04a" stop-opacity="0.1" />
            </linearGradient>
            <filter id="glow">
              <feGaussianBlur stdDeviation="2" result="b" />
              <feMerge>
                <feMergeNode in="b" />
                <feMergeNode in="SourceGraphic" />
              </feMerge>
            </filter>
          </defs>
          <line x1="0" y1="74" x2="600" y2="74" stroke="rgba(255,255,255,0.08)" stroke-width="1" />
          <path
            d={distPath}
            fill="url(#band)"
            stroke="#c5f04a"
            stroke-width="1.4"
            filter="url(#glow)"
          />
          <!-- Médiane -->
          <line x1="300" y1="0" x2="300" y2="78" stroke="#c5f04a" stroke-width="2" />
          <circle cx="300" cy="8" r="3.5" fill="#c5f04a" />
          <text x="306" y="14" font-family="JetBrains Mono" font-size="9" fill="#c5f04a"
            >MÉDIANE</text
          >
          <!-- P5 / P95 ticks -->
          <line
            x1="170"
            y1="50"
            x2="170"
            y2="78"
            stroke="rgba(197,240,74,0.4)"
            stroke-width="1"
            stroke-dasharray="2 3"
          />
          <line
            x1="430"
            y1="56"
            x2="430"
            y2="78"
            stroke="rgba(197,240,74,0.4)"
            stroke-width="1"
            stroke-dasharray="2 3"
          />
          <text
            x="170"
            y="48"
            font-family="JetBrains Mono"
            font-size="8"
            fill="rgba(240,236,227,0.55)"
            text-anchor="middle">P5</text
          >
          <text
            x="430"
            y="54"
            font-family="JetBrains Mono"
            font-size="8"
            fill="rgba(240,236,227,0.55)"
            text-anchor="middle">P95</text
          >
        </svg>
      </div>
      <div class="dist-axis" aria-hidden="true">
        <span>{fmt(co2.p5 * 0.85)}</span>
        <span>{fmt(co2.p5)}</span>
        <span>{fmt(co2.p50)}</span>
        <span>{fmt(co2.p95)}</span>
        <span>{fmt(co2.p95 * 1.15)}</span>
      </div>
    {/if}
  </article>

  <div class="side-metrics">
    {#if energy}
      <div class="side-metric">
        <div class="sm-ico blue">
          <Droplet size={20} strokeWidth={1.6} />
        </div>
        <div class="sm-col">
          <div class="sm-l">Énergie</div>
          <div class="sm-v">
            {fmt(energy.p50)}<span class="u">{energy.unit}</span>
          </div>
          <div class="sm-r">P5–P95 · {fmt(energy.p5)} → {fmt(energy.p95)}</div>
        </div>
      </div>
    {/if}

    {#if water}
      <div class="side-metric">
        <div class="sm-ico blue">
          <Droplet size={20} strokeWidth={1.6} />
        </div>
        <div class="sm-col">
          <div class="sm-l">Eau (refroidissement)</div>
          <div class="sm-v">
            {fmt(water.p50, 3)}<span class="u">{water.unit}</span>
          </div>
          <div class="sm-r">P5–P95 · {fmt(water.p5, 3)} → {fmt(water.p95, 3)}</div>
        </div>
      </div>
    {/if}

    {#if metals}
      <div class="side-metric">
        <div class="sm-ico amber">
          <Hexagon size={20} strokeWidth={1.6} />
        </div>
        <div class="sm-col">
          <div class="sm-l">Métaux critiques</div>
          <div class="sm-v">
            {fmt(metals.p50, 3)}<span class="u">{metals.unit}</span>
          </div>
          <div class="sm-r">embodied amorti · estimé</div>
        </div>
      </div>
    {:else}
      <div class="side-metric ghost">
        <div class="sm-ico amber">
          <Hexagon size={20} strokeWidth={1.6} />
        </div>
        <div class="sm-col">
          <div class="sm-l">Métaux critiques</div>
          <div class="sm-v subtle">non&nbsp;modélisé</div>
          <div class="sm-r">amorti embodied à venir (C10)</div>
        </div>
      </div>
    {/if}
  </div>
</section>

<style>
  .result-block {
    margin-top: 40px;
    display: grid;
    grid-template-columns: 1.4fr 1fr;
    gap: 24px;
    align-items: stretch;
  }

  .hero-metric {
    position: relative;
    padding: 40px 36px 36px;
    background: linear-gradient(
      160deg,
      rgba(197, 240, 74, 0.06),
      rgba(197, 240, 74, 0.01) 60%,
      rgba(255, 255, 255, 0.015)
    );
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
    overflow: hidden;
  }
  .hero-metric::after {
    content: '';
    position: absolute;
    top: -60%;
    right: -20%;
    width: 80%;
    height: 200%;
    background: radial-gradient(ellipse at center, rgba(197, 240, 74, 0.18), transparent 60%);
    filter: blur(40px);
    pointer-events: none;
  }

  .hm-label {
    position: relative;
    z-index: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    font: 500 11px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--ivory-3);
    margin: 0 0 18px;
  }
  .hm-label :global(svg) {
    color: var(--lime);
  }

  .hm-value {
    position: relative;
    z-index: 1;
    font: 400 144px/0.9 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.04em;
    margin-bottom: 8px;
    display: flex;
    align-items: baseline;
    text-shadow: 0 0 80px rgba(197, 240, 74, 0.12);
    animation: pop 800ms var(--ease-spring) backwards;
  }
  @keyframes pop {
    from {
      opacity: 0;
      transform: translateY(20px) scale(0.92);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }
  .hm-value .unit {
    font: 400 28px/1 var(--font-ui);
    font-style: normal;
    color: var(--ivory-2);
    letter-spacing: 0.01em;
    margin-left: 14px;
  }
  .hm-range {
    position: relative;
    z-index: 1;
    font: 400 13px/1 var(--font-mono);
    color: var(--ivory-3);
    margin-bottom: 28px;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--border);
    border-radius: 999px;
  }
  .hm-range .key {
    color: var(--ivory-4);
  }
  .hm-range .lime {
    color: var(--lime);
  }

  .dist {
    position: relative;
    z-index: 1;
    height: 80px;
    margin-bottom: 12px;
  }
  .dist svg {
    width: 100%;
    height: 100%;
    display: block;
    overflow: visible;
  }
  .dist-axis {
    display: flex;
    justify-content: space-between;
    font: 400 10px/1 var(--font-mono);
    color: var(--ivory-4);
    padding: 0 2px;
  }

  .side-metrics {
    display: grid;
    grid-template-rows: 1fr 1fr 1fr;
    gap: 12px;
  }
  .side-metric {
    position: relative;
    padding: 20px 22px;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    display: flex;
    gap: 16px;
    align-items: center;
    overflow: hidden;
    transition: all var(--dur-base) var(--ease);
  }
  .side-metric:hover {
    background: var(--surface-hi);
    border-color: var(--border-hi);
    transform: translateY(-1px);
  }
  .side-metric.ghost {
    opacity: 0.65;
  }
  .side-metric .sm-ico {
    width: 40px;
    height: 40px;
    display: grid;
    place-items: center;
    background: rgba(255, 255, 255, 0.04);
    border-radius: 12px;
    color: var(--ivory-2);
    flex-shrink: 0;
    transition: transform 500ms var(--ease-spring);
  }
  .side-metric:hover .sm-ico {
    transform: rotate(-10deg) scale(1.1);
  }
  .side-metric .sm-ico.blue {
    background: rgba(126, 182, 255, 0.1);
    color: var(--blue);
  }
  .side-metric .sm-ico.amber {
    background: rgba(245, 183, 105, 0.1);
    color: var(--amber);
  }
  .side-metric .sm-col {
    flex: 1;
    min-width: 0;
  }
  .side-metric .sm-l {
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--ivory-3);
    margin-bottom: 6px;
  }
  .side-metric .sm-v {
    font: 400 32px/1 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.02em;
    transition: color var(--dur-slow) var(--ease);
  }
  .side-metric:hover .sm-v {
    color: var(--lime);
  }
  .side-metric .sm-v.subtle {
    color: var(--ivory-3);
    font-size: 18px;
  }
  .side-metric .sm-v .u {
    font: 400 13px/1 var(--font-ui);
    font-style: normal;
    color: var(--ivory-3);
    margin-left: 6px;
  }
  .side-metric .sm-r {
    font: 400 10px/1.3 var(--font-mono);
    color: var(--ivory-4);
    margin-top: 4px;
  }

  /* Stagger entrance */
  .side-metric {
    animation: rise 600ms var(--ease) backwards;
  }
  .side-metric:nth-child(1) {
    animation-delay: 100ms;
  }
  .side-metric:nth-child(2) {
    animation-delay: 220ms;
  }
  .side-metric:nth-child(3) {
    animation-delay: 340ms;
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(12px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  @media (max-width: 960px) {
    .result-block {
      grid-template-columns: 1fr;
    }
    .hm-value {
      font-size: 96px;
    }
  }
</style>
