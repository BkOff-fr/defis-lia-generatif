<script lang="ts">
  // Module M20 — Territoire FR + Sankey énergétique (C13).
  // Consomme 4 commandes IPC sobria-app : list_industrial_sites_fr,
  // aggregate_industrial_sites_by_region, sankey_fr_data, get_industrial_site_fr.
  // Contrat no-mock : hors Tauri on rend la coque + bannière, aucune carte
  // factice (les markers Leaflet sont absents en l'absence de données réelles).
  //
  // Voir :
  //   - briefs/chantiers/C13-territoire-fr-sankey.md
  //   - crates/sobria-app/src/dto.rs (bloc "territoire_fr" + "sankey_fr")
  //   - docs/CAHIER-DES-CHARGES-v1.0.md §4 M20

  import {
    AlertTriangle,
    Info,
    PlugZap,
    HelpCircle,
    Lock,
    Database,
    Terminal,
    MapPin
  } from '@lucide/svelte';
  import {
    isBackendAvailable,
    listIndustrialSitesFr,
    aggregateIndustrialSitesByRegion,
    sankeyFrData,
    SobriaIpcError,
    type IndustrialSiteSummaryDto,
    type RegionFrAggregateDto,
    type SankeyDataDto,
    type IpcErrorCode
  } from '$lib/api';
  import { preferences, type ModuleId } from '$lib/preferences';

  import TerritoireMap from '$lib/components/m20/TerritoireMap.svelte';
  import TerritoireFilters, {
    type TerritoireFilterState
  } from '$lib/components/m20/TerritoireFilters.svelte';
  import SiteDrillDown from '$lib/components/m20/SiteDrillDown.svelte';
  import RegionDrillDown from '$lib/components/m20/RegionDrillDown.svelte';
  import SankeyChart from '$lib/components/m20/SankeyChart.svelte';

  const MODULE_ID: ModuleId = 'm20';

  // Module gating
  $effect(() => {
    if ($preferences.loaded && !$preferences.enabled_modules.includes(MODULE_ID)) {
      window.location.replace('/?disabled=' + MODULE_ID);
    }
  });

  // ─── State ───────────────────────────────────────────────────────────────
  let sites = $state<IndustrialSiteSummaryDto[]>([]);
  let regions = $state<RegionFrAggregateDto[]>([]);
  let sankey = $state<SankeyDataDto | null>(null);

  let bootstrapping = $state(true);
  let loadError = $state<{ code: IpcErrorCode; message: string } | null>(null);

  let selectedSite = $state<IndustrialSiteSummaryDto | null>(null);
  let selectedRegion = $state<RegionFrAggregateDto | null>(null);

  let filters = $state<TerritoireFilterState>({
    enabledRegions: new Set<string>(),
    minConsumptionGwh: 0
  });

  const backendAvailable = $derived(isBackendAvailable());

  // ─── Bootstrap ───────────────────────────────────────────────────────────
  $effect(() => {
    void (async () => {
      if (!backendAvailable) {
        bootstrapping = false;
        loadError = {
          code: 'tauri_unavailable',
          message:
            "La carte Territoire FR (RTE IRIS, maille industrielle) nécessite l'application de bureau Sobr.ia — les données sont chargées en local par le backend Rust."
        };
        return;
      }
      try {
        // Charge en parallèle : sites top 200, agrégats régionaux, Sankey.
        const [s, r, sk] = await Promise.all([
          listIndustrialSitesFr(200, 0),
          aggregateIndustrialSitesByRegion(),
          sankeyFrData()
        ]);
        sites = s;
        regions = r;
        sankey = sk;
      } catch (err) {
        if (err instanceof SobriaIpcError) {
          loadError = { code: err.code, message: err.message };
        } else {
          loadError = { code: 'internal', message: 'Échec du chargement Territoire FR' };
        }
      } finally {
        bootstrapping = false;
      }
    })();
  });

  // ─── Dérivés : filtrage côté front ──────────────────────────────────────
  const filteredSites = $derived.by(() => {
    return sites.filter((s) => {
      if (filters.enabledRegions.size > 0 && !filters.enabledRegions.has(s.region_iso)) {
        return false;
      }
      if (s.consumption_total_mwh / 1000 < filters.minConsumptionGwh) return false;
      return true;
    });
  });

  const filteredRegions = $derived.by(() => {
    if (filters.enabledRegions.size === 0) return regions;
    return regions.filter((r) => filters.enabledRegions.has(r.region_iso));
  });

  // ─── Actions ─────────────────────────────────────────────────────────────
  function selectSite(s: IndustrialSiteSummaryDto) {
    selectedSite = s;
    selectedRegion = null;
  }

  function selectRegion(r: RegionFrAggregateDto) {
    selectedRegion = r;
    selectedSite = null;
  }

  function closeSite() {
    selectedSite = null;
  }

  function closeRegion() {
    selectedRegion = null;
  }

  function resetFilters() {
    filters = { enabledRegions: new Set(), minConsumptionGwh: 0 };
  }

  // ─── Error labels + tone ────────────────────────────────────────────────
  const ERROR_LABELS: Record<string, string> = {
    tauri_unavailable: 'Application de bureau requise',
    data_not_ingested: 'Données territoriales non ingérées',
    internal: 'Erreur interne',
    io_error: 'Lecture du dataset impossible'
  };
  function errorLabel(code: string): string {
    return ERROR_LABELS[code] ?? 'Erreur';
  }

  const errorTone = $derived.by<'info' | 'warn' | 'error'>(() => {
    if (!loadError) return 'info';
    if (loadError.code === 'tauri_unavailable') return 'warn';
    if (loadError.code === 'data_not_ingested') return 'warn';
    return 'error';
  });

  // ── data_not_ingested → message empty state + commande CLI proposée.
  const showDataNotIngested = $derived(loadError?.code === 'data_not_ingested');
</script>

<svelte:head>
  <title>Sobr.ia · Territoire France</title>
</svelte:head>

<div class="canvas-inner">
  <!-- TopBar -->
  <div class="topbar">
    <nav class="breadcrumb" aria-label="Fil d'Ariane">
      Atelier <span class="sep">/</span>
      <span class="current">Territoire France</span>
    </nav>
    <div class="spacer"></div>
    <span class="local-pill">
      <Lock size={12} strokeWidth={1.8} />
      Datasets ODRÉ 100 % locaux
    </span>
    <a class="icon-btn" href="/methodo" aria-label="Méthodologie">
      <HelpCircle size={16} strokeWidth={1.6} />
    </a>
  </div>

  <!-- Hero -->
  <section class="hero">
    <div class="hero-eyebrow">
      <span class="pulse" aria-hidden="true"></span>
      Cartographie IRIS · 13 régions
    </div>
    <h1 class="hero-h1">
      L'angle <em>territorial</em> français de l'impact IA.
    </h1>
    <p class="hero-sub">
      200 sites industriels FR top-consommateurs (RTE IRIS / ODRÉ), agrégés par région avec leur mix
      énergétique. Sankey national en bas de page pour situer l'usage IA dans la production
      électrique réelle 2023.
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

  <!-- État `data_not_ingested` : invite à lancer la CLI. -->
  {#if showDataNotIngested}
    <div class="ingest-card">
      <div class="ic-h">
        <Database size={16} strokeWidth={1.8} />
        <strong>Téléchargez les données officielles ODRÉ</strong>
      </div>
      <p>
        Le backend n'a pas trouvé <span class="mono">territoire_fr.json</span> ni
        <span class="mono">rte_mix_fr.json</span>. Lance la commande ci-dessous pour fetcher les
        datasets RTE IRIS (sites industriels) et RTE eco2mix (mix électrique) depuis l'API ODRÉ —
        environ 12 MB, Etalab 2.0.
      </p>
      <div class="cli mono">
        <Terminal size={11} strokeWidth={1.8} />
        cargo run -p sobria-ingest -- fetch territoire-fr
      </div>
      <div class="cli mono">
        <Terminal size={11} strokeWidth={1.8} />
        cargo run -p sobria-ingest -- fetch rte-mix
      </div>
      <p class="hint">
        Bouton « Télécharger les données officielles » in-app prévu en v1.1 (chantier C18, IPC
        <span class="mono">fetch_official_dataset</span>).
      </p>
    </div>
  {/if}

  {#if !bootstrapping && backendAvailable && !showDataNotIngested && sites.length > 0}
    <!-- Grid 3 colonnes : filtres / carte / drill-down -->
    <div class="grid">
      <div class="col-l">
        <TerritoireFilters {regions} {sites} bind:state={filters} onreset={resetFilters} />
      </div>

      <div class="col-c">
        <TerritoireMap
          sites={filteredSites}
          regions={filteredRegions}
          {selectedSite}
          {selectedRegion}
          onSelectSite={selectSite}
          onSelectRegion={selectRegion}
        />
      </div>

      <div class="col-r">
        {#if selectedSite}
          <SiteDrillDown site={selectedSite} onclose={closeSite} />
        {:else if selectedRegion}
          <RegionDrillDown region={selectedRegion} onclose={closeRegion} />
        {:else}
          <div class="placeholder">
            <MapPin size={18} strokeWidth={1.6} />
            <h4>Cliquez un marker</h4>
            <p>
              Sites individuels au zoom ≥ 6, agrégats régionaux en dessous. Le détail de la
              sélection s'affiche ici.
            </p>
          </div>
        {/if}
      </div>
    </div>
  {:else if !bootstrapping}
    <!-- Coque sans données (no-mock contract). -->
    <div class="empty-shell">
      <MapPin size={20} strokeWidth={1.6} />
      <h4>Carte et Sankey indisponibles</h4>
      <p>
        Installez l'application de bureau (datasets territoire-fr ingérés) pour activer la carte et
        rte-mix pour activer la cartographie.
      </p>
    </div>
  {/if}

  <!-- Sankey énergétique national -->
  {#if sankey}
    <SankeyChart data={sankey} />
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

  /* TopBar */
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
    font: 500 12px/1 var(--font-ui);
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
    padding-bottom: 14px;
    border-bottom: 1px solid var(--border);
  }
  .hero-eyebrow {
    font: 500 12px/1 var(--font-ui);
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

  /* Bannière */
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

  /* Ingest card (data_not_ingested empty state) */
  .ingest-card {
    padding: 22px 26px;
    background: rgba(245, 183, 105, 0.04);
    border: 1px solid rgba(245, 183, 105, 0.25);
    border-radius: var(--radius-xl);
  }
  .ic-h {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font: 600 14px/1 var(--font-ui);
    color: var(--amber);
    margin-bottom: 10px;
  }
  .ingest-card p {
    margin: 0 0 12px;
    font: 400 13px/1.55 var(--font-ui);
    color: var(--ivory-2);
    max-width: 720px;
  }
  .ingest-card .cli {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 10px 16px;
    background: rgba(0, 0, 0, 0.4);
    border: 1px solid rgba(245, 183, 105, 0.2);
    border-radius: var(--radius-sm);
    font: 500 12px/1 var(--font-mono);
    color: var(--ivory);
    margin-bottom: 6px;
    user-select: all;
  }
  .ingest-card .cli :global(svg) {
    color: var(--amber);
  }
  .ingest-card .hint {
    font: 400 12px/1.4 var(--font-ui);
    color: var(--ivory-3);
    font-style: italic;
    margin-top: 12px;
  }

  /* Grid */
  .grid {
    display: grid;
    grid-template-columns: minmax(240px, 1fr) minmax(0, 2.2fr) minmax(280px, 1.1fr);
    gap: 16px;
    align-items: stretch;
  }
  .col-l,
  .col-r {
    min-width: 0;
  }
  .col-c {
    min-width: 0;
    display: flex;
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
    .grid {
      grid-template-columns: 1fr;
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
