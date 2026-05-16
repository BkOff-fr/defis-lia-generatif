<script lang="ts">
  import { onMount } from 'svelte';
  import { apiGet, ApiError } from '$lib/api';
  import { formatCO2, formatCount, formatEnergy, formatWater } from '$lib/format';
  import MetricCard from '$lib/components/MetricCard.svelte';

  interface UsageTotals {
    count: number;
    tokens_in: number;
    tokens_out: number;
    gco2eq_p50_g: number;
    water_ml: number;
    energy_wh: number;
  }
  interface MyUsage {
    user_id: string;
    totals: UsageTotals;
  }

  let data = $state<MyUsage | null>(null);
  let loading = $state(true);
  let error = $state('');

  async function load() {
    loading = true;
    error = '';
    try {
      data = await apiGet<MyUsage>('/api/v1/me/usage');
    } catch (e) {
      error = e instanceof ApiError ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  onMount(load);

  const co2 = $derived(formatCO2(data?.totals.gco2eq_p50_g ?? 0));
  const energy = $derived(formatEnergy(data?.totals.energy_wh ?? 0));
  const water = $derived(formatWater(data?.totals.water_ml ?? 0));
</script>

<section class="head">
  <div>
    <h2>Mon usage</h2>
    <p class="muted">
      Vue personnelle de mes prompts agrégés par le serveur équipe. Vos prompts
      individuels ne quittent jamais votre poste — seules les métadonnées
      (modèle, tokens, gCO₂eq calculé localement) sont remontées.
    </p>
  </div>
</section>

{#if error}<p class="error">{error}</p>{/if}

<section class="cards">
  <MetricCard
    eyebrow="Prompts"
    value={formatCount(data?.totals.count ?? 0)}
    hint="depuis l'enrôlement"
  />
  <MetricCard eyebrow="Empreinte CO₂eq" value={co2.value} unit={co2.unit} />
  <MetricCard eyebrow="Eau consommée" value={water.value} unit={water.unit} />
  <MetricCard eyebrow="Énergie" value={energy.value} unit={energy.unit} />
</section>

<div class="card hint-card">
  <h3>Ce qui est partagé avec l'équipe</h3>
  <ul>
    <li>Modèle utilisé (ex : <code>llama-3-1-70b</code>)</li>
    <li>Nombre de tokens entrée / sortie</li>
    <li>Empreinte calculée localement (AFNOR ou EcoLogits)</li>
    <li>Méthodologie utilisée + région d'inférence</li>
    <li>Horodatage</li>
  </ul>
  <h3>Ce qui ne l'est jamais</h3>
  <ul>
    <li>Le contenu de vos prompts (rester local à votre navigateur / app)</li>
    <li>Le contenu des réponses</li>
    <li>L'URL de la conversation</li>
  </ul>
</div>

<style>
  .head {
    margin-bottom: var(--sp-5);
  }
  .cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: var(--sp-4);
    margin-bottom: var(--sp-5);
  }
  .hint-card ul {
    margin: var(--sp-2) 0 var(--sp-4);
    padding-left: var(--sp-5);
    color: var(--ivory-2);
  }
  .hint-card h3 {
    margin-top: var(--sp-3);
  }
</style>
