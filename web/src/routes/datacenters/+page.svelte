<script lang="ts">
  // Module M12 — Datacenters Europe (C12).
  // Consomme 3 commandes IPC : list_datacenters, get_datacenter_detail,
  // aggregate_datacenters_by_country. Contrat no-mock : hors Tauri, aucune
  // carte ni drill-down rendu (placeholder explicite uniquement).
  //
  // Voir :
  //   - briefs/chantiers/C12-datacenters-europe.md
  //   - crates/sobria-app/src/dto.rs (bloc "datacenters")
  //   - docs/sources/CATALOGUE-DATACENTERS.md
  //   - docs/CAHIER-DES-CHARGES-v1.0.md §4 M12

  import { AlertTriangle, Info, PlugZap, HelpCircle, Lock, Server, Globe } from '@lucide/svelte';
  import {
    isTauriContext,
    listDatacenters,
    aggregateDatacentersByCountry,
    getDatacenterDetail,
    SobriaIpcError,
    type DatacenterSummaryDto,
    type DatacenterDetailDto,
    type CountryAggregateDto,
    type IpcErrorCode
  } from '$lib/api';
  import { preferences, type ModuleId } from '$lib/preferences';

  import DatacenterMap from '$lib/components/m12/DatacenterMap.svelte';
  import DatacenterDrillDown from '$lib/components/m12/DatacenterDrillDown.svelte';
  import CountryDrillDown from '$lib/components/m12/CountryDrillDown.svelte';
  import DatacenterFilters, {
    type DatacenterFilterState
  } from '$lib/components/m12/DatacenterFilters.svelte';

  const MODULE_ID: ModuleId = 'm12';

  // Module gating
  $effect(() => {
    if ($preferences.loaded && !$preferences.enabled_modules.includes(MODULE_ID)) {
      window.location.replace('/?disabled=' + MODULE_ID);
    }
  });

  // ─── State ───────────────────────────────────────────────────────────────
  let datacenters = $state<DatacenterSummaryDto[]>([]);
  let countries = $state<CountryAggregateDto[]>([]);
  let bootstrapping = $state(true);
  let loadError = $state<{ code: IpcErrorCode; message: string } | null>(null);

  let selectedDc = $state<DatacenterSummaryDto | null>(null);
  let selectedCountry = $state<CountryAggregateDto | null>(null);

  // Détail du DC sélectionné — lazy-loaded au click marker
  let dcDetail = $state<DatacenterDetailDto | null>(null);
  let dcDetailLoading = $state(false);
  let dcDetailError = $state<{ code: IpcErrorCode; message: string } | null>(null);

  let filters = $state<DatacenterFilterState>({
    enabledOperators: new Set<string>(),
    enabledCountries: new Set<string>()
  });

  const tauriAvailable = $derived(isTauriContext());

  // ─── Bootstrap ───────────────────────────────────────────────────────────
  $effect(() => {
    void (async () => {
      if (!tauriAvailable) {
        bootstrapping = false;
        loadError = {
          code: 'tauri_unavailable',
          message:
            "L'application doit être lancée via `cargo run -p sobria-app` (ou `cargo tauri dev`). Les 28 datacenters européens sont embarqués dans le binaire — pas de fetch externe."
        };
        return;
      }
      try {
        const [dcs, cs] = await Promise.all([listDatacenters(), aggregateDatacentersByCountry()]);
        datacenters = dcs;
        countries = cs;
      } catch (err) {
        if (err instanceof SobriaIpcError) {
          loadError = { code: err.code, message: err.message };
        } else {
          loadError = { code: 'internal', message: 'Échec du chargement des datacenters' };
        }
      } finally {
        bootstrapping = false;
      }
    })();
  });

  // ─── Lazy load du détail à chaque sélection ─────────────────────────────
  $effect(() => {
    if (!selectedDc) {
      dcDetail = null;
      dcDetailError = null;
      dcDetailLoading = false;
      return;
    }
    const dcId = selectedDc.id;
    void (async () => {
      dcDetailLoading = true;
      dcDetailError = null;
      try {
        const d = await getDatacenterDetail(dcId);
        if (selectedDc?.id === dcId) dcDetail = d;
      } catch (err) {
        if (err instanceof SobriaIpcError) {
          dcDetailError = { code: err.code, message: err.message };
        } else {
          dcDetailError = { code: 'internal', message: 'Échec du chargement du détail' };
        }
      } finally {
        dcDetailLoading = false;
      }
    })();
  });

  // ─── Dérivés : filtrage côté front ──────────────────────────────────────
  const filteredDatacenters = $derived.by(() => {
    return datacenters.filter((dc) => {
      if (filters.enabledOperators.size > 0 && !filters.enabledOperators.has(dc.operator))
        return false;
      if (filters.enabledCountries.size > 0 && !filters.enabledCountries.has(dc.country_iso))
        return false;
      return true;
    });
  });

  const filteredCountries = $derived.by(() => {
    if (filters.enabledCountries.size === 0) return countries;
    return countries.filter((c) => filters.enabledCountries.has(c.country_iso));
  });

  // ─── Actions ─────────────────────────────────────────────────────────────
  function selectDc(dc: DatacenterSummaryDto) {
    selectedDc = dc;
    selectedCountry = null;
  }
  function selectCountry(c: CountryAggregateDto) {
    selectedCountry = c;
    selectedDc = null;
  }
  function closeDc() {
    selectedDc = null;
  }
  function closeCountry() {
    selectedCountry = null;
  }
  function resetFilters() {
    filters = { enabledOperators: new Set(), enabledCountries: new Set() };
  }

  // ─── Error labels + tone ────────────────────────────────────────────────
  const ERROR_LABELS: Record<string, string> = {
    tauri_unavailable: 'Application non lancée via Tauri',
    not_found: 'Datacenter inconnu',
    internal: 'Erreur interne'
  };
  function errorLabel(code: string): string {
    return ERROR_LABELS[code] ?? 'Erreur';
  }

  const errorTone = $derived.by<'info' | 'warn' | 'error'>(() => {
    if (!loadError) return 'info';
    if (loadError.code === 'tauri_unavailable') return 'warn';
    return 'error';
  });
</script>

<svelte:head>
  <title>Sobr.ia · Datacenters Europe</title>
</svelte:head>

<div class="canvas-inner">
  <!-- TopBar -->
  <div class="topbar">
    <nav class="breadcrumb" aria-label="Fil d'Ariane">
      Atelier <span class="sep">/</span>
      <span class="current">Datacenters Europe</span>
    </nav>
    <div class="spacer"></div>
    <span class="local-pill">
      <Lock size={12} strokeWidth={1.8} />
      Dataset 28 DC embarqué
    </span>
    <a class="icon-btn" href="/methodo" aria-label="Méthodologie">
      <HelpCircle size={16} strokeWidth={1.6} />
    </a>
  </div>

  <!-- Hero -->
  <section class="hero">
    <div class="hero-eyebrow">
      <span class="pulse" aria-hidden="true"></span>
      Module M12 · 28 datacenters · 13 pays
    </div>
    <h1 class="hero-h1">
      Où tournent <em>physiquement</em> vos prompts ?
    </h1>
    <p class="hero-sub">
      Les 28 datacenters européens servant l'inférence LLM des hyperscalers et opérateurs
      souverains. Drill-down par site : PUE, WUE, intensité carbone locale (Electricity Maps) et
      profil de charge 24h typique (forme modélisée v1.0, pull ENTSO-E live prévu v1.1).
    </p>
  </section>

  <!-- Bannière erreur -->
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

  {#if !bootstrapping && tauriAvailable && datacenters.length > 0}
    <div class="dc-route">
      <div class="dc-map-fill">
        <DatacenterMap
          datacenters={filteredDatacenters}
          countries={filteredCountries}
          selectedDcId={selectedDc?.id ?? null}
          selectedCountryIso={selectedCountry?.country_iso ?? null}
          onSelectDc={selectDc}
          onSelectCountry={selectCountry}
        />
      </div>

      <div class="dc-filters-overlay">
        <DatacenterFilters {datacenters} bind:state={filters} onreset={resetFilters} />
      </div>

      {#if selectedDc}
        <div class="dc-drill-overlay">
          <DatacenterDrillDown
            detail={dcDetail}
            loading={dcDetailLoading}
            error={dcDetailError}
            onclose={closeDc}
          />
        </div>
      {:else if selectedCountry}
        <div class="dc-drill-overlay">
          <CountryDrillDown
            country={selectedCountry}
            {datacenters}
            onclose={closeCountry}
            onSelectDc={selectDc}
          />
        </div>
      {:else}
        <div class="dc-drill-overlay">
          <div class="placeholder">
            <Server size={18} strokeWidth={1.6} />
            <h4>Cliquez un marker</h4>
            <p>
              Pays agrégés au zoom &lt; 5, datacenters individuels au zoom ≥ 5. Le détail s'affiche
              ici.
            </p>
          </div>
        </div>
      {/if}
    </div>
  {:else if !bootstrapping}
    <div class="empty-shell">
      <Globe size={20} strokeWidth={1.6} />
      <h4>Carte indisponible</h4>
      <p>
        Lance <span class="mono">cargo run -p sobria-app</span> pour activer la cartographie des 28 datacenters
        européens (dataset embarqué dans le binaire).
      </p>
    </div>
  {/if}
</div>

<style>
  .canvas-inner {
    max-width: 1440px;
    margin: 0 auto;
    padding: 40px 56px 80px;
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

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
    max-width: 800px;
    margin: 0 0 8px;
  }
  .hero-h1 em {
    font-style: normal;
    color: var(--lime);
  }
  .hero-sub {
    font: 400 15px/1.55 var(--font-ui);
    color: var(--ivory-2);
    max-width: 760px;
    margin: 0;
  }

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

  .dc-route {
    position: relative;
    width: 100%;
    height: 100%;
    min-height: calc(100vh - var(--app-header-h, 64px));
    overflow: hidden;
    border-radius: var(--radius-xl);
    border: 1px solid var(--border);
  }
  .dc-map-fill {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
  }
  .dc-filters-overlay {
    position: absolute;
    top: 16px;
    left: 16px;
    z-index: 5;
    max-width: 280px;
  }
  .dc-drill-overlay {
    position: absolute;
    top: 16px;
    right: 16px;
    bottom: 16px;
    width: 340px;
    z-index: 5;
    overflow-y: auto;
  }

  .placeholder {
    padding: 24px 22px;
    background: rgba(255, 255, 255, 0.015);
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 6px;
  }
  .placeholder :global(svg) {
    color: var(--ivory-3);
  }
  .placeholder h4 {
    font: 400 18px/1.15 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    margin: 0;
  }
  .placeholder p {
    margin: 0;
    font: 400 12px/1.5 var(--font-ui);
    color: var(--ivory-3);
  }

  .empty-shell {
    padding: 40px;
    background: rgba(255, 255, 255, 0.015);
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    text-align: center;
  }
  .empty-shell :global(svg) {
    color: var(--ivory-3);
  }
  .empty-shell h4 {
    font: 400 22px/1.15 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    margin: 0;
  }
  .empty-shell p {
    margin: 0;
    font: 400 13px/1.55 var(--font-ui);
    color: var(--ivory-3);
    max-width: 560px;
  }

  .mono {
    font-family: var(--font-mono);
  }

  @media (max-width: 1180px) {
    .dc-drill-overlay {
      width: 300px;
    }
    .dc-filters-overlay {
      max-width: 240px;
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
