<script lang="ts">
  // Module M15 — Dashboard personnel (C19).
  // Consomme la commande IPC `get_dashboard_summary` exposée par sobria-app.
  // Contrat no-mock : hors Tauri, la coque pédagogique est rendue (header +
  // switch périodes désactivés + bannière), mais aucune métrique mockée.
  //
  // Voir :
  //   - briefs/chantiers/C19-dashboard-eco-budget.md §1 M15
  //   - briefs/chantiers/C19-PROMPTS-CLAUDE-CODE-M15-M25.md
  //   - crates/sobria-app/src/dto.rs (bloc "dashboard + eco-budget")

  import { onMount } from 'svelte';
  import {
    AlertTriangle,
    PlugZap,
    HelpCircle,
    Lock,
    LayoutDashboard,
    Activity,
    Leaf,
    Zap,
    Droplet,
    Loader2,
    TrendingUp,
    TrendingDown,
    Trophy,
    ArrowUpRight,
    BarChart3
  } from '@lucide/svelte';
  import {
    isTauriContext,
    getDashboardSummary,
    listModels,
    SobriaIpcError,
    type DashboardSummaryDto,
    type DashboardPeriod,
    type ModelPresetDto,
    type Calibration,
    type IpcErrorCode
  } from '$lib/api';
  import { preferences, type ModuleId } from '$lib/preferences';

  const MODULE_ID: ModuleId = 'm15';

  // Module gating (cf. ADR-0010)
  $effect(() => {
    if ($preferences.loaded && !$preferences.enabled_modules.includes(MODULE_ID)) {
      window.location.replace('/?disabled=' + MODULE_ID);
    }
  });

  // ─── Référentiel périodes ───────────────────────────────────────────────
  // Source de vérité côté Rust : `DashboardPeriod::parse()` dans
  // `crates/sobria-app/src/dashboard.rs`. Toute dérive ici cassera
  // l'appel IPC avec `invalid_request`.

  type PeriodOption = { id: DashboardPeriod; label: string };
  const PERIODS: ReadonlyArray<PeriodOption> = [
    { id: 'today', label: "Aujourd'hui" },
    { id: 'last_7_days', label: '7 derniers jours' },
    { id: 'this_month', label: 'Ce mois-ci' },
    { id: 'last_month', label: 'Mois précédent' },
    { id: 'this_year', label: 'Cette année' }
  ];

  // ─── State ──────────────────────────────────────────────────────────────

  let period = $state<DashboardPeriod>('last_7_days');
  let summary = $state<DashboardSummaryDto | null>(null);
  let loading = $state(true);
  let loadError = $state<{ code: IpcErrorCode; message: string } | null>(null);
  let models = $state<ModelPresetDto[]>([]);
  let hoverIdx = $state<number | null>(null);

  const tauriAvailable = $derived(isTauriContext());

  // ─── Charge initiale ────────────────────────────────────────────────────

  async function reload(p: DashboardPeriod): Promise<void> {
    if (!tauriAvailable) {
      loading = false;
      return;
    }
    loading = true;
    loadError = null;
    try {
      summary = await getDashboardSummary(p);
    } catch (err) {
      if (err instanceof SobriaIpcError) {
        loadError = { code: err.code, message: err.message };
      } else {
        loadError = { code: 'internal', message: 'Échec du chargement du dashboard' };
      }
      summary = null;
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    if (!tauriAvailable) {
      loading = false;
      return;
    }
    // Liste modèles : sert à enrichir le top 5 avec le display_name +
    // calibration. Si l'appel échoue, on retombe sur le model_id brut —
    // pas bloquant.
    void listModels()
      .then((m) => {
        models = m;
      })
      .catch(() => {
        models = [];
      });
    void reload(period);
  });

  // ─── Switch période ─────────────────────────────────────────────────────

  async function pickPeriod(next: DashboardPeriod): Promise<void> {
    if (next === period) return;
    period = next;
    hoverIdx = null;
    await reload(next);
  }

  function onTabKeydown(e: KeyboardEvent): void {
    if (e.key !== 'ArrowRight' && e.key !== 'ArrowLeft' && e.key !== 'Home' && e.key !== 'End')
      return;
    e.preventDefault();
    const idx = PERIODS.findIndex((p) => p.id === period);
    let next = idx;
    if (e.key === 'ArrowRight') next = (idx + 1) % PERIODS.length;
    else if (e.key === 'ArrowLeft') next = (idx - 1 + PERIODS.length) % PERIODS.length;
    else if (e.key === 'Home') next = 0;
    else if (e.key === 'End') next = PERIODS.length - 1;
    const target = PERIODS[next];
    if (!target) return;
    void pickPeriod(target.id);
    // Renvoie le focus sur l'onglet sélectionné (pattern aria-tabs).
    if (typeof document !== 'undefined') {
      const el = document.getElementById(`m15-tab-${target.id}`);
      el?.focus();
    }
  }

  // ─── Formatage numérique ────────────────────────────────────────────────

  function fmt(n: number, digits = 1): string {
    if (!Number.isFinite(n)) return '—';
    return new Intl.NumberFormat('fr-FR', { maximumFractionDigits: digits }).format(n);
  }

  function fmtInt(n: number): string {
    if (!Number.isFinite(n)) return '—';
    return new Intl.NumberFormat('fr-FR', { maximumFractionDigits: 0 }).format(n);
  }

  type AutoScale = { v: string; u: string };

  function fmtCo2(g: number): AutoScale {
    if (!Number.isFinite(g)) return { v: '—', u: 'gCO₂eq' };
    if (g >= 1e6) return { v: fmt(g / 1e6, 2), u: 't CO₂eq' };
    if (g >= 1e3) return { v: fmt(g / 1e3, 2), u: 'kg CO₂eq' };
    return { v: fmt(g, 2), u: 'g CO₂eq' };
  }

  function fmtEnergy(wh: number): AutoScale {
    if (!Number.isFinite(wh)) return { v: '—', u: 'Wh' };
    if (wh >= 1e6) return { v: fmt(wh / 1e6, 2), u: 'MWh' };
    if (wh >= 1e3) return { v: fmt(wh / 1e3, 2), u: 'kWh' };
    return { v: fmt(wh, 2), u: 'Wh' };
  }

  function fmtWater(l: number): AutoScale {
    if (!Number.isFinite(l)) return { v: '—', u: 'L' };
    if (l >= 1000) return { v: fmt(l / 1000, 2), u: 'm³' };
    if (l >= 1) return { v: fmt(l, 2), u: 'L' };
    return { v: fmt(l * 1000, 1), u: 'mL' };
  }

  function fmtDateShort(yyyymmdd: string): string {
    // Reçoit "YYYY-MM-DD" depuis daily_series. Renvoie "DD MMM" en FR.
    const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(yyyymmdd);
    if (!m) return yyyymmdd;
    const [, , month, day] = m;
    const MONTHS_FR = [
      'janv',
      'févr',
      'mars',
      'avr',
      'mai',
      'juin',
      'juil',
      'août',
      'sept',
      'oct',
      'nov',
      'déc'
    ];
    const mi = Number(month) - 1;
    if (mi < 0 || mi > 11) return yyyymmdd;
    return `${day} ${MONTHS_FR[mi]}`;
  }

  function fmtDateLong(yyyymmdd: string): string {
    const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(yyyymmdd);
    if (!m) return yyyymmdd;
    return `${m[3]}/${m[2]}/${m[1]}`;
  }

  function fmtRange(iso: string): string {
    // RFC 3339 → DD/MM/YYYY (ignore heure).
    const m = /^(\d{4})-(\d{2})-(\d{2})/.exec(iso);
    if (!m) return iso;
    return `${m[3]}/${m[2]}/${m[1]}`;
  }

  function fmtDeltaPct(pct: number): string {
    if (!Number.isFinite(pct)) return '—';
    const sign = pct > 0 ? '+' : '';
    return `${sign}${new Intl.NumberFormat('fr-FR', { maximumFractionDigits: 1 }).format(pct)}%`;
  }

  // ─── Dérivés graphe / top ───────────────────────────────────────────────

  const dailyMax = $derived.by<number>(() => {
    const s = summary?.daily_series;
    if (!s || s.length === 0) return 1;
    return Math.max(1, ...s.map((d) => d.request_count));
  });

  const dailyMean = $derived.by<number>(() => {
    const s = summary?.daily_series;
    if (!s || s.length === 0) return 0;
    const total = s.reduce((acc, d) => acc + d.request_count, 0);
    return total / s.length;
  });

  const topMax = $derived.by<number>(() => {
    const t = summary?.top_models;
    if (!t || t.length === 0) return 1;
    return Math.max(1, ...t.map((m) => m.total_co2eq_g_p50));
  });

  /** "lime" si < moyenne, "coral" si > 1.5× moyenne, sinon "neutral". */
  function barTone(count: number, mean: number): 'lime' | 'coral' | 'neutral' {
    if (mean <= 0) return 'neutral';
    if (count < mean) return 'lime';
    if (count > mean * 1.5) return 'coral';
    return 'neutral';
  }

  function modelDisplayName(id: string): string {
    const m = models.find((mm) => mm.id === id);
    return m?.display_name ?? id;
  }

  function modelCalibration(id: string): Calibration | null {
    const m = models.find((mm) => mm.id === id);
    return m?.calibration ?? null;
  }

  function calibrationLabel(c: Calibration): string {
    if (c === 'validated') return 'Validé';
    if (c === 'indicative') return 'Indicatif';
    return 'Extrapolé';
  }

  function calibrationTone(c: Calibration): string {
    if (c === 'validated') return 'var(--lime)';
    if (c === 'indicative') return 'var(--amber)';
    return 'var(--coral)';
  }

  // ─── Tooltip chart ──────────────────────────────────────────────────────

  function onBarEnter(i: number): void {
    hoverIdx = i;
  }
  function onBarLeave(): void {
    hoverIdx = null;
  }

  // ─── Erreurs ────────────────────────────────────────────────────────────

  const ERROR_LABELS: Record<string, string> = {
    tauri_unavailable: 'Application non lancée via Tauri',
    invalid_request: 'Période invalide',
    audit_error: "Erreur de lecture du journal d'audit",
    internal: 'Erreur interne'
  };

  function errorLabel(code: string): string {
    return ERROR_LABELS[code] ?? 'Erreur';
  }

  function errorHelp(code: string): string {
    if (code === 'invalid_request') {
      return "Période inconnue : valeurs acceptées 'today', 'last_7_days', 'this_month', 'last_month', 'this_year'.";
    }
    if (code === 'tauri_unavailable') {
      return "L'écran s'ouvre uniquement via `cargo run -p sobria-app`. En navigateur seul, l'IPC est indisponible.";
    }
    return '';
  }

  // ─── Layout du graphe ───────────────────────────────────────────────────
  // viewBox basé sur le nombre de points : largeur totale = N * (BAR_W + GAP).
  // Le SVG occupe 100 % de la largeur du conteneur en `preserveAspectRatio='none'`.

  const CHART_H = 180;
  const BAR_TOP = 10;
  const BAR_AREA_H = CHART_H - BAR_TOP - 26; // 26 px réservés pour l'axe X
  const BAR_W = 18;
  const BAR_GAP = 6;

  const chartWidth = $derived.by<number>(() => {
    const n = summary?.daily_series.length ?? 0;
    return Math.max(1, n * (BAR_W + BAR_GAP) - BAR_GAP);
  });

  function barY(count: number): number {
    if (dailyMax <= 0) return BAR_TOP + BAR_AREA_H;
    const h = (count / dailyMax) * BAR_AREA_H;
    return BAR_TOP + BAR_AREA_H - h;
  }

  function barH(count: number): number {
    if (dailyMax <= 0) return 0;
    return Math.max(1.5, (count / dailyMax) * BAR_AREA_H);
  }

  /** Étiquettes X éparses : on n'affiche qu'un sous-ensemble pour éviter le chevauchement. */
  function shouldShowXLabel(i: number, n: number): boolean {
    if (n <= 10) return true;
    if (n <= 31) return i % 3 === 0 || i === n - 1;
    return i % 7 === 0 || i === n - 1;
  }
</script>

<svelte:head>
  <title>Sobr.ia · Tableau de bord personnel</title>
</svelte:head>

<div class="canvas-inner">
  <!-- TopBar -->
  <div class="topbar">
    <nav class="breadcrumb" aria-label="Fil d'Ariane">
      Atelier <span class="sep">/</span>
      <span class="current">Tableau de bord personnel</span>
    </nav>
    <div class="spacer"></div>
    <span class="local-pill">
      <Lock size={12} strokeWidth={1.8} />
      Lecture 100 % locale
    </span>
    <a class="icon-btn" href="/methodo" aria-label="Méthodologie">
      <HelpCircle size={16} strokeWidth={1.6} />
    </a>
  </div>

  <!-- Hero -->
  <section class="hero">
    <div class="hero-eyebrow">
      <span class="pulse" aria-hidden="true"></span>
      Module M15 · Dashboard personnel
    </div>
    <h1 class="hero-h1">
      Ton <em>empreinte IA</em>, période après période.
    </h1>
    <p class="hero-sub">
      Visualise tes requêtes, CO₂eq, énergie et eau cumulés sur 5 fenêtres temporelles — et compare
      vs la période précédente. Les chiffres sont calculés en P50 depuis le ledger d'audit local.
    </p>
  </section>

  <!-- Bannière hors-Tauri -->
  {#if !tauriAvailable}
    <div class="banner" data-tone="warn" role="alert">
      <span class="banner-ico" aria-hidden="true">
        <AlertTriangle size={18} strokeWidth={1.8} />
      </span>
      <div class="banner-body">
        <strong>Application non lancée via Tauri</strong>
        <span>
          L'application doit être lancée via <span class="mono">cargo run -p sobria-app</span> (ou
          <span class="mono">cargo tauri dev</span>). Le ledger d'audit reste 100 % local — aucun
          envoi externe.
        </span>
      </div>
    </div>
  {/if}

  <!-- Switch périodes (tablist) -->
  <div class="periods-wrap">
    <div
      role="tablist"
      aria-label="Sélection de la période"
      class="periods"
      tabindex="-1"
      onkeydown={onTabKeydown}
    >
      {#each PERIODS as p (p.id)}
        {@const selected = p.id === period}
        <button
          type="button"
          role="tab"
          id="m15-tab-{p.id}"
          aria-selected={selected}
          aria-controls="m15-panel"
          tabindex={selected ? 0 : -1}
          class="period-btn"
          class:active={selected}
          disabled={!tauriAvailable || loading}
          onclick={() => void pickPeriod(p.id)}
        >
          {p.label}
        </button>
      {/each}
    </div>
    {#if loading && tauriAvailable}
      <span class="loading-pill" aria-live="polite">
        <Loader2 size={12} strokeWidth={2} class="spin" /> Chargement…
      </span>
    {/if}
  </div>

  <!-- Panel principal -->
  <div id="m15-panel" role="tabpanel" aria-labelledby="m15-tab-{period}" tabindex="0" class="panel">
    {#if loadError}
      <div class="form-err" role="alert">
        <span class="err-ico"><PlugZap size={14} strokeWidth={1.8} /></span>
        <div>
          <strong>{errorLabel(loadError.code)}</strong>
          <span>{loadError.message}</span>
          {#if errorHelp(loadError.code)}
            <span class="help">{errorHelp(loadError.code)}</span>
          {/if}
        </div>
      </div>
    {/if}

    <!-- ── Synthèse : 4 cards ─────────────────────────────────────────── -->
    <section class="stats" aria-live="polite" aria-label="Synthèse de la période">
      <article class="stat-card">
        <div class="stat-l">
          <Activity size={11} strokeWidth={1.8} /> Total requêtes
        </div>
        {#if summary}
          <div class="stat-v">{fmtInt(summary.total_requests)}</div>
          {#if summary.vs_previous}
            {@const d = summary.vs_previous.delta_requests_pct}
            <div class="stat-delta" class:lime={d < 0} class:coral={d > 0} class:neutral={d === 0}>
              {#if d < 0}
                <TrendingDown size={11} strokeWidth={2} />
              {:else if d > 0}
                <TrendingUp size={11} strokeWidth={2} />
              {/if}
              {fmtDeltaPct(d)}
              <span class="d-vs">vs précédente</span>
            </div>
          {:else}
            <div class="stat-delta neutral">— pas de période précédente</div>
          {/if}
        {:else}
          <div class="stat-v skel">—</div>
        {/if}
      </article>

      <article class="stat-card">
        <div class="stat-l">
          <Leaf size={11} strokeWidth={1.8} /> CO₂eq cumulé (P50)
        </div>
        {#if summary}
          {@const co2 = fmtCo2(summary.total_co2eq_g_p50)}
          <div class="stat-v">{co2.v}<span class="u">{co2.u}</span></div>
          {#if summary.vs_previous}
            {@const d = summary.vs_previous.delta_co2eq_pct}
            <div class="stat-delta" class:lime={d < 0} class:coral={d > 0} class:neutral={d === 0}>
              {#if d < 0}
                <TrendingDown size={11} strokeWidth={2} />
              {:else if d > 0}
                <TrendingUp size={11} strokeWidth={2} />
              {/if}
              {fmtDeltaPct(d)}
              <span class="d-vs">vs précédente</span>
            </div>
          {:else}
            <div class="stat-delta neutral">— pas de période précédente</div>
          {/if}
        {:else}
          <div class="stat-v skel">—</div>
        {/if}
      </article>

      <article class="stat-card">
        <div class="stat-l">
          <Zap size={11} strokeWidth={1.8} /> Énergie cumulée (P50)
        </div>
        {#if summary}
          {@const en = fmtEnergy(summary.total_energy_wh_p50)}
          <div class="stat-v">{en.v}<span class="u">{en.u}</span></div>
        {:else}
          <div class="stat-v skel">—</div>
        {/if}
      </article>

      <article class="stat-card">
        <div class="stat-l">
          <Droplet size={11} strokeWidth={1.8} /> Eau cumulée (P50)
        </div>
        {#if summary}
          {@const w = fmtWater(summary.total_water_l_p50)}
          <div class="stat-v">{w.v}<span class="u">{w.u}</span></div>
        {:else}
          <div class="stat-v skel">—</div>
        {/if}
      </article>
    </section>

    <!-- ── Évolution journalière ──────────────────────────────────────── -->
    <section class="card">
      <header class="ch">
        <div class="ch-l">
          <span class="ch-ico"><BarChart3 size={13} strokeWidth={1.8} /></span>
          <div>
            <div class="eyebrow">Évolution journalière</div>
            <h3>Requêtes par jour</h3>
          </div>
        </div>
        {#if summary && summary.daily_series.length > 0}
          <span class="ch-mean mono">
            ⌀ {fmt(dailyMean, 1)} req/j · max {fmtInt(dailyMax)}
          </span>
        {/if}
      </header>

      {#if !summary}
        <p class="empty">
          {tauriAvailable
            ? 'Aucune donnée encore — sélectionne une période.'
            : "Lance l'app via cargo run -p sobria-app pour charger ton ledger d'audit."}
        </p>
      {:else if summary.daily_series.length === 0}
        <p class="empty">Aucune requête enregistrée sur cette période.</p>
      {:else}
        <div class="chart-host">
          <div
            class="chart"
            role="img"
            aria-label={`Histogramme des requêtes journalières sur ${PERIODS.find((p) => p.id === period)?.label}. Voir le tableau ci-dessous pour les valeurs détaillées.`}
          >
            <svg
              viewBox={`0 0 ${chartWidth} ${CHART_H}`}
              preserveAspectRatio="none"
              role="presentation"
              aria-hidden="true"
            >
              <!-- Grille horizontale -->
              {#each [0, 0.25, 0.5, 0.75, 1] as g (g)}
                <line
                  x1="0"
                  y1={BAR_TOP + BAR_AREA_H - g * BAR_AREA_H}
                  x2={chartWidth}
                  y2={BAR_TOP + BAR_AREA_H - g * BAR_AREA_H}
                  stroke="var(--plot-grid)"
                  stroke-width="1"
                  stroke-dasharray="2 4"
                />
              {/each}

              <!-- Barres -->
              {#each summary.daily_series as d, i (d.date)}
                {@const x = i * (BAR_W + BAR_GAP)}
                {@const tone = barTone(d.request_count, dailyMean)}
                <rect
                  {x}
                  y={barY(d.request_count)}
                  width={BAR_W}
                  height={barH(d.request_count)}
                  rx="2"
                  class="bar"
                  data-tone={tone}
                  data-hover={hoverIdx === i}
                />
                <!-- zone d'interaction élargie -->
                <rect
                  x={x - BAR_GAP / 2}
                  y={BAR_TOP}
                  width={BAR_W + BAR_GAP}
                  height={BAR_AREA_H}
                  fill="transparent"
                  role="presentation"
                  onmouseenter={() => onBarEnter(i)}
                  onmouseleave={onBarLeave}
                  onfocus={() => onBarEnter(i)}
                  onblur={onBarLeave}
                >
                  <title>
                    {fmtDateLong(d.date)} — {fmtInt(d.request_count)} req · {fmtCo2(d.co2eq_g_p50)
                      .v}
                    {fmtCo2(d.co2eq_g_p50).u}
                  </title>
                </rect>
              {/each}

              <!-- Axe X (étiquettes éparses) -->
              {#each summary.daily_series as d, i (d.date)}
                {#if shouldShowXLabel(i, summary.daily_series.length)}
                  <text
                    x={i * (BAR_W + BAR_GAP) + BAR_W / 2}
                    y={CHART_H - 6}
                    text-anchor="middle"
                    class="x-axis"
                  >
                    {fmtDateShort(d.date)}
                  </text>
                {/if}
              {/each}
            </svg>

            {#if hoverIdx !== null}
              {@const d = summary.daily_series[hoverIdx]}
              {#if d}
                {@const co2 = fmtCo2(d.co2eq_g_p50)}
                {@const en = fmtEnergy(d.energy_wh_p50)}
                {@const wa = fmtWater(d.water_l_p50)}
                {@const xPct = (((hoverIdx + 0.5) * (BAR_W + BAR_GAP)) / chartWidth) * 100}
                <div class="tooltip" style="left: {xPct}%" aria-hidden="true">
                  <div class="tt-date mono">{fmtDateLong(d.date)}</div>
                  <div class="tt-row">
                    <Activity size={10} strokeWidth={2} />
                    {fmtInt(d.request_count)} req
                  </div>
                  <div class="tt-row">
                    <Leaf size={10} strokeWidth={2} />
                    {co2.v}
                    {co2.u}
                  </div>
                  <div class="tt-row">
                    <Zap size={10} strokeWidth={2} />
                    {en.v}
                    {en.u}
                  </div>
                  <div class="tt-row">
                    <Droplet size={10} strokeWidth={2} />
                    {wa.v}
                    {wa.u}
                  </div>
                </div>
              {/if}
            {/if}
          </div>
          <!-- Fallback table accessible -->
          <table class="sr-table">
            <caption>Détail des requêtes journalières</caption>
            <thead>
              <tr>
                <th scope="col">Date</th>
                <th scope="col">Requêtes</th>
                <th scope="col">CO₂eq (g)</th>
                <th scope="col">Énergie (Wh)</th>
                <th scope="col">Eau (L)</th>
              </tr>
            </thead>
            <tbody>
              {#each summary.daily_series as d (d.date)}
                <tr>
                  <td>{fmtDateLong(d.date)}</td>
                  <td>{fmtInt(d.request_count)}</td>
                  <td>{fmt(d.co2eq_g_p50, 2)}</td>
                  <td>{fmt(d.energy_wh_p50, 2)}</td>
                  <td>{fmt(d.water_l_p50, 3)}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>

        <div class="legend">
          <span class="leg-item"
            ><span class="dot" style="background: var(--lime)"></span> Sous la moyenne</span
          >
          <span class="leg-item"
            ><span class="dot" style="background: var(--ivory-3)"></span> Moyenne</span
          >
          <span class="leg-item"
            ><span class="dot" style="background: var(--coral)"></span> Pic (&gt; 1.5× moyenne)</span
          >
        </div>
      {/if}
    </section>

    <!-- ── Top 5 modèles ──────────────────────────────────────────────── -->
    <section class="card">
      <header class="ch">
        <div class="ch-l">
          <span class="ch-ico"><Trophy size={13} strokeWidth={1.8} /></span>
          <div>
            <div class="eyebrow">Top modèles</div>
            <h3>Les 5 modèles les plus émetteurs (CO₂eq P50)</h3>
          </div>
        </div>
      </header>

      {#if !summary}
        <p class="empty">—</p>
      {:else if summary.top_models.length === 0}
        <p class="empty">Aucun modèle utilisé sur cette période.</p>
      {:else}
        <ol class="top-list">
          {#each summary.top_models as m, i (m.model_id)}
            {@const co2 = fmtCo2(m.total_co2eq_g_p50)}
            {@const pct = topMax > 0 ? (m.total_co2eq_g_p50 / topMax) * 100 : 0}
            {@const cal = modelCalibration(m.model_id)}
            <li class="top-row">
              <span class="rank mono">#{i + 1}</span>
              <a
                class="model-link"
                href="/m9?id={encodeURIComponent(m.model_id)}"
                aria-label={`Ouvrir la fiche du modèle ${modelDisplayName(m.model_id)}`}
              >
                <span class="model-name">{modelDisplayName(m.model_id)}</span>
                {#if cal}
                  <span
                    class="cal-badge mono"
                    style="color: {calibrationTone(cal)}; border-color: {calibrationTone(cal)}"
                    title="Statut de calibration"
                  >
                    {calibrationLabel(cal)}
                  </span>
                {/if}
                <ArrowUpRight size={11} strokeWidth={2} />
              </a>
              <span class="model-count mono">{fmtInt(m.request_count)} req</span>
              <div
                class="bar-row"
                role="img"
                aria-label={`${co2.v} ${co2.u} cumulés (${fmt(pct, 0)} % du leader)`}
              >
                <div class="bar-bg">
                  <div class="bar-fg" style="width: {pct}%"></div>
                </div>
                <span class="bar-val">{co2.v}<span class="u">{co2.u}</span></span>
              </div>
            </li>
          {/each}
        </ol>
      {/if}
    </section>

    <!-- ── Pied de page : période affichée ────────────────────────────── -->
    <footer class="pfoot">
      {#if summary}
        <span class="pf-label">Période affichée :</span>
        <span class="pf-range mono">
          {fmtRange(summary.period_start)} → {fmtRange(summary.period_end)}
        </span>
        <span class="pf-sep">·</span>
        <a class="pf-link" href="/journal">
          Voir le journal d'audit <ArrowUpRight size={11} strokeWidth={2} />
        </a>
      {:else}
        <span class="pf-label">
          <LayoutDashboard size={11} strokeWidth={1.8} />
          Les chiffres se chargeront depuis le ledger d'audit dès que l'app sera lancée via Tauri.
        </span>
      {/if}
    </footer>
  </div>
</div>

<style>
  .canvas-inner {
    max-width: 1080px;
    margin: 0 auto;
    padding: 40px 56px 80px;
    display: flex;
    flex-direction: column;
    gap: 22px;
  }

  /* ── TopBar ──────────────────────────────────────────────────────────── */
  .topbar {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 4px;
  }
  .breadcrumb {
    font: 400 13px/1 var(--font-ui);
    color: var(--ivory-3);
  }
  .breadcrumb .sep {
    color: var(--ivory-4);
    margin: 0 8px;
  }
  .breadcrumb .current {
    color: var(--ivory-2);
  }
  .spacer {
    flex: 1;
  }
  .local-pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 28px;
    padding: 0 12px;
    background: var(--lime-soft);
    border: 1px solid rgba(197, 240, 74, 0.25);
    border-radius: 999px;
    font: 500 11px/1 var(--font-ui);
    color: var(--lime);
  }
  .icon-btn {
    width: 32px;
    height: 32px;
    display: grid;
    place-items: center;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--ivory-2);
    cursor: pointer;
    text-decoration: none;
    transition: all var(--dur-base) var(--ease);
  }
  .icon-btn:hover {
    background: var(--surface-hi);
    border-color: var(--border-hi);
    color: var(--ivory);
  }

  /* ── Hero ────────────────────────────────────────────────────────────── */
  .hero {
    padding-bottom: 14px;
    border-bottom: 1px solid var(--border);
  }
  .hero-eyebrow {
    font: 500 11px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.16em;
    color: var(--ivory-3);
    margin-bottom: 14px;
    display: inline-flex;
    align-items: center;
    gap: 8px;
  }
  .hero-eyebrow .pulse {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--lime);
    box-shadow: 0 0 10px var(--lime);
  }
  .hero-h1 {
    font: 400 42px/1.1 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.02em;
    margin: 0 0 8px;
  }
  .hero-h1 em {
    font-style: normal;
    color: var(--lime);
  }
  .hero-sub {
    font: 400 15px/1.55 var(--font-ui);
    color: var(--ivory-2);
    max-width: 720px;
    margin: 0;
  }

  /* ── Bannière warn ───────────────────────────────────────────────────── */
  .banner {
    display: flex;
    gap: 14px;
    align-items: flex-start;
    padding: 14px 18px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-hi);
  }
  .banner[data-tone='warn'] {
    background: rgba(245, 183, 105, 0.08);
    border-color: rgba(245, 183, 105, 0.25);
  }
  .banner-ico {
    display: inline-flex;
    flex-shrink: 0;
    padding-top: 2px;
  }
  .banner-body {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font: 400 13px/1.4 var(--font-ui);
    color: var(--ivory);
  }
  .banner-body strong {
    font-weight: 600;
  }
  .banner-body span {
    color: var(--ivory-2);
  }

  /* ── Switch périodes ─────────────────────────────────────────────────── */
  .periods-wrap {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }
  .periods {
    display: inline-flex;
    gap: 4px;
    padding: 4px;
    background: rgba(0, 0, 0, 0.32);
    border: 1px solid var(--border);
    border-radius: var(--radius-pill);
  }
  .period-btn {
    appearance: none;
    border: none;
    background: transparent;
    color: var(--ivory-2);
    padding: 8px 14px;
    border-radius: 999px;
    font: 500 12px/1 var(--font-ui);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
  }
  .period-btn:hover:not(:disabled):not(.active) {
    background: var(--surface-hi);
    color: var(--ivory);
  }
  .period-btn:focus-visible {
    outline: 2px solid var(--lime);
    outline-offset: 2px;
  }
  .period-btn.active {
    background: var(--lime);
    color: var(--ink);
    font-weight: 600;
    box-shadow:
      0 0 0 1px rgba(197, 240, 74, 0.35),
      0 6px 18px -8px rgba(197, 240, 74, 0.6);
  }
  .period-btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  .loading-pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 999px;
    font: 500 11px/1 var(--font-mono);
    color: var(--ivory-3);
  }
  .loading-pill :global(svg.spin) {
    animation: spin 1s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  /* ── Panel ───────────────────────────────────────────────────────────── */
  .panel {
    display: flex;
    flex-direction: column;
    gap: 18px;
  }

  /* ── Stats 4 cards ───────────────────────────────────────────────────── */
  .stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 12px;
  }
  .stat-card {
    padding: 18px 20px;
    background: linear-gradient(180deg, rgba(255, 255, 255, 0.025), rgba(255, 255, 255, 0.005));
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
  }
  .stat-l {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--ivory-3);
    margin-bottom: 10px;
  }
  .stat-v {
    font: 400 36px/1 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.01em;
  }
  .stat-v.skel {
    color: var(--ivory-4);
  }
  .stat-v .u {
    font: 400 13px/1 var(--font-ui);
    font-style: normal;
    color: var(--ivory-3);
    margin-left: 6px;
  }
  .stat-delta {
    margin-top: 8px;
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 3px 9px;
    border-radius: 999px;
    font: 600 11px/1 var(--font-mono);
  }
  .stat-delta.lime {
    background: var(--lime-soft);
    color: var(--lime);
    border: 1px solid rgba(197, 240, 74, 0.3);
  }
  .stat-delta.coral {
    background: rgba(240, 108, 90, 0.1);
    color: var(--coral);
    border: 1px solid rgba(240, 108, 90, 0.32);
  }
  .stat-delta.neutral {
    background: var(--surface);
    color: var(--ivory-3);
    border: 1px solid var(--border);
  }
  .d-vs {
    font-weight: 400;
    color: var(--ivory-3);
    margin-left: 2px;
  }

  /* ── Cards génériques ────────────────────────────────────────────────── */
  .card {
    padding: 24px 26px;
    background: linear-gradient(180deg, rgba(255, 255, 255, 0.025), rgba(255, 255, 255, 0.005));
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
  }
  .ch {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 18px;
    padding-bottom: 14px;
    border-bottom: 1px solid var(--border);
  }
  .ch-l {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .ch-ico {
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
  .ch .eyebrow {
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--ivory-3);
    margin-bottom: 4px;
  }
  .ch h3 {
    font: 400 22px/1.15 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.01em;
    margin: 0;
  }
  .ch-mean {
    font: 500 11px/1 var(--font-mono);
    color: var(--ivory-3);
  }

  .empty {
    margin: 0;
    padding: 18px 4px;
    font: 400 13px/1.5 var(--font-ui);
    font-style: italic;
    color: var(--ivory-3);
  }

  /* ── Chart ───────────────────────────────────────────────────────────── */
  .chart-host {
    position: relative;
  }
  .chart {
    position: relative;
    width: 100%;
    overflow-x: auto;
    overflow-y: visible;
  }
  .chart svg {
    display: block;
    width: 100%;
    min-width: 100%;
    height: 200px;
  }
  .bar {
    fill: var(--ivory-3);
    transition:
      fill var(--dur-base) var(--ease),
      filter var(--dur-base) var(--ease);
  }
  .bar[data-tone='lime'] {
    fill: var(--lime);
  }
  .bar[data-tone='coral'] {
    fill: var(--coral);
  }
  .bar[data-hover='true'] {
    filter: brightness(1.25) drop-shadow(0 0 6px var(--lime-glow));
  }
  .x-axis {
    fill: var(--ivory-4);
    font: 400 9px/1 var(--font-mono);
  }
  .tooltip {
    position: absolute;
    top: -4px;
    transform: translate(-50%, -100%);
    padding: 8px 10px;
    background: rgba(10, 13, 11, 0.96);
    border: 1px solid var(--border-hi);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-soft);
    pointer-events: none;
    z-index: 5;
    min-width: 140px;
  }
  .tt-date {
    font: 600 11px/1 var(--font-mono);
    color: var(--lime);
    margin-bottom: 6px;
    padding-bottom: 4px;
    border-bottom: 1px solid var(--border);
  }
  .tt-row {
    display: flex;
    align-items: center;
    gap: 5px;
    font: 400 11px/1.4 var(--font-ui);
    color: var(--ivory);
  }
  .legend {
    margin-top: 10px;
    display: flex;
    flex-wrap: wrap;
    gap: 16px;
    font: 400 11px/1 var(--font-ui);
    color: var(--ivory-3);
  }
  .leg-item {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .leg-item .dot {
    width: 8px;
    height: 8px;
    border-radius: 2px;
  }

  /* table accessible (off-screen, lecteurs d'écran uniquement) */
  .sr-table {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0 0 0 0);
    clip-path: inset(50%);
    white-space: nowrap;
  }

  /* ── Top list ────────────────────────────────────────────────────────── */
  .top-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .top-row {
    display: grid;
    grid-template-columns: 36px minmax(140px, auto) auto 1fr;
    align-items: center;
    gap: 14px;
    padding: 10px 14px;
    background: rgba(0, 0, 0, 0.22);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
  }
  .rank {
    font: 600 13px/1 var(--font-mono);
    color: var(--lime);
  }
  .model-link {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    border: none;
    text-decoration: none;
    padding: 0;
    color: var(--ivory);
  }
  .model-link:hover {
    color: var(--lime);
  }
  .model-name {
    font: 500 13px/1 var(--font-ui);
  }
  .cal-badge {
    display: inline-flex;
    align-items: center;
    padding: 2px 7px;
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid var(--border);
    border-radius: 999px;
    font: 500 9px/1 var(--font-mono);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }
  .model-count {
    font: 500 11px/1 var(--font-mono);
    color: var(--ivory-3);
  }
  .bar-row {
    display: grid;
    grid-template-columns: 1fr auto;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }
  .bar-bg {
    height: 8px;
    background: rgba(255, 255, 255, 0.04);
    border-radius: 4px;
    overflow: hidden;
  }
  .bar-fg {
    height: 100%;
    background: linear-gradient(90deg, var(--lime-deep), var(--lime));
    border-radius: 4px;
    transition: width var(--dur-slow) var(--ease);
  }
  .bar-val {
    font: 500 12px/1 var(--font-mono);
    color: var(--ivory);
    white-space: nowrap;
  }
  .bar-val .u {
    color: var(--ivory-3);
    margin-left: 4px;
    font-weight: 400;
  }

  /* ── Erreur ──────────────────────────────────────────────────────────── */
  .form-err {
    display: flex;
    gap: 8px;
    align-items: flex-start;
    padding: 10px 14px;
    background: rgba(240, 108, 90, 0.08);
    border: 1px solid rgba(240, 108, 90, 0.3);
    border-radius: var(--radius-md);
  }
  .err-ico {
    color: var(--coral);
    flex-shrink: 0;
    padding-top: 2px;
  }
  .form-err > div {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font: 400 12px/1.4 var(--font-ui);
    color: var(--ivory);
  }
  .form-err strong {
    color: var(--coral);
    font-weight: 600;
  }
  .form-err .help {
    color: var(--ivory-3);
    font-style: italic;
    font-size: 11px;
  }

  /* ── Pied de page ────────────────────────────────────────────────────── */
  .pfoot {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 10px;
    padding-top: 10px;
    border-top: 1px solid var(--border);
    font: 400 12px/1.4 var(--font-ui);
    color: var(--ivory-3);
  }
  .pf-label {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .pf-range {
    color: var(--ivory);
    background: var(--surface);
    padding: 3px 8px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
  }
  .pf-sep {
    color: var(--ivory-4);
  }
  .pf-link {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: var(--lime);
    border-bottom: 1px dashed rgba(197, 240, 74, 0.4);
    padding-bottom: 1px;
  }

  .mono {
    font-family: var(--font-mono);
  }

  /* ── Responsive ──────────────────────────────────────────────────────── */
  @media (max-width: 960px) {
    .canvas-inner {
      padding: 24px 16px 60px;
    }
    .hero-h1 {
      font-size: 32px;
    }
    .top-row {
      grid-template-columns: 32px 1fr;
      grid-template-rows: auto auto;
      row-gap: 6px;
    }
    .top-row .model-link {
      grid-column: 2;
    }
    .top-row .model-count {
      grid-column: 1 / 3;
    }
    .top-row .bar-row {
      grid-column: 1 / 3;
    }
  }
</style>
