<script lang="ts">
  // Module M13 — Simulateur « Et si...? » (C11).
  // Consomme la commande IPC `simulate_scenarios` exposée par `sobria-app`.
  // Contrat no-mock : hors Tauri on affiche une bannière d'erreur explicite.
  //
  // Voir :
  //   - briefs/chantiers/C11-simulateur-et-si.md
  //   - crates/sobria-app/src/dto.rs (bloc "simulation")
  //   - docs/CAHIER-DES-CHARGES-v1.0.md §4 M13

  import {
    AlertTriangle,
    Info,
    PlugZap,
    HelpCircle,
    Lock,
    Cpu,
    Hash,
    Sparkles
  } from '@lucide/svelte';
  import {
    isTauriContext,
    listModels,
    simulateScenarios,
    SobriaIpcError,
    type IpcErrorCode,
    type ModelPresetDto,
    type ScenarioDto,
    type SimulationRequestDto,
    type SimulationResultDto
  } from '$lib/api';
  import { preferences, type ModuleId } from '$lib/preferences';

  import LeverPanel, {
    makeInitialLeverState,
    PUE_MIN,
    MIX_MIN,
    EMBODIED_MIN,
    WUE_MIN,
    type LeverState
  } from '$lib/components/m13/LeverPanel.svelte';
  import Verdict from '$lib/components/m13/Verdict.svelte';
  import DominantLever, { type IsolateKey } from '$lib/components/m13/DominantLever.svelte';
  import Waterfall, { type WaterfallStep } from '$lib/components/m13/Waterfall.svelte';
  import Forecast from '$lib/components/m13/Forecast.svelte';

  const MODULE_ID: ModuleId = 'm13';

  // ─── Module gating ───────────────────────────────────────────────────────
  $effect(() => {
    if ($preferences.loaded && !$preferences.enabled_modules.includes(MODULE_ID)) {
      window.location.replace('/?disabled=' + MODULE_ID);
    }
  });

  // ─── State ───────────────────────────────────────────────────────────────
  let models = $state<ModelPresetDto[]>([]);
  let bootstrapping = $state(true);
  let loadError = $state<{ code: IpcErrorCode; message: string } | null>(null);

  // Baseline (column gauche)
  let baselineModelId = $state<string>('');
  let baselineTokensIn = $state<number>(100);
  let baselineTokensOut = $state<number>(500);

  // Lever state (right panel)
  let leverState = $state<LeverState>(makeInitialLeverState('', 500));

  // Forecast inputs
  let forecastVolume = $state(100);
  let forecastGrowth = $state(5);

  // Simulation result
  let result = $state<SimulationResultDto | null>(null);
  let simulating = $state(false);
  let simError = $state<{ code: IpcErrorCode; message: string } | null>(null);

  const tauriAvailable = $derived(isTauriContext());

  // ─── Bootstrap : charge la liste des modèles ─────────────────────────────
  $effect(() => {
    void (async () => {
      if (!tauriAvailable) {
        bootstrapping = false;
        loadError = {
          code: 'tauri_unavailable',
          message:
            "L'application doit être lancée via `cargo run -p sobria-app` (ou `cargo tauri dev`). La simulation ne peut pas tourner dans un navigateur seul."
        };
        return;
      }
      try {
        const m = await listModels();
        models = m;
        if (m.length > 0) {
          const firstId = m[0]?.id ?? '';
          baselineModelId = firstId;
          leverState = makeInitialLeverState(firstId, baselineTokensOut);
        }
      } catch (err) {
        if (err instanceof SobriaIpcError) {
          loadError = { code: err.code, message: err.message };
        } else {
          loadError = { code: 'internal', message: 'Échec du chargement du référentiel modèles' };
        }
      } finally {
        bootstrapping = false;
      }
    })();
  });

  // ─── Sync baseline → lever state (si non-touché) ─────────────────────────
  $effect(() => {
    if (!leverState.touched.model && leverState.modelId !== baselineModelId) {
      leverState.modelId = baselineModelId;
    }
  });
  $effect(() => {
    if (!leverState.touched.tokens && leverState.tokensOut !== baselineTokensOut) {
      leverState.tokensOut = baselineTokensOut;
    }
  });

  // ─── Construction de la requête IPC ──────────────────────────────────────
  //
  // On envoie au max 12 scénarios :
  //   - 1  "Configuration actuelle" (combinaison de TOUS les overrides)
  //   - 5  isolation scenarios (1 par lever paramétrique, à valeur neutre)
  //   - 0-6 waterfall scenarios (cumulatif sur les levers touchés)
  //
  // Ordre déterministe d'application des levers pour le waterfall :
  //   model → tokens → mix → pue → embodied → wue

  const WATERFALL_ORDER: Array<{
    key: keyof LeverState['touched'];
    label: string;
    apply: (s: ScenarioDto['overrides'], state: LeverState) => void;
  }> = [
    {
      key: 'model',
      label: '+ Modèle',
      apply: (o, s) => {
        o.model_id = s.modelId;
      }
    },
    {
      key: 'tokens',
      label: '+ Tokens out',
      apply: (o, s) => {
        o.tokens_out = s.tokensOut;
      }
    },
    {
      key: 'mix',
      label: '+ Mix élec',
      apply: (o, s) => {
        o.if_electrical_g_per_kwh = s.ifMix;
      }
    },
    {
      key: 'pue',
      label: '+ PUE',
      apply: (o, s) => {
        o.pue = s.pue;
      }
    },
    {
      key: 'embodied',
      label: '+ Embodied',
      apply: (o, s) => {
        o.embodied_g_per_request = s.embodied;
      }
    },
    {
      key: 'wue',
      label: '+ WUE',
      apply: (o, s) => {
        o.wue_l_per_kwh = s.wue;
      }
    }
  ];

  function buildFullOverrides(s: LeverState): ScenarioDto['overrides'] {
    const o: ScenarioDto['overrides'] = {};
    if (s.touched.model) o.model_id = s.modelId;
    if (s.touched.tokens) o.tokens_out = s.tokensOut;
    if (s.touched.pue) o.pue = s.pue;
    if (s.touched.mix) o.if_electrical_g_per_kwh = s.ifMix;
    if (s.touched.embodied) o.embodied_g_per_request = s.embodied;
    if (s.touched.wue) o.wue_l_per_kwh = s.wue;
    return o;
  }

  function buildScenarios(s: LeverState): ScenarioDto[] {
    const scenarios: ScenarioDto[] = [];
    const fullOv = buildFullOverrides(s);
    const anyTouched = Object.keys(fullOv).length > 0;

    // 1) "Configuration actuelle"
    if (anyTouched) {
      scenarios.push({ label: 'Configuration actuelle', overrides: fullOv });
    }

    // 2) Isolation (toujours, pour le dominant lever)
    scenarios.push({ label: 'isolate:pue', overrides: { pue: PUE_MIN } });
    scenarios.push({ label: 'isolate:mix', overrides: { if_electrical_g_per_kwh: MIX_MIN } });
    scenarios.push({ label: 'isolate:tokens', overrides: { tokens_out: 1 } });
    scenarios.push({
      label: 'isolate:embodied',
      overrides: { embodied_g_per_request: EMBODIED_MIN }
    });
    scenarios.push({ label: 'isolate:wue', overrides: { wue_l_per_kwh: WUE_MIN } });

    // 3) Waterfall (cumulatif, sur les levers touchés uniquement)
    const cumOverrides: ScenarioDto['overrides'] = {};
    for (const step of WATERFALL_ORDER) {
      if (!s.touched[step.key]) continue;
      step.apply(cumOverrides, s);
      scenarios.push({
        label: `wf:${step.label}`,
        overrides: { ...cumOverrides }
      });
    }

    return scenarios;
  }

  function buildRequest(): SimulationRequestDto | null {
    if (!baselineModelId) return null;
    return {
      baseline: {
        model_id: baselineModelId,
        tokens_in: baselineTokensIn,
        tokens_out_estimated: baselineTokensOut
      },
      scenarios: buildScenarios(leverState),
      ...(forecastVolume > 0
        ? {
            forecast: {
              months: 12,
              monthly_growth_pct: forecastGrowth,
              base_volume_per_day: forecastVolume
            }
          }
        : {})
    };
  }

  // ─── Effect debounced : recalcule à chaque changement ────────────────────

  let debounceHandle: ReturnType<typeof setTimeout> | null = null;
  let lastSerialized = '';

  function scheduleSimulation(req: SimulationRequestDto) {
    if (debounceHandle !== null) clearTimeout(debounceHandle);
    const ser = JSON.stringify(req);
    if (ser === lastSerialized) return;
    debounceHandle = setTimeout(() => {
      lastSerialized = ser;
      void runSimulation(req);
    }, 300);
  }

  async function runSimulation(req: SimulationRequestDto) {
    if (!tauriAvailable) return;
    simulating = true;
    try {
      const r = await simulateScenarios(req);
      result = r;
      simError = null;
    } catch (err) {
      if (err instanceof SobriaIpcError) {
        simError = { code: err.code, message: err.message };
      } else {
        simError = { code: 'internal', message: 'Échec de la simulation' };
      }
    } finally {
      simulating = false;
    }
  }

  // Effect : reconstruit + redéclenche dès qu'un input baseline / lever / forecast change.
  $effect(() => {
    // Dépendances explicites (Svelte 5 piste les accès aux $state) :
    void baselineModelId;
    void baselineTokensIn;
    void baselineTokensOut;
    void leverState.modelId;
    void leverState.tokensOut;
    void leverState.pue;
    void leverState.ifMix;
    void leverState.embodied;
    void leverState.wue;
    void leverState.touched.model;
    void leverState.touched.tokens;
    void leverState.touched.pue;
    void leverState.touched.mix;
    void leverState.touched.embodied;
    void leverState.touched.wue;
    void forecastVolume;
    void forecastGrowth;

    if (!tauriAvailable || bootstrapping) return;
    const req = buildRequest();
    if (req) scheduleSimulation(req);
  });

  function resetLevers() {
    leverState = makeInitialLeverState(baselineModelId, baselineTokensOut);
  }

  function onForecastChange(vol: number, growth: number) {
    forecastVolume = vol;
    forecastGrowth = growth;
  }

  // ─── Dérivés : extraction des scénarios par type ────────────────────────
  const currentScenario = $derived(
    result?.scenarios.find((s) => s.label === 'Configuration actuelle') ?? null
  );

  const isolationOutcomes = $derived.by(() => {
    if (!result) return [];
    const keys: Array<{ label: string; key: IsolateKey }> = [
      { label: 'isolate:pue', key: 'pue' },
      { label: 'isolate:mix', key: 'mix' },
      { label: 'isolate:tokens', key: 'tokens' },
      { label: 'isolate:embodied', key: 'embodied' },
      { label: 'isolate:wue', key: 'wue' }
    ];
    return keys
      .map(({ label, key }) => {
        const sc = result?.scenarios.find((s) => s.label === label);
        if (!sc) return null;
        return { key, deltaCo2eqG: sc.delta_co2eq_g };
      })
      .filter((x): x is { key: IsolateKey; deltaCo2eqG: number } => x !== null);
  });

  const waterfallSteps = $derived.by<WaterfallStep[]>(() => {
    if (!result) return [];
    const steps: WaterfallStep[] = [];
    for (const sc of result.scenarios) {
      if (!sc.label.startsWith('wf:')) continue;
      const label = sc.label.slice('wf:'.length);
      const co2 = sc.result.indicators.find((i) => i.indicator === 'co2eq');
      if (co2) {
        steps.push({ label, cumulativeCo2eqG: co2.p50 });
      }
    }
    return steps;
  });

  const baselineCo2eqG = $derived.by<number>(() => {
    if (!result) return 0;
    const co2 = result.baseline.indicators.find((i) => i.indicator === 'co2eq');
    return co2?.p50 ?? 0;
  });

  const errorTone = $derived.by<'info' | 'warn' | 'error'>(() => {
    const e = loadError ?? simError;
    if (!e) return 'info';
    if (e.code === 'tauri_unavailable') return 'warn';
    return 'error';
  });

  const ERROR_LABELS: Record<string, string> = {
    tauri_unavailable: 'Application non lancée via Tauri',
    unknown_model: 'Modèle inconnu',
    invalid_request: 'Requête invalide',
    estimator_error: 'Erreur estimateur',
    internal: 'Erreur interne'
  };
  function errorLabel(code: string): string {
    return ERROR_LABELS[code] ?? 'Erreur';
  }

  const activeError = $derived(loadError ?? simError);
</script>

<svelte:head>
  <title>Sobr.ia · Simulateur « Et si...? »</title>
</svelte:head>

<div class="canvas-inner">
  <!-- TopBar -->
  <div class="topbar">
    <nav class="breadcrumb" aria-label="Fil d'Ariane">
      Atelier <span class="sep">/</span>
      <span class="current">Simuler</span>
    </nav>
    <div class="spacer"></div>
    <span class="local-pill">
      <Lock size={12} strokeWidth={1.8} />
      Simulation 100 % locale
    </span>
    <a class="icon-btn" href="/methodo" aria-label="Méthodologie">
      <HelpCircle size={16} strokeWidth={1.6} />
    </a>
  </div>

  <!-- Hero -->
  <section class="hero">
    <div class="hero-eyebrow">
      <span class="pulse" aria-hidden="true"></span>
      Module M13 · 7 leviers temps réel
    </div>
    <h1 class="hero-h1">
      Et si on changeait <em>un seul levier</em> ?
    </h1>
    <p class="hero-sub">
      Construisez un baseline, actionnez un curseur, voyez l'impact CO₂eq se recalibrer
      instantanément. Le simulateur identifie le levier dominant et projette votre usage sur 12
      mois.
    </p>
  </section>

  <!-- Bannière erreur -->
  {#if activeError}
    <div class="banner" data-tone={errorTone} role="alert">
      <span class="banner-ico" aria-hidden="true">
        {#if errorTone === 'warn'}
          <AlertTriangle size={18} strokeWidth={1.8} />
        {:else if errorTone === 'error'}
          <PlugZap size={18} strokeWidth={1.8} />
        {:else}
          <Info size={18} strokeWidth={1.8} />
        {/if}
      </span>
      <div class="banner-body">
        <strong>{errorLabel(activeError.code)}</strong>
        <span>{activeError.message}</span>
      </div>
    </div>
  {/if}

  {#if tauriAvailable && !bootstrapping}
    <div class="grid">
      <!-- ─── Col gauche : baseline + lever panel ─────────────────────── -->
      <div class="col-l">
        <section class="baseline-card" aria-label="Configuration baseline">
          <header class="bh">
            <div class="eyebrow">
              <Sparkles size={11} strokeWidth={1.8} />
              Baseline · point de référence
            </div>
            <h2>Configurez votre baseline</h2>
          </header>

          <div class="b-fields">
            <label class="b-field">
              <span><Cpu size={11} strokeWidth={1.8} /> Modèle</span>
              <select bind:value={baselineModelId} class="b-select">
                {#each models as m (m.id)}
                  <option value={m.id}>{m.display_name} · {m.provider}</option>
                {/each}
              </select>
            </label>
            <label class="b-field">
              <span><Hash size={11} strokeWidth={1.8} /> Tokens entrée</span>
              <input
                type="number"
                min="1"
                max="100000"
                bind:value={baselineTokensIn}
                class="b-num mono"
              />
            </label>
            <label class="b-field">
              <span><Hash size={11} strokeWidth={1.8} /> Tokens sortie</span>
              <input
                type="number"
                min="1"
                max="100000"
                bind:value={baselineTokensOut}
                class="b-num mono"
              />
            </label>
          </div>
        </section>

        <LeverPanel
          {models}
          {baselineModelId}
          {baselineTokensOut}
          bind:state={leverState}
          onreset={resetLevers}
        />
      </div>

      <!-- ─── Col droite : verdict + dominant + waterfall + forecast ──── -->
      <div class="col-r">
        {#if result}
          <Verdict scenario={currentScenario} baseline={result.baseline} />
          <DominantLever outcomes={isolationOutcomes} {baselineCo2eqG} />
          <Waterfall {baselineCo2eqG} steps={waterfallSteps} />
          <Forecast
            forecast={result.forecast ?? null}
            volumePerDay={forecastVolume}
            growthPct={forecastGrowth}
            onchange={onForecastChange}
          />
        {:else if simulating}
          <div class="loading">
            <span class="dot"></span><span class="dot"></span><span class="dot"></span>
            Simulation en cours…
          </div>
        {:else}
          <div class="loading">Préparation du baseline…</div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .canvas-inner {
    max-width: 1440px;
    margin: 0 auto;
    padding: 40px 56px 80px;
  }

  /* TopBar */
  .topbar {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 28px;
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

  /* Hero */
  .hero {
    padding-bottom: 24px;
    margin-bottom: 24px;
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
    max-width: 720px;
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

  /* Bannière */
  .banner {
    display: flex;
    gap: 14px;
    align-items: flex-start;
    padding: 14px 18px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-hi);
    margin-bottom: 20px;
  }
  .banner[data-tone='warn'] {
    background: rgba(245, 183, 105, 0.08);
    border-color: rgba(245, 183, 105, 0.25);
  }
  .banner[data-tone='error'] {
    background: rgba(240, 108, 90, 0.08);
    border-color: rgba(240, 108, 90, 0.3);
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

  /* Grid */
  .grid {
    display: grid;
    grid-template-columns: minmax(360px, 1fr) minmax(0, 1.4fr);
    gap: 24px;
    align-items: start;
  }
  .col-l {
    display: flex;
    flex-direction: column;
    gap: 20px;
    position: sticky;
    top: 20px;
  }
  .col-r {
    display: flex;
    flex-direction: column;
    gap: 20px;
    min-width: 0;
  }

  /* Baseline card */
  .baseline-card {
    padding: 20px 22px 18px;
    background: linear-gradient(180deg, rgba(255, 255, 255, 0.025), rgba(255, 255, 255, 0.005));
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
  }
  .bh {
    margin-bottom: 14px;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--border);
  }
  .bh .eyebrow {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--ivory-3);
    margin-bottom: 6px;
  }
  .bh h2 {
    font: 400 22px/1.15 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.01em;
    margin: 0;
  }

  .b-fields {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .b-field {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 9px 12px;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
  }
  .b-field > span:first-child {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--ivory-3);
    min-width: 130px;
  }
  .b-select {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    color: var(--ivory);
    font: 400 13px/1.2 var(--font-ui);
    cursor: pointer;
  }
  .b-num {
    width: 100px;
    margin-left: auto;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--ivory);
    font: 600 12px/1 var(--font-mono);
    text-align: right;
    padding: 4px 8px;
  }
  .mono {
    font-family: var(--font-mono);
  }

  /* Loading state */
  .loading {
    padding: 60px 20px;
    text-align: center;
    font: 400 14px/1.5 var(--font-ui);
    color: var(--ivory-3);
    background: rgba(255, 255, 255, 0.015);
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
  }
  .dot {
    display: inline-block;
    width: 6px;
    height: 6px;
    background: var(--lime);
    border-radius: 50%;
    animation: blink 1.4s infinite;
  }
  .dot:nth-child(2) {
    animation-delay: 0.2s;
  }
  .dot:nth-child(3) {
    animation-delay: 0.4s;
  }
  @keyframes blink {
    0%,
    80%,
    100% {
      opacity: 0.25;
    }
    40% {
      opacity: 1;
    }
  }

  @media (max-width: 1080px) {
    .grid {
      grid-template-columns: 1fr;
    }
    .col-l {
      position: static;
    }
  }
  @media (max-width: 960px) {
    .canvas-inner {
      padding: 24px 16px 60px;
    }
    .hero-h1 {
      font-size: 32px;
    }
  }
</style>
