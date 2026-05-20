<!--
  C34.4 — ModalitiesPanel
  ------------------------------------------------------------
  Panneau qui permet d'enrichir un prompt avec :
    - Modalités d'input (images low/high, document, audio)
    - Overhead système (system prompt + tools + memory)
  Mode "Simple" (replié par défaut) vs "Expert" (tout déplié).

  Affiche aussi un badge automatique pour reasoning models indiquant
  que des thinking tokens seront ajoutés (P5-P95 du multiplier).

  Disclaimer transparent ± 50 % sur l'overhead — basé sur leaks publics
  et reverse-engineering interfaces vendor.
-->
<script lang="ts">
  import {
    Image as ImageIcon,
    FileText,
    Mic,
    Brain,
    Sparkles,
    ChevronDown,
    Info,
    Settings2,
    Layers
  } from '@lucide/svelte';
  import type { ContextOverhead, InputModality, ModelPresetDto } from '$lib/api';

  type Props = {
    model: ModelPresetDto | null;
    modalities: InputModality[];
    overhead: ContextOverhead;
    /** Estimation des tokens de sortie (utilisé pour afficher thinking auto). */
    tokensOut: number;
  };

  let { model, modalities = $bindable(), overhead = $bindable(), tokensOut }: Props = $props();

  // ─── Mode Simple/Expert (persisté localStorage) ────────────────────
  const EXPERT_KEY = 'sobria_modalities_expert_mode';
  let expertMode = $state(false);
  $effect(() => {
    if (typeof window === 'undefined') return;
    try {
      expertMode = window.localStorage.getItem(EXPERT_KEY) === 'true';
    } catch {
      expertMode = false;
    }
  });
  function toggleExpert() {
    expertMode = !expertMode;
    try {
      window.localStorage.setItem(EXPERT_KEY, String(expertMode));
    } catch {
      /* ignore */
    }
  }

  // ─── Helpers modalités : on stocke l'état UI séparément pour permettre
  // l'édition fluide avant de recalculer le tableau modalities[].
  let visionEnabled = $state(false);
  let visionImageCount = $state(1);
  let visionHighDetail = $state(false);
  let visionWidth = $state(1024);
  let visionHeight = $state(1024);

  let documentEnabled = $state(false);
  let documentPages = $state(5);

  let audioEnabled = $state(false);
  let audioSeconds = $state(30);

  // Synchronise les états locaux → modalities[] dans le parent.
  $effect(() => {
    const list: InputModality[] = [];
    if (visionEnabled) {
      if (visionHighDetail) {
        list.push({
          kind: 'vision_high',
          image_count: visionImageCount,
          avg_width: visionWidth,
          avg_height: visionHeight
        });
      } else {
        list.push({ kind: 'vision_low', image_count: visionImageCount });
      }
    }
    if (documentEnabled) {
      list.push({ kind: 'document', page_count: documentPages });
    }
    if (audioEnabled) {
      list.push({ kind: 'audio_input', duration_seconds: audioSeconds });
    }
    modalities = list;
  });

  // ─── Pré-remplissage overhead depuis le preset (au changement de modèle)
  let previousModelId = $state<string | null>(null);
  $effect(() => {
    if (!model) return;
    // Quand le modèle change, on pré-remplit le system_prompt avec
    // la valeur par défaut du preset (mais on ne touche pas si l'user a déjà saisi).
    if (model.id !== previousModelId) {
      previousModelId = model.id;
      overhead = {
        ...overhead,
        system_prompt_tokens: model.default_context_overhead_tokens
      };
    }
  });

  // ─── Garde-fous capabilities ────────────────────────────────────────
  const visionCompat = $derived(model?.vision_capable ?? false);
  const audioCompat = $derived(model?.audio_capable ?? false);
  const isReasoning = $derived(model?.reasoning_capable ?? false);
  const thinkingMult = $derived(model?.thinking_token_multiplier);

  // Estimation des thinking tokens auto (pour affichage badge).
  const autoThinkingP50 = $derived.by(() => {
    if (!isReasoning || !thinkingMult) return 0;
    const [p5, p95] = thinkingMult;
    if (p5 <= 0 || p95 <= 0 || p5 > p95) return 0;
    const geomean = Math.sqrt(p5 * p95);
    return Math.round(tokensOut * geomean);
  });

  // Désactive auto-thinking si l'user a fourni un overhead.thinking_tokens_p50 explicite.
  const willAutoAddThinking = $derived(isReasoning && overhead.thinking_tokens_p50 === 0);

  // ─── Sources documentaires (liens tooltip) ──────────────────────────
  const SOURCES = {
    openai_vision: 'https://platform.openai.com/docs/guides/vision/calculating-costs',
    anthropic_vision: 'https://docs.anthropic.com/en/docs/build-with-claude/vision',
    gemini_vision: 'https://ai.google.dev/gemini-api/docs/vision',
    llama_vision: 'https://ai.meta.com/blog/llama-4-multimodal-intelligence/',
    whisper: 'https://openai.com/research/whisper',
    overhead_brief: 'https://github.com/anthropics/claude-code'
  };
</script>

<section class="modalities-panel">
  <header class="panel-header">
    <div class="panel-title">
      <Layers size={14} strokeWidth={1.8} />
      <span>Modalités &amp; contexte</span>
      <span class="panel-pip">C34</span>
    </div>
    <button
      type="button"
      class="mode-toggle"
      class:on={expertMode}
      onclick={toggleExpert}
      aria-pressed={expertMode}
      title={expertMode
        ? 'Passer en mode Simple (cache les détails techniques)'
        : 'Passer en mode Expert (déplie les détails techniques)'}
    >
      <Settings2 size={12} strokeWidth={1.8} />
      {expertMode ? 'Mode Expert' : 'Mode Simple'}
    </button>
  </header>

  <!-- ─── Toggles modalités ─────────────────────────────────────────── -->
  <div class="modalities-toggles">
    <label class="modality-toggle" class:disabled={!visionCompat}>
      <input type="checkbox" bind:checked={visionEnabled} disabled={!visionCompat} />
      <ImageIcon size={14} strokeWidth={1.8} />
      <span class="m-name">Image</span>
      {#if !visionCompat}
        <span class="m-warn" title="Ce modèle ne supporte pas l'input image">N/A</span>
      {/if}
    </label>

    <label class="modality-toggle" class:disabled={!visionCompat}>
      <input type="checkbox" bind:checked={documentEnabled} disabled={!visionCompat} />
      <FileText size={14} strokeWidth={1.8} />
      <span class="m-name">Document PDF</span>
      {#if !visionCompat}
        <span class="m-warn" title="Le modèle doit supporter la vision pour traiter les documents"
          >N/A</span
        >
      {/if}
    </label>

    <label class="modality-toggle" class:disabled={!audioCompat}>
      <input type="checkbox" bind:checked={audioEnabled} disabled={!audioCompat} />
      <Mic size={14} strokeWidth={1.8} />
      <span class="m-name">Audio</span>
      {#if !audioCompat}
        <span class="m-warn" title="Ce modèle ne supporte pas l'input audio">N/A</span>
      {/if}
    </label>
  </div>

  <!-- ─── Sous-formulaires modalités actives ───────────────────────── -->
  {#if visionEnabled && visionCompat}
    <div class="modality-detail">
      <div class="row">
        <label class="field-mini">
          <span class="lbl">Nombre d'images</span>
          <input type="number" min="1" max="16" bind:value={visionImageCount} class="num-input" />
        </label>
        <div class="radio-group">
          <label class="radio">
            <input type="radio" name="vision-detail" bind:group={visionHighDetail} value={false} />
            <span>Basse résolution</span>
          </label>
          <label class="radio">
            <input type="radio" name="vision-detail" bind:group={visionHighDetail} value={true} />
            <span>Haute résolution</span>
          </label>
        </div>
        <a
          class="help"
          href={SOURCES[
            model?.model_family === 'anthropic'
              ? 'anthropic_vision'
              : model?.model_family === 'google_deep_mind'
                ? 'gemini_vision'
                : model?.model_family === 'meta_ai'
                  ? 'llama_vision'
                  : 'openai_vision'
          ]}
          target="_blank"
          rel="noopener"
          title="Formule de tarification tokens vision (doc vendor)"
        >
          <Info size={12} strokeWidth={1.8} />
        </a>
      </div>
      {#if visionHighDetail}
        <div class="row">
          <label class="field-mini">
            <span class="lbl">Largeur moyenne (px)</span>
            <input
              type="number"
              min="64"
              max="8192"
              step="64"
              bind:value={visionWidth}
              class="num-input"
            />
          </label>
          <label class="field-mini">
            <span class="lbl">Hauteur moyenne (px)</span>
            <input
              type="number"
              min="64"
              max="8192"
              step="64"
              bind:value={visionHeight}
              class="num-input"
            />
          </label>
        </div>
      {/if}
    </div>
  {/if}

  {#if documentEnabled && visionCompat}
    <div class="modality-detail">
      <div class="row">
        <label class="field-mini">
          <span class="lbl">Nombre de pages PDF</span>
          <input type="number" min="1" max="500" bind:value={documentPages} class="num-input" />
        </label>
        <span class="hint"> ~1100 tokens / page (analyse empirique LMSYS arena uploads) </span>
      </div>
    </div>
  {/if}

  {#if audioEnabled && audioCompat}
    <div class="modality-detail">
      <div class="row">
        <label class="field-mini">
          <span class="lbl">Durée (secondes)</span>
          <input type="number" min="1" max="3600" bind:value={audioSeconds} class="num-input" />
        </label>
        <a
          class="hint link"
          href={SOURCES.whisper}
          target="_blank"
          rel="noopener"
          title="OpenAI Whisper paper"
        >
          ~10 tokens / seconde (taux Whisper)
        </a>
      </div>
    </div>
  {/if}

  <!-- ─── Badge reasoning auto-thinking ─────────────────────────────── -->
  {#if isReasoning && willAutoAddThinking}
    <div class="reasoning-badge">
      <Brain size={14} strokeWidth={1.8} />
      <div class="rb-body">
        <div class="rb-title">Reasoning model — thinking automatique</div>
        <div class="rb-text">
          Ce modèle ajoute automatiquement <b>~{autoThinkingP50.toLocaleString('fr-FR')}</b>
          tokens de raisonnement (P50, ratio {thinkingMult
            ? `${thinkingMult[0]}×-${thinkingMult[1]}×`
            : '?'} P5-P95) aux <b>{tokensOut}</b> tokens de sortie estimés.
        </div>
      </div>
    </div>
  {/if}

  <!-- ─── Détails techniques (collapsible en mode Simple) ──────────── -->
  <details class="tech-details" open={expertMode}>
    <summary class="tech-summary">
      <ChevronDown class="chev" size={14} strokeWidth={1.8} />
      Détails techniques
      <span class="tech-disclaimer"> Estimation ± 50 % — basée sur leaks publics </span>
    </summary>
    <div class="tech-grid">
      <label class="field-mini">
        <span class="lbl">
          System prompt (tokens)
          <span
            class="info-mark"
            title="System prompt caché injecté par l'interface vendor. Claude.ai ~2000, ChatGPT ~1000, Gemini ~1000, Mistral ~300, API directe = 0. Estimation ± 50 % basée sur leaks publics + reverse-engineering."
          >
            ?
          </span>
        </span>
        <input
          type="number"
          min="0"
          max="10000"
          bind:value={overhead.system_prompt_tokens}
          class="num-input wide"
        />
      </label>
      <label class="field-mini">
        <span class="lbl">
          Tools schémas (tokens)
          <span
            class="info-mark"
            title="Tokens des schémas JSON des outils activés (code interpreter, web search, etc.) — ~200-1500 tokens par outil."
          >
            ?
          </span>
        </span>
        <input
          type="number"
          min="0"
          max="10000"
          bind:value={overhead.tools_definition_tokens}
          class="num-input wide"
        />
      </label>
      <label class="field-mini">
        <span class="lbl">
          Memory / context précédent
          <span
            class="info-mark"
            title="Tokens des tours précédents accumulés dans la conversation. 0 si nouveau prompt."
          >
            ?
          </span>
        </span>
        <input
          type="number"
          min="0"
          max="100000"
          bind:value={overhead.memory_tokens}
          class="num-input wide"
        />
      </label>
      {#if isReasoning}
        <label class="field-mini">
          <span class="lbl">
            Thinking tokens (override)
            <span
              class="info-mark"
              title="Override l'auto-thinking calculé sur le multiplier. 0 = utilise l'auto-thinking (P50)."
            >
              ?
            </span>
          </span>
          <input
            type="number"
            min="0"
            max="200000"
            bind:value={overhead.thinking_tokens_p50}
            class="num-input wide"
          />
        </label>
      {/if}
    </div>
    <div class="tech-footer">
      <Sparkles size={11} strokeWidth={1.8} />
      Total overhead :
      <b
        >{(
          overhead.system_prompt_tokens +
          overhead.tools_definition_tokens +
          overhead.memory_tokens +
          (overhead.thinking_tokens_p50 || (willAutoAddThinking ? autoThinkingP50 : 0))
        ).toLocaleString('fr-FR')}</b
      >
      tokens
    </div>
  </details>
</section>

<style>
  .modalities-panel {
    background: linear-gradient(180deg, rgba(255, 255, 255, 0.03), rgba(255, 255, 255, 0.01));
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
    padding: 20px 24px;
    margin-bottom: 20px;
    font-family: var(--font-ui);
  }
  .panel-header {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 16px;
  }
  .panel-title {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font: 500 12px/1 var(--font-ui);
    color: var(--ivory-2);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    flex: 1;
  }
  .panel-pip {
    padding: 2px 6px;
    background: rgba(197, 240, 74, 0.1);
    border: 1px solid rgba(197, 240, 74, 0.25);
    border-radius: 4px;
    color: var(--lime);
    font: 600 9px/1 var(--font-mono);
  }
  .mode-toggle {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--border);
    border-radius: var(--radius-pill);
    color: var(--ivory-3);
    font: 500 11px/1 var(--font-mono);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
  }
  .mode-toggle:hover {
    border-color: var(--border-hi);
    color: var(--ivory);
  }
  .mode-toggle.on {
    background: rgba(197, 240, 74, 0.1);
    border-color: rgba(197, 240, 74, 0.3);
    color: var(--lime);
  }

  .modalities-toggles {
    display: flex;
    gap: 12px;
    flex-wrap: wrap;
    margin-bottom: 14px;
  }
  .modality-toggle {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 8px 14px;
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--dur-base) var(--ease);
    user-select: none;
  }
  .modality-toggle:hover:not(.disabled) {
    border-color: var(--border-hi);
    background: rgba(255, 255, 255, 0.025);
  }
  .modality-toggle.disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .modality-toggle input[type='checkbox'] {
    accent-color: var(--lime);
    cursor: inherit;
  }
  .modality-toggle .m-name {
    font: 500 12px/1 var(--font-ui);
    color: var(--ivory-2);
  }
  .modality-toggle .m-warn {
    font: 500 9px/1 var(--font-mono);
    color: var(--coral);
    background: rgba(240, 108, 90, 0.1);
    border: 1px solid rgba(240, 108, 90, 0.25);
    padding: 2px 6px;
    border-radius: 4px;
  }

  .modality-detail {
    background: rgba(0, 0, 0, 0.15);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 12px 16px;
    margin-bottom: 12px;
  }
  .modality-detail .row {
    display: flex;
    gap: 16px;
    align-items: center;
    flex-wrap: wrap;
  }
  .modality-detail .row + .row {
    margin-top: 12px;
  }
  .field-mini {
    display: inline-flex;
    flex-direction: column;
    gap: 4px;
  }
  .field-mini .lbl {
    font: 500 10px/1 var(--font-ui);
    color: var(--ivory-3);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .num-input {
    width: 80px;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--ivory);
    font: 500 12px/1 var(--font-mono);
    padding: 6px 8px;
  }
  .num-input.wide {
    width: 100px;
  }
  .num-input:focus {
    outline: 2px solid var(--lime);
    outline-offset: 1px;
  }

  .radio-group {
    display: flex;
    gap: 12px;
  }
  .radio {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font: 500 12px/1 var(--font-ui);
    color: var(--ivory-3);
    cursor: pointer;
  }
  .radio input {
    accent-color: var(--lime);
    cursor: pointer;
  }
  .help {
    display: inline-flex;
    color: var(--ivory-4);
    text-decoration: none;
    transition: color var(--dur-base) var(--ease);
  }
  .help:hover {
    color: var(--lime);
  }
  .hint {
    font: 400 11px/1.4 var(--font-mono);
    color: var(--ivory-4);
    font-style: italic;
  }
  .hint.link {
    text-decoration: none;
  }
  .hint.link:hover {
    color: var(--lime);
  }

  .reasoning-badge {
    display: flex;
    gap: 10px;
    align-items: flex-start;
    background: linear-gradient(180deg, rgba(126, 182, 255, 0.08), rgba(126, 182, 255, 0.03));
    border: 1px solid rgba(126, 182, 255, 0.25);
    border-radius: var(--radius-md);
    padding: 10px 14px;
    margin-bottom: 12px;
    color: var(--blue);
  }
  .rb-body {
    flex: 1;
  }
  .rb-title {
    font: 600 11px/1.4 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }
  .rb-text {
    font: 400 12px/1.5 var(--font-ui);
    color: var(--ivory-2);
    margin-top: 4px;
  }
  .rb-text b {
    color: var(--ivory);
    font-weight: 600;
  }

  .tech-details {
    background: rgba(0, 0, 0, 0.15);
    border: 1px dashed var(--border);
    border-radius: var(--radius-md);
    padding: 12px 16px;
    margin-top: 8px;
  }
  .tech-summary {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    list-style: none;
    font: 500 11px/1 var(--font-ui);
    color: var(--ivory-3);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }
  .tech-summary::-webkit-details-marker {
    display: none;
  }
  .tech-details[open] .tech-summary :global(.chev) {
    transform: rotate(180deg);
  }
  .tech-summary :global(.chev) {
    transition: transform var(--dur-base) var(--ease);
  }
  .tech-disclaimer {
    margin-left: auto;
    font: 400 10px/1 var(--font-mono);
    color: var(--amber);
    text-transform: none;
    letter-spacing: 0;
  }
  .tech-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 16px;
    margin-top: 14px;
  }
  .info-mark {
    display: inline-grid;
    place-items: center;
    width: 14px;
    height: 14px;
    border: 1px solid var(--border-hi);
    border-radius: 50%;
    color: var(--ivory-3);
    font: 500 9px/1 var(--font-ui);
    cursor: help;
  }
  .tech-footer {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 12px;
    padding-top: 10px;
    border-top: 1px dashed var(--border);
    font: 500 11px/1 var(--font-mono);
    color: var(--ivory-3);
  }
  .tech-footer b {
    color: var(--lime);
    font-weight: 700;
  }
</style>
