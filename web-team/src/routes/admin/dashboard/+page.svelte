<script lang="ts">
  import { onMount } from 'svelte';
  import { apiGet } from '$lib/api';
  import { formatCO2, formatCount, formatEnergy, formatWater } from '$lib/format';
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
  interface UserTop {
    user_id: string;
    fingerprint: string;
    display_name: string | null;
    count: number;
    gco2eq_g: number;
  }
  interface MethodBreakdown {
    method: string;
    count: number;
    gco2eq_g: number;
  }
  interface Analytics {
    from: string;
    to: string;
    group_by: string;
    series: TimeBucket[];
    top_models: ModelTop[];
    top_users: UserTop[];
    method_breakdown: MethodBreakdown[];
  }

  let data = $state<Analytics | null>(null);
  let loading = $state(true);
  let error = $state('');
  let groupBy = $state<'day' | 'week' | 'month'>('day');
  let anonymizeUsers = $state(false);

  async function load() {
    loading = true;
    error = '';
    try {
      data = await apiGet<Analytics>(`/api/v1/admin/analytics?group_by=${groupBy}`);
    } catch (e) {
      error = (e as Error).message;
    } finally {
      loading = false;
    }
  }

  onMount(load);

  const totals = $derived(
    (data?.series ?? []).reduce(
      (acc, b) => ({
        count: acc.count + b.count,
        tokens_in: acc.tokens_in + b.tokens_in,
        tokens_out: acc.tokens_out + b.tokens_out,
        gco2eq_g: acc.gco2eq_g + b.gco2eq_g,
        water_ml: acc.water_ml + b.water_ml,
        energy_wh: acc.energy_wh + b.energy_wh
      }),
      { count: 0, tokens_in: 0, tokens_out: 0, gco2eq_g: 0, water_ml: 0, energy_wh: 0 }
    )
  );

  const co2 = $derived(formatCO2(totals.gco2eq_g));
  const energy = $derived(formatEnergy(totals.energy_wh));
  const water = $derived(formatWater(totals.water_ml));

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
  const userBars = $derived(
    (data?.top_users ?? []).map((u, i) => ({
      label: anonymizeUsers
        ? `Employé #${i + 1}`
        : u.display_name || u.fingerprint,
      value: u.gco2eq_g,
      detail: `${formatCount(u.count)} prompts`
    }))
  );
  const methodSlices = $derived(
    (data?.method_breakdown ?? []).map((m) => ({ label: m.method, value: m.gco2eq_g }))
  );
</script>

<section class="head">
  <div>
    <h2>Vue d'ensemble — équipe</h2>
    <p class="muted">
      Empreinte agrégée sur les 30 derniers jours · Self-hosted, aucune donnée ne quitte votre infra.
    </p>
  </div>
  <div class="controls">
    <select bind:value={groupBy} onchange={load}>
      <option value="day">Jour</option>
      <option value="week">Semaine</option>
      <option value="month">Mois</option>
    </select>
  </div>
</section>

{#if error}<p class="error">{error}</p>{/if}

<section class="cards">
  <MetricCard eyebrow="Prompts" value={formatCount(totals.count)} hint="sur la fenêtre" />
  <MetricCard eyebrow="Empreinte CO₂eq" value={co2.value} unit={co2.unit} />
  <MetricCard eyebrow="Eau" value={water.value} unit={water.unit} />
  <MetricCard eyebrow="Énergie" value={energy.value} unit={energy.unit} />
</section>

<section class="chart card">
  <h3>Évolution gCO₂eq par {groupBy === 'day' ? 'jour' : groupBy === 'week' ? 'semaine' : 'mois'}</h3>
  {#if loading}<p class="muted">Chargement…</p>{:else}
    <LineChart points={seriesPoints} valueFormat={(v) => formatCO2(v).value} />
  {/if}
</section>

<section class="grid-2">
  <div class="card">
    <h3>Top modèles (gCO₂eq)</h3>
    {#if loading}<p class="muted">Chargement…</p>{:else}
      <BarChart bars={modelBars} color="var(--lime)" valueFormat={(v) => `${formatCO2(v).value} ${formatCO2(v).unit}`} />
    {/if}
  </div>
  <div class="card">
    <div class="row">
      <h3>Top employés</h3>
      <label class="anon">
        <input type="checkbox" bind:checked={anonymizeUsers} />
        Anonymiser
      </label>
    </div>
    {#if loading}<p class="muted">Chargement…</p>{:else}
      <BarChart bars={userBars} color="var(--amber)" valueFormat={(v) => `${formatCO2(v).value} ${formatCO2(v).unit}`} />
    {/if}
  </div>
</section>

<section class="card">
  <h3>Répartition par méthodologie</h3>
  {#if loading}<p class="muted">Chargement…</p>{:else}
    <DonutChart slices={methodSlices} />
  {/if}
</section>

<style>
  .head {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    margin-bottom: var(--sp-5);
    gap: var(--sp-4);
    flex-wrap: wrap;
  }
  .controls {
    display: flex;
    gap: var(--sp-2);
  }
  .controls select {
    width: auto;
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
    margin-bottom: var(--sp-5);
  }
  @media (max-width: 900px) {
    .grid-2 {
      grid-template-columns: 1fr;
    }
  }
  .row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--sp-3);
  }
  .anon {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-2);
    text-transform: none;
    letter-spacing: 0;
    margin: 0;
    color: var(--ivory-2);
    font-size: var(--fs-caption);
  }
  .anon input {
    width: auto;
  }
</style>
