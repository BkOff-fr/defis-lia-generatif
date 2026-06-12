<script lang="ts">
  // Module M9 — Référentiel modèles (C18).
  // Consomme 2 commandes IPC : `list_models` (catalogue résumé) et
  // `get_model_detail` (fiche distributionnelle). Contrat no-mock : hors
  // Tauri, aucune fiche n'est rendue (bannière + empty shell uniquement).
  //
  // Voir :
  //   - briefs/chantiers/C18-referentiel-modeles.md
  //   - crates/sobria-app/src/dto.rs (bloc "model detail")
  //   - docs/methodology/MODEL-PRESETS.md
  //   - docs/CAHIER-DES-CHARGES-v1.0.md §4 M9

  import {
    AlertTriangle,
    Info,
    PlugZap,
    HelpCircle,
    Lock,
    BookOpen,
    Inbox,
    Check,
    X
  } from '@lucide/svelte';
  import {
    isBackendAvailable,
    listModels,
    getModelDetail,
    listVendorComparison,
    SobriaIpcError,
    type ModelPresetDto,
    type ModelDetailDto,
    type VendorComparisonRowDto,
    type IpcErrorCode
  } from '$lib/api';
  import { preferences, type ModuleId } from '$lib/preferences';

  import ModelCard from '$lib/components/m9/ModelCard.svelte';
  import ModelDetailDrawer from '$lib/components/m9/ModelDetailDrawer.svelte';
  import ModelFilters, {
    emptyFilters,
    type FilterState
  } from '$lib/components/m9/ModelFilters.svelte';

  const MODULE_ID: ModuleId = 'm9';

  // Module gating — mêmes règles que /datacenters.
  $effect(() => {
    if ($preferences.loaded && !$preferences.enabled_modules.includes(MODULE_ID)) {
      window.location.replace('/?disabled=' + MODULE_ID);
    }
  });

  // ─── State ───────────────────────────────────────────────────────────────
  let models = $state<ModelPresetDto[]>([]);
  let bootstrapping = $state(true);
  let loadError = $state<{ code: IpcErrorCode; message: string } | null>(null);

  // Index baseline P50 par id, alimenté à la demande (au survol/clic).
  // On précharge les 8 premiers modèles affichés pour donner aux cards une
  // valeur immédiate sans bloquer le rendu de la grille (cf. brief §B).
  let baselineCo2eq = $state<Record<string, number>>({});

  // C32.4 — Table comparaison vendor disclosure (5 fabricants).
  let vendorComparison = $state<VendorComparisonRowDto[]>([]);

  // Modèle sélectionné — id stocké pour pouvoir survivre à un re-tri.
  let selectedId = $state<string | null>(null);
  let detail = $state<ModelDetailDto | null>(null);
  let detailLoading = $state(false);
  let detailError = $state<{ code: IpcErrorCode; message: string } | null>(null);

  let filters = $state<FilterState>(emptyFilters());

  const backendAvailable = $derived(isBackendAvailable());

  // ─── Bootstrap ───────────────────────────────────────────────────────────
  $effect(() => {
    void (async () => {
      if (!backendAvailable) {
        bootstrapping = false;
        loadError = {
          code: 'tauri_unavailable',
          message:
            "Cette fonctionnalité nécessite l'application de bureau Sobr.ia. La démo web présente des données d'exemple sur : Estimer, Comparer, Bibliothèque de modèles, Datacenters et Tableau de bord."
        };
        return;
      }
      try {
        const list = await listModels();
        models = list;
        // C32.4 — charge la table comparaison en parallèle (IPC séparée).
        try {
          vendorComparison = await listVendorComparison();
        } catch {
          // Non-fatal — la table reste masquée.
        }
        // Précharge les 8 premiers détails pour peupler les baselines sur les
        // cards visibles. On ignore les erreurs individuelles (les cards
        // afficheront simplement "—").
        const head = list.slice(0, 8);
        await Promise.all(
          head.map(async (m) => {
            try {
              const d = await getModelDetail(m.id);
              baselineCo2eq = { ...baselineCo2eq, [m.id]: d.baseline_co2eq_p50_g };
            } catch {
              // Ignoré — la card retombe sur "—".
            }
          })
        );
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

  // ─── Lazy load du détail à chaque sélection ─────────────────────────────
  $effect(() => {
    if (selectedId === null) {
      detail = null;
      detailError = null;
      detailLoading = false;
      return;
    }
    const id = selectedId;
    void (async () => {
      detailLoading = true;
      detailError = null;
      try {
        const d = await getModelDetail(id);
        if (selectedId === id) {
          detail = d;
          baselineCo2eq = { ...baselineCo2eq, [id]: d.baseline_co2eq_p50_g };
        }
      } catch (err) {
        if (err instanceof SobriaIpcError) {
          detailError = { code: err.code, message: err.message };
        } else {
          detailError = { code: 'internal', message: 'Échec du chargement de la fiche' };
        }
      } finally {
        detailLoading = false;
      }
    })();
  });

  // ─── Filtrage + tri côté front ──────────────────────────────────────────
  const visibleModels = $derived.by(() => {
    const f = filters;
    const filtered = models.filter((m) => {
      if (f.enabledProviders.size > 0 && !f.enabledProviders.has(m.provider)) return false;
      if (f.enabledCalibrations.size > 0 && !f.enabledCalibrations.has(m.calibration)) return false;
      if (f.openness !== null && m.openness !== f.openness) return false;
      return true;
    });
    const sorted = [...filtered];
    switch (f.sort) {
      case 'name':
        sorted.sort((a, b) => a.display_name.localeCompare(b.display_name, 'fr'));
        break;
      case 'params_asc':
        sorted.sort((a, b) => a.approx_params_billions - b.approx_params_billions);
        break;
      case 'params_desc':
        sorted.sort((a, b) => b.approx_params_billions - a.approx_params_billions);
        break;
      case 'co2eq_asc':
        // Modèles sans baseline préchargée → en queue (ordre stable par nom).
        sorted.sort((a, b) => {
          const va = baselineCo2eq[a.id];
          const vb = baselineCo2eq[b.id];
          if (va === undefined && vb === undefined)
            return a.display_name.localeCompare(b.display_name, 'fr');
          if (va === undefined) return 1;
          if (vb === undefined) return -1;
          return va - vb;
        });
        break;
    }
    return sorted;
  });

  // ─── Actions ─────────────────────────────────────────────────────────────
  function selectModel(id: string) {
    selectedId = id;
  }
  function closeDrawer() {
    selectedId = null;
  }
  function resetFilters() {
    filters = emptyFilters();
  }

  // ─── Labels erreur ──────────────────────────────────────────────────────
  const ERROR_LABELS: Record<string, string> = {
    tauri_unavailable: 'Application de bureau requise',
    not_found: 'Modèle inconnu',
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
  <title>Sobr.ia · Bibliothèque de modèles</title>
</svelte:head>

<div class="canvas-inner">
  <!-- TopBar -->
  <div class="topbar">
    <nav class="breadcrumb" aria-label="Fil d'Ariane">
      Pédagogie <span class="sep">/</span>
      <span class="current">Bibliothèque de modèles</span>
    </nav>
    <div class="spacer"></div>
    <span class="local-pill">
      <Lock size={12} strokeWidth={1.8} />
      Référentiel embarqué
    </span>
    <a class="icon-btn" href="/methodo" aria-label="Méthodologie">
      <HelpCircle size={16} strokeWidth={1.6} />
    </a>
  </div>

  <!-- Hero -->
  <section class="hero">
    <div class="hero-eyebrow">
      <span class="pulse" aria-hidden="true"></span>
      Catalogue transparent des modèles
    </div>
    <h1 class="hero-h1">
      Les chiffres derrière <em>chaque modèle</em>.
    </h1>
    <p class="hero-sub">
      Plages distributionnelles P5/P50/P95 utilisées par l'estimateur Monte-Carlo, sources
      documentaires, baseline contextuel. Conçu pour qu'un chercheur ou un journaliste puisse
      reproduire les calculs jusqu'à la décimale.
    </p>
  </section>

  <!-- C32.4 — Table comparaison vendor disclosure (5 fabricants) -->
  {#if vendorComparison.length > 0}
    <section class="vendor-table" data-testid="vendor-comparison">
      <header class="vt-head">
        <h2 class="vt-title">Données vendor disclosure officielles</h2>
        <p class="vt-sub">
          Tableau de transparence : quels fabricants publient des chiffres environnementaux
          officiels, et à quel niveau de granularité. Sobr.ia agrège ces données quand elles
          existent et l'affiche dans la fiche du modèle.
        </p>
      </header>
      <div class="vt-wrap">
        <table>
          <thead>
            <tr>
              <th scope="col">Fabricant</th>
              <th scope="col">Prompt-level</th>
              <th scope="col">Training</th>
              <th scope="col">Source</th>
            </tr>
          </thead>
          <tbody>
            {#each vendorComparison as row (row.vendor)}
              <tr>
                <th scope="row">{row.vendor}</th>
                <td>
                  {#if row.has_prompt_level}
                    <span class="ok" role="img" aria-label="Oui"
                      ><Check size={14} strokeWidth={2.5} /></span
                    >
                  {:else}
                    <span class="ko" role="img" aria-label="Non"
                      ><X size={14} strokeWidth={2.5} /></span
                    >
                  {/if}
                </td>
                <td>
                  {#if row.has_training}
                    <span class="ok" role="img" aria-label="Oui"
                      ><Check size={14} strokeWidth={2.5} /></span
                    >
                  {:else}
                    <span class="ko" role="img" aria-label="Non"
                      ><X size={14} strokeWidth={2.5} /></span
                    >
                  {/if}
                </td>
                <td>
                  {#if row.primary_source_url}
                    <a
                      href={row.primary_source_url}
                      target="_blank"
                      rel="noopener noreferrer"
                      class="src-link"
                    >
                      Voir
                    </a>
                  {:else}
                    <span class="muted">Pas de disclosure officielle</span>
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </section>
  {/if}

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

  {#if !bootstrapping && backendAvailable && models.length > 0}
    <div class="layout">
      <div class="col-l">
        <ModelFilters {models} bind:state={filters} onreset={resetFilters} />
      </div>

      <div class="col-r">
        <div class="grid-meta mono" aria-live="polite">
          {visibleModels.length} / {models.length} modèles
        </div>
        {#if visibleModels.length === 0}
          <div class="empty-grid">
            <Inbox size={20} strokeWidth={1.6} />
            <h4>Aucun modèle ne matche</h4>
            <p>Élargis les filtres ou clique « Reset » pour réafficher le catalogue complet.</p>
          </div>
        {:else}
          <div
            class="grid"
            role="grid"
            aria-label="Catalogue des modèles ({visibleModels.length} visibles)"
          >
            {#each visibleModels as m (m.id)}
              <div class="row" role="row">
                <ModelCard
                  model={m}
                  baselineCo2eqP50G={baselineCo2eq[m.id] ?? null}
                  selected={selectedId === m.id}
                  onselect={selectModel}
                />
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  {:else if !bootstrapping}
    <div class="empty-shell">
      <BookOpen size={20} strokeWidth={1.6} />
      <h4>Référentiel indisponible</h4>
      <p>
        Lance <span class="mono">cargo run -p sobria-app</span> pour activer le catalogue des modèles
        (registre embarqué dans le binaire).
      </p>
    </div>
  {/if}
</div>

{#if selectedId !== null}
  <ModelDetailDrawer {detail} loading={detailLoading} error={detailError} onclose={closeDrawer} />
{/if}

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

  .layout {
    display: grid;
    grid-template-columns: 260px 1fr;
    gap: 20px;
    align-items: flex-start;
  }
  .col-l,
  .col-r {
    min-width: 0;
  }

  .grid-meta {
    font: 500 12px/1 var(--font-mono);
    color: var(--ivory-4);
    margin-bottom: 10px;
    letter-spacing: 0.04em;
  }
  .grid {
    list-style: none;
    padding: 0;
    margin: 0;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 14px;
  }
  .row {
    display: flex;
  }
  .row > :global(*) {
    flex: 1;
    min-width: 0;
  }

  /* C32.4 — Table comparaison vendor disclosure ─────────────────────── */
  .vendor-table {
    padding: 20px 24px;
    background: rgba(197, 240, 74, 0.03);
    border: 1px solid rgba(197, 240, 74, 0.18);
    border-radius: var(--radius-lg);
  }
  .vt-head {
    margin-bottom: 16px;
  }
  .vt-title {
    margin: 0 0 6px;
    font: 400 22px/1.15 var(--font-display);
    font-style: italic;
    color: var(--ivory);
  }
  .vt-sub {
    margin: 0;
    font: 400 13px/1.5 var(--font-ui);
    color: var(--ivory-3);
    max-width: 720px;
  }
  .vt-wrap {
    overflow-x: auto;
  }
  .vendor-table table {
    width: 100%;
    border-collapse: collapse;
    font: 400 13px/1.4 var(--font-ui);
  }
  .vendor-table th,
  .vendor-table td {
    padding: 10px 14px;
    text-align: left;
    border-bottom: 1px solid var(--border);
  }
  .vendor-table thead th {
    color: var(--ivory-3);
    font: 500 12px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .vendor-table tbody th {
    color: var(--ivory);
    font-weight: 500;
  }
  .vendor-table tbody tr:last-child th,
  .vendor-table tbody tr:last-child td {
    border-bottom: none;
  }
  .vendor-table .ok {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: rgba(197, 240, 74, 0.18);
    color: var(--lime);
  }
  .vendor-table .ko {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: rgba(240, 108, 90, 0.15);
    color: var(--coral, #f06c5a);
  }
  .vendor-table .src-link {
    color: var(--lime);
    text-decoration: none;
    border-bottom: 1px dashed rgba(197, 240, 74, 0.4);
  }
  .vendor-table .src-link:hover {
    color: var(--ivory);
    border-bottom-color: var(--ivory);
  }
  .vendor-table .muted {
    color: var(--ivory-4);
    font-style: italic;
    font-size: 12px;
  }

  .empty-grid {
    padding: 40px 32px;
    background: rgba(255, 255, 255, 0.015);
    border: 1px dashed var(--border);
    border-radius: var(--radius-xl);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    text-align: center;
  }
  .empty-grid :global(svg) {
    color: var(--ivory-3);
  }
  .empty-grid h4 {
    font: 400 20px/1.15 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    margin: 0;
  }
  .empty-grid p {
    margin: 0;
    font: 400 12px/1.55 var(--font-ui);
    color: var(--ivory-3);
    max-width: 480px;
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
    .layout {
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
