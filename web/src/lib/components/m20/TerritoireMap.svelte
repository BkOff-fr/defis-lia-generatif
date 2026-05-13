<script lang="ts">
  // Carte Leaflet pour M20 (Territoire FR) — CARTO dark, agrégat régional
  // au zoom faible, sites individuels au zoom élevé.
  //
  // Lazy-load de Leaflet : le module accède à `window` au top-level, donc
  // import via `await import()` dans `onMount` pour ne pas casser le SSR
  // SvelteKit (cf. ADR-0011 si on en formalise un).

  import 'leaflet/dist/leaflet.css';
  import { onMount } from 'svelte';
  import type { Map as LMap, LayerGroup, CircleMarker, TileLayer } from 'leaflet';
  import type { IndustrialSiteSummaryDto, RegionFrAggregateDto } from '$lib/api';

  type Props = {
    sites: IndustrialSiteSummaryDto[];
    regions: RegionFrAggregateDto[];
    selectedSite: IndustrialSiteSummaryDto | null;
    selectedRegion: RegionFrAggregateDto | null;
    onSelectSite: (s: IndustrialSiteSummaryDto) => void;
    onSelectRegion: (r: RegionFrAggregateDto) => void;
  };

  let { sites, regions, selectedSite, selectedRegion, onSelectSite, onSelectRegion }: Props =
    $props();

  let mapDiv: HTMLDivElement | undefined = $state();
  let map: LMap | null = null;
  let siteLayer: LayerGroup | null = null;
  let regionLayer: LayerGroup | null = null;
  let currentZoom = $state(5);
  let ready = $state(false);

  // Seuil de bascule : sous Z6, on agrège par région.
  const ZOOM_REGION_TO_SITE = 6;

  // Couleurs alignées sur le design system (cf. app.css tokens).
  const COL_SITE = '#c5f04a'; // lime
  const COL_REGION = '#7eb6ff'; // blue
  const COL_SELECTED = '#f5b769'; // amber

  function siteRadius(consMwh: number): number {
    // Échelle log compressée — top 200 sites s'étalent sur 4 ordres de grandeur.
    if (consMwh <= 0) return 4;
    return Math.max(4, Math.min(14, 4 + Math.log10(consMwh) * 1.6));
  }

  function regionRadius(count: number, max: number): number {
    if (max <= 0) return 10;
    return 10 + (count / max) * 22;
  }

  function refreshMarkers(L: typeof import('leaflet')) {
    if (!map || !siteLayer || !regionLayer) return;
    siteLayer.clearLayers();
    regionLayer.clearLayers();

    if (currentZoom < ZOOM_REGION_TO_SITE) {
      // Agrégats régionaux
      const maxCount = regions.reduce((m, r) => (r.site_count > m ? r.site_count : m), 0);
      for (const r of regions) {
        const isSelected = selectedRegion?.region_iso === r.region_iso;
        const marker: CircleMarker = L.circleMarker([r.centroid_lat, r.centroid_lon], {
          radius: regionRadius(r.site_count, maxCount),
          color: isSelected ? COL_SELECTED : COL_REGION,
          weight: 1.6,
          fillColor: isSelected ? COL_SELECTED : COL_REGION,
          fillOpacity: isSelected ? 0.5 : 0.25
        });
        marker.bindTooltip(
          `<strong>${r.region_name}</strong><br>` +
            `${r.site_count} sites · ${(r.total_consumption_mwh / 1000).toFixed(0)} GWh`,
          { direction: 'top', offset: [0, -6], className: 'm20-tooltip' }
        );
        marker.on('click', () => {
          onSelectRegion(r);
          map?.setView([r.centroid_lat, r.centroid_lon], 7, { animate: true });
        });
        regionLayer.addLayer(marker);
      }
    } else {
      // Sites individuels
      for (const s of sites) {
        const isSelected = selectedSite?.code_iris === s.code_iris;
        const marker: CircleMarker = L.circleMarker([s.lat, s.lon], {
          radius: siteRadius(s.consumption_total_mwh),
          color: isSelected ? COL_SELECTED : COL_SITE,
          weight: 1.4,
          fillColor: isSelected ? COL_SELECTED : COL_SITE,
          fillOpacity: isSelected ? 0.55 : 0.3
        });
        marker.bindTooltip(
          `<strong>${s.commune}</strong><br>` +
            `IRIS ${s.code_iris}<br>` +
            `${(s.consumption_total_mwh / 1000).toFixed(2)} GWh/an`,
          { direction: 'top', offset: [0, -6], className: 'm20-tooltip' }
        );
        marker.on('click', () => onSelectSite(s));
        siteLayer.addLayer(marker);
      }
    }
  }

  onMount(() => {
    const container = mapDiv;
    if (!container) return;

    let cleanup: () => void = () => {};

    void (async () => {
      const L = (await import('leaflet')).default;
      const initialView: [number, number] = [46.5, 2.5]; // Centre approximatif de la France métropolitaine.
      map = L.map(container, {
        zoomControl: true,
        attributionControl: true,
        keyboard: true,
        scrollWheelZoom: true
      }).setView(initialView, 5);

      const tiles: TileLayer = L.tileLayer(
        'https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}.png',
        {
          maxZoom: 18,
          minZoom: 4,
          attribution: '© OpenStreetMap · © CARTO',
          subdomains: 'abcd'
        }
      );
      tiles.addTo(map);

      siteLayer = L.layerGroup().addTo(map);
      regionLayer = L.layerGroup().addTo(map);

      currentZoom = map.getZoom();
      map.on('zoomend', () => {
        if (!map) return;
        currentZoom = map.getZoom();
        refreshMarkers(L);
      });

      refreshMarkers(L);
      ready = true;

      cleanup = () => {
        map?.remove();
        map = null;
      };
    })();

    return () => cleanup();
  });

  // Re-render markers when data or selection changes.
  $effect(() => {
    if (!ready) return;
    // Dépendances explicites
    void sites;
    void regions;
    void selectedSite?.code_iris;
    void selectedRegion?.region_iso;
    void (async () => {
      const L = (await import('leaflet')).default;
      refreshMarkers(L);
    })();
  });
</script>

<div class="map-wrapper">
  <div class="map" bind:this={mapDiv} aria-label="Carte des sites industriels FR"></div>
  <div class="zoom-hint mono">
    Zoom {currentZoom}/18 ·
    {#if currentZoom < ZOOM_REGION_TO_SITE}
      vue régionale (13 régions)
    {:else}
      sites individuels ({sites.length} markers)
    {/if}
  </div>
</div>

<style>
  .map-wrapper {
    position: relative;
    height: 100%;
    min-height: 520px;
    border-radius: var(--radius-xl);
    overflow: hidden;
    border: 1px solid var(--border);
  }
  .map {
    height: 100%;
    width: 100%;
    background: var(--ink-2);
  }
  .zoom-hint {
    position: absolute;
    bottom: 14px;
    left: 14px;
    z-index: 500;
    padding: 5px 11px;
    background: rgba(10, 13, 11, 0.85);
    border: 1px solid var(--border);
    border-radius: 999px;
    font: 500 10px/1 var(--font-mono);
    color: var(--ivory-2);
    letter-spacing: 0.06em;
    pointer-events: none;
    backdrop-filter: blur(8px);
  }
  .mono {
    font-family: var(--font-mono);
  }

  /* Tooltip CARTO dark — surchargé via :global pour échapper au scoping. */
  :global(.leaflet-container) {
    background: var(--ink-2) !important;
    font-family: var(--font-ui);
  }
  :global(.m20-tooltip) {
    background: rgba(10, 13, 11, 0.92);
    border: 1px solid rgba(197, 240, 74, 0.25);
    border-radius: 6px;
    box-shadow: 0 8px 24px -8px rgba(0, 0, 0, 0.6);
    color: var(--ivory);
    font: 400 12px/1.4 var(--font-ui);
    padding: 6px 10px;
  }
  :global(.m20-tooltip::before),
  :global(.leaflet-tooltip-top::before) {
    border-top-color: rgba(197, 240, 74, 0.4) !important;
  }
  :global(.leaflet-control-zoom a) {
    background: rgba(10, 13, 11, 0.85) !important;
    color: var(--ivory) !important;
    border: 1px solid var(--border) !important;
  }
  :global(.leaflet-control-zoom a:hover) {
    background: var(--surface-hi) !important;
    color: var(--lime) !important;
  }
  :global(.leaflet-control-attribution) {
    background: rgba(10, 13, 11, 0.75) !important;
    color: var(--ivory-3) !important;
    font: 400 9px/1 var(--font-mono) !important;
  }
  :global(.leaflet-control-attribution a) {
    color: var(--ivory-2) !important;
  }
</style>
