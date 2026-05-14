<script lang="ts">
  import {
    X,
    BookOpen,
    AlertTriangle,
    Loader2,
    ArrowUpRight,
    Activity,
    Cpu,
    Droplet,
    Zap,
    FileText,
    Unlock,
    Key,
    Lock
  } from '@lucide/svelte';
  import type { ModelDetailDto, TripletDto } from '$lib/api';
  import { CALIB_LABEL, CALIB_TONE, OPENNESS_LABEL } from './ModelCard.svelte';

  type Props = {
    detail: ModelDetailDto | null;
    loading: boolean;
    error: { code: string; message: string } | null;
    onclose: () => void;
  };
  const { detail, loading, error, onclose }: Props = $props();

  let panel = $state<HTMLElement | null>(null);
  let restoreFocus: HTMLElement | null = null;

  $effect(() => {
    if (!panel) return;
    // Sauvegarde focus pour restauration à la fermeture (accessibilité).
    restoreFocus = (document.activeElement as HTMLElement) ?? null;
    const closeBtn = panel.querySelector<HTMLButtonElement>('button.x-btn');
    closeBtn?.focus();

    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') {
        e.preventDefault();
        onclose();
        return;
      }
      if (e.key === 'Tab' && panel) {
        // Trap focus dans le drawer pendant qu'il est ouvert.
        const focusables = panel.querySelectorAll<HTMLElement>(
          'a[href], button:not([disabled]), [tabindex]:not([tabindex="-1"])'
        );
        if (focusables.length === 0) return;
        const first = focusables[0];
        const last = focusables[focusables.length - 1];
        if (!first || !last) return;
        const active = document.activeElement as HTMLElement | null;
        if (e.shiftKey && active === first) {
          e.preventDefault();
          last.focus();
        } else if (!e.shiftKey && active === last) {
          e.preventDefault();
          first.focus();
        }
      }
    }
    document.addEventListener('keydown', onKey);
    return () => {
      document.removeEventListener('keydown', onKey);
      restoreFocus?.focus();
    };
  });

  // ── Formatage numérique ────────────────────────────────────────────────
  function fmt(value: number, digits = 2): string {
    if (!Number.isFinite(value)) return '—';
    return new Intl.NumberFormat('fr-FR', { maximumFractionDigits: digits }).format(value);
  }
  function autoCo2(value: number): { v: string; u: string } {
    if (!Number.isFinite(value)) return { v: '—', u: 'g CO₂eq' };
    if (value >= 1000) return { v: fmt(value / 1000, 2), u: 'kg CO₂eq' };
    if (value >= 1) return { v: fmt(value, 2), u: 'g CO₂eq' };
    return { v: fmt(value * 1000, 1), u: 'mg CO₂eq' };
  }
  function autoEnergy(value: number): { v: string; u: string } {
    if (!Number.isFinite(value)) return { v: '—', u: 'Wh' };
    if (value >= 1000) return { v: fmt(value / 1000, 2), u: 'kWh' };
    if (value >= 1) return { v: fmt(value, 2), u: 'Wh' };
    return { v: fmt(value * 1000, 1), u: 'mWh' };
  }
  function autoWater(value: number): { v: string; u: string } {
    if (!Number.isFinite(value)) return { v: '—', u: 'L' };
    if (value >= 1) return { v: fmt(value, 2), u: 'L' };
    if (value >= 1e-3) return { v: fmt(value * 1000, 1), u: 'mL' };
    return { v: fmt(value * 1e6, 0), u: 'µL' };
  }

  // ── Sources ─────────────────────────────────────────────────────────────
  function isUrl(s: string): boolean {
    return /^https?:\/\//i.test(s);
  }
  function prettySource(s: string, max = 72): string {
    const stripped = s.replace(/^https?:\/\//i, '');
    return stripped.length > max ? `${stripped.slice(0, max - 1)}…` : stripped;
  }

  // ── 3 mini-graphes barres "candle" P5/P50/P95 ──────────────────────────
  //
  // Chaque graphe = un viewBox SVG normalisé à l'échelle (log auto pour
  // embodied qui peut être très petit). On affiche une "candle" horizontale :
  //   - segment fin P5 → P95
  //   - rectangle plus épais centré sur P50
  //   - tick vertical sur P50
  //
  // Le test fallback `<table>` (sr-only) fournit les chiffres aux lecteurs.

  type Triplet = { p5: number; p50: number; p95: number };

  function triplet(t: TripletDto): Triplet {
    return { p5: t.p5, p50: t.p50, p95: t.p95 };
  }

  // Décide si on passe en log10 : plage de >2 ordres de grandeur (P95/P5 > 100).
  function shouldLog(t: Triplet): boolean {
    if (t.p5 <= 0) return false;
    return t.p95 / t.p5 > 100;
  }

  // Convertit une valeur dans l'espace [0..1] selon l'échelle choisie.
  function scale(t: Triplet, value: number, log: boolean): number {
    if (!log) {
      const span = Math.max(t.p95 - 0, 1e-12);
      return Math.max(0, Math.min(1, value / span));
    }
    const lo = Math.log10(Math.max(t.p5 * 0.5, 1e-12));
    const hi = Math.log10(Math.max(t.p95 * 1.2, t.p5 * 2));
    const v = Math.log10(Math.max(value, 1e-12));
    return Math.max(0, Math.min(1, (v - lo) / (hi - lo || 1)));
  }

  // 3 spécifications de graphes. Calculé via $derived pour réagir à `detail`.
  type ChartSpec = {
    key: string;
    label: string;
    unit: string;
    fmt: string;
    triplet: Triplet;
    tone: string;
    log: boolean;
  };

  const charts = $derived.by<ChartSpec[]>(() => {
    if (!detail) return [];
    const ep = triplet(detail.epsilon_prefill_mj_per_token);
    const ed = triplet(detail.epsilon_decode_mj_per_token);
    const em = triplet(detail.embodied_g_per_request);
    return [
      {
        key: 'prefill',
        label: 'ε prefill',
        unit: 'mJ / token in',
        fmt: `${fmt(ep.p5, 3)} – ${fmt(ep.p50, 3)} – ${fmt(ep.p95, 3)}`,
        triplet: ep,
        tone: '#c5f04a',
        log: shouldLog(ep)
      },
      {
        key: 'decode',
        label: 'ε decode',
        unit: 'mJ / token out',
        fmt: `${fmt(ed.p5, 3)} – ${fmt(ed.p50, 3)} – ${fmt(ed.p95, 3)}`,
        triplet: ed,
        tone: '#7eb6ff',
        log: shouldLog(ed)
      },
      {
        key: 'embodied',
        label: 'Embodied',
        unit: 'g CO₂eq / requête',
        fmt: `${fmt(em.p5, 4)} – ${fmt(em.p50, 4)} – ${fmt(em.p95, 4)}`,
        triplet: em,
        tone: '#b794f4',
        log: shouldLog(em)
      }
    ];
  });

  // Pour le SVG : viewBox 200x40.
  const CHART_W = 200;
  const CHART_H = 40;
  const PAD = 8;

  function tickX(c: ChartSpec, v: number): number {
    return PAD + scale(c.triplet, v, c.log) * (CHART_W - 2 * PAD);
  }

  // ── Méthodologie : phrase selon calibration ─────────────────────────────
  const METHOD_TEXT: Record<ModelDetailDto['calibration'], string> = {
    validated: 'Reproduction usage-only ±20-25% contre EcoLogits 2026-01.',
    indicative: 'Calibré par ordre de grandeur depuis HF AI Energy Score.',
    extrapolated: 'Extrapolé depuis un modèle ouvert comparable.'
  };

  const co2Baseline = $derived(detail ? autoCo2(detail.baseline_co2eq_p50_g) : null);
  const energyBaseline = $derived(detail ? autoEnergy(detail.baseline_energy_wh_p50) : null);
  const waterBaseline = $derived(detail ? autoWater(detail.baseline_water_l_p50) : null);
  const co2P5 = $derived(detail ? autoCo2(detail.baseline_co2eq_p5_g) : null);
  const co2P95 = $derived(detail ? autoCo2(detail.baseline_co2eq_p95_g) : null);
</script>

<!-- Backdrop scrim — bouton pour respecter les règles d'a11y. -->
<button class="scrim" type="button" aria-label="Fermer la fiche" tabindex="-1" onclick={onclose}
></button>

<div
  class="drawer"
  role="dialog"
  aria-modal="true"
  aria-label="Fiche détaillée du modèle"
  bind:this={panel}
  tabindex="-1"
>
  <header class="dh">
    {#if detail}
      <div class="dh-l">
        <div class="eyebrow">{detail.provider} · {detail.family}</div>
        <h2>{detail.display_name}</h2>
        <div class="sub mono">{detail.id} · ~{fmt(detail.approx_params_billions, 1)} B</div>
        <div class="badges">
          <span class="badge calib" data-tone={CALIB_TONE[detail.calibration]}>
            {CALIB_LABEL[detail.calibration]}
          </span>
          <span class="badge openness" data-openness={detail.openness}>
            {#if detail.openness === 'open'}
              <Unlock size={11} strokeWidth={1.8} />
            {:else if detail.openness === 'open_weights'}
              <Key size={11} strokeWidth={1.8} />
            {:else}
              <Lock size={11} strokeWidth={1.8} />
            {/if}
            {OPENNESS_LABEL[detail.openness]}
          </span>
        </div>
      </div>
    {:else}
      <div class="dh-l">
        <h2>—</h2>
      </div>
    {/if}
    <button class="x-btn" type="button" onclick={onclose} aria-label="Fermer la fiche">
      <X size={14} strokeWidth={1.8} />
    </button>
  </header>

  {#if loading}
    <div class="loading">
      <Loader2 size={14} strokeWidth={1.8} class="spin" /> Chargement de la fiche…
    </div>
  {:else if error}
    <div class="err" role="alert">
      <AlertTriangle size={14} strokeWidth={1.8} />
      <div>
        <strong>{error.code}</strong>
        <span>{error.message}</span>
      </div>
    </div>
  {:else if detail}
    <!-- Plage de référence : 3 candles -->
    <section class="block">
      <div class="block-h">
        <Activity size={11} strokeWidth={1.8} /> Plage de référence
        <span class="block-counter mono">P5 · P50 · P95</span>
      </div>

      <ul class="charts">
        {#each charts as c (c.key)}
          {@const x5 = tickX(c, c.triplet.p5)}
          {@const x50 = tickX(c, c.triplet.p50)}
          {@const x95 = tickX(c, c.triplet.p95)}
          <li class="chart">
            <div class="chart-l">
              <span class="chart-label">{c.label}</span>
              <span class="chart-unit mono">
                {c.unit}{c.log ? ' · log' : ''}
              </span>
            </div>
            <svg
              viewBox="0 0 {CHART_W} {CHART_H}"
              role="img"
              aria-label="{c.label} · P5 {fmt(c.triplet.p5, 4)} · P50 {fmt(
                c.triplet.p50,
                4
              )} · P95 {fmt(c.triplet.p95, 4)} {c.unit}"
            >
              <!-- baseline track -->
              <line
                x1={PAD}
                y1={CHART_H / 2}
                x2={CHART_W - PAD}
                y2={CHART_H / 2}
                stroke="rgba(255,255,255,0.06)"
                stroke-width="1"
              />
              <!-- P5-P95 line -->
              <line
                x1={x5}
                y1={CHART_H / 2}
                x2={x95}
                y2={CHART_H / 2}
                stroke={c.tone}
                stroke-width="2"
                opacity="0.45"
                stroke-linecap="round"
              />
              <!-- candle body (P5..P95 rectangle, more visible center) -->
              <rect
                x={x5}
                y={CHART_H / 2 - 5}
                width={Math.max(2, x95 - x5)}
                height={10}
                fill={c.tone}
                opacity="0.18"
                rx="2"
              />
              <!-- P50 vertical tick -->
              <line
                x1={x50}
                y1={CHART_H / 2 - 9}
                x2={x50}
                y2={CHART_H / 2 + 9}
                stroke={c.tone}
                stroke-width="2.5"
                stroke-linecap="round"
              />
              <!-- end ticks -->
              <line
                x1={x5}
                y1={CHART_H / 2 - 5}
                x2={x5}
                y2={CHART_H / 2 + 5}
                stroke={c.tone}
                stroke-width="1.5"
              />
              <line
                x1={x95}
                y1={CHART_H / 2 - 5}
                x2={x95}
                y2={CHART_H / 2 + 5}
                stroke={c.tone}
                stroke-width="1.5"
              />
            </svg>
            <span class="chart-vals mono">{c.fmt}</span>
          </li>
        {/each}
      </ul>

      <!-- Fallback table accessible (sr-only) -->
      <table class="sr-only">
        <caption>Plage P5/P50/P95 des paramètres distributionnels.</caption>
        <thead>
          <tr>
            <th>Paramètre</th>
            <th>Unité</th>
            <th>P5</th>
            <th>P50</th>
            <th>P95</th>
          </tr>
        </thead>
        <tbody>
          {#each charts as c (c.key)}
            <tr>
              <td>{c.label}</td>
              <td>{c.unit}</td>
              <td>{fmt(c.triplet.p5, 4)}</td>
              <td>{fmt(c.triplet.p50, 4)}</td>
              <td>{fmt(c.triplet.p95, 4)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </section>

    <!-- Baseline contextuel -->
    <section class="block baseline">
      <div class="block-h">
        <Cpu size={11} strokeWidth={1.8} /> Baseline contextuel
      </div>
      <p class="baseline-ctx">
        Pour un prompt <span class="mono">100 tokens in / 500 tokens out</span> sur
        <span class="mono">PUE 1,3 · IF FR</span> :
      </p>

      <div class="baseline-grid">
        <div class="kpi kpi-co2">
          <span class="kpi-l"><Zap size={11} strokeWidth={1.8} /> CO₂eq</span>
          <span class="kpi-v mono">
            {co2Baseline?.v ?? '—'}<span class="u">{co2Baseline?.u ?? ''}</span>
          </span>
          <span class="kpi-sub mono">
            P5 {co2P5?.v}<span class="u">{co2P5?.u}</span> · P95 {co2P95?.v}<span class="u"
              >{co2P95?.u}</span
            >
          </span>
        </div>
        <div class="kpi kpi-energy">
          <span class="kpi-l"><Activity size={11} strokeWidth={1.8} /> Énergie</span>
          <span class="kpi-v mono">
            {energyBaseline?.v ?? '—'}<span class="u">{energyBaseline?.u ?? ''}</span>
          </span>
          <span class="kpi-sub">médiane</span>
        </div>
        <div class="kpi kpi-water">
          <span class="kpi-l"><Droplet size={11} strokeWidth={1.8} /> Eau</span>
          <span class="kpi-v mono">
            {waterBaseline?.v ?? '—'}<span class="u">{waterBaseline?.u ?? ''}</span>
          </span>
          <span class="kpi-sub">médiane</span>
        </div>
      </div>
    </section>

    <!-- Sources -->
    <section class="block">
      <div class="block-h">
        <FileText size={11} strokeWidth={1.8} /> Sources documentaires
        <span class="block-counter mono">{detail.sources.length}</span>
      </div>
      {#if detail.sources.length === 0}
        <p class="empty">Aucune source documentée pour ce modèle.</p>
      {:else}
        <ul class="sources">
          {#each detail.sources as s (s)}
            <li>
              {#if isUrl(s)}
                <a href={s} target="_blank" rel="noopener noreferrer" title={s}>
                  <span class="src-text">{prettySource(s)}</span>
                  <ArrowUpRight size={11} strokeWidth={2} />
                </a>
              {:else}
                <span class="src-plain">{s}</span>
              {/if}
            </li>
          {/each}
        </ul>
      {/if}
    </section>

    <!-- Méthodologie -->
    <section class="block method">
      <div class="block-h">
        <BookOpen size={11} strokeWidth={1.8} /> Méthodologie
      </div>
      <p class="method-text">{METHOD_TEXT[detail.calibration]}</p>
      <a class="method-link" href="/methodo">
        Voir la méthodologie complète <ArrowUpRight size={11} strokeWidth={2} />
      </a>
    </section>
  {/if}
</div>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    backdrop-filter: blur(2px);
    -webkit-backdrop-filter: blur(2px);
    z-index: 90;
    border: none;
    padding: 0;
    margin: 0;
    cursor: pointer;
    animation: fade 200ms var(--ease) backwards;
  }
  @keyframes fade {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }
  .drawer {
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
    width: min(520px, 100vw);
    z-index: 100;
    background: rgba(14, 18, 16, 0.96);
    backdrop-filter: blur(24px) saturate(140%);
    -webkit-backdrop-filter: blur(24px) saturate(140%);
    border-left: 1px solid var(--border-hi);
    padding: 20px 22px 24px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 18px;
    animation: slide 280ms var(--ease) backwards;
    outline: none;
  }
  @keyframes slide {
    from {
      transform: translateX(40px);
      opacity: 0;
    }
    to {
      transform: translateX(0);
      opacity: 1;
    }
  }

  .dh {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 12px;
    padding-bottom: 14px;
    border-bottom: 1px solid var(--border);
  }
  .dh-l {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .eyebrow {
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--ivory-3);
  }
  h2 {
    font: 400 30px/1.1 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.01em;
    margin: 0;
    overflow-wrap: anywhere;
  }
  .sub {
    font: 400 11px/1.2 var(--font-mono);
    color: var(--ivory-3);
    overflow-wrap: anywhere;
  }
  .badges {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
    margin-top: 6px;
  }
  .badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 22px;
    padding: 0 9px;
    border-radius: 999px;
    font: 500 10px/1 var(--font-ui);
    letter-spacing: 0.02em;
    border: 1px solid transparent;
  }
  .badge.calib[data-tone='lime'] {
    background: var(--lime-soft);
    color: var(--lime);
    border-color: rgba(197, 240, 74, 0.3);
  }
  .badge.calib[data-tone='amber'] {
    background: rgba(245, 183, 105, 0.1);
    color: var(--amber);
    border-color: rgba(245, 183, 105, 0.32);
  }
  .badge.calib[data-tone='coral'] {
    background: rgba(240, 108, 90, 0.1);
    color: var(--coral);
    border-color: rgba(240, 108, 90, 0.32);
  }
  .badge.openness {
    background: rgba(255, 255, 255, 0.04);
    color: var(--ivory-2);
    border-color: var(--border-hi);
  }

  .x-btn {
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--ivory-2);
    cursor: pointer;
    flex-shrink: 0;
  }
  .x-btn:hover {
    background: var(--surface-hi);
    border-color: var(--border-hi);
    color: var(--ivory);
  }
  .x-btn:focus-visible {
    outline: none;
    border-color: var(--lime);
    box-shadow: 0 0 0 3px rgba(197, 240, 74, 0.18);
  }

  .loading {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 20px 4px;
    font: 400 13px/1 var(--font-ui);
    color: var(--ivory-3);
  }
  .loading :global(svg.spin) {
    animation: spin 1s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .err {
    display: flex;
    gap: 8px;
    align-items: flex-start;
    padding: 10px 12px;
    background: rgba(240, 108, 90, 0.08);
    border: 1px solid rgba(240, 108, 90, 0.3);
    border-radius: var(--radius-sm);
    color: var(--ivory);
    font: 400 12px/1.4 var(--font-ui);
  }
  .err :global(svg) {
    color: var(--coral);
    flex-shrink: 0;
    margin-top: 2px;
  }
  .err strong {
    color: var(--coral);
    font-weight: 600;
    margin-right: 6px;
  }

  .block {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .block-h {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--ivory-3);
  }
  .block-counter {
    margin-left: auto;
    font: 500 10px/1 var(--font-mono);
    color: var(--ivory-4);
  }

  .charts {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .chart {
    display: grid;
    grid-template-columns: 100px 1fr;
    grid-template-rows: auto auto;
    gap: 4px 12px;
    padding: 10px 12px;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }
  .chart-l {
    grid-column: 1;
    grid-row: 1 / span 2;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .chart-label {
    font: 500 11px/1.1 var(--font-ui);
    color: var(--ivory);
  }
  .chart-unit {
    font: 400 9px/1.2 var(--font-mono);
    color: var(--ivory-4);
    letter-spacing: 0.02em;
  }
  .chart svg {
    grid-column: 2;
    grid-row: 1;
    width: 100%;
    height: 32px;
    display: block;
  }
  .chart-vals {
    grid-column: 2;
    grid-row: 2;
    font: 500 10px/1 var(--font-mono);
    color: var(--ivory-2);
    text-align: right;
  }

  .baseline-ctx {
    margin: 0;
    font: 400 12px/1.4 var(--font-ui);
    color: var(--ivory-2);
  }
  .baseline-grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: 6px;
  }
  .kpi {
    display: grid;
    grid-template-columns: auto 1fr;
    grid-template-rows: auto auto;
    gap: 2px 10px;
    align-items: baseline;
    padding: 10px 12px;
    background: rgba(0, 0, 0, 0.22);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }
  .kpi-l {
    grid-column: 1;
    grid-row: 1;
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--ivory-3);
  }
  .kpi-v {
    grid-column: 2;
    grid-row: 1;
    text-align: right;
    font: 600 16px/1 var(--font-mono);
    color: var(--ivory);
  }
  .kpi-v .u {
    color: var(--ivory-3);
    margin-left: 4px;
    font: 500 10px/1 var(--font-ui);
  }
  .kpi-sub {
    grid-column: 1 / -1;
    grid-row: 2;
    font: 400 10px/1 var(--font-ui);
    color: var(--ivory-4);
    font-style: italic;
  }
  .kpi-sub .u {
    margin-left: 2px;
  }
  .kpi-co2 .kpi-l :global(svg) {
    color: var(--lime);
  }
  .kpi-energy .kpi-l :global(svg) {
    color: #7eb6ff;
  }
  .kpi-water .kpi-l :global(svg) {
    color: #b794f4;
  }

  .empty {
    margin: 0;
    font: 400 12px/1.4 var(--font-ui);
    color: var(--ivory-3);
    font-style: italic;
  }
  .sources {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .sources li a,
  .sources li .src-plain {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 7px 10px;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    font: 400 11px/1.3 var(--font-ui);
    color: var(--ivory-2);
    text-decoration: none;
    transition: all var(--dur-base) var(--ease);
    overflow-wrap: anywhere;
  }
  .sources li a:hover {
    border-color: rgba(197, 240, 74, 0.3);
    color: var(--ivory);
  }
  .sources li a :global(svg) {
    color: var(--ivory-3);
    flex-shrink: 0;
  }
  .sources li a:hover :global(svg) {
    color: var(--lime);
  }
  .src-text {
    overflow-wrap: anywhere;
  }

  .method {
    padding: 12px 14px;
    background: rgba(126, 182, 255, 0.05);
    border: 1px solid rgba(126, 182, 255, 0.15);
    border-radius: var(--radius-md);
  }
  .method-text {
    margin: 0;
    font: 400 12px/1.5 var(--font-ui);
    color: var(--ivory-2);
  }
  .method-link {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    margin-top: 6px;
    font: 500 11px/1 var(--font-ui);
    color: #7eb6ff;
    text-decoration: none;
  }
  .method-link:hover {
    color: var(--ivory);
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }

  .mono {
    font-family: var(--font-mono);
  }
</style>
