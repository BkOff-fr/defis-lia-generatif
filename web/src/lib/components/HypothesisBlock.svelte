<script lang="ts">
  import { FlaskConical, ExternalLink, ArrowUpRight } from '@lucide/svelte';
  import type { HypothesisDto } from '$lib/api';

  type Props = { hypotheses: HypothesisDto[] };
  const { hypotheses }: Props = $props();

  function fmtValue(v: unknown): string {
    if (typeof v === 'number') {
      return new Intl.NumberFormat('fr-FR', {
        maximumFractionDigits: 3
      }).format(v);
    }
    if (v === null || v === undefined) return '—';
    if (typeof v === 'string') return v;
    return JSON.stringify(v);
  }

  function isUrl(source: string): boolean {
    return /^https?:\/\//i.test(source);
  }
</script>

<section class="hyp-block" aria-label="Hypothèses utilisées">
  <div class="hh">
    <FlaskConical size={18} strokeWidth={1.6} />
    <div class="t">Hypothèses utilisées</div>
    <div class="spc"></div>
    <a class="btn-ghost-mini" href="/methodo">
      <ExternalLink size={14} strokeWidth={1.8} />
      Voir la méthodologie complète
    </a>
  </div>

  {#if hypotheses.length === 0}
    <p class="empty">Aucune hypothèse exportée par le moteur pour cette estimation.</p>
  {:else}
    <div class="hyp-grid">
      {#each hypotheses as h (h.key)}
        <div class="hyp-row">
          <span class="sym">{h.key}</span>
          <span class="val">{fmtValue(h.value)}</span>
          {#if isUrl(h.source)}
            <a class="src" href={h.source} target="_blank" rel="noopener noreferrer">
              {h.source.replace(/^https?:\/\//, '').slice(0, 32)}…
              <ArrowUpRight size={10} strokeWidth={2} />
            </a>
          {:else}
            <span class="src">
              {h.source}
              <ArrowUpRight size={10} strokeWidth={2} />
            </span>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</section>

<style>
  .hyp-block {
    margin-top: 28px;
    padding: 28px 32px;
    background: var(--ink-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
  }
  .hh {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 20px;
    padding-bottom: 16px;
    border-bottom: 1px solid var(--border);
  }
  .hh :global(svg) {
    color: var(--lime);
    flex-shrink: 0;
  }
  .hh .t {
    font: 400 22px/1 var(--font-display);
    font-style: italic;
    color: var(--ivory);
  }
  .hh .spc {
    flex: 1;
  }

  .btn-ghost-mini {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 32px;
    padding: 0 12px;
    background: transparent;
    color: var(--ivory-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    font: 500 12px/1 var(--font-ui);
    text-decoration: none;
    transition: all var(--dur-base) var(--ease);
  }
  .btn-ghost-mini:hover {
    border-color: var(--border-hi);
    color: var(--ivory);
  }

  .hyp-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 6px 32px;
  }
  .hyp-row {
    display: grid;
    grid-template-columns: auto 1fr auto;
    gap: 14px;
    padding: 12px 4px;
    border-bottom: 1px dashed var(--border);
    align-items: center;
    border-radius: 4px;
    transition: background var(--dur-base) var(--ease);
  }
  .hyp-row:hover {
    background: rgba(255, 255, 255, 0.015);
  }
  .hyp-row:nth-last-child(-n + 2) {
    border-bottom: none;
  }
  .sym {
    font: 400 14px/1 var(--font-mono);
    color: var(--lime);
    width: 90px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .val {
    font: 500 13px/1.3 var(--font-mono);
    color: var(--ivory);
  }
  .src {
    font: 400 11px/1 var(--font-ui);
    color: var(--ivory-3);
    display: inline-flex;
    align-items: center;
    gap: 4px;
    transition: color var(--dur-base) var(--ease);
    border-bottom: none;
    text-decoration: none;
  }
  .src:hover {
    color: var(--blue);
  }

  .empty {
    font: 400 13px/1.5 var(--font-ui);
    color: var(--ivory-3);
    font-style: italic;
  }

  @media (max-width: 720px) {
    .hyp-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
