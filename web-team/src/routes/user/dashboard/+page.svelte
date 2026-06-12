<script lang="ts">
  import { onMount } from 'svelte';
  import { apiGet, apiPut, ApiError } from '$lib/api';
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
  let shareIdentified = $state(false);
  let shareSaving = $state(false);
  // C44 — politique de visibilité de l'organisation (ADR-0016) : chaque
  // salarié sait sous quel régime il travaille.
  let policy = $state<'anonymous' | 'opt_in' | 'identified' | null>(null);

  const POLICY_TEXT: Record<string, string> = {
    anonymous:
      'Politique de votre organisation : ANONYME STRICT — l’administrateur ne voit que des agrégats k-anonymes, jamais d’identification individuelle (même volontaire).',
    opt_in:
      'Politique de votre organisation : OPT-IN (défaut) — vous seul·e décidez d’apparaître nommément, via le réglage ci-dessous.',
    identified:
      'Politique de votre organisation : NOMINATIF, attestée par votre employeur (CSE et salariés informés — ADR-0016). Vos consommations sont visibles par l’administrateur.'
  };

  async function load() {
    loading = true;
    error = '';
    try {
      data = await apiGet<MyUsage>('/api/v1/me/usage');
      const sharing = await apiGet<{ share_identified: boolean; policy?: string }>(
        '/api/v1/me/sharing'
      );
      shareIdentified = sharing.share_identified;
      policy = (sharing.policy as typeof policy) ?? null;
    } catch (e) {
      error = e instanceof ApiError ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  // ADR-0015 §3 : le consentement appartient au salarié — ce toggle est la
  // SEULE écriture possible du flag (aucune route admin).
  async function toggleSharing() {
    shareSaving = true;
    error = '';
    try {
      const next = await apiPut<{ share_identified: boolean }>('/api/v1/me/sharing', {
        share_identified: !shareIdentified
      });
      shareIdentified = next.share_identified;
    } catch (e) {
      error = e instanceof ApiError ? e.message : String(e);
    } finally {
      shareSaving = false;
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
      Vue personnelle de mes prompts agrégés par le serveur équipe. Vos prompts individuels ne
      quittent jamais votre poste — seules les métadonnées (modèle, tokens, gCO₂eq calculé
      localement) sont remontées.
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

<div class="card share-card">
  {#if policy}
    <p class="policy-line" data-policy={policy}>{POLICY_TEXT[policy]}</p>
  {/if}
  <div class="share-row">
    <div>
      <h3>Partage identifié avec l'admin</h3>
      <p class="muted">
        {#if policy === 'identified'}
          La politique nominative de votre organisation s'applique : ce réglage personnel est sans
          effet sur les vues admin tant qu'elle est active.
        {:else if policy === 'anonymous'}
          La politique « anonyme strict » de votre organisation s'applique : personne n'apparaît
          nommément, même volontairement. Ce réglage sera honoré si la politique change.
        {:else}
          Par défaut, l'administrateur ne voit que des
          <strong>agrégats anonymes d'équipe</strong> (seuil k-anonymat). Si vous l'activez, votre
          nom et vos totaux apparaîtront dans les vues admin. Révocable à tout moment — vous seul·e
          contrôlez ce réglage.
        {/if}
      </p>
    </div>
    <button
      class="share-toggle"
      class:active={shareIdentified}
      onclick={toggleSharing}
      disabled={shareSaving || loading}
      aria-pressed={shareIdentified}
    >
      {shareIdentified ? 'Activé — me retirer' : 'Désactivé — activer'}
    </button>
  </div>
</div>

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
  .policy-line {
    margin-bottom: var(--sp-4);
    padding-bottom: var(--sp-3);
    border-bottom: 1px solid var(--border, rgba(255, 255, 255, 0.07));
    font-size: var(--fs-caption);
    color: var(--ivory-2);
  }
  .policy-line[data-policy='identified'] {
    color: var(--amber);
  }
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
  .share-card {
    margin-bottom: var(--sp-5);
    border-left: 3px solid var(--lime);
  }
  .share-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--sp-5);
    flex-wrap: wrap;
  }
  .share-toggle {
    flex: none;
  }
  .share-toggle.active {
    background: var(--lime);
    color: var(--ink);
  }
</style>
