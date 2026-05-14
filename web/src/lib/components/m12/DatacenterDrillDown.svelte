<script lang="ts" module>
  // Drapeau ASCII pour quelques pays (alt text + tooltip). Pas de
  // dépendance externe, juste les 13 pays du dataset v1.0.

  export const COUNTRY_FLAG: Record<string, string> = {
    FR: '🇫🇷',
    DE: '🇩🇪',
    IE: '🇮🇪',
    NL: '🇳🇱',
    GB: '🇬🇧',
    SE: '🇸🇪',
    FI: '🇫🇮',
    ES: '🇪🇸',
    IT: '🇮🇹',
    PL: '🇵🇱',
    CH: '🇨🇭',
    AT: '🇦🇹',
    DK: '🇩🇰'
  };
</script>

<script lang="ts">
  import {
    Server,
    Building2,
    Zap,
    Droplet,
    Leaf,
    Clock,
    Activity,
    X,
    ArrowUpRight,
    Loader2,
    AlertTriangle
  } from '@lucide/svelte';
  import type { DatacenterDetailDto } from '$lib/api';

  type Props = {
    detail: DatacenterDetailDto | null;
    loading: boolean;
    error: { code: string; message: string } | null;
    onclose: () => void;
  };
  const { detail, loading, error, onclose }: Props = $props();

  function fmt(value: number, digits = 2): string {
    if (!Number.isFinite(value)) return '—';
    return new Intl.NumberFormat('fr-FR', { maximumFractionDigits: digits }).format(value);
  }

  function fmtAuto(value: number, baseUnit: 'g' | 'Wh' | 'L'): { v: string; u: string } {
    if (!Number.isFinite(value)) return { v: '—', u: baseUnit };
    if (baseUnit === 'g') {
      if (value >= 1000) return { v: fmt(value / 1000, 2), u: 'kg CO₂eq' };
      if (value >= 1) return { v: fmt(value, 2), u: 'g CO₂eq' };
      return { v: fmt(value * 1000, 1), u: 'mg CO₂eq' };
    }
    if (baseUnit === 'Wh') {
      if (value >= 1000) return { v: fmt(value / 1000, 2), u: 'kWh' };
      if (value >= 1) return { v: fmt(value, 2), u: 'Wh' };
      return { v: fmt(value * 1000, 1), u: 'mWh' };
    }
    if (value >= 1) return { v: fmt(value, 2), u: 'L' };
    if (value >= 1e-3) return { v: fmt(value * 1000, 1), u: 'mL' };
    return { v: fmt(value * 1e6, 0), u: 'µL' };
  }

  // ─── Intensity gauge (donut "carbon intensity") ──────────────────────────
  //
  // On n'a pas la décomposition mix par source (cf. brief C12 §1 + DoD :
  // "détail par source en v1.1"). On rend une jauge circulaire qui situe la
  // valeur IF du DC sur une échelle 0-800 gCO₂/kWh avec un dégradé couleur.

  const IF_SCALE_MAX = 800; // gCO₂eq/kWh (PL = 633, plafond confortable).

  const ifPct = $derived(detail ? Math.min(1, detail.if_electrical_g_per_kwh / IF_SCALE_MAX) : 0);
  const ifTone = $derived.by<'lime' | 'amber' | 'coral'>(() => {
    if (!detail) return 'lime';
    if (detail.if_electrical_g_per_kwh < 100) return 'lime';
    if (detail.if_electrical_g_per_kwh < 300) return 'amber';
    return 'coral';
  });
  const TONE_COLORS = { lime: '#c5f04a', amber: '#f5b769', coral: '#f06c5a' } as const;

  // Donut SVG : viewBox 100x100, cercle de rayon 42 stroke 14.
  const DONUT_R = 42;
  const DONUT_CIRC = 2 * Math.PI * DONUT_R; // ≈ 263.89
  const dashoffset = $derived(DONUT_CIRC * (1 - ifPct));

  // ─── Barres (3 indicateurs P50 baseline) ─────────────────────────────────
  //
  // Échelle relative entre les 3 métriques : log normalisée pour rester
  // lisible quand co2eq et energy ne sont pas dans le même ordre.

  type BarItem = { label: string; raw: number; pretty: string; unit: string; tone: string };
  const bars = $derived.by<BarItem[]>(() => {
    if (!detail) return [];
    const co2 = fmtAuto(detail.baseline_co2eq_p50_g, 'g');
    const en = fmtAuto(detail.baseline_energy_wh_p50, 'Wh');
    const wa = fmtAuto(detail.baseline_water_l_p50, 'L');
    return [
      {
        label: 'CO₂eq',
        raw: detail.baseline_co2eq_p50_g,
        pretty: co2.v,
        unit: co2.u,
        tone: '#c5f04a'
      },
      {
        label: 'Énergie',
        raw: detail.baseline_energy_wh_p50,
        pretty: en.v,
        unit: en.u,
        tone: '#7eb6ff'
      },
      {
        label: 'Eau',
        raw: detail.baseline_water_l_p50,
        pretty: wa.v,
        unit: wa.u,
        tone: '#b794f4'
      }
    ];
  });

  // Normalisation log10 pour la barre. Toutes les valeurs sont positives ;
  // on prend log(1 + max) pour éviter les zéros.
  const maxLog = $derived(Math.max(0.01, ...bars.map((b) => Math.log10(1 + Math.abs(b.raw)))));
  function barWidthPct(raw: number): number {
    if (raw <= 0) return 2;
    return Math.max(3, (Math.log10(1 + Math.abs(raw)) / maxLog) * 100);
  }

  // ─── Profil 24h ─────────────────────────────────────────────────────────
  //
  // viewBox 600x100 — courbe normalisée [0..1] avec area fill + points clés.
  const PROFILE_W = 600;
  const PROFILE_H = 100;
  const PROFILE_PAD = 8;

  function profilePath(samples: number[]): string {
    if (samples.length === 0) return `M 0 ${PROFILE_H - PROFILE_PAD}`;
    const n = samples.length;
    const max = samples.reduce((m, v) => (v > m ? v : m), 0);
    if (max <= 0) return `M 0 ${PROFILE_H - PROFILE_PAD}`;
    const segs: string[] = [];
    for (let i = 0; i < n; i++) {
      const x = (i / (n - 1)) * PROFILE_W;
      const y = PROFILE_H - PROFILE_PAD - ((samples[i] ?? 0) / max) * (PROFILE_H - 2 * PROFILE_PAD);
      segs.push(`${i === 0 ? 'M' : 'L'} ${x.toFixed(1)} ${y.toFixed(1)}`);
    }
    return segs.join(' ');
  }

  const profPath = $derived(detail ? profilePath(detail.hourly_profile_24h) : '');
  const profAreaPath = $derived(
    detail
      ? `${profPath} L ${PROFILE_W} ${PROFILE_H - PROFILE_PAD} L 0 ${PROFILE_H - PROFILE_PAD} Z`
      : ''
  );

  // Index de l'heure pic (max de hourly_profile_24h).
  const peakHour = $derived.by<number | null>(() => {
    if (!detail || detail.hourly_profile_24h.length === 0) return null;
    let idx = 0;
    let max = -Infinity;
    for (let i = 0; i < detail.hourly_profile_24h.length; i++) {
      const v = detail.hourly_profile_24h[i] ?? 0;
      if (v > max) {
        max = v;
        idx = i;
      }
    }
    return idx;
  });

  // ─── Helpers ─────────────────────────────────────────────────────────────
  function isUrl(s: string): boolean {
    return /^https?:\/\//i.test(s);
  }
  function prettySource(s: string, max = 60): string {
    const stripped = s.replace(/^https?:\/\//i, '');
    return stripped.length > max ? `${stripped.slice(0, max - 1)}…` : stripped;
  }
</script>

<article class="dc-dd glass" aria-label="Détail datacenter">
  <header class="dh">
    <div class="dh-l">
      {#if detail}
        <span class="flag" aria-label={detail.country_iso}>
          {COUNTRY_FLAG[detail.country_iso] ?? '🇪🇺'}
        </span>
        <div>
          <div class="eyebrow">{detail.operator} · {detail.city}</div>
          <h3>{detail.name}</h3>
          <div class="sub mono">{detail.id}</div>
        </div>
      {:else}
        <span class="ico"><Server size={14} strokeWidth={1.8} /></span>
        <div>
          <div class="eyebrow">Datacenter européen</div>
          <h3>—</h3>
        </div>
      {/if}
    </div>
    <button class="x-btn" type="button" onclick={onclose} aria-label="Fermer le détail">
      <X size={14} strokeWidth={1.8} />
    </button>
  </header>

  {#if loading}
    <div class="loading">
      <Loader2 size={14} strokeWidth={1.8} class="spin" /> Chargement du détail…
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
    <!-- Donut Intensity Gauge -->
    <section class="block donut-block">
      <div class="block-h">
        <Leaf size={11} strokeWidth={1.8} /> Intensité carbone · mix élec pays
      </div>
      <div class="donut-wrap">
        <svg viewBox="0 0 100 100" class="donut" aria-label="Jauge IF carbone">
          <circle
            cx="50"
            cy="50"
            r={DONUT_R}
            stroke="rgba(255,255,255,0.06)"
            stroke-width="10"
            fill="none"
          />
          <circle
            cx="50"
            cy="50"
            r={DONUT_R}
            stroke={TONE_COLORS[ifTone]}
            stroke-width="10"
            fill="none"
            stroke-dasharray={DONUT_CIRC}
            stroke-dashoffset={dashoffset}
            stroke-linecap="round"
            transform="rotate(-90 50 50)"
            style="transition: stroke-dashoffset 500ms var(--ease)"
          />
          <text
            x="50"
            y="48"
            text-anchor="middle"
            font-family="Instrument Serif, serif"
            font-style="italic"
            font-size="20"
            fill={TONE_COLORS[ifTone]}
          >
            {fmt(detail.if_electrical_g_per_kwh, 0)}
          </text>
          <text
            x="50"
            y="62"
            text-anchor="middle"
            font-family="JetBrains Mono, monospace"
            font-size="6"
            fill="rgba(240,236,227,0.55)"
          >
            gCO₂/kWh
          </text>
        </svg>
      </div>
      <p class="donut-hint">
        Décomposition par source (nucléaire / renouv / gaz / charbon) en v1.1 — pour l'instant on
        affiche le mix moyen annuel du pays (source Electricity Maps).
      </p>
    </section>

    <!-- Barres P50 -->
    <section class="block">
      <div class="block-h">
        <Activity size={11} strokeWidth={1.8} /> Indicateurs baseline · gpt-4o-mini 100/500 tok
      </div>
      <ul class="bars">
        {#each bars as b (b.label)}
          <li class="bar-row">
            <span class="bar-label">{b.label}</span>
            <span class="bar-track" aria-hidden="true">
              <span
                class="bar-fill"
                style="width: {barWidthPct(b.raw).toFixed(1)}%; background: {b.tone}"
              ></span>
            </span>
            <span class="bar-val mono">
              {b.pretty}<span class="u">{b.unit}</span>
            </span>
          </li>
        {/each}
      </ul>
    </section>

    <!-- Profil 24h -->
    <section class="block">
      <div class="block-h">
        <Clock size={11} strokeWidth={1.8} /> Profil de charge 24h
        {#if peakHour !== null}
          <span class="block-counter mono">pic {String(peakHour).padStart(2, '0')}h</span>
        {/if}
      </div>
      <div class="profile" aria-label="Profil horaire normalisé">
        <svg viewBox="0 0 {PROFILE_W} {PROFILE_H}" preserveAspectRatio="none">
          <defs>
            <linearGradient id="profile-area" x1="0" x2="0" y1="0" y2="1">
              <stop offset="0%" stop-color="#7eb6ff" stop-opacity="0.35" />
              <stop offset="100%" stop-color="#7eb6ff" stop-opacity="0" />
            </linearGradient>
          </defs>
          <path d={profAreaPath} fill="url(#profile-area)" />
          <path
            d={profPath}
            fill="none"
            stroke="#7eb6ff"
            stroke-width="1.6"
            stroke-linejoin="round"
          />
          {#if peakHour !== null}
            {@const px = (peakHour / 23) * PROFILE_W}
            <line
              x1={px}
              y1="0"
              x2={px}
              y2={PROFILE_H - PROFILE_PAD}
              stroke="#f5b769"
              stroke-width="1.2"
              stroke-dasharray="2 2"
              opacity="0.7"
            />
          {/if}
        </svg>
        <div class="profile-axis mono" aria-hidden="true">
          <span>00h</span>
          <span>06h</span>
          <span>12h</span>
          <span>18h</span>
          <span>24h</span>
        </div>
      </div>
    </section>

    <!-- Specs DC -->
    <section class="specs">
      <div class="spec">
        <Server size={11} strokeWidth={1.8} />
        <span class="spec-l">PUE</span>
        <span class="spec-v mono">{fmt(detail.pue, 2)}</span>
      </div>
      <div class="spec">
        <Droplet size={11} strokeWidth={1.8} />
        <span class="spec-l">WUE</span>
        <span class="spec-v mono">
          {detail.wue_l_per_kwh !== undefined && detail.wue_l_per_kwh !== null
            ? `${fmt(detail.wue_l_per_kwh, 2)} L/kWh`
            : 'n.c.'}
        </span>
      </div>
      <div class="spec">
        <Building2 size={11} strokeWidth={1.8} />
        <span class="spec-l">Capacité</span>
        <span class="spec-v mono">
          {detail.capacity_mw !== undefined && detail.capacity_mw !== null
            ? `${fmt(detail.capacity_mw, 0)} MW`
            : 'n.c.'}
        </span>
      </div>
      <div class="spec">
        <Zap size={11} strokeWidth={1.8} />
        <span class="spec-l">IF élec</span>
        <span class="spec-v mono">{fmt(detail.if_electrical_g_per_kwh, 0)} g/kWh</span>
      </div>
    </section>

    <!-- Sources -->
    {#if detail.sources.length > 0}
      <section class="block">
        <div class="block-h">Sources ({detail.sources.length})</div>
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
      </section>
    {/if}
  {/if}
</article>

<style>
  .dc-dd {
    padding: 18px 20px 16px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    animation: rise 320ms var(--ease) backwards;
  }
  /* C25 B3 — surface verre dépoli (mirroir B2) */
  .glass {
    background: color-mix(in oklab, var(--surface) 70%, transparent);
    backdrop-filter: blur(14px) saturate(1.2);
    -webkit-backdrop-filter: blur(14px) saturate(1.2);
    border: 1px solid color-mix(in oklab, var(--ink-mute) 12%, transparent);
    border-radius: 14px;
    box-shadow: 0 8px 24px color-mix(in oklab, black 12%, transparent);
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
    padding-bottom: 10px;
    border-bottom: 1px solid var(--border);
  }
  .dh-l {
    display: flex;
    gap: 10px;
    align-items: flex-start;
    min-width: 0;
  }
  .dh-l .flag {
    font-size: 22px;
    line-height: 1;
    flex-shrink: 0;
    padding-top: 2px;
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
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--ivory-3);
    margin-bottom: 4px;
  }
  h3 {
    font: 400 20px/1.15 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.01em;
    margin: 0 0 2px;
  }
  .sub {
    font: 400 10px/1 var(--font-mono);
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
    flex-shrink: 0;
  }
  .x-btn:hover {
    background: var(--surface-hi);
    border-color: var(--border-hi);
    color: var(--ivory);
  }

  .loading {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 20px 4px;
    font: 400 12px/1 var(--font-ui);
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
    gap: 8px;
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
    font: 600 10px/1 var(--font-mono);
    color: var(--amber);
  }

  /* Donut */
  .donut-block {
    align-items: stretch;
  }
  .donut-wrap {
    display: flex;
    justify-content: center;
    padding: 4px 0;
  }
  .donut {
    width: 140px;
    height: 140px;
  }
  .donut-hint {
    margin: 0;
    font: 400 10px/1.4 var(--font-ui);
    color: var(--ivory-3);
    font-style: italic;
    text-align: center;
  }

  /* Bars */
  .bars {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
  .bar-row {
    display: grid;
    grid-template-columns: 70px 1fr 90px;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    background: rgba(0, 0, 0, 0.22);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }
  .bar-label {
    font: 500 11px/1 var(--font-ui);
    color: var(--ivory-2);
  }
  .bar-track {
    position: relative;
    height: 6px;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 999px;
    overflow: hidden;
  }
  .bar-fill {
    display: block;
    height: 100%;
    border-radius: 999px;
    opacity: 0.85;
    transition: width 350ms var(--ease);
  }
  .bar-val {
    text-align: right;
    font: 600 11px/1 var(--font-mono);
    color: var(--ivory);
  }
  .bar-val .u {
    color: var(--ivory-3);
    margin-left: 3px;
    font-weight: 400;
  }

  /* Profile 24h */
  .profile {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .profile svg {
    width: 100%;
    height: 90px;
    display: block;
  }
  .profile-axis {
    display: flex;
    justify-content: space-between;
    font: 400 9px/1 var(--font-mono);
    color: var(--ivory-4);
    letter-spacing: 0.04em;
  }

  /* Specs grid */
  .specs {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 6px;
    padding-top: 10px;
    border-top: 1px solid var(--border);
  }
  .spec {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 10px;
    background: rgba(0, 0, 0, 0.18);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }
  .spec :global(svg) {
    color: var(--ivory-3);
    flex-shrink: 0;
  }
  .spec-l {
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--ivory-3);
  }
  .spec-v {
    margin-left: auto;
    font: 600 11px/1 var(--font-mono);
    color: var(--ivory);
  }

  /* Sources */
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
    padding: 6px 10px;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    font: 400 11px/1.3 var(--font-ui);
    color: var(--ivory-2);
    text-decoration: none;
    transition: all var(--dur-base) var(--ease);
  }
  .sources li a:hover {
    border-color: rgba(197, 240, 74, 0.3);
    color: var(--ivory);
  }
  .sources li a :global(svg) {
    color: var(--ivory-3);
  }
  .sources li a:hover :global(svg) {
    color: var(--lime);
  }
  .src-text {
    overflow-wrap: anywhere;
  }
  .mono {
    font-family: var(--font-mono);
  }
</style>
