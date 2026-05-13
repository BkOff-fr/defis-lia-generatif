<script lang="ts">
  import {
    Sparkles,
    Trophy,
    HelpCircle,
    Lock,
    Info,
    AlertTriangle,
    PlugZap,
    RotateCcw,
    Check,
    Loader
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

  // Poids du score composite (somme libre, on normalise au moment du calcul).
  let wCO2 = $state(60);
  let wEnergy = $state(25);
  let wWater = $state(15);

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
    // Valeurs brutes en unité canonique (Rust) pour normalisation.
    co2: number | undefined;
    energy: number | undefined;
    water: number | undefined;
    score: number;
    // Décomposition du score par indicateur — rend l'effet des poids
    // visible dans la matrice même quand les 3 indicateurs sont
    // corrélés (auquel cas le total bouge peu mais les contributions
    // changent fortement).
    contribCo2: number;
    contribEnergy: number;
    contribWater: number;
  };

  const rows = $derived.by<Row[]>(() => {
    const ids = [...selectedIds];
    const ms = ids
      .map((id) => models.find((m) => m.id === id))
      .filter((m): m is ModelPresetDto => !!m);

    const rawRows: Row[] = ms.map((model) => {
      const r = results[model.id];
      const err = errorById[model.id];
      return {
        model,
        result: r,
        error: err,
        co2: r ? pickInd(r, 'co2eq')?.p50 : undefined,
        energy: r ? pickInd(r, 'energy')?.p50 : undefined,
        water: r ? pickInd(r, 'water')?.p50 : undefined,
        score: 0,
        contribCo2: 0,
        contribEnergy: 0,
        contribWater: 0
      };
    });

    // Score composite : on normalise chaque indicateur sur l'intervalle
    // [min, max] de la colonne (modèles comparés ensemble) puis on
    // applique les poids. Plus c'est BAS, mieux c'est ; donc le score
    // final est `100 × (1 - moyenne pondérée des normalisés)`.
    const totalW = wCO2 + wEnergy + wWater || 1;
    type ContribKey = 'contribCo2' | 'contribEnergy' | 'contribWater';
    const cols: Array<{ key: 'co2' | 'energy' | 'water'; w: number; contrib: ContribKey }> = [
      { key: 'co2', w: wCO2 / totalW, contrib: 'contribCo2' },
      { key: 'energy', w: wEnergy / totalW, contrib: 'contribEnergy' },
      { key: 'water', w: wWater / totalW, contrib: 'contribWater' }
    ];
    for (const c of cols) {
      const values = rawRows
        .map((r) => r[c.key])
        .filter((v): v is number => typeof v === 'number' && Number.isFinite(v));
      if (values.length === 0) continue;
      const min = Math.min(...values);
      const max = Math.max(...values);
      const span = max - min || 1;
      for (const r of rawRows) {
        const v = r[c.key];
        if (typeof v !== 'number') continue;
        const norm = (v - min) / span; // [0, 1], 0 = meilleur
        const contribution = (1 - norm) * c.w * 100;
        r.score += contribution;
        r[c.contrib] = contribution;
      }
    }

    return rawRows;
  });

  /** Normalisé [0, 1] de la valeur dans sa colonne (0 = meilleur). */
  function normCell(row: Row, key: IndKey): number | undefined {
    const value = key === 'co2eq' ? row.co2 : key === 'energy' ? row.energy : row.water;
    if (typeof value !== 'number') return undefined;
    const values = rows
      .map((r) => (key === 'co2eq' ? r.co2 : key === 'energy' ? r.energy : r.water))
      .filter((v): v is number => typeof v === 'number');
    if (values.length === 0) return undefined;
    const min = Math.min(...values);
    const max = Math.max(...values);
    if (max === min) return 0;
    return (value - min) / (max - min);
  }

  function cellTone(normalized: number | undefined): string {
    if (typeof normalized !== 'number') return '';
    if (normalized < 0.34) return 'good';
    if (normalized < 0.67) return 'mid';
    return 'bad';
  }

  // ─── Formatage des cellules ─────────────────────────────────────────
  type Scale = { mult: number; unit: string };

  // Chaînes d'unités (sous-ensemble des chaînes ResultBlock — on a besoin
  // ici de CO₂eq / Wh / L uniquement pour la matrice).
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

  /** Choisit l'échelle d'affichage commune en se basant sur le P50
   *  médian des modèles comparés — toutes les cellules d'une colonne
   *  utilisent la même unité pour qu'on puisse comparer à l'œil. */
  function pickColScale(values: number[], baseUnit: string): Scale {
    const chain = UNIT_CHAINS[baseUnit];
    const fallback: Scale = { mult: 1, unit: baseUnit };
    if (!chain || chain.length === 0 || values.length === 0) return fallback;
    // Médiane pour ignorer les outliers.
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

  const co2ColScale = $derived(
    pickColScale(
      rows.map((r) => r.co2).filter((v): v is number => typeof v === 'number'),
      'gCO2eq'
    )
  );
  const energyColScale = $derived(
    pickColScale(
      rows.map((r) => r.energy).filter((v): v is number => typeof v === 'number'),
      'Wh'
    )
  );
  const waterColScale = $derived(
    pickColScale(
      rows.map((r) => r.water).filter((v): v is number => typeof v === 'number'),
      'L'
    )
  );

  function fmtNum(value: number): string {
    return new Intl.NumberFormat('fr-FR', {
      maximumSignificantDigits: 3,
      minimumSignificantDigits: 1
    }).format(value);
  }

  // ─── Classement (top 3) ─────────────────────────────────────────────
  const ranking = $derived.by(() => {
    return [...rows].filter((r) => r.result).sort((a, b) => b.score - a.score);
  });

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

    <!-- ─── Résultats : score composite + matrice ───────────────── -->
    {#if ranking.length > 0 && !comparing}
      <!-- Top 3 podium -->
      <section class="podium" aria-label="Classement composite">
        <div class="podium-head">
          <Trophy size={16} strokeWidth={1.8} />
          <h2>Classement composite</h2>
          <span class="podium-hint mono">poids ajustables ci-dessous · plus haut = mieux</span>
        </div>
        <div class="podium-list">
          {#each ranking.slice(0, 3) as r, i (r.model.id)}
            <div class="podium-row" data-rank={i + 1}>
              <span class="podium-rank">{(i + 1).toString().padStart(2, '0')}</span>
              <div class="podium-meta">
                <div class="podium-name">{r.model.display_name}</div>
                <div class="podium-prov mono">{r.model.provider}</div>
              </div>
              <div class="podium-score">
                <span class="podium-score-v">{fmtNum(r.score)}</span>
                <span class="podium-score-l">/ 100</span>
              </div>
            </div>
          {/each}
        </div>
      </section>

      <!-- Pondération du score composite -->
      <section class="weights" aria-label="Poids du score composite">
        <header class="weights-head">
          <h3>Poids du score composite</h3>
          <span class="weights-hint mono">
            Σ = {wCO2 + wEnergy + wWater}% · renormalisé automatiquement
          </span>
        </header>
        <div class="weight-row">
          <label for="w-co2">CO₂eq</label>
          <input id="w-co2" type="range" min="0" max="100" bind:value={wCO2} />
          <span class="weight-val mono">{wCO2}%</span>
        </div>
        <div class="weight-row">
          <label for="w-energy">Énergie</label>
          <input id="w-energy" type="range" min="0" max="100" bind:value={wEnergy} />
          <span class="weight-val mono">{wEnergy}%</span>
        </div>
        <div class="weight-row">
          <label for="w-water">Eau</label>
          <input id="w-water" type="range" min="0" max="100" bind:value={wWater} />
          <span class="weight-val mono">{wWater}%</span>
        </div>
      </section>
    {/if}

    {#if rows.length > 0}
      <section class="matrix" aria-label="Matrice d'indicateurs">
        <div class="table-wrap scrollable">
          <table class="cmp-table">
            <thead>
              <tr>
                <th class="th-model">Modèle</th>
                <th class="th-calib">Calibration</th>
                <th class="th-ind">
                  CO₂eq P50 <small class="mono">({co2ColScale.unit})</small>
                </th>
                <th class="th-ind">
                  Énergie P50 <small class="mono">({energyColScale.unit})</small>
                </th>
                <th class="th-ind">
                  Eau P50 <small class="mono">({waterColScale.unit})</small>
                </th>
                <th class="th-score">Score</th>
              </tr>
            </thead>
            <tbody>
              {#each rows as r (r.model.id)}
                {@const co2N = normCell(r, 'co2eq')}
                {@const eN = normCell(r, 'energy')}
                {@const wN = normCell(r, 'water')}
                <tr>
                  <td class="td-model">
                    <span class="td-model-name">{r.model.display_name}</span>
                    <span class="td-model-prov mono">{r.model.provider}</span>
                  </td>
                  <td>
                    <span class="badge calib" data-tone={calibTone(r.model.calibration)}>
                      {calibLabel[r.model.calibration]}
                    </span>
                  </td>
                  <td class="td-cell" data-tone={cellTone(co2N)}>
                    {#if comparing}<Loader size={11} strokeWidth={2} />
                    {:else if r.error}<span class="td-err" title={r.error}>échec</span>
                    {:else if typeof r.co2 === 'number'}{fmtNum(r.co2 * co2ColScale.mult)}
                    {:else}—
                    {/if}
                  </td>
                  <td class="td-cell" data-tone={cellTone(eN)}>
                    {#if comparing}<Loader size={11} strokeWidth={2} />
                    {:else if typeof r.energy === 'number'}{fmtNum(r.energy * energyColScale.mult)}
                    {:else}—
                    {/if}
                  </td>
                  <td class="td-cell" data-tone={cellTone(wN)}>
                    {#if comparing}<Loader size={11} strokeWidth={2} />
                    {:else if typeof r.water === 'number'}{fmtNum(r.water * waterColScale.mult)}
                    {:else}—
                    {/if}
                  </td>
                  <td class="td-score">
                    {#if r.result}
                      <div class="score-cell">
                        <div class="score-val mono">{fmtNum(r.score)}</div>
                        <div
                          class="score-bar"
                          role="img"
                          aria-label={`CO₂eq ${fmtNum(r.contribCo2)} · Énergie ${fmtNum(r.contribEnergy)} · Eau ${fmtNum(r.contribWater)}`}
                          title={`Décomposition · CO₂eq ${fmtNum(r.contribCo2)} + Énergie ${fmtNum(r.contribEnergy)} + Eau ${fmtNum(r.contribWater)}`}
                        >
                          <span class="seg seg-co2" style="flex: {Math.max(0.01, r.contribCo2)}"
                          ></span>
                          <span
                            class="seg seg-energy"
                            style="flex: {Math.max(0.01, r.contribEnergy)}"
                          ></span>
                          <span class="seg seg-water" style="flex: {Math.max(0.01, r.contribWater)}"
                          ></span>
                          <span class="seg seg-rest" style="flex: {Math.max(0.01, 100 - r.score)}"
                          ></span>
                        </div>
                      </div>
                    {:else}
                      <span class="mono">—</span>
                    {/if}
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>

        <p class="matrix-footnote">
          <Info size={12} strokeWidth={1.8} />
          Cellule
          <span class="legend good"></span> meilleur ·
          <span class="legend mid"></span> milieu ·
          <span class="legend bad"></span> moins bon — normalisation par colonne sur les modèles
          affichés. Barre Score :
          <span class="legend lime-bar"></span> CO₂eq ·
          <span class="legend blue-bar"></span> énergie ·
          <span class="legend cyan-bar"></span> eau. Largeur du segment = contribution réelle au total
          — c'est ce qui bouge quand vous ajustez les poids.
        </p>
      </section>
    {/if}
  {/if}
</div>

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

  /* Podium */
  .podium {
    background: linear-gradient(180deg, rgba(197, 240, 74, 0.04), rgba(255, 255, 255, 0.01));
    border: 1px solid rgba(197, 240, 74, 0.25);
    border-radius: var(--radius-md);
    padding: 18px 22px;
    margin-bottom: 16px;
    animation: rise 400ms var(--ease);
  }
  .podium-head {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 14px;
  }
  .podium-head :global(svg) {
    color: var(--lime);
  }
  .podium-head h2 {
    font: 400 22px/1 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    margin: 0;
    flex: 1;
  }
  .podium-hint {
    font-size: 11px;
    color: var(--ivory-3);
  }
  .podium-list {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: 8px;
  }
  .podium-row {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: 12px;
    padding: 12px 14px;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
  }
  .podium-row[data-rank='1'] {
    border-color: rgba(197, 240, 74, 0.4);
    background: rgba(197, 240, 74, 0.06);
  }
  .podium-rank {
    font: 400 22px/1 var(--font-display);
    font-style: italic;
    color: var(--ivory-3);
  }
  .podium-row[data-rank='1'] .podium-rank {
    color: var(--lime);
  }
  .podium-meta {
    min-width: 0;
  }
  .podium-name {
    font: 500 13px/1.2 var(--font-ui);
    color: var(--ivory);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .podium-prov {
    font: 400 10px/1.4 var(--font-mono);
    color: var(--ivory-3);
  }
  .podium-score {
    display: inline-flex;
    align-items: baseline;
    gap: 2px;
  }
  .podium-score-v {
    font: 400 22px/1 var(--font-display);
    font-style: italic;
    color: var(--lime);
  }
  .podium-score-l {
    font: 400 10px/1 var(--font-ui);
    color: var(--ivory-3);
  }

  /* Pondération */
  .weights {
    background: rgba(255, 255, 255, 0.015);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 16px 20px;
    margin-bottom: 16px;
  }
  .weights-head {
    display: flex;
    align-items: baseline;
    gap: 12px;
    margin-bottom: 10px;
  }
  .weights-head h3 {
    font: 500 11px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--ivory-3);
    margin: 0;
    flex: 1;
  }
  .weights-hint {
    font-size: 11px;
    color: var(--ivory-4);
  }
  .weight-row {
    display: grid;
    grid-template-columns: 80px 1fr 50px;
    align-items: center;
    gap: 14px;
    padding: 6px 0;
  }
  .weight-row label {
    font: 500 12px/1 var(--font-ui);
    color: var(--ivory);
  }
  .weight-row input[type='range'] {
    width: 100%;
    accent-color: var(--lime);
  }
  .weight-val {
    font: 500 12px/1 var(--font-mono);
    color: var(--lime);
    text-align: right;
  }

  /* Matrice */
  .matrix {
    margin-top: 8px;
  }
  .table-wrap {
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: rgba(255, 255, 255, 0.015);
    overflow: auto;
  }
  .cmp-table {
    width: 100%;
    border-collapse: collapse;
    table-layout: fixed;
    font: 400 13px/1 var(--font-ui);
  }
  .cmp-table thead th {
    text-align: left;
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--ivory-3);
    padding: 12px 14px;
    background: var(--ink-2);
    border-bottom: 1px solid var(--border);
  }
  .cmp-table thead th small {
    color: var(--ivory-4);
    text-transform: none;
    letter-spacing: 0;
    font-size: 10px;
    margin-left: 4px;
  }
  .th-model {
    width: 24%;
  }
  .th-calib {
    width: 110px;
  }
  .th-ind {
    width: auto;
    text-align: right !important;
  }
  .th-score {
    width: 160px;
    text-align: right !important;
  }
  .cmp-table tbody td {
    padding: 12px 14px;
    border-bottom: 1px solid var(--border);
    vertical-align: middle;
  }
  .cmp-table tbody tr:last-child td {
    border-bottom: none;
  }
  .td-model {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .td-model-name {
    font: 500 13px/1.2 var(--font-ui);
    color: var(--ivory);
  }
  .td-model-prov {
    font: 400 10px/1 var(--font-mono);
    color: var(--ivory-3);
  }
  .badge.calib {
    display: inline-flex;
    padding: 3px 8px;
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

  .td-cell {
    text-align: right;
    font: 500 13px/1 var(--font-mono);
    color: var(--ivory);
    position: relative;
  }
  .td-cell[data-tone='good'] {
    color: var(--lime);
    background: linear-gradient(90deg, transparent 0%, rgba(197, 240, 74, 0.08) 100%);
  }
  .td-cell[data-tone='mid'] {
    color: var(--amber);
    background: linear-gradient(90deg, transparent 0%, rgba(245, 183, 105, 0.08) 100%);
  }
  .td-cell[data-tone='bad'] {
    color: var(--coral);
    background: linear-gradient(90deg, transparent 0%, rgba(240, 108, 90, 0.08) 100%);
  }
  .td-cell :global(svg) {
    color: var(--ivory-3);
    animation: spin 1.4s linear infinite;
  }
  .td-err {
    color: var(--coral);
    font-style: italic;
    font-size: 12px;
  }
  .td-score {
    text-align: right;
  }
  .score-cell {
    display: flex;
    flex-direction: column;
    gap: 5px;
    align-items: flex-end;
  }
  .score-val {
    font: 600 14px/1 var(--font-mono);
    color: var(--lime);
  }
  /* Barre segmentée : chaque segment est dimensionné par sa contribution
     au score. Quand l'utilisateur déplace un slider de poids, les widths
     changent instantanément — c'est le signal visuel principal de
     l'effet des poids, même si le total bouge peu (indicateurs corrélés). */
  .score-bar {
    display: flex;
    width: 100%;
    height: 8px;
    border-radius: 4px;
    overflow: hidden;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid var(--border);
  }
  .score-bar .seg {
    display: block;
    height: 100%;
    transition:
      flex 250ms var(--ease),
      background 200ms var(--ease);
  }
  .score-bar .seg-co2 {
    background: var(--lime);
  }
  .score-bar .seg-energy {
    background: var(--blue);
  }
  .score-bar .seg-water {
    background: #5ec3d6;
  }
  .score-bar .seg-rest {
    background: transparent;
  }

  .matrix-footnote {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 4px 8px;
    margin: 12px 0 0;
    font: 400 11px/1.6 var(--font-ui);
    color: var(--ivory-3);
  }
  .matrix-footnote :global(svg) {
    color: var(--ivory-4);
    flex-shrink: 0;
  }
  .legend {
    display: inline-block;
    width: 10px;
    height: 10px;
    border-radius: 3px;
    margin: 0 4px 0 8px;
    vertical-align: middle;
  }
  .legend.good {
    background: var(--lime);
  }
  .legend.mid {
    background: var(--amber);
  }
  .legend.bad {
    background: var(--coral);
  }
  .legend.lime-bar {
    background: var(--lime);
  }
  .legend.blue-bar {
    background: var(--blue);
  }
  .legend.cyan-bar {
    background: #5ec3d6;
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
    .weight-row {
      grid-template-columns: 70px 1fr 44px;
      gap: 10px;
    }
  }
</style>
