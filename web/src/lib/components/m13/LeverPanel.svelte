<script lang="ts" module>
  // Bornes des sliders alignées sur le brief C11 §1.
  //
  // Note méthodologique : ces bornes UI sont volontairement plus serrées que
  // la validation Rust côté `sobria-estimator::params` — elles guident
  // l'utilisateur vers des plages réalistes (pas de PUE 3,0 fantaisiste).

  export const PUE_MIN = 1.05;
  export const PUE_MAX = 1.6;
  export const PUE_DEFAULT = 1.3;

  export const MIX_MIN = 10;
  export const MIX_MAX = 800;
  export const MIX_DEFAULT = 412;

  export const TOKENS_OUT_MIN = 1;
  export const TOKENS_OUT_MAX = 10_000;

  export const EMBODIED_MIN = 0.0001;
  export const EMBODIED_MAX = 1.0;
  export const EMBODIED_DEFAULT = 0.02;

  export const WUE_MIN = 0;
  export const WUE_MAX = 5;
  export const WUE_DEFAULT = 1.5;

  // Presets régionaux pour le mix électrique (gCO₂eq/kWh).
  // Sources : Ember Climate (2024), ADEME Base Carbone v23.0 pour FR.
  export type RegionCode = 'FR' | 'SE' | 'GB' | 'IE' | 'NL' | 'DE' | 'US' | 'COAL';

  export const REGION_PRESETS: Record<RegionCode, { label: string; ifValue: number }> = {
    FR: { label: 'France', ifValue: 56 },
    SE: { label: 'Suède', ifValue: 45 },
    GB: { label: 'Royaume-Uni', ifValue: 260 },
    IE: { label: 'Irlande', ifValue: 350 },
    NL: { label: 'Pays-Bas', ifValue: 300 },
    DE: { label: 'Allemagne', ifValue: 380 },
    US: { label: 'États-Unis (moy.)', ifValue: 400 },
    COAL: { label: 'Charbon (worst-case)', ifValue: 633 }
  };

  export interface LeverState {
    modelId: string;
    tokensOut: number;
    pue: number;
    ifMix: number;
    embodied: number;
    wue: number;
    region: RegionCode | null;
    touched: {
      model: boolean;
      tokens: boolean;
      pue: boolean;
      mix: boolean;
      embodied: boolean;
      wue: boolean;
    };
  }

  export function makeInitialLeverState(
    baselineModelId: string,
    baselineTokensOut: number
  ): LeverState {
    return {
      modelId: baselineModelId,
      tokensOut: baselineTokensOut,
      pue: PUE_DEFAULT,
      ifMix: MIX_DEFAULT,
      embodied: EMBODIED_DEFAULT,
      wue: WUE_DEFAULT,
      region: null,
      touched: {
        model: false,
        tokens: false,
        pue: false,
        mix: false,
        embodied: false,
        wue: false
      }
    };
  }
</script>

<script lang="ts">
  import { Cpu, Globe, Server, Zap, Hash, CircuitBoard, Droplet, RotateCcw } from '@lucide/svelte';
  import type { ModelPresetDto } from '$lib/api';

  type Props = {
    models: ModelPresetDto[];
    baselineModelId: string;
    baselineTokensOut: number;
    state: LeverState;
    onreset: () => void;
  };

  let {
    models,
    baselineModelId,
    baselineTokensOut,
    state = $bindable(),
    onreset
  }: Props = $props();

  function touch(key: keyof LeverState['touched']) {
    state.touched = { ...state.touched, [key]: true };
  }

  function onRegionChange(region: RegionCode) {
    state.region = region;
    state.ifMix = REGION_PRESETS[region].ifValue;
    touch('mix');
  }

  function nFmt(value: number, digits = 2): string {
    return new Intl.NumberFormat('fr-FR', { maximumFractionDigits: digits }).format(value);
  }

  const anyTouched = $derived(Object.values(state.touched).some((v) => v));
</script>

<aside class="lever-panel" aria-label="Panneau des leviers">
  <header class="ph">
    <div>
      <div class="eyebrow">7 leviers · « Et si...? »</div>
      <h2>Modifiez un curseur pour voir l'impact</h2>
    </div>
    {#if anyTouched}
      <button class="reset-btn" type="button" onclick={onreset}>
        <RotateCcw size={12} strokeWidth={1.8} /> Réinitialiser
      </button>
    {/if}
  </header>

  <!-- ─── Lever 1 : Modèle ─────────────────────────────────────────── -->
  <section class="lever" class:on={state.touched.model}>
    <label for="lev-model" class="lev-label">
      <span class="ico"><Cpu size={13} strokeWidth={1.8} /></span>
      Modèle (override)
      <span class="lev-status">
        {state.touched.model ? 'Modifié' : `Baseline : ${baselineModelId}`}
      </span>
    </label>
    <select
      id="lev-model"
      class="lev-select"
      bind:value={state.modelId}
      onchange={() => touch('model')}
    >
      {#each models as m (m.id)}
        <option value={m.id}>{m.display_name} · {m.provider}</option>
      {/each}
    </select>
  </section>

  <!-- ─── Lever 2 : Région DC ──────────────────────────────────────── -->
  <section class="lever" class:on={state.touched.mix && state.region !== null}>
    <fieldset class="region-fs">
      <legend class="lev-label">
        <span class="ico"><Globe size={13} strokeWidth={1.8} /></span>
        Région datacenter
        <span class="lev-status">
          {state.region
            ? `${REGION_PRESETS[state.region].label} · ${REGION_PRESETS[state.region].ifValue} g/kWh`
            : 'Aucun preset appliqué'}
        </span>
      </legend>
      <div class="region-grid">
        {#each Object.entries(REGION_PRESETS) as [code, p] (code)}
          <label class="region-radio" class:on={state.region === code}>
            <input
              type="radio"
              name="lev-region"
              value={code}
              checked={state.region === code}
              onchange={() => onRegionChange(code as RegionCode)}
            />
            <span class="r-label">{code}</span>
            <span class="r-value mono">{p.ifValue}</span>
          </label>
        {/each}
      </div>
    </fieldset>
  </section>

  <!-- ─── Lever 3 : PUE ─────────────────────────────────────────────── -->
  <section class="lever" class:on={state.touched.pue}>
    <label for="lev-pue" class="lev-label">
      <span class="ico"><Server size={13} strokeWidth={1.8} /></span>
      PUE datacenter
      <span class="lev-status mono">
        {nFmt(state.pue, 2)}
      </span>
    </label>
    <input
      id="lev-pue"
      class="lev-slider"
      type="range"
      min={PUE_MIN}
      max={PUE_MAX}
      step="0.01"
      bind:value={state.pue}
      oninput={() => touch('pue')}
      aria-valuemin={PUE_MIN}
      aria-valuemax={PUE_MAX}
      aria-valuenow={state.pue}
    />
    <div class="slider-axis">
      <span>{PUE_MIN} (idéal)</span>
      <span>{PUE_MAX} (peu efficace)</span>
    </div>
  </section>

  <!-- ─── Lever 4 : Mix élec ────────────────────────────────────────── -->
  <section class="lever" class:on={state.touched.mix}>
    <label for="lev-mix" class="lev-label">
      <span class="ico"><Zap size={13} strokeWidth={1.8} /></span>
      Mix électrique
      <span class="lev-status mono">{nFmt(state.ifMix, 0)} gCO₂/kWh</span>
    </label>
    <input
      id="lev-mix"
      class="lev-slider"
      type="range"
      min={MIX_MIN}
      max={MIX_MAX}
      step="1"
      bind:value={state.ifMix}
      oninput={() => {
        state.region = null;
        touch('mix');
      }}
      aria-valuemin={MIX_MIN}
      aria-valuemax={MIX_MAX}
      aria-valuenow={state.ifMix}
    />
    <div class="slider-axis">
      <span>{MIX_MIN} g (renouv.)</span>
      <span>{MIX_MAX} g (charbon)</span>
    </div>
  </section>

  <!-- ─── Lever 5 : Tokens out ──────────────────────────────────────── -->
  <section class="lever" class:on={state.touched.tokens}>
    <label for="lev-tokens" class="lev-label">
      <span class="ico"><Hash size={13} strokeWidth={1.8} /></span>
      Tokens de sortie (override)
      <span class="lev-status">
        {state.touched.tokens ? 'Modifié' : `Baseline : ${baselineTokensOut}`}
      </span>
    </label>
    <input
      id="lev-tokens"
      class="lev-number mono"
      type="number"
      min={TOKENS_OUT_MIN}
      max={TOKENS_OUT_MAX}
      bind:value={state.tokensOut}
      oninput={() => touch('tokens')}
      aria-valuemin={TOKENS_OUT_MIN}
      aria-valuemax={TOKENS_OUT_MAX}
      aria-valuenow={state.tokensOut}
    />
  </section>

  <!-- ─── Lever 6 : Embodied ─────────────────────────────────────────── -->
  <section class="lever" class:on={state.touched.embodied}>
    <label for="lev-emb" class="lev-label">
      <span class="ico"><CircuitBoard size={13} strokeWidth={1.8} /></span>
      Embodied carbon / requête
      <span class="lev-status mono">{nFmt(state.embodied, 4)} g</span>
    </label>
    <input
      id="lev-emb"
      class="lev-slider"
      type="range"
      min={EMBODIED_MIN}
      max={EMBODIED_MAX}
      step="0.0001"
      bind:value={state.embodied}
      oninput={() => touch('embodied')}
      aria-valuemin={EMBODIED_MIN}
      aria-valuemax={EMBODIED_MAX}
      aria-valuenow={state.embodied}
    />
    <div class="slider-axis">
      <span>{EMBODIED_MIN} g</span>
      <span>{EMBODIED_MAX} g</span>
    </div>
  </section>

  <!-- ─── Lever 7 : WUE ──────────────────────────────────────────────── -->
  <section class="lever" class:on={state.touched.wue}>
    <label for="lev-wue" class="lev-label">
      <span class="ico"><Droplet size={13} strokeWidth={1.8} /></span>
      WUE refroidissement
      <span class="lev-status mono">{nFmt(state.wue, 2)} L/kWh</span>
    </label>
    <input
      id="lev-wue"
      class="lev-slider"
      type="range"
      min={WUE_MIN}
      max={WUE_MAX}
      step="0.05"
      bind:value={state.wue}
      oninput={() => touch('wue')}
      aria-valuemin={WUE_MIN}
      aria-valuemax={WUE_MAX}
      aria-valuenow={state.wue}
    />
    <div class="slider-axis">
      <span>{WUE_MIN} (sec)</span>
      <span>{WUE_MAX} (humide)</span>
    </div>
  </section>
</aside>

<style>
  .lever-panel {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 22px;
    background: linear-gradient(180deg, rgba(255, 255, 255, 0.025), rgba(255, 255, 255, 0.005));
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
    min-width: 0;
  }

  .ph {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    gap: 12px;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--border);
  }
  .ph .eyebrow {
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--ivory-3);
    margin-bottom: 6px;
  }
  .ph h2 {
    font: 400 22px/1.15 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.01em;
    margin: 0;
  }
  .reset-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 6px 10px;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: var(--radius-pill);
    font: 500 11px/1 var(--font-ui);
    color: var(--ivory-2);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
    white-space: nowrap;
  }
  .reset-btn:hover {
    border-color: var(--border-hi);
    color: var(--ivory);
  }

  .lever {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 14px 14px 12px;
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    transition: border-color var(--dur-base) var(--ease);
  }
  .lever.on {
    border-color: rgba(197, 240, 74, 0.35);
    background: rgba(197, 240, 74, 0.03);
  }

  .lev-label {
    display: flex;
    align-items: center;
    gap: 8px;
    font: 500 11px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--ivory-2);
  }
  .lev-label .ico {
    display: inline-grid;
    place-items: center;
    width: 22px;
    height: 22px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--ivory-3);
    flex-shrink: 0;
  }
  .lever.on .lev-label .ico {
    background: var(--lime-soft);
    border-color: rgba(197, 240, 74, 0.3);
    color: var(--lime);
  }
  .lev-status {
    margin-left: auto;
    font: 400 11px/1 var(--font-ui);
    color: var(--ivory-3);
    text-transform: none;
    letter-spacing: 0;
  }
  .lev-status.mono {
    font-family: var(--font-mono);
    color: var(--lime);
    font-weight: 600;
  }
  .lever.on .lev-status:not(.mono) {
    color: var(--lime);
  }

  .lev-select {
    width: 100%;
    padding: 9px 12px;
    background: rgba(0, 0, 0, 0.4);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--ivory);
    font: 400 13px/1.2 var(--font-ui);
    cursor: pointer;
  }
  .lev-select:focus {
    outline: 2px solid var(--lime);
    outline-offset: 1px;
    border-color: rgba(197, 240, 74, 0.4);
  }

  .lev-number {
    width: 120px;
    padding: 7px 10px;
    background: rgba(0, 0, 0, 0.4);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--ivory);
    font: 600 13px/1 var(--font-mono);
    text-align: right;
  }
  .lev-number:focus {
    outline: 2px solid var(--lime);
    outline-offset: 1px;
  }

  .lev-slider {
    -webkit-appearance: none;
    appearance: none;
    width: 100%;
    height: 4px;
    border-radius: 999px;
    background: linear-gradient(
      90deg,
      var(--lime) 0%,
      var(--lime) var(--pct, 0%),
      rgba(255, 255, 255, 0.08) var(--pct, 0%),
      rgba(255, 255, 255, 0.08) 100%
    );
    margin: 4px 0 0;
    cursor: pointer;
  }
  .lev-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--lime);
    border: 2px solid var(--ink);
    cursor: pointer;
    box-shadow: 0 0 0 0 var(--lime-glow);
    transition: box-shadow 200ms var(--ease);
  }
  .lev-slider:focus::-webkit-slider-thumb {
    box-shadow: 0 0 0 4px var(--lime-glow);
  }
  .lev-slider::-moz-range-thumb {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--lime);
    border: 2px solid var(--ink);
    cursor: pointer;
  }
  .lev-slider:focus {
    outline: none;
  }
  .lev-slider:focus::-moz-range-thumb {
    box-shadow: 0 0 0 4px var(--lime-glow);
  }

  .slider-axis {
    display: flex;
    justify-content: space-between;
    font: 400 9px/1 var(--font-mono);
    color: var(--ivory-4);
    letter-spacing: 0.04em;
    margin-top: 4px;
  }

  /* Region radio grid */
  .region-fs {
    border: none;
    padding: 0;
    margin: 0;
  }
  .region-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 6px;
    margin-top: 6px;
  }
  .region-radio {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
    padding: 8px 4px;
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
    text-align: center;
  }
  .region-radio:hover {
    border-color: var(--border-hi);
  }
  .region-radio input {
    position: absolute;
    inset: 0;
    opacity: 0;
    cursor: pointer;
  }
  .region-radio.on {
    background: var(--lime-soft);
    border-color: rgba(197, 240, 74, 0.5);
  }
  .r-label {
    font: 600 11px/1 var(--font-ui);
    color: var(--ivory);
    letter-spacing: 0.04em;
  }
  .region-radio.on .r-label {
    color: var(--lime);
  }
  .r-value {
    font: 400 9px/1 var(--font-mono);
    color: var(--ivory-3);
  }
  .region-radio.on .r-value {
    color: var(--lime);
  }
</style>
