<script lang="ts">
  import {
    Layers,
    Search,
    X,
    HelpCircle,
    ArrowUpDown,
    ArrowDown,
    ArrowUp,
    ArrowUpRight,
    Sparkles,
    Info,
    AlertTriangle,
    PlugZap,
    Lock,
    RotateCcw,
    Filter
  } from '@lucide/svelte';
  import {
    isTauriContext,
    listModels,
    SobriaIpcError,
    type Calibration,
    type IpcErrorCode,
    type ModelPresetDto,
    type Openness
  } from '$lib/api';
  import { tick } from 'svelte';

  // ─── State ───────────────────────────────────────────────────────────
  let models = $state<ModelPresetDto[]>([]);
  let bootstrapping = $state(true);
  let loadError = $state<{ code: IpcErrorCode; message: string } | null>(null);

  let query = $state('');
  let providerFilter = $state<Set<string>>(new Set());
  let calibFilter = $state<Set<Calibration>>(new Set());
  let opennessFilter = $state<Set<Openness>>(new Set());

  type SortKey = 'display_name' | 'provider' | 'family' | 'approx_params_billions' | 'calibration';
  let sortKey = $state<SortKey>('provider');
  let sortDir = $state<'asc' | 'desc'>('asc');

  let selected = $state<ModelPresetDto | null>(null);

  const tauriAvailable = $derived(isTauriContext());

  // ─── Bootstrap ─────────────────────────────────────────────────────
  $effect(() => {
    void (async () => {
      if (!tauriAvailable) {
        bootstrapping = false;
        loadError = {
          code: 'tauri_unavailable',
          message:
            "L'application doit être lancée via `cargo run -p sobria-app`. Le référentiel modèles n'est pas accessible dans un navigateur seul."
        };
        return;
      }
      try {
        models = await listModels();
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

  // ─── Dérivés : providers disponibles + filtre + tri ────────────────
  const providers = $derived.by(() => {
    const set = new Set<string>();
    for (const m of models) set.add(m.provider);
    return [...set].sort();
  });

  const calibLabel: Record<Calibration, string> = {
    validated: 'Validé',
    indicative: 'Indicatif',
    extrapolated: 'Extrapolé'
  };
  const opennessLabel: Record<Openness, string> = {
    open: 'Open',
    open_weights: 'Open weights',
    closed: 'Closed'
  };

  function matchesQuery(m: ModelPresetDto, q: string): boolean {
    if (q.length === 0) return true;
    const norm = q.toLocaleLowerCase('fr');
    return (
      m.id.toLocaleLowerCase('fr').includes(norm) ||
      m.display_name.toLocaleLowerCase('fr').includes(norm) ||
      m.provider.toLocaleLowerCase('fr').includes(norm) ||
      m.family.toLocaleLowerCase('fr').includes(norm)
    );
  }

  const filtered = $derived(
    models.filter(
      (m) =>
        matchesQuery(m, query) &&
        (providerFilter.size === 0 || providerFilter.has(m.provider)) &&
        (calibFilter.size === 0 || calibFilter.has(m.calibration)) &&
        (opennessFilter.size === 0 || opennessFilter.has(m.openness))
    )
  );

  const sorted = $derived.by(() => {
    const arr = [...filtered];
    const dir = sortDir === 'asc' ? 1 : -1;
    arr.sort((a, b) => {
      switch (sortKey) {
        case 'approx_params_billions':
          return (a.approx_params_billions - b.approx_params_billions) * dir;
        case 'display_name':
        case 'provider':
        case 'family':
        case 'calibration':
          return a[sortKey].localeCompare(b[sortKey], 'fr') * dir;
      }
    });
    return arr;
  });

  const filtersActive = $derived(
    query.length > 0 || providerFilter.size > 0 || calibFilter.size > 0 || opennessFilter.size > 0
  );

  // ─── Actions filtres ───────────────────────────────────────────────
  function toggleProvider(p: string) {
    const next = new Set(providerFilter);
    if (next.has(p)) next.delete(p);
    else next.add(p);
    providerFilter = next;
  }
  function toggleCalib(c: Calibration) {
    const next = new Set(calibFilter);
    if (next.has(c)) next.delete(c);
    else next.add(c);
    calibFilter = next;
  }
  function toggleOpenness(o: Openness) {
    const next = new Set(opennessFilter);
    if (next.has(o)) next.delete(o);
    else next.add(o);
    opennessFilter = next;
  }
  function resetFilters() {
    query = '';
    providerFilter = new Set();
    calibFilter = new Set();
    opennessFilter = new Set();
  }

  function toggleSort(k: SortKey) {
    if (sortKey === k) {
      sortDir = sortDir === 'asc' ? 'desc' : 'asc';
    } else {
      sortKey = k;
      sortDir = 'asc';
    }
  }

  // ─── Drawer ──────────────────────────────────────────────────────
  function openModel(m: ModelPresetDto) {
    selected = m;
  }
  function closeDrawer() {
    selected = null;
  }
  function handleEscape(e: KeyboardEvent) {
    if (e.key === 'Escape' && selected) closeDrawer();
  }

  // Focus initial du drawer (a11y) — on shift le focus sur le bouton fermer
  // dès qu'il s'ouvre.
  let closeBtn: HTMLButtonElement | undefined = $state();
  $effect(() => {
    if (selected) {
      void tick().then(() => closeBtn?.focus());
    }
  });

  // ─── Helpers ─────────────────────────────────────────────────────
  function fmtBn(value: number): string {
    if (!Number.isFinite(value)) return '—';
    return `${new Intl.NumberFormat('fr-FR', {
      maximumSignificantDigits: 3
    }).format(value)} B`;
  }

  function isUrl(s: string): boolean {
    return /^https?:\/\//i.test(s);
  }
  function prettySource(s: string, max = 56): string {
    const stripped = s.replace(/^https?:\/\//i, '');
    return stripped.length > max ? stripped.slice(0, max - 1) + '…' : stripped;
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

  const SKEL_ROWS = [0, 1, 2, 3, 4, 5, 6, 7] as const;
</script>

<svelte:window onkeydown={handleEscape} />

<svelte:head>
  <title>Sobr.ia · Workbench</title>
</svelte:head>

<div class="canvas-inner">
  <!-- ─── TopBar ─────────────────────────────────────────────── -->
  <div class="topbar">
    <nav class="breadcrumb" aria-label="Fil d'Ariane">
      Atelier <span class="sep">/</span>
      <span class="current">Workbench</span>
    </nav>
    <div class="spacer"></div>
    <span class="local-pill">
      <Lock size={12} strokeWidth={1.8} />
      Référentiel 100 % local
    </span>
    <a class="icon-btn" href="/methodo" aria-label="Méthodologie">
      <HelpCircle size={16} strokeWidth={1.6} />
    </a>
  </div>

  <!-- ─── Hero ───────────────────────────────────────────────── -->
  <section class="hero">
    <div class="hero-eyebrow">
      <span class="pulse" aria-hidden="true"></span>
      Module M3 · référentiel des modèles
    </div>
    <h1 class="hero-h1">
      Tous les modèles que <em>Sobr.ia</em> sait estimer.
    </h1>
    <p class="hero-sub">
      Explorez les presets distributionnels disponibles. Chaque ligne porte son statut de
      calibration et ses sources : cliquez pour ouvrir la fiche, ou lancez directement une
      estimation depuis le drawer.
    </p>
  </section>

  <!-- ─── Bannière erreur ────────────────────────────────────── -->
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

  <!-- ─── Filtres ───────────────────────────────────────────── -->
  {#if tauriAvailable}
    <div class="filters">
      <div class="search">
        <Search size={14} strokeWidth={1.8} />
        <input
          type="search"
          placeholder="Rechercher un modèle, un provider, une famille…"
          bind:value={query}
          aria-label="Recherche dans le référentiel"
        />
        {#if query.length > 0}
          <button
            class="search-clear"
            type="button"
            onclick={() => (query = '')}
            aria-label="Effacer"
          >
            <X size={12} strokeWidth={2} />
          </button>
        {/if}
      </div>

      <div class="chip-group" aria-label="Filtre providers">
        <span class="chip-group-label">
          <Filter size={11} strokeWidth={1.8} /> Provider
        </span>
        {#each providers as p (p)}
          <button
            type="button"
            class="chip"
            class:on={providerFilter.has(p)}
            onclick={() => toggleProvider(p)}
          >
            {p}
          </button>
        {/each}
      </div>

      <div class="chip-group" aria-label="Filtre calibration">
        <span class="chip-group-label">Statut</span>
        {#each ['validated', 'indicative', 'extrapolated'] as Calibration[] as c (c)}
          <button
            type="button"
            class="chip"
            data-tone={c}
            class:on={calibFilter.has(c)}
            onclick={() => toggleCalib(c)}
          >
            {calibLabel[c]}
          </button>
        {/each}
      </div>

      <div class="chip-group" aria-label="Filtre ouverture">
        <span class="chip-group-label">Ouverture</span>
        {#each ['open', 'open_weights', 'closed'] as Openness[] as o (o)}
          <button
            type="button"
            class="chip"
            class:on={opennessFilter.has(o)}
            onclick={() => toggleOpenness(o)}
          >
            {opennessLabel[o]}
          </button>
        {/each}
      </div>

      <div class="filter-actions">
        {#if filtersActive}
          <button class="btn-ghost-mini" type="button" onclick={resetFilters}>
            <RotateCcw size={13} strokeWidth={1.8} /> Réinitialiser
          </button>
        {/if}
        <span class="count mono">
          {sorted.length} / {models.length} modèles
        </span>
      </div>
    </div>

    <!-- ─── Table ──────────────────────────────────────────────── -->
    <div class="table-wrap scrollable">
      <table class="models-table" aria-label="Référentiel des modèles">
        <thead>
          <tr>
            <th
              class="th-name"
              aria-sort={sortKey === 'display_name'
                ? sortDir === 'asc'
                  ? 'ascending'
                  : 'descending'
                : 'none'}
            >
              <button type="button" class="th-sort" onclick={() => toggleSort('display_name')}>
                Modèle
                {#if sortKey === 'display_name'}
                  {#if sortDir === 'asc'}<ArrowUp size={11} strokeWidth={2} />{:else}<ArrowDown
                      size={11}
                      strokeWidth={2}
                    />{/if}
                {:else}<ArrowUpDown size={11} strokeWidth={2} />{/if}
              </button>
            </th>
            <th
              class="th-prov"
              aria-sort={sortKey === 'provider'
                ? sortDir === 'asc'
                  ? 'ascending'
                  : 'descending'
                : 'none'}
            >
              <button type="button" class="th-sort" onclick={() => toggleSort('provider')}>
                Provider
                {#if sortKey === 'provider'}
                  {#if sortDir === 'asc'}<ArrowUp size={11} strokeWidth={2} />{:else}<ArrowDown
                      size={11}
                      strokeWidth={2}
                    />{/if}
                {:else}<ArrowUpDown size={11} strokeWidth={2} />{/if}
              </button>
            </th>
            <th class="th-family">Famille</th>
            <th
              class="th-params"
              aria-sort={sortKey === 'approx_params_billions'
                ? sortDir === 'asc'
                  ? 'ascending'
                  : 'descending'
                : 'none'}
            >
              <button
                type="button"
                class="th-sort right"
                onclick={() => toggleSort('approx_params_billions')}
              >
                Paramètres
                {#if sortKey === 'approx_params_billions'}
                  {#if sortDir === 'asc'}<ArrowUp size={11} strokeWidth={2} />{:else}<ArrowDown
                      size={11}
                      strokeWidth={2}
                    />{/if}
                {:else}<ArrowUpDown size={11} strokeWidth={2} />{/if}
              </button>
            </th>
            <th class="th-openness">Ouverture</th>
            <th
              class="th-calib"
              aria-sort={sortKey === 'calibration'
                ? sortDir === 'asc'
                  ? 'ascending'
                  : 'descending'
                : 'none'}
            >
              <button type="button" class="th-sort" onclick={() => toggleSort('calibration')}>
                Calibration
                {#if sortKey === 'calibration'}
                  {#if sortDir === 'asc'}<ArrowUp size={11} strokeWidth={2} />{:else}<ArrowDown
                      size={11}
                      strokeWidth={2}
                    />{/if}
                {:else}<ArrowUpDown size={11} strokeWidth={2} />{/if}
              </button>
            </th>
            <th class="th-sources">Sources</th>
          </tr>
        </thead>
        <tbody>
          {#if bootstrapping}
            {#each SKEL_ROWS as i (i)}
              <tr class="row-skel">
                <td colspan="7"><span class="skel-bar"></span></td>
              </tr>
            {/each}
          {:else if sorted.length === 0}
            <tr class="row-empty">
              <td colspan="7">
                Aucun modèle ne correspond à vos filtres.
                {#if filtersActive}<button class="link-btn" onclick={resetFilters}
                    >Réinitialiser</button
                  >{/if}
              </td>
            </tr>
          {:else}
            {#each sorted as m (m.id)}
              <tr
                class="row"
                class:active={selected?.id === m.id}
                onclick={() => openModel(m)}
                tabindex="0"
                onkeydown={(ev) => {
                  if (ev.key === 'Enter' || ev.key === ' ') {
                    ev.preventDefault();
                    openModel(m);
                  }
                }}
              >
                <td class="td-name">
                  <span class="td-name-display">{m.display_name}</span>
                  <span class="td-name-id mono">{m.id}</span>
                </td>
                <td class="td-prov">{m.provider}</td>
                <td class="td-family mono">{m.family}</td>
                <td class="td-params mono">{fmtBn(m.approx_params_billions)}</td>
                <td class="td-openness">
                  <span class="badge openness {m.openness}">{opennessLabel[m.openness]}</span>
                </td>
                <td class="td-calib">
                  <span class="badge calib" data-tone={m.calibration}>
                    {calibLabel[m.calibration]}
                  </span>
                </td>
                <td class="td-sources mono">{m.sources.length}</td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<!-- ─── Drawer ────────────────────────────────────────────────── -->
{#if selected}
  <button class="drawer-backdrop" type="button" aria-label="Fermer le détail" onclick={closeDrawer}
  ></button>
  <div class="drawer" role="dialog" aria-modal="true" aria-labelledby="drawer-title">
    <header class="drawer-head">
      <div>
        <div class="drawer-eye">Modèle · preset distributionnel</div>
        <div id="drawer-title" class="drawer-title">{selected.display_name}</div>
        <div class="drawer-sub mono">{selected.id}</div>
      </div>
      <button
        class="icon-btn"
        type="button"
        bind:this={closeBtn}
        onclick={closeDrawer}
        aria-label="Fermer"
      >
        <X size={16} strokeWidth={1.8} />
      </button>
    </header>

    <div class="drawer-body scrollable">
      <dl class="drawer-grid">
        <dt>Provider</dt>
        <dd>{selected.provider}</dd>

        <dt>Famille</dt>
        <dd class="mono">{selected.family}</dd>

        <dt>Paramètres</dt>
        <dd class="mono">{fmtBn(selected.approx_params_billions)}</dd>

        <dt>Ouverture</dt>
        <dd>
          <span class="badge openness {selected.openness}">
            {opennessLabel[selected.openness]}
          </span>
        </dd>

        <dt>Calibration</dt>
        <dd>
          <span class="badge calib" data-tone={selected.calibration}>
            {calibLabel[selected.calibration]}
          </span>
          {#if selected.calibration === 'extrapolated'}
            <p class="status-note">
              Taille du modèle estimée publiquement (provider ne publie pas N<sub>params</sub>).
              Marge d'incertitude élargie.
            </p>
          {:else if selected.calibration === 'indicative'}
            <p class="status-note">
              Caractéristiques extrapolées depuis HF AI Energy Score pour modèle ouvert de même
              famille. Statut « validated » à venir après reproduction paper-based.
            </p>
          {:else}
            <p class="status-note">Reproduit à ±15 % vs paper de référence.</p>
          {/if}
        </dd>
      </dl>

      <div class="sec-h">Sources ({selected.sources.length})</div>
      {#if selected.sources.length === 0}
        <p class="empty">Aucune source listée pour ce preset.</p>
      {:else}
        <ul class="sources">
          {#each selected.sources as s (s)}
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

      <div class="drawer-actions">
        <a class="btn-primary" href={`/?model=${encodeURIComponent(selected.id)}`}>
          <Sparkles size={14} strokeWidth={2} />
          Estimer avec ce modèle
        </a>
        <a class="btn-ghost" href="/methodo#methode">
          <Layers size={14} strokeWidth={1.8} />
          Méthodologie
        </a>
      </div>
    </div>
  </div>
{/if}

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
    max-width: 620px;
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

  /* Filtres */
  .filters {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px 14px;
    margin-bottom: 18px;
    padding: 14px 16px;
    background: rgba(255, 255, 255, 0.015);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
  }
  .search {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1 1 280px;
    min-width: 240px;
    padding: 0 12px;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    height: 34px;
    transition: border-color var(--dur-base) var(--ease);
  }
  .search:focus-within {
    border-color: rgba(197, 240, 74, 0.4);
  }
  .search :global(svg) {
    color: var(--ivory-3);
    flex-shrink: 0;
  }
  .search input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    color: var(--ivory);
    font: 400 13px/1 var(--font-ui);
  }
  .search input::placeholder {
    color: var(--ivory-4);
  }
  .search-clear {
    background: none;
    border: none;
    color: var(--ivory-3);
    cursor: pointer;
    padding: 2px;
    display: inline-flex;
    transition: color var(--dur-base) var(--ease);
  }
  .search-clear:hover {
    color: var(--ivory);
  }

  .chip-group {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    flex-wrap: wrap;
  }
  .chip-group-label {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--ivory-3);
    margin-right: 4px;
  }
  .chip {
    display: inline-flex;
    align-items: center;
    height: 26px;
    padding: 0 10px;
    background: var(--surface);
    color: var(--ivory-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-pill);
    font: 500 11px/1 var(--font-ui);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
  }
  .chip:hover {
    border-color: var(--border-hi);
    color: var(--ivory);
  }
  .chip.on {
    background: var(--lime-soft);
    border-color: rgba(197, 240, 74, 0.4);
    color: var(--lime);
  }
  .chip[data-tone='indicative'].on {
    background: rgba(245, 183, 105, 0.12);
    border-color: rgba(245, 183, 105, 0.4);
    color: var(--amber);
  }
  .chip[data-tone='extrapolated'].on {
    background: rgba(240, 108, 90, 0.12);
    border-color: rgba(240, 108, 90, 0.4);
    color: var(--coral);
  }

  .filter-actions {
    display: inline-flex;
    align-items: center;
    gap: 12px;
    margin-left: auto;
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
  .count {
    font: 400 11px/1 var(--font-mono);
    color: var(--ivory-3);
    letter-spacing: 0.04em;
  }

  /* Table */
  .table-wrap {
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: rgba(255, 255, 255, 0.015);
    overflow: auto;
    max-height: 65vh;
  }
  .models-table {
    width: 100%;
    border-collapse: collapse;
    table-layout: fixed;
    font: 400 13px/1 var(--font-ui);
  }
  .models-table thead th {
    text-align: left;
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--ivory-3);
    padding: 12px 16px;
    background: var(--ink-2);
    border-bottom: 1px solid var(--border);
    position: sticky;
    top: 0;
    z-index: 1;
  }
  .th-name {
    width: 26%;
  }
  .th-prov {
    width: 14%;
  }
  .th-family {
    width: 14%;
  }
  .th-params {
    width: 100px;
  }
  .th-openness {
    width: 130px;
  }
  .th-calib {
    width: 120px;
  }
  .th-sources {
    width: 80px;
    text-align: center;
  }

  .th-sort {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    background: none;
    border: none;
    color: inherit;
    font: inherit;
    cursor: pointer;
    padding: 0;
    letter-spacing: inherit;
    text-transform: inherit;
    transition: color var(--dur-base) var(--ease);
  }
  .th-sort:hover {
    color: var(--ivory);
  }
  .th-sort.right {
    margin-left: auto;
  }
  .th-sort :global(svg) {
    color: var(--ivory-4);
  }

  .row {
    cursor: pointer;
    transition: background var(--dur-fast) var(--ease);
  }
  .row td {
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
    color: var(--ivory-2);
    vertical-align: middle;
  }
  .row:hover td {
    background: rgba(255, 255, 255, 0.03);
    color: var(--ivory);
  }
  .row.active td {
    background: rgba(197, 240, 74, 0.05);
  }
  .td-name {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .td-name-display {
    font: 500 13px/1.2 var(--font-ui);
    color: var(--ivory);
  }
  .td-name-id {
    font: 400 10px/1.2 var(--font-mono);
    color: var(--ivory-3);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .td-prov {
    font: 500 12px/1 var(--font-ui);
    color: var(--ivory);
  }
  .td-family,
  .td-params,
  .td-sources {
    font: 400 12px/1 var(--font-mono);
    color: var(--ivory-2);
  }
  .td-params {
    text-align: right;
  }
  .td-sources {
    text-align: center;
    color: var(--lime);
  }

  .badge {
    display: inline-flex;
    align-items: center;
    padding: 3px 8px;
    border-radius: var(--radius-pill);
    font: 500 10px/1 var(--font-mono);
    letter-spacing: 0.04em;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--ivory-2);
  }
  .badge.openness.open {
    background: rgba(197, 240, 74, 0.08);
    border-color: rgba(197, 240, 74, 0.3);
    color: var(--lime);
  }
  .badge.openness.open_weights {
    background: rgba(126, 182, 255, 0.08);
    border-color: rgba(126, 182, 255, 0.3);
    color: var(--blue);
  }
  .badge.openness.closed {
    background: rgba(255, 255, 255, 0.03);
    border-color: var(--border-hi);
    color: var(--ivory-3);
  }
  .badge.calib[data-tone='validated'] {
    background: var(--lime-soft);
    border-color: rgba(197, 240, 74, 0.3);
    color: var(--lime);
  }
  .badge.calib[data-tone='indicative'] {
    background: rgba(245, 183, 105, 0.1);
    border-color: rgba(245, 183, 105, 0.3);
    color: var(--amber);
  }
  .badge.calib[data-tone='extrapolated'] {
    background: rgba(240, 108, 90, 0.1);
    border-color: rgba(240, 108, 90, 0.3);
    color: var(--coral);
  }

  .row-skel td {
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
  }
  .skel-bar {
    display: block;
    height: 14px;
    background: linear-gradient(
      90deg,
      rgba(255, 255, 255, 0.02),
      rgba(255, 255, 255, 0.06),
      rgba(255, 255, 255, 0.02)
    );
    background-size: 200% 100%;
    border-radius: 4px;
    animation: shimmer 1.4s linear infinite;
  }
  @keyframes shimmer {
    from {
      background-position: 200% 0;
    }
    to {
      background-position: -200% 0;
    }
  }
  .row-empty td {
    padding: 30px 16px;
    text-align: center;
    color: var(--ivory-3);
    font: 400 13px/1.5 var(--font-ui);
  }
  .link-btn {
    background: none;
    border: none;
    color: var(--lime);
    cursor: pointer;
    border-bottom: 1px dashed rgba(197, 240, 74, 0.4);
    font: inherit;
    padding: 0;
    margin-left: 8px;
  }

  /* Drawer */
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
    width: 440px;
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
    gap: 16px;
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
  .drawer-grid {
    display: grid;
    grid-template-columns: 130px 1fr;
    gap: 12px 16px;
    margin: 0 0 20px;
  }
  .drawer-grid dt {
    font: 500 10px/1.2 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--ivory-3);
    padding-top: 4px;
  }
  .drawer-grid dd {
    font: 400 13px/1.4 var(--font-ui);
    color: var(--ivory);
    margin: 0;
    min-width: 0;
  }
  .drawer-grid dd.mono {
    font-family: var(--font-mono);
    color: var(--ivory-2);
  }
  .status-note {
    margin: 6px 0 0;
    font: 400 11px/1.5 var(--font-ui);
    color: var(--ivory-3);
    font-style: italic;
  }
  .sec-h {
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--ivory-3);
    margin: 18px 0 10px;
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
  .sources {
    list-style: none;
    padding: 0;
    margin: 0 0 16px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .sources li a,
  .sources li .src-plain {
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
  .empty {
    font: 400 13px/1.5 var(--font-ui);
    color: var(--ivory-3);
    font-style: italic;
    margin: 0 0 16px;
  }

  .drawer-actions {
    display: flex;
    gap: 8px;
    margin-top: 18px;
    flex-wrap: wrap;
  }
  .btn-primary,
  .btn-ghost {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    height: 38px;
    padding: 0 16px;
    border-radius: var(--radius-md);
    font: 500 13px/1 var(--font-ui);
    text-decoration: none;
    border-bottom: none;
    transition: all var(--dur-base) var(--ease);
  }
  .btn-primary {
    background: var(--lime);
    color: var(--ink);
    border: 1px solid var(--lime);
    font-weight: 600;
    box-shadow:
      0 0 0 0 var(--lime-glow),
      0 4px 16px -6px rgba(197, 240, 74, 0.5);
  }
  .btn-primary:hover {
    transform: translateY(-1px);
    box-shadow:
      0 0 0 4px rgba(197, 240, 74, 0.15),
      0 8px 24px -6px rgba(197, 240, 74, 0.6);
  }
  .btn-ghost {
    background: transparent;
    color: var(--ivory-2);
    border: 1px solid var(--border);
  }
  .btn-ghost:hover {
    border-color: var(--border-hi);
    color: var(--ivory);
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
    .filter-actions {
      margin-left: 0;
      width: 100%;
      justify-content: space-between;
    }
  }
</style>
