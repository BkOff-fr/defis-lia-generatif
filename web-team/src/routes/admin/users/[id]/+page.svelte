<script lang="ts">
  // C44 — Détail de consommation d'UN employé (ADR-0016).
  //
  // L'accès est gouverné CÔTÉ SERVEUR par la politique de visibilité :
  // `identified` → tous ; `opt_in` → uniquement partage activé ; sinon
  // 403. Cette page se contente d'afficher — et d'expliquer le 403.
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { apiGet, ApiError } from '$lib/api';
  import { formatCO2, formatCount, formatDateTime, formatEnergy, formatWater } from '$lib/format';
  import MetricCard from '$lib/components/MetricCard.svelte';
  import LineChart from '$charts/LineChart.svelte';
  import BarChart from '$charts/BarChart.svelte';
  import DonutChart from '$charts/DonutChart.svelte';

  interface TimeBucket {
    bucket: string;
    count: number;
    tokens_in: number;
    tokens_out: number;
    gco2eq_g: number;
    water_ml: number;
    energy_wh: number;
  }
  interface ModelTop {
    model_id: string;
    count: number;
    gco2eq_g: number;
  }
  interface MethodBreakdown {
    method: string;
    count: number;
    gco2eq_g: number;
  }
  interface UsageTotals {
    count: number;
    tokens_in: number;
    tokens_out: number;
    gco2eq_p50_g: number;
    water_ml: number;
    energy_wh: number;
  }
  interface UserDetail {
    policy: 'anonymous' | 'opt_in' | 'identified';
    from: string;
    to: string;
    user: {
      id: string;
      display_name: string | null;
      fingerprint: string;
      share_identified: boolean;
    };
    totals: UsageTotals;
    series: TimeBucket[];
    top_models: ModelTop[];
    method_breakdown: MethodBreakdown[];
  }

  let data = $state<UserDetail | null>(null);
  let loading = $state(true);
  let forbidden = $state(false);
  let error = $state('');

  const userId = $derived($page.params.id);

  onMount(async () => {
    try {
      data = await apiGet<UserDetail>(`/api/v1/admin/users/${userId}/analytics`);
    } catch (e) {
      if (e instanceof ApiError && e.status === 403) {
        forbidden = true;
      } else {
        error = e instanceof ApiError ? e.message : String(e);
      }
    } finally {
      loading = false;
    }
  });

  const co2 = $derived(formatCO2(data?.totals.gco2eq_p50_g ?? 0));
  const energy = $derived(formatEnergy(data?.totals.energy_wh ?? 0));
  const water = $derived(formatWater(data?.totals.water_ml ?? 0));
  const seriesPoints = $derived(
    (data?.series ?? []).map((b) => ({ label: b.bucket.slice(5), value: b.gco2eq_g }))
  );
  const modelBars = $derived(
    (data?.top_models ?? []).map((m) => ({
      label: m.model_id,
      value: m.gco2eq_g,
      detail: `${formatCount(m.count)} prompts`
    }))
  );
  const methodSlices = $derived(
    (data?.method_breakdown ?? []).map((m) => ({ label: m.method, value: m.gco2eq_g }))
  );
</script>

<section class="head">
  <div>
    <p class="crumb"><a href="/admin/users">← Employés</a></p>
    <h2>
      {data ? data.user.display_name || data.user.fingerprint : 'Détail employé'}
    </h2>
    {#if data}
      <p class="muted">
        Fenêtre : {formatDateTime(data.from)} → {formatDateTime(data.to)} ·
        {data.policy === 'identified'
          ? 'politique nominative attestée (ADR-0016)'
          : 'partage identifié activé par l’employé (ADR-0015)'}
      </p>
    {/if}
  </div>
</section>

{#if loading}
  <p class="muted">Chargement…</p>
{:else if forbidden}
  <div class="card forbidden" role="status">
    <h3>Détail individuel non accessible</h3>
    <p class="muted">
      La politique de visibilité de votre organisation ne permet pas cette vue : soit le mode est
      « anonyme strict » (aucune identification, ADR-0016), soit cet employé n'a pas activé son
      partage identifié (ADR-0015). Le consentement appartient au salarié — il peut l'activer
      depuis son espace « Mon usage ».
    </p>
  </div>
{:else if error}
  <p class="error">{error}</p>
{:else if data}
  <section class="cards">
    <MetricCard eyebrow="Prompts" value={formatCount(data.totals.count)} hint="30 derniers jours" />
    <MetricCard eyebrow="Empreinte CO₂eq" value={co2.value} unit={co2.unit} />
    <MetricCard eyebrow="Eau" value={water.value} unit={water.unit} />
    <MetricCard eyebrow="Énergie" value={energy.value} unit={energy.unit} />
  </section>

  <section class="chart card">
    <h3>Évolution gCO₂eq par jour</h3>
    {#if seriesPoints.length === 0}
      <p class="muted">Aucune mesure sur la fenêtre.</p>
    {:else}
      <LineChart points={seriesPoints} valueFormat={(v) => formatCO2(v).value} />
    {/if}
  </section>

  <section class="grid-2">
    <div class="card">
      <h3>Modèles utilisés (gCO₂eq)</h3>
      {#if modelBars.length === 0}
        <p class="muted">—</p>
      {:else}
        <BarChart
          bars={modelBars}
          color="var(--lime)"
          valueFormat={(v) => `${formatCO2(v).value} ${formatCO2(v).unit}`}
        />
      {/if}
    </div>
    <div class="card">
      <h3>Méthodologies</h3>
      {#if methodSlices.length === 0}
        <p class="muted">—</p>
      {:else}
        <DonutChart slices={methodSlices} />
      {/if}
    </div>
  </section>
{/if}

<style>
  .head {
    margin-bottom: var(--sp-5);
  }
  .crumb {
    margin-bottom: var(--sp-2);
    font-size: var(--fs-caption);
  }
  .cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: var(--sp-4);
    margin-bottom: var(--sp-5);
  }
  .chart {
    margin-bottom: var(--sp-5);
  }
  .grid-2 {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--sp-5);
  }
  @media (max-width: 900px) {
    .grid-2 {
      grid-template-columns: 1fr;
    }
  }
  .forbidden {
    border-left: 3px solid var(--amber);
  }
</style>
