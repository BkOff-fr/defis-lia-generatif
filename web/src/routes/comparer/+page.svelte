<script lang="ts">
  import {
    Sparkles,
    Award,
    HelpCircle,
    Lock,
    Info,
    AlertTriangle,
    PlugZap,
    RotateCcw,
    Check,
    Loader,
    X,
    ArrowUpRight,
    FlaskConical
  } from '@lucide/svelte';
  import {
    estimatePrompt,
    isTauriContext,
    listModels,
    SobriaIpcError,
    type EstimationResultDto,
    type IndicatorDto,
    type IpcErrorCode,
    type ModelPresetDto
  } from '$lib/api';
  import { tick } from 'svelte';

  // ─── State ───────────────────────────────────────────────────────────
  const MAX_SELECTED = 8;
  const MIN_SELECTED = 2;

  let models = $state<ModelPresetDto[]>([]);
  let bootstrapping = $state(true);
  let loadError = $state<{ code: IpcErrorCode; message: string } | null>(null);

  let selectedIds = $state<Set<string>>(new Set());
  // Prompt (source de vérité pour tokens_in). Ratio FR 3,3 chars/token —
  // même heuristique que Composer Estimer. Si l'utilisateur le laisse vide,
  // on retombe sur le `manualTokensIn` ci-dessous.
  let prompt = $state('');
  let manualTokensIn = $state(100);
  let tokensOut = $state(500);
  const CHARS_PER_TOKEN_FR = 3.3;
  const tokensIn = $derived(
    prompt.trim().length > 0
      ? Math.max(1, Math.ceil(prompt.length / CHARS_PER_TOKEN_FR))
      : Math.max(1, manualTokensIn)
  );

  let comparing = $state(false);
  let results = $state<Record<string, EstimationResultDto>>({});
  let errorById = $state<Record<string, string>>({});

  // Modèle dont le détail de note est affiché dans le drawer (clic sur le
  // badge A-F dans la matrice).
  let selectedDetail = $state<string | null>(null);

  const tauriAvailable = $derived(isTauriContext());

  // ─── Bootstrap ─────────────────────────────────────────────────────
  $effect(() => {
    void (async () => {
      if (!tauriAvailable) {
        bootstrapping = false;
        loadError = {
          code: 'tauri_unavailable',
          message:
            "L'application doit être lancée via `cargo run -p sobria-app`. Le comparateur exécute N estimations Monte-Carlo en parallèle via le moteur Rust local — pas accessible dans un navigateur seul."
        };
        return;
      }
      try {
        const list = await listModels();
        models = list.sort((a, b) =>
          a.provider === b.provider
            ? a.display_name.localeCompare(b.display_name, 'fr')
            : a.provider.localeCompare(b.provider, 'fr')
        );
        // Lecture des paramètres d'URL (depuis l'écran Estimer : CTA
        // « Comparer avec d'autres modèles » → /comparer?prompt=…&
        // tokensOut=…&model=…).
        if (typeof window !== 'undefined') {
          const params = new URLSearchParams(window.location.search);
          const pParam = params.get('prompt');
          if (pParam) prompt = pParam;
          const toParam = params.get('tokensOut');
          if (toParam) {
            const n = Number.parseInt(toParam, 10);
            if (Number.isFinite(n) && n > 0) tokensOut = n;
          }
        }
        // Préselection : 4 modèles d'entrée de gamme + le modèle source de
        // la redirection si fourni.
        const defaults = ['gpt-4o-mini', 'claude-3-5-sonnet', 'mistral-large-2', 'llama-3-1-70b'];
        for (const id of defaults) {
          if (list.find((m) => m.id === id)) selectedIds.add(id);
        }
        if (typeof window !== 'undefined') {
          const params = new URLSearchParams(window.location.search);
          const modelParam = params.get('model');
          if (modelParam && list.find((m) => m.id === modelParam)) {
            selectedIds.add(modelParam);
          }
        }
        // Respecte la borne MAX_SELECTED (le `model` source pourrait pousser
        // à 5 modèles, on tronque pour rester sous 8 — pas critique ici).
        selectedIds = new Set([...selectedIds].slice(0, MAX_SELECTED));
      } catch (err) {
        if (err instanceof SobriaIpcError) {
          loadError = { code: err.code, message: err.message };
        } else {
          loadError = { code: 'internal', message: 'Échec du chargement du référentiel' };
        }
      } finally {
        bootstrapping = false;
      }
    })();
  });

  // ─── Sélection ─────────────────────────────────────────────────────
  function toggleModel(id: string) {
    const next = new Set(selectedIds);
    if (next.has(id)) {
      next.delete(id);
    } else {
      if (next.size >= MAX_SELECTED) return; // bloque au-delà de 8
      next.add(id);
    }
    selectedIds = next;
    // Invalide les résultats si le set change : on évite d'afficher une
    // matrice incohérente avec ce que l'utilisateur voit en sélection.
    results = {};
    errorById = {};
  }

  function resetSelection() {
    selectedIds = new Set();
    results = {};
    errorById = {};
  }

  const canCompare = $derived(
    selectedIds.size >= MIN_SELECTED &&
      selectedIds.size <= MAX_SELECTED &&
      tokensIn > 0 &&
      tokensOut > 0
  );

  // ─── Comparaison : fan-out estimatePrompt ──────────────────────────
  async function runComparison() {
    if (!canCompare) return;
    comparing = true;
    results = {};
    errorById = {};

    const ids = [...selectedIds];
    // Parallèle : SQLite WAL gère l'écriture concurrente dans le ledger,
    // donc on lance tout en même temps pour minimiser le perçu < 1 s.
    const settled = await Promise.allSettled(
      ids.map((id) =>
        estimatePrompt({
          model_id: id,
          tokens_in: Math.max(1, tokensIn),
          tokens_out_estimated: Math.max(1, tokensOut)
        }).then((r) => [id, r] as const)
      )
    );

    const nextResults: Record<string, EstimationResultDto> = {};
    const nextErrors: Record<string, string> = {};
    for (const s of settled) {
      if (s.status === 'fulfilled') {
        const [id, r] = s.value;
        nextResults[id] = r;
      } else {
        // En cas d'erreur on ne sait pas à quel id elle se rapporte
        // depuis la Promise — on remonte un message générique côté table.
        // Pour identifier précisément, l'ordre est préservé par allSettled,
        // donc on retombe sur l'index dans `ids`.
        const idx = settled.indexOf(s);
        const id = ids[idx];
        if (id) {
          nextErrors[id] =
            s.reason instanceof SobriaIpcError
              ? s.reason.message
              : "Échec de l'estimation pour ce modèle";
        }
      }
    }
    results = nextResults;
    errorById = nextErrors;
    comparing = false;
  }

  // ─── Dérivés : matrice + score composite ──────────────────────────
  type IndKey = 'co2eq' | 'energy' | 'water';

  function pickInd(r: EstimationResultDto, name: IndKey): IndicatorDto | undefined {
    return r.indicators.find((i) => i.indicator === name);
  }

  type Row = {
    model: ModelPresetDto;
    result: EstimationResultDto | undefined;
    error: string | undefined;
    // Valeurs P5/P50/P95 par indicateur en unité canonique (Rust).
    co2: IndicatorDto | undefined;
    energy: IndicatorDto | undefined;
    water: IndicatorDto | undefined;
    totalTokens: number;
  };

  const rows = $derived.by<Row[]>(() => {
    const ids = [...selectedIds];
    const ms = ids
      .map((id) => models.find((m) => m.id === id))
      .filter((m): m is ModelPresetDto => !!m);

    return ms.map((model) => {
      const r = results[model.id];
      const err = errorById[model.id];
      if (!r) {
        return {
          model,
          result: undefined,
          error: err,
          co2: undefined,
          energy: undefined,
          water: undefined,
          totalTokens: 0
        };
      }
      return {
        model,
        result: r,
        error: err,
        co2: pickInd(r, 'co2eq'),
        energy: pickInd(r, 'energy'),
        water: pickInd(r, 'water'),
        totalTokens: r.request.tokens_in + r.request.tokens_out_estimated
      };
    });
  });

  // ─── Drawer : modèle sélectionné pour détail de note ───────────────
  const detailRow = $derived(
    selectedDetail ? rows.find((r) => r.model.id === selectedDetail) : undefined
  );

  function openDetail(modelId: string) {
    selectedDetail = modelId;
  }
  function closeDetail() {
    selectedDetail = null;
  }
  function handleEsc(e: KeyboardEvent) {
    if (e.key === 'Escape' && selectedDetail) closeDetail();
  }
  let closeBtn: HTMLButtonElement | undefined = $state();
  $effect(() => {
    if (selectedDetail) {
      void tick().then(() => closeBtn?.focus());
    }
  });

  // ─── Échelles d'unités partagées (mêmes chaînes que ResultBlock) ───
  type Scale = { mult: number; unit: string };
  const UNIT_CHAINS: Record<string, Scale[]> = {
    gCO2eq: [
      { mult: 1e-3, unit: 'kg CO₂eq' },
      { mult: 1, unit: 'g CO₂eq' },
      { mult: 1e3, unit: 'mg CO₂eq' },
      { mult: 1e6, unit: 'µg CO₂eq' },
      { mult: 1e9, unit: 'ng CO₂eq' }
    ],
    Wh: [
      { mult: 1e-3, unit: 'kWh' },
      { mult: 1, unit: 'Wh' },
      { mult: 1e3, unit: 'mWh' },
      { mult: 1e6, unit: 'µWh' },
      { mult: 1e9, unit: 'nWh' }
    ],
    L: [
      { mult: 1, unit: 'L' },
      { mult: 1e3, unit: 'mL' },
      { mult: 1e6, unit: 'µL' },
      { mult: 1e9, unit: 'nL' }
    ]
  };

  /** Échelle commune à tous les modèles comparés sur cet indicateur,
   *  choisie sur la médiane des P50 — toutes les cards affichent la
   *  même unité pour pouvoir comparer à l'œil. */
  function pickColScale(values: number[], baseUnit: string): Scale {
    const chain = UNIT_CHAINS[baseUnit];
    const fallback: Scale = { mult: 1, unit: baseUnit };
    if (!chain || chain.length === 0 || values.length === 0) return fallback;
    const sorted = [...values].sort((a, b) => a - b);
    const mid = sorted[Math.floor(sorted.length / 2)] ?? 0;
    if (!Number.isFinite(mid) || mid === 0) {
      return chain.find((s) => s.mult === 1) ?? chain[0] ?? fallback;
    }
    for (const s of chain) {
      const v = Math.abs(mid * s.mult);
      if (v >= 1 && v < 1000) return s;
    }
    return chain[chain.length - 1] ?? fallback;
  }

  const co2Scale = $derived(
    pickColScale(
      rows.map((r) => r.co2?.p50).filter((v): v is number => typeof v === 'number'),
      'gCO2eq'
    )
  );
  const energyScale = $derived(
    pickColScale(
      rows.map((r) => r.energy?.p50).filter((v): v is number => typeof v === 'number'),
      'Wh'
    )
  );
  const waterScale = $derived(
    pickColScale(
      rows.map((r) => r.water?.p50).filter((v): v is number => typeof v === 'number'),
      'L'
    )
  );

  function fmtNum(value: number): string {
    if (!Number.isFinite(value)) return '—';
    return new Intl.NumberFormat('fr-FR', {
      maximumSignificantDigits: 3,
      minimumSignificantDigits: 1
    }).format(value);
  }

  // ─── Verdict : meilleur vs pire CO₂eq ──────────────────────────────
  type Verdict = {
    best: Row;
    worst: Row;
    ratio: number; // worst.p50 / best.p50
    percentSaved: number; // (1 − best/worst) × 100
  };

  const verdict = $derived.by<Verdict | undefined>(() => {
    const scored = rows.filter(
      (r): r is Row & { co2: IndicatorDto } => !!r.co2 && Number.isFinite(r.co2.p50)
    );
    if (scored.length < 2) return undefined;
    const sorted = [...scored].sort((a, b) => a.co2.p50 - b.co2.p50);
    const best = sorted[0];
    const worst = sorted[sorted.length - 1];
    if (!best || !worst || best === worst) return undefined;
    if (best.co2.p50 <= 0) return undefined;
    const ratio = worst.co2.p50 / best.co2.p50;
    const percentSaved = (1 - best.co2.p50 / worst.co2.p50) * 100;
    return { best, worst, ratio, percentSaved };
  });

  /** Largeur de barre relative pour un P50 dans sa colonne (0–100 %) ;
   *  utilisée dans la card pour rendre le ratio visuel. */
  function relBarPct(value: number | undefined, key: 'co2' | 'energy' | 'water'): number {
    if (typeof value !== 'number' || !Number.isFinite(value)) return 0;
    const values = rows
      .map((r) => (key === 'co2' ? r.co2?.p50 : key === 'energy' ? r.energy?.p50 : r.water?.p50))
      .filter((v): v is number => typeof v === 'number' && Number.isFinite(v));
    if (values.length === 0) return 0;
    const max = Math.max(...values);
    if (max <= 0) return 0;
    return Math.min(100, Math.max(2, (value / max) * 100));
  }

  const errorTone = $derived.by<'info' | 'warn' | 'error'>(() => {
    if (!loadError) return 'info';
    if (loadError.code === 'tauri_unavailable') return 'warn';
    return 'error';
  });

  const ERROR_LABELS: Record<string, string> = {
    tauri_unavailable: 'Application non lancée via Tauri',
    internal: 'Erreur interne'
  };
  function errorLabel(code: string): string {
    return ERROR_LABELS[code] ?? 'Erreur';
  }

  function calibTone(c: ModelPresetDto['calibration']): string {
    return c === 'validated' ? 'good' : c === 'indicative' ? 'mid' : 'bad';
  }
  const calibLabel: Record<ModelPresetDto['calibration'], string> = {
    validated: 'Validé',
    indicative: 'Indicatif',
    extrapolated: 'Extrapolé'
  };
</script>

<svelte:head>
  <title>Sobr.ia · Comparer</title>
</svelte:head>

<div class="canvas-inner">
  <!-- ─── TopBar ─────────────────────────────────────────────── -->
  <div class="topbar">
    <nav class="breadcrumb" aria-label="Fil d'Ariane">
      Atelier <span class="sep">/</span>
      <span class="current">Comparer</span>
    </nav>
    <div class="spacer"></div>
    <span class="local-pill">
      <Lock size={12} strokeWidth={1.8} />
      Calculs 100 % locaux
    </span>
    <a class="icon-btn" href="/methodo" aria-label="Méthodologie">
      <HelpCircle size={16} strokeWidth={1.6} />
    </a>
  </div>

  <!-- ─── Hero ───────────────────────────────────────────────── -->
  <section class="hero">
    <div class="hero-eyebrow">
      <span class="pulse" aria-hidden="true"></span>
      Module M5 · comparateur de modèles
    </div>
    <h1 class="hero-h1">
      Le bon LLM <em>pour le bon usage</em>, indicateurs à l'appui.
    </h1>
    <p class="hero-sub">
      Sélectionnez 2 à 8 modèles, fixez les paramètres communs (tokens d'entrée/sortie), lancez la
      comparaison. Sobr.ia exécute les estimations Monte-Carlo en parallèle et restitue une matrice
      normalisée plus un score composite ajustable.
    </p>
  </section>

  {#if loadError}
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
        <strong>{errorLabel(loadError.code)}</strong>
        <span>{loadError.message}</span>
      </div>
    </div>
  {/if}

  {#if tauriAvailable}
    <!-- ─── Sélection modèles ──────────────────────────────────── -->
    <section class="selector">
      <header class="selector-head">
        <h2>Modèles à comparer</h2>
        <div class="counter mono">
          {selectedIds.size} / {MAX_SELECTED}
          <span class="counter-hint">· min {MIN_SELECTED}</span>
        </div>
        {#if selectedIds.size > 0}
          <button class="btn-ghost-mini" type="button" onclick={resetSelection}>
            <RotateCcw size={13} strokeWidth={1.8} />
            Réinitialiser
          </button>
        {/if}
      </header>

      {#if bootstrapping}
        <div class="selector-skel">Chargement du référentiel…</div>
      {:else}
        <div class="model-chips" role="group" aria-label="Sélecteur de modèles">
          {#each models as m (m.id)}
            {@const isOn = selectedIds.has(m.id)}
            {@const isMaxed = !isOn && selectedIds.size >= MAX_SELECTED}
            <button
              type="button"
              class="model-chip"
              class:on={isOn}
              class:maxed={isMaxed}
              disabled={isMaxed}
              onclick={() => toggleModel(m.id)}
              aria-pressed={isOn}
              title={`${m.display_name} · ${m.provider} · ~${m.approx_params_billions} B`}
            >
              {#if isOn}<Check size={11} strokeWidth={2.5} />{/if}
              <span class="chip-name">{m.display_name}</span>
              <span class="chip-prov mono">· {m.provider}</span>
            </button>
          {/each}
        </div>
      {/if}
    </section>

    <!-- ─── Prompt + paramètres communs ────────────────────────── -->
    <section class="params">
      <div class="prompt-block">
        <label for="cmp-prompt">Prompt à comparer</label>
        <textarea
          id="cmp-prompt"
          bind:value={prompt}
          placeholder="(Optionnel) Écrivez votre prompt — Sobr.ia estime alors les tokens d'entrée automatiquement. Sinon, saisissez directement le nombre de tokens ci-dessous."
          rows="2"
        ></textarea>
        <div class="prompt-meta mono">
          {#if prompt.trim().length > 0}
            <span>Tokens entrée (auto)&nbsp;: <b>{tokensIn}</b></span>
            <span class="hint">~ 3,3 chars/token FR · tokenizer réel en v0.3</span>
          {:else}
            <span class="hint">Pas de prompt → tokens entrée saisis ci-dessous.</span>
          {/if}
        </div>
      </div>

      <div class="param-row">
        <div class="param-field">
          <label for="tokens-in">Tokens d'entrée</label>
          <input
            id="tokens-in"
            type="number"
            min="1"
            max="100000"
            disabled={prompt.trim().length > 0}
            bind:value={manualTokensIn}
            title={prompt.trim().length > 0
              ? 'Désactivé : valeur dérivée du prompt ci-dessus.'
              : 'Tokens d’entrée pour les N estimations.'}
          />
        </div>
        <div class="param-field">
          <label for="tokens-out">Tokens de sortie estimés</label>
          <input id="tokens-out" type="number" min="1" max="100000" bind:value={tokensOut} />
        </div>
        <div class="param-spacer"></div>
        <button
          class="btn-primary"
          type="button"
          disabled={!canCompare || comparing}
          onclick={runComparison}
        >
          {#if comparing}
            <Loader size={16} strokeWidth={2} />
            Comparaison en cours… ({selectedIds.size} estimations parallèles)
          {:else}
            <Sparkles size={16} strokeWidth={2} />
            Comparer
          {/if}
        </button>
      </div>
    </section>

    <!-- ─── Verdict ────────────────────────────────────────────── -->
    {#if verdict && !comparing}
      <section class="verdict" aria-label="Verdict de la comparaison">
        <div class="verdict-body">
          <span class="verdict-eye">
            <Award size={12} strokeWidth={1.8} />
            Verdict · plus sobre en CO₂eq
          </span>
          <h2 class="verdict-h">
            <em>{verdict.best.model.display_name}</em>
            est <i>{fmtNum(verdict.ratio)}×</i>
            plus sobre que <span class="loser">{verdict.worst.model.display_name}</span>.
          </h2>
          <p class="verdict-why">
            À tokens identiques ({verdict.best.result?.request.tokens_in ?? '?'} entrée +
            {verdict.best.result?.request.tokens_out_estimated ?? '?'}
            sortie), {verdict.best.model.display_name} émet
            <b>{fmtNum(verdict.percentSaved)} %</b> de moins. L'écart vient principalement de la
            taille du modèle (N<sub>params</sub>) — en v0.3 le datacenter du provider entrera aussi
            dans le calcul (mix électrique, PUE, WUE par fournisseur).
          </p>
        </div>
        <div class="verdict-delta">
          <span class="verdict-delta-v">−{Math.round(verdict.percentSaved)} %</span>
          <span class="verdict-delta-u">CO₂eq économisé</span>
        </div>
      </section>
    {/if}

    <!-- ─── Cards par modèle ──────────────────────────────────── -->
    {#if rows.length > 0}
      <section class="cards-section" aria-label="Profils détaillés des modèles">
        <header class="cards-head">
          <h2>Profils détaillés</h2>
          <span class="cards-hint mono">
            unité commune CO₂eq : {co2Scale.unit} · énergie : {energyScale.unit} · eau : {waterScale.unit}
          </span>
        </header>

        <div class="cmp-grid" data-count={rows.length}>
          {#each rows as r (r.model.id)}
            {@const isBest = verdict?.best.model.id === r.model.id}
            {@const isWorst = verdict?.worst.model.id === r.model.id}
            <article
              class="cmp-card"
              class:winner={isBest}
              class:loser={isWorst}
              aria-label={`Profil ${r.model.display_name}`}
            >
              <header class="cmp-card-head">
                <span class="cmp-card-logo">
                  {r.model.display_name
                    .replace(/[^A-Za-z0-9]/g, '')
                    .slice(0, 2)
                    .toUpperCase()}
                </span>
                <div class="cmp-card-id">
                  <div class="cmp-card-name">{r.model.display_name}</div>
                  <div class="cmp-card-meta mono">
                    {r.model.provider} · ~{r.model.approx_params_billions} B
                  </div>
                </div>
                {#if isBest}
                  <span class="cmp-trophy" title="Plus sobre du panel">
                    <Award size={14} strokeWidth={2} />
                  </span>
                {/if}
              </header>

              {#if comparing}
                <div class="cmp-card-skel" aria-busy="true">
                  <Loader size={14} strokeWidth={2} />
                  Estimation Monte-Carlo en cours…
                </div>
              {:else if r.error}
                <div class="cmp-card-err">
                  <AlertTriangle size={14} strokeWidth={1.8} />
                  {r.error}
                </div>
              {:else if r.result}
                <!-- 3 metric rows -->
                <div class="cmp-metric primary">
                  <div class="cmp-metric-l">CO₂eq · médiane</div>
                  <div class="cmp-metric-v">
                    {#if r.co2}{fmtNum(r.co2.p50 * co2Scale.mult)}<span class="cmp-metric-u"
                        >{co2Scale.unit}</span
                      >{:else}—{/if}
                  </div>
                  {#if r.co2}
                    <div class="cmp-metric-r">
                      P5–P95 · {fmtNum(r.co2.p5 * co2Scale.mult)} → {fmtNum(
                        r.co2.p95 * co2Scale.mult
                      )}
                    </div>
                    <div class="cmp-bar">
                      <div
                        class="cmp-bar-fill primary"
                        style="width: {relBarPct(r.co2.p50, 'co2')}%"
                      ></div>
                    </div>
                  {/if}
                </div>

                <div class="cmp-metric">
                  <div class="cmp-metric-l">Énergie · médiane</div>
                  <div class="cmp-metric-v">
                    {#if r.energy}{fmtNum(r.energy.p50 * energyScale.mult)}<span
                        class="cmp-metric-u">{energyScale.unit}</span
                      >{:else}—{/if}
                  </div>
                  {#if r.energy}
                    <div class="cmp-metric-r">
                      P5–P95 · {fmtNum(r.energy.p5 * energyScale.mult)} → {fmtNum(
                        r.energy.p95 * energyScale.mult
                      )}
                    </div>
                    <div class="cmp-bar">
                      <div
                        class="cmp-bar-fill energy"
                        style="width: {relBarPct(r.energy.p50, 'energy')}%"
                      ></div>
                    </div>
                  {/if}
                </div>

                <div class="cmp-metric">
                  <div class="cmp-metric-l">Eau · médiane</div>
                  <div class="cmp-metric-v">
                    {#if r.water}{fmtNum(r.water.p50 * waterScale.mult)}<span class="cmp-metric-u"
                        >{waterScale.unit}</span
                      >{:else}—{/if}
                  </div>
                  {#if r.water}
                    <div class="cmp-metric-r">
                      P5–P95 · {fmtNum(r.water.p5 * waterScale.mult)} → {fmtNum(
                        r.water.p95 * waterScale.mult
                      )}
                    </div>
                    <div class="cmp-bar">
                      <div
                        class="cmp-bar-fill water"
                        style="width: {relBarPct(r.water.p50, 'water')}%"
                      ></div>
                    </div>
                  {/if}
                </div>

                <footer class="cmp-card-foot">
                  <span class="badge calib" data-tone={calibTone(r.model.calibration)}>
                    {calibLabel[r.model.calibration]}
                  </span>
                  <span class="spacer-flex"></span>
                  <button
                    type="button"
                    class="cmp-detail-btn"
                    onclick={() => openDetail(r.model.id)}
                  >
                    <FlaskConical size={12} strokeWidth={1.8} />
                    Hypothèses
                  </button>
                  <a
                    class="cmp-detail-btn"
                    href={`/journal?focus=${r.result.audit_id}`}
                    title="Voir l'entrée dans le ledger d'audit"
                  >
                    Ledger #{r.result.audit_id}
                    <ArrowUpRight size={11} strokeWidth={2} />
                  </a>
                </footer>
              {/if}
            </article>
          {/each}
        </div>

        <p class="cards-footnote">
          <Info size={12} strokeWidth={1.8} />
          Barres normalisées au pire modèle du panel (100 % = pire CO₂eq / énergie / eau). Toutes les
          valeurs proviennent du moteur Monte-Carlo Rust, 10 000 tirages par modèle, journalisées dans
          le ledger d'audit local.
        </p>
      </section>
    {/if}
  {/if}
</div>

<!-- ─── Drawer : hypothèses + sources du modèle ────────────────── -->
{#if detailRow}
  <button class="drawer-backdrop" type="button" aria-label="Fermer le détail" onclick={closeDetail}
  ></button>
  <div class="drawer" role="dialog" aria-modal="true" aria-labelledby="cmp-detail-title">
    <header class="drawer-head">
      <div>
        <div class="drawer-eye">Hypothèses · {detailRow.model.provider}</div>
        <div id="cmp-detail-title" class="drawer-title">{detailRow.model.display_name}</div>
        <div class="drawer-sub mono">
          {detailRow.model.id} · ~{detailRow.model.approx_params_billions} B params
        </div>
      </div>
      <button
        class="icon-btn"
        type="button"
        bind:this={closeBtn}
        onclick={closeDetail}
        aria-label="Fermer"
      >
        <X size={16} strokeWidth={1.8} />
      </button>
    </header>

    <div class="drawer-body scrollable">
      {#if detailRow.result && detailRow.result.hypotheses.length > 0}
        <div class="sec-h">Hypothèses Monte-Carlo</div>
        <ul class="hyp-list">
          {#each detailRow.result.hypotheses as h (h.key)}
            <li class="hyp-item">
              <span class="hyp-key mono">{h.key}</span>
              <span class="hyp-val mono">{String(h.value)}</span>
              <span class="hyp-src">{h.source}</span>
            </li>
          {/each}
        </ul>
      {:else}
        <p class="empty">Pas d'hypothèse exportée par le moteur pour ce modèle.</p>
      {/if}

      <div class="sec-h">Sources du preset</div>
      {#if detailRow.model.sources.length > 0}
        <ul class="src-list">
          {#each detailRow.model.sources as s (s)}
            <li>
              {#if /^https?:\/\//i.test(s)}
                <a href={s} target="_blank" rel="noopener noreferrer" title={s}>
                  <span class="src-text">{s.replace(/^https?:\/\//i, '').slice(0, 56)}</span>
                  <ArrowUpRight size={11} strokeWidth={2} />
                </a>
              {:else}
                <span class="src-plain">{s}</span>
              {/if}
            </li>
          {/each}
        </ul>
      {:else}
        <p class="empty">Aucune source listée pour ce preset.</p>
      {/if}

      {#if detailRow.result}
        <div class="sec-h">Lien ledger</div>
        <a class="drawer-ledger" href={`/journal?focus=${detailRow.result.audit_id}`}>
          Voir l'entrée #{detailRow.result.audit_id} dans le journal d'audit
          <ArrowUpRight size={12} strokeWidth={2} />
        </a>
      {/if}
    </div>
  </div>
{/if}

<svelte:window onkeydown={handleEsc} />

<style>
  .canvas-inner {
    max-width: 1240px;
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
    max-width: 740px;
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

  /* Banner */
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

  /* Sélection */
  .selector {
    background: rgba(255, 255, 255, 0.015);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 18px 20px;
    margin-bottom: 16px;
  }
  .selector-head {
    display: flex;
    align-items: baseline;
    gap: 12px;
    margin-bottom: 14px;
  }
  .selector-head h2 {
    font: 500 13px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--ivory-3);
    margin: 0;
    flex: 1;
  }
  .counter {
    font: 500 12px/1 var(--font-mono);
    color: var(--lime);
  }
  .counter-hint {
    color: var(--ivory-4);
  }
  .btn-ghost-mini {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 26px;
    padding: 0 10px;
    background: transparent;
    color: var(--ivory-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    font: 500 11px/1 var(--font-ui);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
  }
  .btn-ghost-mini:hover {
    border-color: var(--border-hi);
    color: var(--ivory);
  }

  .selector-skel {
    padding: 30px;
    text-align: center;
    color: var(--ivory-3);
    font: 400 13px/1 var(--font-mono);
  }
  .model-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .model-chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 30px;
    padding: 0 12px;
    background: var(--surface);
    color: var(--ivory-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-pill);
    font: 500 12px/1 var(--font-ui);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
  }
  .model-chip:hover {
    border-color: var(--border-hi);
    color: var(--ivory);
  }
  .model-chip.on {
    background: var(--lime-soft);
    border-color: rgba(197, 240, 74, 0.4);
    color: var(--lime);
  }
  .model-chip.maxed {
    opacity: 0.35;
    cursor: not-allowed;
  }
  .chip-name {
    font-weight: 500;
  }
  .chip-prov {
    font-size: 10px;
    color: var(--ivory-4);
  }
  .model-chip.on .chip-prov {
    color: rgba(197, 240, 74, 0.7);
  }

  /* Prompt + paramètres communs */
  .params {
    display: flex;
    flex-direction: column;
    gap: 14px;
    margin-bottom: 24px;
    padding: 16px 20px;
    background: rgba(255, 255, 255, 0.015);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
  }
  .prompt-block {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .prompt-block label {
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--ivory-3);
  }
  .prompt-block textarea {
    width: 100%;
    min-height: 60px;
    padding: 10px 12px;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--ivory);
    font: 400 14px/1.5 var(--font-ui);
    resize: vertical;
    transition: border-color var(--dur-base) var(--ease);
  }
  .prompt-block textarea:focus {
    outline: none;
    border-color: rgba(197, 240, 74, 0.4);
  }
  .prompt-block textarea::placeholder {
    color: var(--ivory-4);
  }
  .prompt-meta {
    display: flex;
    gap: 14px;
    align-items: center;
    flex-wrap: wrap;
    font: 500 11px/1.4 var(--font-mono);
    color: var(--ivory-3);
  }
  .prompt-meta b {
    color: var(--ivory);
    font-weight: 600;
  }
  .prompt-meta .hint {
    color: var(--ivory-4);
    font-weight: 400;
  }

  .param-row {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    gap: 14px;
    padding-top: 4px;
    border-top: 1px dashed var(--border);
  }
  .param-field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .param-field label {
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--ivory-3);
  }
  .param-field input {
    width: 120px;
    height: 36px;
    padding: 0 12px;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--ivory);
    font: 500 14px/1 var(--font-mono);
    text-align: right;
    transition: border-color var(--dur-base) var(--ease);
  }
  .param-field input:focus {
    outline: none;
    border-color: rgba(197, 240, 74, 0.4);
  }
  .param-field input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .param-spacer {
    flex: 1;
  }

  .btn-primary {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    height: 42px;
    padding: 0 20px;
    background: var(--lime);
    color: var(--ink);
    border: 1px solid var(--lime);
    border-radius: var(--radius-md);
    font: 600 14px/1 var(--font-ui);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
    box-shadow:
      0 0 0 0 var(--lime-glow),
      0 4px 16px -6px rgba(197, 240, 74, 0.5);
  }
  .btn-primary:hover:not(:disabled) {
    transform: translateY(-1px);
    box-shadow:
      0 0 0 4px rgba(197, 240, 74, 0.15),
      0 8px 24px -6px rgba(197, 240, 74, 0.6);
  }
  .btn-primary:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .btn-primary :global(svg) {
    animation: spin 1.6s linear infinite;
    animation-play-state: paused;
  }
  .btn-primary:disabled :global(svg) {
    animation-play-state: running;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  /* ─── Verdict banner ──────────────────────────────────────── */
  .verdict {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 24px;
    align-items: center;
    padding: 26px 30px;
    margin-bottom: 18px;
    background: linear-gradient(160deg, rgba(197, 240, 74, 0.08), rgba(197, 240, 74, 0.01));
    border: 1px solid rgba(197, 240, 74, 0.3);
    border-radius: var(--radius-lg);
    animation: rise 400ms var(--ease);
  }
  .verdict-eye {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.18em;
    color: var(--lime);
  }
  .verdict-eye :global(svg) {
    color: var(--lime);
  }
  .verdict-h {
    font: 400 32px/1.15 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.015em;
    margin: 8px 0 6px;
  }
  .verdict-h em {
    font-style: italic;
    color: var(--lime);
  }
  .verdict-h i {
    font-style: italic;
    color: var(--lime);
    font-weight: 500;
  }
  .verdict-h .loser {
    font-style: italic;
    color: var(--coral);
  }
  .verdict-why {
    font: 400 13px/1.5 var(--font-ui);
    color: var(--ivory-2);
    margin: 0;
    max-width: 640px;
  }
  .verdict-why b {
    color: var(--lime);
    font-weight: 500;
  }
  .verdict-delta {
    text-align: right;
  }
  .verdict-delta-v {
    display: block;
    font: 400 52px/1 var(--font-display);
    font-style: italic;
    color: var(--lime);
    letter-spacing: -0.025em;
  }
  .verdict-delta-u {
    display: block;
    margin-top: 4px;
    font: 500 9px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--ivory-3);
  }

  /* ─── Section profils détaillés ───────────────────────────── */
  .cards-section {
    margin-top: 14px;
  }
  .cards-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    margin-bottom: 14px;
    gap: 14px;
    flex-wrap: wrap;
  }
  .cards-head h2 {
    font: 400 24px/1 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    margin: 0;
  }
  .cards-hint {
    font: 400 11px/1.4 var(--font-mono);
    color: var(--ivory-4);
  }

  .cmp-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 14px;
    align-items: stretch;
  }
  /* Avec 2 modèles on force exactly 2 colonnes pour un effet head-to-head
     vraiment lisible (sinon `auto-fit` peut basculer sur 1 colonne
     selon la largeur). */
  .cmp-grid[data-count='2'] {
    grid-template-columns: 1fr 1fr;
  }

  .cmp-card {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 20px 22px;
    background: linear-gradient(160deg, rgba(255, 255, 255, 0.03), rgba(255, 255, 255, 0.005));
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    transition: all var(--dur-base) var(--ease);
    min-width: 0;
  }
  .cmp-card:hover {
    border-color: var(--border-hi);
  }
  .cmp-card.winner {
    border-color: rgba(197, 240, 74, 0.5);
    box-shadow:
      0 0 0 1px rgba(197, 240, 74, 0.1) inset,
      0 12px 32px rgba(0, 0, 0, 0.3);
  }
  .cmp-card.loser {
    border-color: rgba(240, 108, 90, 0.3);
  }

  .cmp-card-head {
    display: flex;
    align-items: center;
    gap: 12px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border);
  }
  .cmp-card-logo {
    width: 40px;
    height: 40px;
    flex-shrink: 0;
    display: grid;
    place-items: center;
    border-radius: var(--radius-md);
    background: linear-gradient(135deg, #c5f04a, #7a9a32);
    color: var(--ivory-inv);
    font: 600 14px/1 var(--font-ui);
    letter-spacing: 0.04em;
  }
  .cmp-card.loser .cmp-card-logo {
    background: linear-gradient(135deg, #f08c5a, #b56340);
    color: var(--ivory);
  }
  .cmp-card-id {
    flex: 1;
    min-width: 0;
  }
  .cmp-card-name {
    font: 400 18px/1.2 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.01em;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .cmp-card-meta {
    font: 400 11px/1.4 var(--font-mono);
    color: var(--ivory-3);
    margin-top: 2px;
  }
  .cmp-trophy {
    display: inline-grid;
    place-items: center;
    width: 26px;
    height: 26px;
    background: var(--lime);
    color: var(--ink);
    border-radius: 50%;
    flex-shrink: 0;
  }

  .cmp-card-skel {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 30px 20px;
    color: var(--ivory-3);
    font: 400 12px/1 var(--font-mono);
    justify-content: center;
  }
  .cmp-card-skel :global(svg) {
    color: var(--lime);
    animation: spin 1.4s linear infinite;
  }
  .cmp-card-err {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 14px 16px;
    background: rgba(240, 108, 90, 0.08);
    border: 1px solid rgba(240, 108, 90, 0.25);
    border-radius: var(--radius-md);
    color: var(--coral);
    font: 400 12px/1.4 var(--font-ui);
    font-style: italic;
  }

  .cmp-metric {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .cmp-metric-l {
    font: 500 9px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--ivory-3);
  }
  .cmp-metric-v {
    font: 400 22px/1.1 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.01em;
    display: flex;
    align-items: baseline;
    gap: 6px;
  }
  .cmp-metric.primary .cmp-metric-v {
    font-size: 36px;
    color: var(--lime);
  }
  .cmp-card.loser .cmp-metric.primary .cmp-metric-v {
    color: var(--coral);
  }
  .cmp-metric-u {
    font: 400 11px/1 var(--font-ui);
    font-style: normal;
    color: var(--ivory-3);
  }
  .cmp-metric-r {
    font: 400 10px/1.3 var(--font-mono);
    color: var(--ivory-4);
  }
  .cmp-bar {
    height: 4px;
    background: rgba(255, 255, 255, 0.04);
    border-radius: 2px;
    overflow: hidden;
    margin-top: 4px;
  }
  .cmp-bar-fill {
    height: 100%;
    border-radius: 2px;
    transition: width 350ms var(--ease);
  }
  .cmp-bar-fill.primary {
    background: linear-gradient(90deg, #7a9a32, #c5f04a);
  }
  .cmp-card.loser .cmp-bar-fill.primary {
    background: linear-gradient(90deg, #7a3a1a, #f08c5a);
  }
  .cmp-bar-fill.energy {
    background: linear-gradient(90deg, #3a6db5, #7eb6ff);
  }
  .cmp-bar-fill.water {
    background: linear-gradient(90deg, #2e6a78, #5ec3d6);
  }

  .cmp-card-foot {
    display: flex;
    align-items: center;
    gap: 8px;
    padding-top: 10px;
    border-top: 1px dashed var(--border);
    flex-wrap: wrap;
  }
  .spacer-flex {
    flex: 1;
  }
  .cmp-detail-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 28px;
    padding: 0 10px;
    background: transparent;
    color: var(--ivory-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-pill);
    font: 500 11px/1 var(--font-ui);
    cursor: pointer;
    text-decoration: none;
    transition: all var(--dur-base) var(--ease);
  }
  .cmp-detail-btn:hover {
    border-color: var(--border-hi);
    color: var(--ivory);
  }

  .badge.calib {
    display: inline-flex;
    align-items: center;
    height: 22px;
    padding: 0 9px;
    border-radius: var(--radius-pill);
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--ivory-2);
    font: 500 10px/1 var(--font-mono);
    letter-spacing: 0.04em;
  }
  .badge.calib[data-tone='good'] {
    background: var(--lime-soft);
    border-color: rgba(197, 240, 74, 0.3);
    color: var(--lime);
  }
  .badge.calib[data-tone='mid'] {
    background: rgba(245, 183, 105, 0.1);
    border-color: rgba(245, 183, 105, 0.3);
    color: var(--amber);
  }
  .badge.calib[data-tone='bad'] {
    background: rgba(240, 108, 90, 0.1);
    border-color: rgba(240, 108, 90, 0.3);
    color: var(--coral);
  }

  .cards-footnote {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    margin: 14px 0 0;
    font: 400 11px/1.5 var(--font-ui);
    color: var(--ivory-3);
  }
  .cards-footnote :global(svg) {
    color: var(--ivory-4);
    flex-shrink: 0;
    margin-top: 2px;
  }

  /* ─── Drawer (commun) ────────────────────────────────────── */
  .drawer-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(10, 13, 11, 0.6);
    backdrop-filter: blur(4px);
    z-index: 40;
    border: none;
    padding: 0;
    cursor: default;
    animation: fade 200ms var(--ease);
  }
  .drawer {
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
    width: 460px;
    max-width: 92vw;
    background: var(--ink-3);
    border-left: 1px solid var(--border-hi);
    z-index: 50;
    box-shadow: var(--shadow-modal);
    display: flex;
    flex-direction: column;
    animation: slide-in 280ms var(--ease);
  }
  @keyframes slide-in {
    from {
      transform: translateX(100%);
    }
    to {
      transform: translateX(0);
    }
  }
  @keyframes fade {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }
  .drawer-head {
    padding: 20px 24px 16px;
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 14px;
    border-bottom: 1px solid var(--border);
  }
  .drawer-eye {
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--ivory-3);
    margin-bottom: 6px;
  }
  .drawer-title {
    font: 400 26px/1.1 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    margin-bottom: 4px;
  }
  .drawer-sub {
    font: 400 11px/1.4 var(--font-mono);
    color: var(--ivory-3);
  }
  .drawer-body {
    padding: 20px 24px 24px;
    overflow-y: auto;
    flex: 1;
  }

  .sec-h {
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--ivory-3);
    margin: 6px 0 10px;
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .sec-h::after {
    content: '';
    flex: 1;
    height: 1px;
    background: var(--border);
  }

  /* Liste d'hypothèses */
  .hyp-list {
    list-style: none;
    padding: 0;
    margin: 0 0 16px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .hyp-item {
    display: grid;
    grid-template-columns: 90px 1fr;
    gap: 4px 10px;
    padding: 10px 12px;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }
  .hyp-key {
    font: 400 12px/1.2 var(--font-mono);
    color: var(--lime);
    grid-row: span 2;
    align-self: center;
  }
  .hyp-val {
    font: 500 12px/1.3 var(--font-mono);
    color: var(--ivory);
    overflow-wrap: anywhere;
  }
  .hyp-src {
    font: 400 10px/1.3 var(--font-ui);
    color: var(--ivory-3);
    font-style: italic;
  }

  /* Liste de sources */
  .src-list {
    list-style: none;
    padding: 0;
    margin: 0 0 16px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .src-list li a,
  .src-list li .src-plain {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 8px 12px;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    font: 400 12px/1.3 var(--font-ui);
    color: var(--ivory-2);
    text-decoration: none;
    border-bottom: 1px solid var(--border);
    transition: all var(--dur-base) var(--ease);
  }
  .src-list li a:hover {
    border-color: rgba(197, 240, 74, 0.3);
    color: var(--ivory);
  }
  .src-text {
    overflow-wrap: anywhere;
  }

  .drawer-ledger {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 10px 14px;
    background: var(--lime-soft);
    border: 1px solid rgba(197, 240, 74, 0.3);
    border-radius: var(--radius-md);
    color: var(--lime);
    font: 500 12px/1 var(--font-ui);
    text-decoration: none;
    transition: all var(--dur-base) var(--ease);
  }
  .drawer-ledger:hover {
    background: rgba(197, 240, 74, 0.18);
  }

  .empty {
    font: 400 13px/1.5 var(--font-ui);
    color: var(--ivory-3);
    font-style: italic;
    margin: 0 0 16px;
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

  @media (max-width: 960px) {
    .canvas-inner {
      padding: 24px 16px 60px;
    }
    .hero-h1 {
      font-size: 32px;
    }
    .drawer {
      width: 100%;
    }
  }
</style>
