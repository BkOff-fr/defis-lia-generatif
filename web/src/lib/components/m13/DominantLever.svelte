<script lang="ts" module>
  // Labels canoniques des leviers "isolés" — alignés avec les ScenarioDto
  // construits dans `/simuler/+page.svelte` (préfixe "isolate:").
  //
  // L'idée méthodologique (cf. brief C11 §4bis) : pour identifier le levier
  // dominant on simule, pour chaque composante, un scénario où ce levier est
  // ramené à sa valeur "neutre" (PUE=1, mix=0, embodied=0, etc.). Le |Δ|
  // produit isole la contribution de cette composante au total. La composante
  // dont la suppression cause la plus grosse chute = le levier dominant.

  export const ISOLATE_LABELS = {
    embodied: 'Embodied carbon',
    mix: 'Mix électrique',
    pue: 'Surcharge PUE',
    wue: 'Empreinte eau',
    tokens: 'Tokens de sortie'
  } as const;

  export type IsolateKey = keyof typeof ISOLATE_LABELS;
</script>

<script lang="ts">
  import { Crosshair, Info } from '@lucide/svelte';

  type IsolateOutcome = { key: IsolateKey; deltaCo2eqG: number };

  type Props = {
    outcomes: IsolateOutcome[];
    /** baseline P50 absolu (gCO₂eq) pour calculer la part % du total. */
    baselineCo2eqG: number;
  };
  const { outcomes, baselineCo2eqG }: Props = $props();

  /**
   * Calcule la part de chaque lever dans le total : on prend la somme des
   * |Δ| comme dénominateur (vu comme proxy de l'empreinte décomposable).
   *
   * Pour le dominant on prend simplement le max(|Δ|).
   */
  const ranked = $derived.by(() => {
    const items = outcomes.map((o) => ({
      ...o,
      abs: Math.abs(o.deltaCo2eqG)
    }));
    const totalAbs = items.reduce((sum, i) => sum + i.abs, 0);
    items.sort((a, b) => b.abs - a.abs);
    return items.map((i) => ({
      ...i,
      sharePct: totalAbs > 0 ? (i.abs / totalAbs) * 100 : 0
    }));
  });

  const dominant = $derived(ranked[0]);
  const dominantLabel = $derived(dominant ? ISOLATE_LABELS[dominant.key] : '');

  function fmt(value: number, digits = 1): string {
    if (!Number.isFinite(value)) return '—';
    return new Intl.NumberFormat('fr-FR', { maximumFractionDigits: digits }).format(value);
  }
</script>

<article class="dl">
  <header class="dh">
    <div class="dh-l">
      <span class="ico"><Crosshair size={13} strokeWidth={1.8} /></span>
      <span>Levier dominant sur ce profil</span>
    </div>
    <span class="meta">
      <Info size={11} strokeWidth={1.8} />
      Méthodologie EcoLogits / C11 §4bis
    </span>
  </header>

  {#if dominant}
    <div class="dl-main">
      <div class="dl-headline">
        <em>{dominantLabel}</em>
        <span class="share">{fmt(dominant.sharePct, 1)}%</span>
      </div>
      <p class="dl-narrative">
        Sur la configuration baseline (~{fmt(baselineCo2eqG, 2)} gCO₂eq), votre principal levier est
        <strong>{dominantLabel.toLowerCase()}</strong>. Les autres ont un impact marginal — agir
        d'abord là.
      </p>
    </div>

    <ul class="dl-list" aria-label="Décomposition par lever">
      {#each ranked as r, i (r.key)}
        <li class:lead={i === 0}>
          <span class="ll-key">{ISOLATE_LABELS[r.key]}</span>
          <span class="ll-bar" aria-hidden="true">
            <span class="ll-fill" style="width: {Math.max(2, r.sharePct).toFixed(1)}%"></span>
          </span>
          <span class="ll-pct mono">{fmt(r.sharePct, 1)}%</span>
        </li>
      {/each}
    </ul>
  {:else}
    <p class="dl-empty">Aucune décomposition disponible (simulation en cours…).</p>
  {/if}
</article>

<style>
  .dl {
    padding: 22px 24px 20px;
    background: rgba(255, 255, 255, 0.015);
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
  }
  .dh {
    display: flex;
    align-items: center;
    gap: 14px;
    margin-bottom: 14px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border);
  }
  .dh-l {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font: 500 11px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--ivory-2);
  }
  .dh-l .ico {
    display: inline-grid;
    place-items: center;
    width: 22px;
    height: 22px;
    background: var(--lime-soft);
    border: 1px solid rgba(197, 240, 74, 0.3);
    border-radius: 6px;
    color: var(--lime);
  }
  .meta {
    margin-left: auto;
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font: 400 10px/1 var(--font-mono);
    color: var(--ivory-4);
    letter-spacing: 0.04em;
  }

  .dl-main {
    margin-bottom: 18px;
  }
  .dl-headline {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 8px;
  }
  .dl-headline em {
    font: 400 36px/1.05 var(--font-display);
    font-style: italic;
    color: var(--lime);
    letter-spacing: -0.02em;
  }
  .dl-headline .share {
    font: 600 22px/1 var(--font-mono);
    color: var(--ivory);
    letter-spacing: -0.01em;
  }
  .dl-narrative {
    font: 400 13px/1.55 var(--font-ui);
    color: var(--ivory-2);
    margin: 0;
  }
  .dl-narrative strong {
    color: var(--lime);
    font-weight: 600;
  }

  .dl-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .dl-list li {
    display: grid;
    grid-template-columns: 140px 1fr 60px;
    align-items: center;
    gap: 12px;
    padding: 6px 10px;
    border-radius: var(--radius-sm);
    background: rgba(0, 0, 0, 0.18);
    border: 1px solid var(--border);
  }
  .dl-list li.lead {
    background: rgba(197, 240, 74, 0.06);
    border-color: rgba(197, 240, 74, 0.25);
  }
  .ll-key {
    font: 500 12px/1.2 var(--font-ui);
    color: var(--ivory-2);
  }
  .lead .ll-key {
    color: var(--lime);
  }
  .ll-bar {
    height: 5px;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 999px;
    overflow: hidden;
  }
  .ll-fill {
    display: block;
    height: 100%;
    background: linear-gradient(90deg, rgba(197, 240, 74, 0.4), var(--lime));
    border-radius: 999px;
    transition: width 400ms var(--ease);
  }
  .lead .ll-fill {
    background: linear-gradient(90deg, var(--lime), var(--lime-deep));
  }
  .ll-pct {
    text-align: right;
    font: 600 11px/1 var(--font-mono);
    color: var(--ivory-2);
  }
  .lead .ll-pct {
    color: var(--lime);
  }
  .mono {
    font-family: var(--font-mono);
  }

  .dl-empty {
    font: 400 13px/1.5 var(--font-ui);
    color: var(--ivory-3);
    font-style: italic;
    margin: 0;
  }
</style>
