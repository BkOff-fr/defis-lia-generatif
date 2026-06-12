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
  interface TopUsersShared {
    identified: UserTop[];
    anonymous_users: number;
    anonymous_count: number;
    anonymous_gco2eq_g: number;
  }
  interface KAnonymity {
    required: number;
    active_users: number;
    blocked: boolean;
  }
  interface ProjectBreakdown {
    project: string | null;
    contributors: number;
    count: number;
    gco2eq_g: number;
    energy_wh: number;
    water_ml: number;
    folded: boolean;
  }
  interface Analytics {
    from: string;
    to: string;
    group_by: string;
    policy: 'anonymous' | 'opt_in' | 'identified';
    k_anonymity: KAnonymity;
    series: TimeBucket[];
    top_models: ModelTop[];
    top_users: TopUsersShared;
    projects: ProjectBreakdown[];
    method_breakdown: MethodBreakdown[];
  }

  const POLICY_LABEL: Record<Analytics['policy'], string> = {
    anonymous: 'Anonyme strict — agrégats k-anonymes uniquement (ADR-0016)',
    opt_in: 'Opt-in — identification contrôlée par chaque salarié (défaut, ADR-0015)',
    identified: 'Nominatif — politique attestée par votre organisation (ADR-0016)'
  };

  let data = $state<Analytics | null>(null);
  let loading = $state(true);
  let error = $state('');
  let groupBy = $state<'day' | 'week' | 'month'>('day');

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
  // ADR-0015 §3 : seuls les participants en partage opt-in arrivent
  // nommés du serveur ; le reste est déjà agrégé côté SQL.
  const userBars = $derived.by(() => {
    const tu = data?.top_users;
    if (!tu) return [];
    const bars = tu.identified.map((u) => ({
      label: u.display_name || u.fingerprint,
      value: u.gco2eq_g,
      detail: `${formatCount(u.count)} prompts`
    }));
    if (tu.anonymous_users > 0) {
      bars.push({
        label: `${tu.anonymous_users} participant${tu.anonymous_users > 1 ? 's' : ''} (anonyme)`,
        value: tu.anonymous_gco2eq_g,
        detail: `${formatCount(tu.anonymous_count)} prompts`
      });
    }
    return bars;
  });
  const blocked = $derived(data?.k_anonymity.blocked ?? false);
  const methodSlices = $derived(
    (data?.method_breakdown ?? []).map((m) => ({ label: m.method, value: m.gco2eq_g }))
  );
  // C44 — agrégats par projet (repli « autres projets » sous k déjà
  // appliqué côté serveur selon la politique).
  const projectBars = $derived(
    (data?.projects ?? []).map((p) => ({
      label: p.folded ? 'Autres projets (repli k)' : (p.project ?? 'Hors projet'),
      value: p.gco2eq_g,
      detail: `${formatCount(p.count)} prompts · ${p.contributors}+ contributeur${p.contributors > 1 ? 's' : ''}`
    }))
  );
  // Lien vers le détail par employé : autorisé par le serveur en mode
  // identified (tous) ou opt_in (partage activé → présents dans
  // identified[]). En mode anonymous, identified[] est vide.
  const canDrill = $derived(data?.policy === 'identified' || data?.policy === 'opt_in');
</script>

<section class="head">
  <div>
    <h2>Vue d'ensemble — équipe</h2>
    <p class="muted">
      Empreinte agrégée sur les 30 derniers jours · Self-hosted, aucune donnée ne quitte votre
      infra.
    </p>
    {#if data}
      <p class="policy-badge" data-policy={data.policy} title={POLICY_LABEL[data.policy]}>
        Politique : <strong>{data.policy === 'identified' ? 'Nominatif (attesté)' : data.policy === 'anonymous' ? 'Anonyme strict' : 'Opt-in (défaut)'}</strong>
      </p>
    {/if}
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

{#if data && blocked}
  <div class="card k-blocked" role="status">
    <h3>Agrégats protégés par le k-anonymat</h3>
    <p class="muted">
      {data.k_anonymity.active_users} utilisateur{data.k_anonymity.active_users > 1 ? 's' : ''}
      actif{data.k_anonymity.active_users > 1 ? 's' : ''} sur la fenêtre, alors que le seuil configuré
      est de {data.k_anonymity.required}. En dessous de ce seuil, un agrégat d'équipe reviendrait à
      exposer des données individuelles (ADR-0015) — les analytics se débloqueront quand l'activité
      sera suffisante.
    </p>
  </div>
{/if}

<section class="cards">
  <MetricCard eyebrow="Prompts" value={formatCount(totals.count)} hint="sur la fenêtre" />
  <MetricCard eyebrow="Empreinte CO₂eq" value={co2.value} unit={co2.unit} />
  <MetricCard eyebrow="Eau" value={water.value} unit={water.unit} />
  <MetricCard eyebrow="Énergie" value={energy.value} unit={energy.unit} />
</section>

<section class="chart card">
  <h3>
    Évolution gCO₂eq par {groupBy === 'day' ? 'jour' : groupBy === 'week' ? 'semaine' : 'mois'}
  </h3>
  {#if loading}<p class="muted">Chargement…</p>{:else}
    <LineChart points={seriesPoints} valueFormat={(v) => formatCO2(v).value} />
  {/if}
</section>

<section class="grid-2">
  <div class="card">
    <h3>Top modèles (gCO₂eq)</h3>
    {#if loading}<p class="muted">Chargement…</p>{:else}
      <BarChart
        bars={modelBars}
        color="var(--lime)"
        valueFormat={(v) => `${formatCO2(v).value} ${formatCO2(v).unit}`}
      />
    {/if}
  </div>
  <div class="card">
    <div class="row">
      <h3>Participants</h3>
      <span class="opt-in-hint">
        {data?.policy === 'identified'
          ? 'nominatif — ADR-0016'
          : data?.policy === 'anonymous'
            ? 'anonyme strict — ADR-0016'
            : 'partage opt-in — ADR-0015'}
      </span>
    </div>
    {#if loading}<p class="muted">Chargement…</p>{:else if userBars.length === 0}
      <p class="muted">
        {data?.policy === 'anonymous'
          ? 'Politique « anonyme strict » : aucune identification individuelle, même volontaire. Seuls les agrégats d’équipe sont visibles.'
          : 'Aucun participant n’a activé le partage identifié pour l’instant. Chaque employé décide depuis son espace « Mon usage » ; en attendant, seuls les agrégats anonymes d’équipe sont visibles.'}
      </p>
    {:else}
      <BarChart
        bars={userBars}
        color="var(--amber)"
        valueFormat={(v) => `${formatCO2(v).value} ${formatCO2(v).unit}`}
      />
      {#if canDrill && (data?.top_users.identified.length ?? 0) > 0}
        <ul class="drill-list">
          {#each data?.top_users.identified ?? [] as u (u.user_id)}
            <li>
              <a href={`/admin/users/${u.user_id}`}>
                {u.display_name || u.fingerprint} — détail →
              </a>
            </li>
          {/each}
        </ul>
      {/if}
    {/if}
  </div>
</section>

<section class="grid-2">
  <div class="card">
    <h3>Par projet (gCO₂eq)</h3>
    {#if loading}<p class="muted">Chargement…</p>{:else if projectBars.length === 0}
      <p class="muted">
        Aucune mesure taguée par projet sur la fenêtre. Les projets sont choisis par conversation
        dans l'extension (popup → « Projet de cette conversation »).
      </p>
    {:else}
      <BarChart
        bars={projectBars}
        color="var(--blue, #7eb6ff)"
        valueFormat={(v) => `${formatCO2(v).value} ${formatCO2(v).unit}`}
      />
    {/if}
  </div>
  <div class="card">
    <h3>Lecture</h3>
    <p class="muted">
      Les projets comptant moins de contributeurs que le seuil k sont repliés dans « Autres
      projets » (modes anonyme et opt-in) : un projet d'une seule personne est une personne. En
      mode nominatif attesté, tous les projets sont détaillés.
    </p>
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
  .opt-in-hint {
    color: var(--ivory-2);
    font-size: var(--fs-caption);
  }
  .k-blocked {
    margin-bottom: var(--sp-5);
    border-left: 3px solid var(--amber);
  }
  .policy-badge {
    margin-top: var(--sp-2);
    font-size: var(--fs-caption);
    color: var(--ivory-2);
  }
  .policy-badge[data-policy='identified'] strong {
    color: var(--amber);
  }
  .policy-badge[data-policy='opt_in'] strong,
  .policy-badge[data-policy='anonymous'] strong {
    color: var(--lime);
  }
  .drill-list {
    list-style: none;
    margin: var(--sp-3) 0 0;
    padding: 0;
    display: flex;
    flex-wrap: wrap;
    gap: var(--sp-2) var(--sp-4);
    font-size: var(--fs-caption);
  }
</style>
