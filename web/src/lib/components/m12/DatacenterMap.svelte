<script lang="ts" module>
  // Palette opérateurs — alignée sur les acteurs majeurs européens (cf. brief
  // C12 §1). Chaque DC peut appartenir à un cloud public (hyperscaler), à un
  // opérateur souverain (OVH, Scaleway), ou à un colocateur (Equinix).

  export const OPERATOR_COLORS: Record<string, string> = {
    AWS: '#f5b769', // amber
    GCP: '#7eb6ff', // blue
    Azure: '#b794f4', // violet
    Microsoft: '#b794f4',
    OVH: '#c5f04a', // lime (souverain FR)
    Scaleway: '#88a83a', // lime-deep (souverain FR)
    Equinix: '#f06c5a', // coral
    Hetzner: '#7eb6ff',
    OpenAI: '#c5f04a',
    Anthropic: '#f5b769',
    Mistral: '#c5f04a'
  };

  export function colorForOperator(op: string): string {
    return OPERATOR_COLORS[op] ?? '#f0ece3'; // ivory fallback
  }
</script>

<script lang="ts">
  // Carte Leaflet M12 — datacenters européens, agrégation pays au zoom faible,
  // markers individuels color-codés par opérateur au zoom élevé.

  import 'leaflet/dist/leaflet.css';
  import { onMount } from 'svelte';
  import type { Map as LMap, LayerGroup, CircleMarker, TileLayer } from 'leaflet';
  import type { DatacenterSummaryDto, CountryAggregateDto } from '$lib/api';

  type Props = {
    datacenters: DatacenterSummaryDto[];
    countries: CountryAggregateDto[];
    selectedDcId: string | null;
    selectedCountryIso: string | null;
    onSelectDc: (dc: DatacenterSummaryDto) => void;
    onSelectCountry: (c: CountryAggregateDto) => void;
  };

  let {
    datacenters,
    countries,
    selectedDcId,
    selectedCountryIso,
    onSelectDc,
    onSelectCountry
  }: Props = $props();

  let mapDiv: HTMLDivElement | undefined = $state();
  let map: LMap | null = null;
  let dcLayer: LayerGroup | null = null;
  let countryLayer: LayerGroup | null = null;
  let currentZoom = $state(4);
  let ready = $state(false);

  // Sous Z5 → pays agrégés ; Z5+ → DC individuels (cf. brief C12).
  const ZOOM_COUNTRY_TO_DC = 5;

  const COL_SELECTED = '#f5b769';

  function countryRadius(count: number, max: number): number {
    if (max <= 0) return 12;
    return 12 + (count / max) * 24;
  }

  function refreshMarkers(L: typeof import('leaflet')) {
    if (!map || !dcLayer || !countryLayer) return;
    dcLayer.clearLayers();
    countryLayer.clearLayers();

    if (currentZoom < ZOOM_COUNTRY_TO_DC) {
      const maxCount = countries.reduce(
        (m, c) => (c.datacenter_count > m ? c.datacenter_count : m),
        0
      );
      for (const c of countries) {
        const isSelected = selectedCountryIso === c.country_iso;
        const ifTone =
          c.if_electrical_g_per_kwh < 100
            ? '#c5f04a' // lime (décarboné)
            : c.if_electrical_g_per_kwh < 300
              ? '#f5b769' // amber (mixte)
              : '#f06c5a'; // coral (carboné)
        const marker: CircleMarker = L.circleMarker([c.centroid_lat, c.centroid_lon], {
          radius: countryRadius(c.datacenter_count, maxCount),
          color: isSelected ? COL_SELECTED : ifTone,
          weight: 1.8,
          fillColor: isSelected ? COL_SELECTED : ifTone,
          fillOpacity: isSelected ? 0.55 : 0.28
        });
        marker.bindTooltip(
          `<strong>${c.country_iso}</strong><br>` +
            `${c.datacenter_count} datacenter${c.datacenter_count > 1 ? 's' : ''}<br>` +
            `IF ${c.if_electrical_g_per_kwh.toFixed(0)} g/kWh · PUE ${c.avg_pue.toFixed(2)}`,
          { direction: 'top', offset: [0, -6], className: 'm12-tooltip' }
        );
        marker.on('click', () => {
          onSelectCountry(c);
          map?.setView([c.centroid_lat, c.centroid_lon], 6, { animate: true });
        });
        countryLayer.addLayer(marker);
      }
    } else {
      for (const dc of datacenters) {
        const isSelected = selectedDcId === dc.id;
        const baseColor = colorForOperator(dc.operator);
        const marker: CircleMarker = L.circleMarker([dc.lat, dc.lon], {
          radius: 7,
          color: isSelected ? COL_SELECTED : baseColor,
          weight: 1.8,
          fillColor: isSelected ? COL_SELECTED : baseColor,
          fillOpacity: isSelected ? 0.65 : 0.4
        });
        marker.bindTooltip(
          `<strong>${dc.name}</strong><br>` +
            `${dc.operator} · ${dc.city}<br>` +
            `PUE ${dc.pue.toFixed(2)} · IF ${dc.if_electrical_g_per_kwh.toFixed(0)} g/kWh`,
          { direction: 'top', offset: [0, -6], className: 'm12-tooltip' }
        );
        marker.on('click', () => onSelectDc(dc));
        dcLayer.addLayer(marker);
      }
    }
  }

  onMount(() => {
    const container = mapDiv;
    if (!container) return;

    let cleanup: () => void = () => {};

    void (async () => {
      const L = (await import('leaflet')).default;
      // Centre approximatif de l'Europe (Bbox brief : lat [35, 71], lon [-10, 35]).
      const initialView: [number, number] = [52, 10];
      map = L.map(container, {
        zoomControl: true,
        attributionControl: true,
        keyboard: true,
        scrollWheelZoom: true
      }).setView(initialView, 4);

      const tiles: TileLayer = L.tileLayer(
        'https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}.png',
        {
          maxZoom: 18,
          minZoom: 3,
          attribution: '© OpenStreetMap · © CARTO',
          subdomains: 'abcd'
        }
      );
      tiles.addTo(map);

      dcLayer = L.layerGroup().addTo(map);
      countryLayer = L.layerGroup().addTo(map);

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

  $effect(() => {
    if (!ready) return;
    void datacenters;
    void countries;
    void selectedDcId;
    void selectedCountryIso;
    void (async () => {
      const L = (await import('leaflet')).default;
      refreshMarkers(L);
    })();
  });
</script>

<div class="map-wrapper">
  <div class="map" bind:this={mapDiv} aria-label="Carte des datacenters européens"></div>
  <div class="zoom-hint mono">
    Zoom {currentZoom}/18 ·
    {#if currentZoom < ZOOM_COUNTRY_TO_DC}
      pays agrégés ({countries.length})
    {:else}
      datacenters individuels ({datacenters.length})
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

  :global(.leaflet-container) {
    background: var(--ink-2) !important;
    font-family: var(--font-ui);
  }
  :global(.m12-tooltip) {
    background: rgba(10, 13, 11, 0.92);
    border: 1px solid rgba(197, 240, 74, 0.25);
    border-radius: 6px;
    color: var(--ivory);
    font: 400 12px/1.4 var(--font-ui);
    padding: 6px 10px;
    box-shadow: 0 8px 24px -8px rgba(0, 0, 0, 0.6);
  }
  :global(.m12-tooltip::before),
  :global(.leaflet-tooltip-top::before) {
    border-top-color: rgba(197, 240, 74, 0.4) !important;
  }
</style>
