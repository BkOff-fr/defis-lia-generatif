<script lang="ts">
  // C40 — Boucle « Réduire » : après chaque estimation, propose jusqu'à
  // 3 modèles plus sobres (paramètres actifs inférieurs), réestimés par le
  // VRAI moteur sur la même requête et la même méthodologie
  // (estimate_for_comparison — éphémère, non journalisé, audit_id = 0).
  // Aucune heuristique d'empreinte côté client : uniquement des résultats
  // moteur, conformément à CLAUDE.md §13.
  import { TrendingDown, ArrowRight } from '@lucide/svelte';
  import { estimateForComparison, type EstimationResultDto, type ModelPresetDto } from '$lib/api';

  type Props = {
    /** Résultat de l'estimation principale (référence des deltas). */
    result: EstimationResultDto;
    /** Catalogue chargé par la page hôte (évite un second list_models). */
    models: ModelPresetDto[];
  };
  let { result, models }: Props = $props();

  type Suggestion = {
    preset: ModelPresetDto;
    co2_p50_g: number;
    delta_pct: number;
  };

  let suggestions = $state<Suggestion[]>([]);
  let loading = $state(false);
  let smallestAlready = $state(false);

  function co2P50(r: EstimationResultDto): number {
    return r.indicators.find((i) => i.indicator === 'co2eq')?.p50 ?? 0;
  }

  /** Jusqu'à 3 candidats plus petits (params actifs), fournisseurs variés,
   * du plus proche au plus sobre — pour montrer l'éventail du compromis. */
  function pickCandidates(current: ModelPresetDto): ModelPresetDto[] {
    const smaller = models
      .filter((m) => !m.deprecated && m.id !== current.id)
      .filter((m) => m.active_params_b < current.active_params_b)
      .sort((a, b) => b.active_params_b - a.active_params_b);
    if (smaller.length <= 3) return smaller;
    // Indices valides (length > 3) mais noUncheckedIndexedAccess oblige
    // à filtrer l'undefined théorique.
    const picks = [smaller[0], smaller[Math.floor(smaller.length / 2)], smaller.at(-1)];
    return [...new Set(picks.filter((m): m is ModelPresetDto => m !== undefined))];
  }

  $effect(() => {
    const current = models.find((m) => m.id === result.request.model_id);
    suggestions = [];
    smallestAlready = false;
    if (!current) return;
    const candidates = pickCandidates(current);
    if (candidates.length === 0) {
      smallestAlready = true;
      return;
    }
    loading = true;
    const base = co2P50(result);
    void Promise.allSettled(
      candidates.map(async (preset) => {
        const r = await estimateForComparison(
          {
            model_id: preset.id,
            tokens_in: result.request.tokens_in,
            tokens_out_estimated: result.request.tokens_out_estimated,
            ...(result.request.datacenter_id ? { datacenter_id: result.request.datacenter_id } : {})
          },
          result.method
        );
        return { preset, co2_p50_g: co2P50(r) } satisfies Omit<Suggestion, 'delta_pct'>;
      })
    ).then((settled) => {
      suggestions = settled
        .filter(
          (s): s is PromiseFulfilledResult<Omit<Suggestion, 'delta_pct'>> =>
            s.status === 'fulfilled'
        )
        .map((s) => ({
          ...s.value,
          delta_pct: base > 0 ? ((s.value.co2_p50_g - base) / base) * 100 : 0
        }))
        .filter((s) => s.delta_pct < -1) // ne propose que de vraies baisses
        .sort((a, b) => a.co2_p50_g - b.co2_p50_g);
      loading = false;
    });
  });

  /** Lien /comparer pré-rempli : modèle courant + alternatives + tokens. */
  const compareHref = $derived.by(() => {
    const ids = [result.request.model_id, ...suggestions.map((s) => s.preset.id)];
    const q = new URLSearchParams({
      models: ids.join(','),
      tin: String(result.request.tokens_in),
      tout: String(result.request.tokens_out_estimated)
    });
    return `/comparer?${q.toString()}`;
  });

  function fmtCo2(g: number): string {
    if (g < 0.001) return `${(g * 1e6).toFixed(0)} µg`;
    if (g < 1) return `${(g * 1000).toFixed(1)} mg`;
    if (g < 1000) return `${g.toFixed(2)} g`;
    return `${(g / 1000).toFixed(2)} kg`;
  }
</script>

{#if loading || suggestions.length > 0 || smallestAlready}
  <section class="reduce-suggestions" aria-label="Réduire cette empreinte">
    <header class="reduce-head">
      <span class="reduce-ico" aria-hidden="true">
        <TrendingDown size={16} strokeWidth={1.8} />
      </span>
      <h3>Réduire cette empreinte</h3>
      <span class="reduce-note">mêmes tokens, même méthodologie — moteur Monte-Carlo</span>
    </header>

    {#if loading}
      <p class="reduce-loading">Estimation des alternatives…</p>
    {:else if smallestAlready}
      <p class="reduce-done">
        Vous utilisez déjà l'un des modèles les plus sobres du catalogue pour cette tâche.
      </p>
    {:else}
      <ul class="reduce-list">
        {#each suggestions as s (s.preset.id)}
          <li class="reduce-card">
            <span class="reduce-model">{s.preset.display_name}</span>
            <span class="reduce-delta">{s.delta_pct.toFixed(0)} %</span>
            <span class="reduce-abs">{fmtCo2(s.co2_p50_g)} CO₂eq · P50</span>
            <span class="reduce-params">{s.preset.active_params_b} Mds params actifs</span>
          </li>
        {/each}
      </ul>
      <div class="reduce-actions">
        <a class="reduce-link" href={compareHref}>
          Comparer en détail <ArrowRight size={14} strokeWidth={2} />
        </a>
        <a class="reduce-link" href="/eco-budget">
          Définir un éco-budget <ArrowRight size={14} strokeWidth={2} />
        </a>
      </div>
    {/if}
  </section>
{/if}

<style>
  .reduce-suggestions {
    margin-top: 16px;
    padding: 16px 18px;
    background: var(--lime-soft);
    border: 1px solid rgba(197, 240, 74, 0.25);
    border-radius: var(--radius-lg);
  }
  .reduce-head {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
    margin-bottom: 10px;
  }
  .reduce-ico {
    display: grid;
    place-items: center;
    color: var(--lime);
  }
  .reduce-head h3 {
    font: 600 var(--fs-h3) / var(--lh-h3) var(--font-ui);
    color: var(--ivory);
  }
  .reduce-note {
    font: 400 var(--fs-caption) / var(--lh-caption) var(--font-ui);
    color: var(--ivory-2);
  }
  .reduce-loading,
  .reduce-done {
    font: 400 var(--fs-body-sm) / var(--lh-body-sm) var(--font-ui);
    color: var(--ivory-2);
  }
  .reduce-list {
    list-style: none;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 10px;
    margin: 0 0 12px;
    padding: 0;
  }
  .reduce-card {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 10px 12px;
    background: var(--surface-hi);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
  }
  .reduce-model {
    font: 500 var(--fs-body-sm) / 1.3 var(--font-ui);
    color: var(--ivory);
  }
  .reduce-delta {
    font: 400 1.375rem/1.1 var(--font-display);
    color: var(--lime);
  }
  .reduce-abs,
  .reduce-params {
    font: 400 var(--fs-caption) / var(--lh-caption) var(--font-mono);
    color: var(--ivory-2);
  }
  .reduce-params {
    color: var(--ivory-3);
  }
  .reduce-actions {
    display: flex;
    gap: 16px;
    flex-wrap: wrap;
  }
  .reduce-link {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font: 500 var(--fs-body-sm) / 1 var(--font-ui);
  }
</style>
