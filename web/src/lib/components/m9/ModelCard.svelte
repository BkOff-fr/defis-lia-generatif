<script lang="ts" module>
  import type { ModelPresetDto } from '$lib/api';

  export const CALIB_LABEL: Record<ModelPresetDto['calibration'], string> = {
    validated: 'Validé',
    indicative: 'Indicatif',
    extrapolated: 'Extrapolé'
  };
  export const CALIB_TONE: Record<ModelPresetDto['calibration'], 'lime' | 'amber' | 'coral'> = {
    validated: 'lime',
    indicative: 'amber',
    extrapolated: 'coral'
  };
  export const OPENNESS_LABEL: Record<ModelPresetDto['openness'], string> = {
    open: 'Open source',
    open_weights: 'Poids ouverts',
    closed: 'Fermé'
  };
</script>

<script lang="ts">
  import { Unlock, Key, Lock } from '@lucide/svelte';

  type Props = {
    model: ModelPresetDto;
    baselineCo2eqP50G: number | null;
    selected: boolean;
    onselect: (id: string) => void;
  };
  const { model, baselineCo2eqP50G, selected, onselect }: Props = $props();

  function fmt(value: number, digits = 2): string {
    if (!Number.isFinite(value)) return '—';
    return new Intl.NumberFormat('fr-FR', { maximumFractionDigits: digits }).format(value);
  }

  function autoCo2(value: number | null): { v: string; u: string } {
    if (value === null || !Number.isFinite(value)) return { v: '—', u: 'g CO₂eq' };
    if (value >= 1000) return { v: fmt(value / 1000, 2), u: 'kg CO₂eq' };
    if (value >= 1) return { v: fmt(value, 2), u: 'g CO₂eq' };
    return { v: fmt(value * 1000, 1), u: 'mg CO₂eq' };
  }

  const co2 = $derived(autoCo2(baselineCo2eqP50G));

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      onselect(model.id);
    }
  }
</script>

<div
  class="card"
  class:selected
  role="gridcell"
  tabindex="0"
  aria-label="{model.display_name} — fiche détaillée"
  onclick={() => onselect(model.id)}
  onkeydown={onKey}
>
  <header class="card-head">
    <div class="ids">
      <div class="provider">{model.provider}</div>
      <h3 class="model-name">{model.display_name}</h3>
      <div class="model-id mono">{model.id}</div>
    </div>
    <div class="badges">
      <span class="badge calib" data-tone={CALIB_TONE[model.calibration]}>
        {CALIB_LABEL[model.calibration]}
      </span>
      <span class="badge openness" data-openness={model.openness}>
        {#if model.openness === 'open'}
          <Unlock size={11} strokeWidth={1.8} />
        {:else if model.openness === 'open_weights'}
          <Key size={11} strokeWidth={1.8} />
        {:else}
          <Lock size={11} strokeWidth={1.8} />
        {/if}
        {OPENNESS_LABEL[model.openness]}
      </span>
    </div>
  </header>

  <div class="tagline mono">~{fmt(model.approx_params_billions, 1)} B paramètres</div>

  <section class="baseline" aria-label="Baseline CO₂eq de référence">
    <div class="baseline-h">CO₂eq · 100 in / 500 out</div>
    <div class="baseline-v">
      <span class="big mono">{co2.v}</span>
      <span class="unit">{co2.u}</span>
    </div>
    <div class="baseline-sub">médiane Monte-Carlo · PUE 1,3 · IF FR</div>
  </section>
</div>

<style>
  .card {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 18px 18px 16px;
    background: rgba(255, 255, 255, 0.018);
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
    cursor: pointer;
    transition:
      transform var(--dur-base) var(--ease),
      border-color var(--dur-base) var(--ease),
      background var(--dur-base) var(--ease);
    outline: none;
    min-width: 0;
  }
  .card:hover {
    border-color: var(--border-hi);
    background: rgba(255, 255, 255, 0.03);
    transform: translateY(-2px);
  }
  .card:focus-visible {
    border-color: var(--lime);
    box-shadow: 0 0 0 3px rgba(197, 240, 74, 0.18);
  }
  .card.selected {
    border-color: var(--lime);
    background: rgba(197, 240, 74, 0.04);
  }

  .card-head {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 12px;
  }
  .ids {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .provider {
    font: 500 12px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--ivory-3);
  }
  .model-name {
    font: 400 22px/1.15 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.01em;
    margin: 0;
    overflow-wrap: anywhere;
  }
  .model-id {
    font: 400 12px/1.2 var(--font-mono);
    color: var(--ivory-4);
    overflow-wrap: anywhere;
  }

  .badges {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 4px;
    flex-shrink: 0;
  }
  .badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 22px;
    padding: 0 8px;
    border-radius: 999px;
    font: 500 12px/1 var(--font-ui);
    letter-spacing: 0.02em;
    border: 1px solid transparent;
  }
  .badge.calib[data-tone='lime'] {
    background: var(--lime-soft);
    color: var(--lime);
    border-color: rgba(197, 240, 74, 0.3);
  }
  .badge.calib[data-tone='amber'] {
    background: rgba(245, 183, 105, 0.1);
    color: var(--amber);
    border-color: rgba(245, 183, 105, 0.32);
  }
  .badge.calib[data-tone='coral'] {
    background: rgba(240, 108, 90, 0.1);
    color: var(--coral);
    border-color: rgba(240, 108, 90, 0.32);
  }
  .badge.openness {
    background: rgba(255, 255, 255, 0.04);
    color: var(--ivory-2);
    border-color: var(--border-hi);
  }
  .badge.openness[data-openness='open'] {
    color: var(--ivory);
  }

  .tagline {
    font: 500 12px/1 var(--font-mono);
    color: var(--ivory-3);
    letter-spacing: 0.04em;
  }

  .baseline {
    margin-top: auto;
    padding: 12px 14px;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .baseline-h {
    font: 500 12px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--ivory-3);
  }
  .baseline-v {
    display: flex;
    align-items: baseline;
    gap: 6px;
  }
  .big {
    font: 600 26px/1 var(--font-mono);
    color: var(--ivory);
    letter-spacing: -0.01em;
  }
  .unit {
    font: 500 12px/1 var(--font-ui);
    color: var(--ivory-3);
  }
  .baseline-sub {
    font: 400 12px/1.3 var(--font-ui);
    color: var(--ivory-4);
    font-style: italic;
  }

  .mono {
    font-family: var(--font-mono);
  }
</style>
