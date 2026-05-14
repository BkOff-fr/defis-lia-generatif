<script lang="ts">
  import { Leaf, Droplet, Hexagon, Layers, Settings2 } from '@lucide/svelte';
  import type {
    DistributionBins,
    EmpreinteMethod,
    EstimationResultDto,
    IndicatorDto
  } from '$lib/api';

  type Props = { result: EstimationResultDto };
  const { result }: Props = $props();

  // ─── C24 — Badge méthodologie ──────────────────────────────────────────
  // Le résultat porte la méthodo utilisée (AFNOR ou EcoLogits). On affiche
  // un badge cliquable près du résultat pour que l'utilisateur sache
  // toujours quelle méthodologie a produit ce chiffre.
  const METHOD_LABELS: Record<EmpreinteMethod, string> = {
    afnor_sobria: 'AFNOR SPEC 2314 (Sobr.ia)',
    ecologits: 'EcoLogits 2026-01'
  };
  function methodLabel(m: EmpreinteMethod): string {
    return METHOD_LABELS[m] ?? m;
  }

  // Format FR : virgule décimale, **N chiffres significatifs** plutôt que
  // N décimales fixes. Indispensable parce que les indicateurs Sobr.ia
  // s'étalent sur 6 ordres de grandeur (de quelques nanogrammes de métaux
  // critiques à plusieurs centaines de grammes de CO₂eq pour des gros
  // modèles) — un `maximumFractionDigits: 2` afficherait « 0,00 » pour
  // tout ce qui passe sous 5 mg.
  //
  // Edge cases :
  //   - `value === 0`  → "0"
  //   - non fini       → "—"
  //   - sinon          → Intl avec `maximumSignificantDigits`.
  function fmt(value: number, sig = 3): string {
    if (!Number.isFinite(value)) return '—';
    if (value === 0) return '0';
    return new Intl.NumberFormat('fr-FR', {
      maximumSignificantDigits: sig,
      minimumSignificantDigits: 1
    }).format(value);
  }

  // ─── Auto-rescale des unités ────────────────────────────────────────────
  //
  // Le Rust renvoie les indicateurs dans leur unité « canonique »
  // (`gCO2eq`, `Wh`, `L`, `mg`). Pour les petits modèles, ces valeurs
  // tombent à 10⁻⁵..10⁻⁷ — illisibles. On bascule automatiquement vers la
  // sous-unité où la valeur tombe dans [1, 1000).
  //
  // Le choix de l'échelle est fait UNE FOIS sur le P50 puis appliqué à P5,
  // P95, et aux bornes de l'axe — pour que les trois percentiles d'un
  // même indicateur restent comparables visuellement.

  type Scale = { mult: number; unit: string };
  type UnitChain = readonly Scale[];

  // Chaînes de prefixes SI ordonnées de la plus grande à la plus petite.
  // La fonction `pickScale` itère et choisit la première entrée où la
  // valeur P50 multipliée par `mult` est dans [1, 1000).
  const UNIT_CHAINS: Record<string, UnitChain> = {
    gCO2eq: [
      { mult: 1e-3, unit: 'kg CO₂eq' },
      { mult: 1, unit: 'g CO₂eq' },
      { mult: 1e3, unit: 'mg CO₂eq' },
      { mult: 1e6, unit: 'µg CO₂eq' },
      { mult: 1e9, unit: 'ng CO₂eq' }
    ],
    Wh: [
      { mult: 1e-6, unit: 'MWh' },
      { mult: 1e-3, unit: 'kWh' },
      { mult: 1, unit: 'Wh' },
      { mult: 1e3, unit: 'mWh' },
      { mult: 1e6, unit: 'µWh' },
      { mult: 1e9, unit: 'nWh' }
    ],
    L: [
      { mult: 1e-3, unit: 'm³' },
      { mult: 1, unit: 'L' },
      { mult: 1e3, unit: 'mL' },
      { mult: 1e6, unit: 'µL' },
      { mult: 1e9, unit: 'nL' }
    ],
    mg: [
      { mult: 1e-3, unit: 'g' },
      { mult: 1, unit: 'mg' },
      { mult: 1e3, unit: 'µg' },
      { mult: 1e6, unit: 'ng' },
      { mult: 1e9, unit: 'pg' }
    ],
    EUR: [{ mult: 1, unit: '€' }]
  };

  function pickScale(p50: number, baseUnit: string): Scale {
    const chain = UNIT_CHAINS[baseUnit];
    if (!chain || chain.length === 0) return { mult: 1, unit: baseUnit };
    if (!Number.isFinite(p50) || p50 === 0) {
      // Pas de signal → on garde l'unité canonique du milieu de chaîne.
      return chain.find((s) => s.mult === 1) ?? chain[0] ?? { mult: 1, unit: baseUnit };
    }
    for (const s of chain) {
      const scaled = Math.abs(p50 * s.mult);
      if (scaled >= 1 && scaled < 1000) return s;
    }
    // Valeur hors plage couverte : on prend l'extrême le plus proche
    // pour ne pas afficher zéro.
    const last = chain[chain.length - 1];
    return last ?? { mult: 1, unit: baseUnit };
  }

  // Découpe l'unité affichée en (g, CO₂eq) pour la pill range P5–P95 qui
  // veut juste la sous-unité (« mg ») dans la clé, pas le suffixe (« CO₂eq »).
  function shortUnit(displayed: string): string {
    const space = displayed.indexOf(' ');
    return space === -1 ? displayed : displayed.slice(0, space);
  }

  function pick(name: IndicatorDto['indicator']): IndicatorDto | undefined {
    return result.indicators.find((i) => i.indicator === name);
  }

  const co2 = $derived(pick('co2eq'));
  const energy = $derived(pick('energy'));
  const water = $derived(pick('water'));
  const metals = $derived(pick('critical_metals'));

  // ─── Distribution Monte-Carlo ───────────────────────────────────────────
  //
  // Quand l'indicateur transporte un histogramme (`bins`, option A1 du brief
  // C09), on en restitue la forme log-normale réelle — la queue droite est
  // précisément ce qu'on veut donner à voir (worst-case CSRD).
  //
  // En l'absence (entrées d'audit antérieures à v0.2 ou N trop faible), on
  // retombe sur une approximation gaussienne depuis P5/P50/P95 — visuel
  // uniquement, méthodologie le signale via le drawer d'hypothèses.

  /** Domaine de l'axe x = [xMin, xMax], en unités de l'indicateur. */
  function axisDomain(ind: IndicatorDto): { xMin: number; xMax: number } {
    if (ind.bins) {
      return { xMin: ind.bins.min, xMax: ind.bins.max };
    }
    const span = ind.p95 - ind.p5;
    return { xMin: ind.p5 - span * 0.2, xMax: ind.p95 + span * 0.2 };
  }

  /** Chemin SVG du polygone de distribution (viewBox 0 0 600 80). */
  function distributionPath(ind: IndicatorDto): string {
    return ind.bins ? binsPath(ind.bins) : gaussianPath(ind.p5, ind.p50, ind.p95);
  }

  function binsPath(b: DistributionBins): string {
    if (b.counts.length === 0 || b.max <= b.min) return 'M 0 74 L 600 74 Z';
    const maxCount = b.counts.reduce((acc, c) => (c > acc ? c : acc), 0);
    if (maxCount === 0) return 'M 0 74 L 600 74 Z';
    const n = b.counts.length;
    const segs: string[] = ['M 0 74'];
    for (let i = 0; i < n; i++) {
      const x = ((i + 0.5) / n) * 600;
      const count = b.counts[i] ?? 0;
      const y = 74 - (count / maxCount) * 64;
      segs.push(`L ${x.toFixed(2)} ${y.toFixed(2)}`);
    }
    segs.push('L 600 74 Z');
    return segs.join(' ');
  }

  function gaussianPath(p5: number, p50: number, p95: number): string {
    if (!isFinite(p5) || !isFinite(p95) || p95 <= p5) return 'M 0 74 L 600 74 Z';
    const n = 40;
    const span = p95 - p5;
    const xMin = p5 - span * 0.2;
    const xMax = p95 + span * 0.2;
    const sigma = span / 3.3;
    const pts: Array<[number, number]> = [];
    for (let i = 0; i < n; i++) {
      const x = xMin + ((xMax - xMin) * i) / (n - 1);
      const yGauss = Math.exp(-Math.pow(x - p50, 2) / (2 * sigma * sigma));
      pts.push([(i / (n - 1)) * 600, 74 - yGauss * 64]);
    }
    const first = pts[0];
    if (!first) return 'M 0 74 L 600 74 Z';
    return `M ${first[0]} 74 ${pts.map(([x, y]) => `L ${x} ${y}`).join(' ')} L 600 74 Z`;
  }

  /** Position SVG (0..600) d'une valeur indicateur sur l'axe. */
  function xPos(v: number, dom: { xMin: number; xMax: number }): number {
    if (dom.xMax === dom.xMin) return 300;
    const t = (v - dom.xMin) / (dom.xMax - dom.xMin);
    return Math.max(0, Math.min(600, t * 600));
  }

  const co2Dom = $derived(co2 ? axisDomain(co2) : { xMin: 0, xMax: 1 });
  const distPath = $derived(co2 ? distributionPath(co2) : 'M 0 74 L 600 74 Z');
  const x5 = $derived(co2 ? xPos(co2.p5, co2Dom) : 0);
  const x50 = $derived(co2 ? xPos(co2.p50, co2Dom) : 300);
  const x95 = $derived(co2 ? xPos(co2.p95, co2Dom) : 600);
  const isMcRendered = $derived(!!co2?.bins);

  // Échelles d'affichage : décidées sur le P50 (cf. pickScale) puis
  // appliquées partout pour le même indicateur (cohérence P5/P50/P95 + axe).
  const co2Scale = $derived(co2 ? pickScale(co2.p50, co2.unit) : { mult: 1, unit: '' });
  const energyScale = $derived(energy ? pickScale(energy.p50, energy.unit) : { mult: 1, unit: '' });
  const waterScale = $derived(water ? pickScale(water.p50, water.unit) : { mult: 1, unit: '' });
  const metalsScale = $derived(metals ? pickScale(metals.p50, metals.unit) : { mult: 1, unit: '' });
</script>

<section class="result-block" aria-label="Résultat de l'estimation">
  <!-- C24 — Badge méthodologie utilisée pour produire ce résultat -->
  <a
    class="method-badge"
    href="/methodologies"
    title="Méthodologie utilisée pour ce calcul. Clique pour changer ta méthodologie par défaut."
    data-method={result.method}
  >
    <Layers size={11} strokeWidth={2} />
    <span class="method-badge-label">{methodLabel(result.method)}</span>
    <Settings2 size={10} strokeWidth={1.8} aria-hidden="true" />
  </a>

  <article class="hero-metric">
    <h2 class="hm-label">
      <Leaf size={14} strokeWidth={1.8} />
      Émission CO₂ équivalent · médiane
    </h2>

    {#if co2}
      <div class="hm-value">
        {fmt(co2.p50 * co2Scale.mult)}<span class="unit">{co2Scale.unit}</span>
      </div>
      <div class="hm-range" aria-label="Intervalle d'incertitude P5 à P95">
        <span class="key">P5–P95</span>
        <span class="lime">{fmt(co2.p5 * co2Scale.mult)}</span>
        <span class="key">→</span>
        <span class="lime">{fmt(co2.p95 * co2Scale.mult)}</span>
        <span class="key">{shortUnit(co2Scale.unit)}</span>
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
          <line x1={x50} y1="0" x2={x50} y2="78" stroke="#c5f04a" stroke-width="2" />
          <circle cx={x50} cy="8" r="3.5" fill="#c5f04a" />
          <text x={x50 + 6} y="14" font-family="JetBrains Mono" font-size="9" fill="#c5f04a"
            >MÉDIANE</text
          >
          <!-- P5 / P95 ticks -->
          <line
            x1={x5}
            y1="50"
            x2={x5}
            y2="78"
            stroke="rgba(197,240,74,0.4)"
            stroke-width="1"
            stroke-dasharray="2 3"
          />
          <line
            x1={x95}
            y1="56"
            x2={x95}
            y2="78"
            stroke="rgba(197,240,74,0.4)"
            stroke-width="1"
            stroke-dasharray="2 3"
          />
          <text
            x={x5}
            y="48"
            font-family="JetBrains Mono"
            font-size="8"
            fill="rgba(240,236,227,0.55)"
            text-anchor="middle">P5</text
          >
          <text
            x={x95}
            y="54"
            font-family="JetBrains Mono"
            font-size="8"
            fill="rgba(240,236,227,0.55)"
            text-anchor="middle">P95</text
          >
        </svg>
      </div>
      <div class="dist-axis" aria-hidden="true">
        <span>{fmt(co2Dom.xMin * co2Scale.mult)}</span>
        <span>{fmt(co2.p5 * co2Scale.mult)}</span>
        <span>{fmt(co2.p50 * co2Scale.mult)}</span>
        <span>{fmt(co2.p95 * co2Scale.mult)}</span>
        <span>{fmt(co2Dom.xMax * co2Scale.mult)}</span>
      </div>
      <div
        class="dist-meta"
        title={isMcRendered
          ? 'Distribution Monte-Carlo réelle (10⁴ tirages, 50 bins équi-width)'
          : 'Approximation gaussienne — distribution réelle absente du résultat'}
      >
        {isMcRendered
          ? 'distribution Monte-Carlo · 10⁴ tirages'
          : 'approximation gaussienne (P5/P50/P95)'}
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
            {fmt(energy.p50 * energyScale.mult)}<span class="u">{energyScale.unit}</span>
          </div>
          <div class="sm-r">
            P5–P95 · {fmt(energy.p5 * energyScale.mult)} → {fmt(energy.p95 * energyScale.mult)}
          </div>
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
            {fmt(water.p50 * waterScale.mult)}<span class="u">{waterScale.unit}</span>
          </div>
          <div class="sm-r">
            P5–P95 · {fmt(water.p5 * waterScale.mult)} → {fmt(water.p95 * waterScale.mult)}
          </div>
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
            {fmt(metals.p50 * metalsScale.mult)}<span class="u">{metalsScale.unit}</span>
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
    position: relative;
  }

  /* C24 — Badge méthodologie au-dessus du résultat */
  .method-badge {
    position: absolute;
    top: -14px;
    left: 36px;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 12px 4px 10px;
    background: var(--surface-hi);
    border: 1px solid var(--border-hi);
    border-radius: var(--radius-pill);
    font: 500 11px/1 var(--font-ui);
    color: var(--ivory-2);
    text-decoration: none;
    z-index: 2;
    transition: all var(--dur-base) var(--ease);
  }
  .method-badge:hover {
    border-color: var(--lime);
    color: var(--ivory);
    transform: translateY(-1px);
  }
  .method-badge[data-method='afnor_sobria'] {
    background: linear-gradient(135deg, rgba(197, 240, 74, 0.12), rgba(197, 240, 74, 0.04));
    border-color: rgba(197, 240, 74, 0.3);
    color: var(--lime);
  }
  .method-badge[data-method='ecologits'] {
    background: linear-gradient(135deg, rgba(96, 165, 250, 0.12), rgba(96, 165, 250, 0.04));
    border-color: rgba(96, 165, 250, 0.3);
    color: rgb(147, 197, 253);
  }
  .method-badge-label {
    font-weight: 500;
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
  .dist-meta {
    margin-top: 6px;
    font: 400 10px/1 var(--font-mono);
    color: var(--ivory-3);
    letter-spacing: 0.04em;
    text-transform: lowercase;
    cursor: help;
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
