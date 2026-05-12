<script lang="ts">
  import {
    Cpu,
    MessageSquare,
    ChevronsUpDown,
    ArrowDownToLine,
    ArrowUpFromLine,
    SlidersHorizontal,
    Server,
    Zap,
    Edit2,
    Sparkles,
    Bookmark,
    GitCompare,
    Check
  } from 'lucide-svelte';
  import type { Calibration, ModelPresetDto } from '$lib/api';

  type Props = {
    models: ModelPresetDto[];
    selectedModelId: string;
    prompt: string;
    tokensOut: number;
    estimating: boolean;
    onsubmit: () => void;
  };

  let {
    models,
    selectedModelId = $bindable(),
    prompt = $bindable(),
    tokensOut = $bindable(),
    estimating,
    onsubmit
  }: Props = $props();

  let popoverOpen = $state(false);
  let popoverEl: HTMLDivElement | undefined = $state();

  const selectedModel = $derived(models.find((m) => m.id === selectedModelId) ?? null);

  // Heuristique simple : ~4 caractères / token (cf. OpenAI tokenizer
  // typique). Pour une mesure exacte il faudrait BPE côté Rust — c'est
  // l'objet du chantier futur EF-M2-01.
  const tokensIn = $derived(Math.max(1, Math.ceil(prompt.length / 4)));

  const calibrationLabel: Record<Calibration, string> = {
    validated: 'Validé',
    indicative: 'Indicatif',
    extrapolated: 'Extrapolé'
  };

  function selectModel(id: string) {
    selectedModelId = id;
    popoverOpen = false;
  }

  function handleSubmit(e: Event) {
    e.preventDefault();
    if (estimating || !selectedModelId || tokensIn <= 0 || tokensOut <= 0) {
      return;
    }
    onsubmit();
  }

  function handleClickOutside(e: MouseEvent) {
    if (!popoverOpen) return;
    if (popoverEl && !popoverEl.contains(e.target as Node)) {
      popoverOpen = false;
    }
  }

  $effect(() => {
    if (!popoverOpen) return () => {};
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  });
</script>

<form class="composer" onsubmit={handleSubmit} novalidate>
  <div class="composer-row">
    <div class="field">
      <label class="field-label" id="lbl-model" for="model-select">
        <Cpu size={12} strokeWidth={1.8} />Modèle
      </label>
      <div class="select-wrap" bind:this={popoverEl}>
        <button
          id="model-select"
          class="select-shell"
          type="button"
          aria-haspopup="listbox"
          aria-expanded={popoverOpen}
          aria-labelledby="lbl-model"
          onclick={() => (popoverOpen = !popoverOpen)}
        >
          <div class="left">
            <div class="model-name">
              {selectedModel?.display_name ?? '—'}
            </div>
            <div class="model-prov">
              {#if selectedModel}
                {selectedModel.provider} · ~{selectedModel.approx_params_billions}B paramètres
              {:else}
                Sélectionnez un modèle
              {/if}
            </div>
          </div>
          <div class="spacer"></div>
          {#if selectedModel}
            <span
              class="score-pip"
              class:c={selectedModel.calibration === 'indicative'}
              class:e={selectedModel.calibration === 'extrapolated'}
              title="Statut de calibration scientifique"
            >
              {calibrationLabel[selectedModel.calibration]}
            </span>
          {/if}
          <span class="chev"><ChevronsUpDown size={16} strokeWidth={1.6} /></span>
        </button>

        {#if popoverOpen}
          <ul class="model-popover" role="listbox" aria-label="Modèles disponibles">
            {#each models as model (model.id)}
              <li>
                <button
                  type="button"
                  class="model-row"
                  class:on={model.id === selectedModelId}
                  role="option"
                  aria-selected={model.id === selectedModelId}
                  onclick={() => selectModel(model.id)}
                >
                  <div class="model-row-main">
                    <span class="model-row-name">{model.display_name}</span>
                    <span class="model-row-prov">
                      {model.provider} · ~{model.approx_params_billions}B
                    </span>
                  </div>
                  <span
                    class="score-pip sm"
                    class:c={model.calibration === 'indicative'}
                    class:e={model.calibration === 'extrapolated'}
                  >
                    {calibrationLabel[model.calibration]}
                  </span>
                  {#if model.id === selectedModelId}
                    <span class="check"><Check size={14} strokeWidth={2} /></span>
                  {/if}
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      </div>
    </div>
  </div>

  <label class="field-label" for="prompt-textarea">
    <MessageSquare size={12} strokeWidth={1.8} />Prompt
  </label>
  <div class="prompt-area">
    <textarea
      id="prompt-textarea"
      bind:value={prompt}
      placeholder="Écrivez votre prompt…"
      rows="3"
      aria-describedby="prompt-meta"
    ></textarea>
    <div class="prompt-meta" id="prompt-meta">
      <span class="item">
        <ArrowDownToLine size={12} strokeWidth={1.8} />Tokens entrée
        <b>{tokensIn}</b>
      </span>
      <span class="item">
        <ArrowUpFromLine size={12} strokeWidth={1.8} />Tokens sortie estimés
        <input
          type="number"
          min="1"
          max="100000"
          class="tokens-input"
          bind:value={tokensOut}
          aria-label="Tokens de sortie estimés"
        />
      </span>
      <span class="grow"></span>
      <span class="item muted">
        <SlidersHorizontal size={12} strokeWidth={1.8} />Ajuster
      </span>
    </div>
  </div>

  <div class="context-row">
    <div class="context-card">
      <div class="ico">
        <Server size={18} strokeWidth={1.6} />
      </div>
      <div class="col">
        <div class="ll">Datacenter (auto)</div>
        <div class="vv">US-East · Virginie</div>
        <div class="vm">PUE 1,30 · détecté via géoloc</div>
      </div>
      <span class="edit" aria-hidden="true">
        <Edit2 size={14} strokeWidth={1.8} />
      </span>
    </div>
    <div class="context-card">
      <div class="ico blue">
        <Zap size={18} strokeWidth={1.6} />
      </div>
      <div class="col">
        <div class="ll">Mix électrique · LIVE</div>
        <div class="vv">412 gCO₂eq / kWh</div>
        <div class="vm">Electricity Maps · maj 14h32 UTC</div>
      </div>
      <span class="edit" aria-hidden="true">
        <Edit2 size={14} strokeWidth={1.8} />
      </span>
    </div>
  </div>

  <div class="composer-actions">
    <button
      class="btn-primary"
      type="submit"
      disabled={estimating || !selectedModelId || tokensOut <= 0}
    >
      <Sparkles size={16} strokeWidth={2} />
      {estimating ? 'Estimation en cours…' : "Estimer l'impact"}
    </button>
    <button class="btn-ghost" type="button" disabled aria-disabled="true">
      <Bookmark size={14} strokeWidth={1.8} />Sauvegarder
    </button>
    <button class="btn-ghost" type="button" disabled aria-disabled="true">
      <GitCompare size={14} strokeWidth={1.8} />Comparer
    </button>
    <span class="kbd-hint" aria-hidden="true">
      <kbd class="kbd">Ctrl</kbd><kbd class="kbd">↵</kbd> pour estimer
    </span>
  </div>
</form>

<style>
  .composer {
    background: linear-gradient(180deg, rgba(255, 255, 255, 0.04), rgba(255, 255, 255, 0.015));
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
    padding: 28px 32px;
    margin-bottom: 24px;
    box-shadow:
      0 1px 0 rgba(255, 255, 255, 0.04) inset,
      0 32px 64px -32px rgba(0, 0, 0, 0.6);
    position: relative;
    overflow: hidden;
  }
  .composer::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 1px;
    background: linear-gradient(
      90deg,
      transparent 0%,
      rgba(197, 240, 74, 0.6) 50%,
      transparent 100%
    );
    background-size: 200% 100%;
    animation: beam 6s linear infinite;
  }
  @keyframes beam {
    from {
      background-position: 200% 0;
    }
    to {
      background-position: -200% 0;
    }
  }

  .composer-row {
    display: flex;
    gap: 14px;
    margin-bottom: 20px;
  }
  .field {
    flex: 1;
  }
  .field-label {
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--ivory-3);
    margin-bottom: 8px;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .select-wrap {
    position: relative;
  }
  .select-shell {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    padding: 12px 16px;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition:
      border-color var(--dur-base) var(--ease),
      background var(--dur-base) var(--ease);
    text-align: left;
  }
  .select-shell:hover {
    border-color: var(--border-hi);
    background: rgba(255, 255, 255, 0.015);
  }
  .select-shell .left {
    flex: 0 1 auto;
    min-width: 0;
  }
  .select-shell .model-name {
    font: 500 15px/1.2 var(--font-ui);
    color: var(--ivory);
  }
  .select-shell .model-prov {
    font: 400 12px/1.4 var(--font-ui);
    color: var(--ivory-3);
    margin-top: 2px;
  }
  .select-shell .spacer {
    flex: 1;
  }
  .select-shell .chev {
    color: var(--ivory-3);
    display: inline-flex;
    align-items: center;
  }

  .score-pip {
    display: inline-flex;
    align-items: center;
    padding: 5px 10px;
    background: rgba(197, 240, 74, 0.1);
    border: 1px solid rgba(197, 240, 74, 0.25);
    border-radius: var(--radius-pill);
    font: 600 11px/1 var(--font-mono);
    color: var(--lime);
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }
  .score-pip.c {
    background: rgba(245, 183, 105, 0.1);
    border-color: rgba(245, 183, 105, 0.25);
    color: var(--amber);
  }
  .score-pip.e {
    background: rgba(240, 108, 90, 0.1);
    border-color: rgba(240, 108, 90, 0.25);
    color: var(--coral);
  }
  .score-pip.sm {
    padding: 3px 8px;
    font-size: 10px;
  }

  .model-popover {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    right: 0;
    background: var(--ink-3);
    border: 1px solid var(--border-hi);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-modal);
    list-style: none;
    padding: 4px;
    z-index: 50;
    max-height: 320px;
    overflow-y: auto;
  }
  .model-row {
    display: flex;
    width: 100%;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    border: none;
    background: transparent;
    cursor: pointer;
    text-align: left;
    transition: background var(--dur-fast) var(--ease);
  }
  .model-row:hover {
    background: var(--surface-hi);
  }
  .model-row.on {
    background: var(--lime-soft);
  }
  .model-row-main {
    flex: 1;
    min-width: 0;
  }
  .model-row-name {
    display: block;
    font: 500 13px/1.2 var(--font-ui);
    color: var(--ivory);
  }
  .model-row-prov {
    display: block;
    font: 400 11px/1.4 var(--font-mono);
    color: var(--ivory-3);
    margin-top: 1px;
  }
  .model-row .check {
    color: var(--lime);
    display: inline-flex;
  }

  .prompt-area {
    position: relative;
    border: 1px solid var(--border);
    background: rgba(0, 0, 0, 0.25);
    border-radius: var(--radius-md);
    padding: 16px 18px;
    margin-bottom: 16px;
    transition:
      border-color var(--dur-base) var(--ease),
      box-shadow var(--dur-slow) var(--ease);
  }
  .prompt-area:focus-within {
    border-color: rgba(197, 240, 74, 0.3);
    box-shadow: 0 0 0 4px rgba(197, 240, 74, 0.06);
  }
  .prompt-area textarea {
    width: 100%;
    min-height: 80px;
    background: transparent;
    border: none;
    outline: none;
    resize: vertical;
    color: var(--ivory);
    font: 400 15px/1.6 var(--font-ui);
  }
  .prompt-area textarea::placeholder {
    color: var(--ivory-4);
  }

  .prompt-meta {
    display: flex;
    gap: 24px;
    align-items: center;
    padding-top: 14px;
    border-top: 1px dashed var(--border);
    font: 500 12px/1 var(--font-mono);
    flex-wrap: wrap;
  }
  .prompt-meta .item {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: var(--ivory-3);
  }
  .prompt-meta .item b {
    color: var(--ivory);
    font-weight: 600;
    margin-left: 2px;
  }
  .prompt-meta .item.muted {
    cursor: default;
  }
  .prompt-meta .grow {
    flex: 1;
  }
  .tokens-input {
    width: 70px;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--ivory);
    font: 600 12px/1 var(--font-mono);
    padding: 4px 8px;
    text-align: right;
    margin-left: 4px;
  }
  .tokens-input:focus {
    outline: 2px solid var(--lime);
    outline-offset: 1px;
  }

  .context-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
    margin-bottom: 20px;
  }
  .context-card {
    display: flex;
    gap: 14px;
    align-items: center;
    padding: 14px 16px;
    background: rgba(0, 0, 0, 0.18);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    transition: all 250ms var(--ease);
  }
  .context-card:hover {
    background: rgba(0, 0, 0, 0.3);
    border-color: var(--border-hi);
    transform: translateY(-2px);
  }
  .context-card .ico {
    width: 36px;
    height: 36px;
    display: grid;
    place-items: center;
    background: var(--lime-soft);
    border-radius: 10px;
    color: var(--lime);
    flex-shrink: 0;
    transition: transform 400ms var(--ease-spring);
  }
  .context-card:hover .ico {
    transform: rotate(-6deg) scale(1.08);
  }
  .context-card .ico.blue {
    background: rgba(126, 182, 255, 0.12);
    color: var(--blue);
  }
  .context-card .col {
    flex: 1;
    min-width: 0;
  }
  .context-card .ll {
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--ivory-3);
    margin-bottom: 4px;
  }
  .context-card .vv {
    font: 500 14px/1.3 var(--font-ui);
    color: var(--ivory);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .context-card .vm {
    font: 400 11px/1 var(--font-mono);
    color: var(--ivory-3);
    margin-top: 2px;
  }
  .context-card .edit {
    color: var(--ivory-3);
    cursor: pointer;
  }

  .composer-actions {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }
  .btn-primary {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    height: 44px;
    padding: 0 22px;
    background: var(--lime);
    color: var(--ink);
    border: none;
    border-radius: var(--radius-md);
    font: 600 14px/1 var(--font-ui);
    letter-spacing: 0.01em;
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
    box-shadow:
      0 0 0 0 var(--lime-glow),
      0 6px 24px -8px rgba(197, 240, 74, 0.6);
  }
  .btn-primary:hover:not(:disabled) {
    transform: translateY(-1px);
    box-shadow:
      0 0 0 4px rgba(197, 240, 74, 0.15),
      0 8px 32px -8px rgba(197, 240, 74, 0.7);
  }
  .btn-primary:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .btn-ghost {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    height: 44px;
    padding: 0 16px;
    background: transparent;
    color: var(--ivory-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    font: 500 13px/1 var(--font-ui);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
  }
  .btn-ghost:hover:not(:disabled) {
    border-color: var(--border-hi);
    color: var(--ivory);
  }
  .btn-ghost:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .kbd-hint {
    margin-left: auto;
    font: 400 11px/1 var(--font-mono);
    color: var(--ivory-4);
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .kbd {
    padding: 2px 6px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: rgba(0, 0, 0, 0.3);
    color: var(--ivory-3);
    font: 500 10px/1 var(--font-mono);
  }
</style>
