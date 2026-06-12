<script lang="ts">
  import { TrendingDown, TrendingUp, Minus } from '@lucide/svelte';

  export type WaterfallStep = {
    /** Libellé du lever ajouté à ce step (ex: "+ PUE", "+ Tokens"). */
    label: string;
    /** P50 cumulatif après application du lever (gCO₂eq). */
    cumulativeCo2eqG: number;
  };

  type Props = {
    baselineCo2eqG: number;
    steps: WaterfallStep[];
  };
  const { baselineCo2eqG, steps }: Props = $props();

  type Row = {
    label: string;
    cumulative: number;
    deltaPrev: number;
    deltaFromBaseline: number;
    isBaseline: boolean;
  };

  const rows = $derived.by<Row[]>(() => {
    const out: Row[] = [
      {
        label: 'Baseline',
        cumulative: baselineCo2eqG,
        deltaPrev: 0,
        deltaFromBaseline: 0,
        isBaseline: true
      }
    ];
    let prev = baselineCo2eqG;
    for (const s of steps) {
      out.push({
        label: s.label,
        cumulative: s.cumulativeCo2eqG,
        deltaPrev: s.cumulativeCo2eqG - prev,
        deltaFromBaseline: s.cumulativeCo2eqG - baselineCo2eqG,
        isBaseline: false
      });
      prev = s.cumulativeCo2eqG;
    }
    return out;
  });

  // Échelle horizontale : 0 → max(cumulative, baseline) avec un peu de marge.
  const maxCum = $derived.by(() => {
    let m = baselineCo2eqG;
    for (const r of rows) if (r.cumulative > m) m = r.cumulative;
    return m * 1.1 || 1;
  });

  function fmt(value: number, digits = 2): string {
    if (!Number.isFinite(value)) return '—';
    return new Intl.NumberFormat('fr-FR', { maximumFractionDigits: digits }).format(value);
  }

  function pct(value: number, total: number): number {
    if (total <= 0) return 0;
    return Math.max(0, Math.min(100, (value / total) * 100));
  }
</script>

<article class="wf">
  <header class="wfh">
    <div class="eyebrow">Attribution séquentielle</div>
    <h3>Contribution lever par lever</h3>
    <p class="hint">
      L'ordre d'addition des leviers compte (attribution non-Shapley — cf. méthodologie C11 §2.3).
    </p>
  </header>

  {#if steps.length === 0}
    <p class="wf-empty">Aucun lever activé — actionne un curseur pour voir la cascade.</p>
  {:else}
    <ul class="wf-list">
      {#each rows as r, i (i)}
        <li class="wf-row" class:baseline={r.isBaseline}>
          <span class="wf-label">{r.label}</span>
          <span class="wf-bar" aria-hidden="true">
            <span class="wf-fill" style="width: {pct(r.cumulative, maxCum)}%"></span>
            {#if !r.isBaseline}
              <span
                class="wf-baseline-mark"
                style="left: {pct(baselineCo2eqG, maxCum)}%"
                title="Baseline"
              ></span>
            {/if}
          </span>
          <span class="wf-val mono">{fmt(r.cumulative, 2)}<span class="u">g</span></span>
          <span
            class="wf-delta"
            class:lime={r.deltaPrev < 0}
            class:coral={r.deltaPrev > 0}
            class:zero={r.isBaseline || r.deltaPrev === 0}
          >
            {#if r.isBaseline}
              <Minus size={11} strokeWidth={2} /> ref
            {:else if r.deltaPrev < 0}
              <TrendingDown size={11} strokeWidth={2} />
              {fmt(r.deltaPrev, 2)}g
            {:else if r.deltaPrev > 0}
              <TrendingUp size={11} strokeWidth={2} />
              +{fmt(r.deltaPrev, 2)}g
            {:else}
              <Minus size={11} strokeWidth={2} /> 0
            {/if}
          </span>
        </li>
      {/each}
    </ul>
  {/if}
</article>

<style>
  .wf {
    padding: 22px 24px 22px;
    background: rgba(255, 255, 255, 0.015);
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
  }
  .wfh {
    margin-bottom: 16px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border);
  }
  .wfh .eyebrow {
    font: 500 12px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--ivory-3);
    margin-bottom: 6px;
  }
  .wfh h3 {
    font: 400 22px/1.15 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.01em;
    margin: 0 0 4px;
  }
  .wfh .hint {
    margin: 0;
    font: 400 12px/1.4 var(--font-ui);
    color: var(--ivory-3);
  }

  .wf-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .wf-row {
    display: grid;
    grid-template-columns: 160px 1fr 90px 110px;
    gap: 12px;
    align-items: center;
    padding: 8px 12px;
    background: rgba(0, 0, 0, 0.18);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }
  .wf-row.baseline {
    background: rgba(126, 182, 255, 0.04);
    border-color: rgba(126, 182, 255, 0.2);
  }

  .wf-label {
    font: 500 12px/1.2 var(--font-ui);
    color: var(--ivory);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .wf-row.baseline .wf-label {
    color: var(--blue);
  }

  .wf-bar {
    position: relative;
    height: 8px;
    background: rgba(255, 255, 255, 0.04);
    border-radius: 999px;
    overflow: visible;
  }
  .wf-fill {
    display: block;
    height: 100%;
    background: linear-gradient(90deg, rgba(197, 240, 74, 0.35), var(--lime));
    border-radius: 999px;
    transition: width 350ms var(--ease);
  }
  .wf-row.baseline .wf-fill {
    background: linear-gradient(90deg, rgba(126, 182, 255, 0.4), var(--blue));
  }
  .wf-baseline-mark {
    position: absolute;
    top: -2px;
    bottom: -2px;
    width: 1.5px;
    background: var(--blue);
    box-shadow: 0 0 6px rgba(126, 182, 255, 0.6);
    transform: translateX(-0.75px);
  }

  .wf-val {
    text-align: right;
    font: 600 13px/1 var(--font-mono);
    color: var(--ivory);
  }
  .wf-val .u {
    font: 400 12px/1 var(--font-mono);
    color: var(--ivory-3);
    margin-left: 2px;
  }

  .wf-delta {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    padding: 3px 8px;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--border);
    border-radius: 999px;
    font: 600 12px/1 var(--font-mono);
    color: var(--ivory-3);
  }
  .wf-delta.lime {
    color: var(--lime);
    border-color: rgba(197, 240, 74, 0.25);
    background: rgba(197, 240, 74, 0.06);
  }
  .wf-delta.coral {
    color: var(--coral);
    border-color: rgba(240, 108, 90, 0.25);
    background: rgba(240, 108, 90, 0.06);
  }
  .wf-delta.zero {
    color: var(--ivory-4);
  }

  .wf-empty {
    margin: 0;
    font: 400 13px/1.5 var(--font-ui);
    color: var(--ivory-3);
    font-style: italic;
  }

  .mono {
    font-family: var(--font-mono);
  }
</style>
