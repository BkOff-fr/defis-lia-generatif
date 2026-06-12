<script lang="ts">
  import { onMount } from 'svelte';
  import { apiGet, ApiError } from '$lib/api';
  import { formatCO2, formatCount, formatDateTime } from '$lib/format';

  interface UserRow {
    id: string;
    fingerprint: string;
    display_name: string | null;
    enrollment_code_id: string | null;
    created_at: string;
    last_seen_at: string | null;
    share_identified: boolean;
    /// `null` tant que l'employé n'a pas activé le partage (ADR-0015 §3).
    totals: {
      count: number;
      tokens_in: number;
      tokens_out: number;
      gco2eq_p50_g: number;
      water_ml: number;
      energy_wh: number;
    } | null;
  }

  let users = $state<UserRow[]>([]);
  let loading = $state(true);
  let error = $state('');

  async function load() {
    loading = true;
    error = '';
    try {
      const resp = await apiGet<{ users: UserRow[] }>('/api/v1/admin/users');
      users = resp.users;
    } catch (e) {
      error = e instanceof ApiError ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  onMount(load);
</script>

<section class="head">
  <div>
    <h2>Employés enrôlés</h2>
    <p class="muted">
      {users.length} employé{users.length > 1 ? 's' : ''} avec un fingerprint unique. Les consommations
      individuelles n'apparaissent qu'avec le consentement de chacun (ADR-0015) — cette page sert à gérer
      les enrôlements.
    </p>
  </div>
  <button onclick={load} disabled={loading}>Rafraîchir</button>
</section>

{#if error}<p class="error">{error}</p>{/if}

<div class="card">
  <table>
    <thead>
      <tr>
        <th>Nom / Fingerprint</th>
        <th>Enrôlé le</th>
        <th>Dernière activité</th>
        <th class="num">Prompts</th>
        <th class="num">gCO₂eq</th>
      </tr>
    </thead>
    <tbody>
      {#if loading}
        <tr><td colspan="5" class="muted">Chargement…</td></tr>
      {:else if users.length === 0}
        <tr><td colspan="5" class="muted">Aucun employé enrôlé pour l'instant.</td></tr>
      {:else}
        {#each users as u (u.id)}
          <tr>
            <td>
              <!-- C44 : lien vers le détail individuel — le serveur applique
                   la politique (403 expliqué côté page si non autorisé). -->
              <a class="name" href={`/admin/users/${u.id}`}>{u.display_name || u.fingerprint}</a>
              {#if u.display_name}
                <div class="muted small mono">{u.fingerprint}</div>
              {/if}
            </td>
            <td>{formatDateTime(u.created_at)}</td>
            <td>{formatDateTime(u.last_seen_at)}</td>
            {#if u.totals}
              {@const co2 = formatCO2(u.totals.gco2eq_p50_g)}
              <td class="num">{formatCount(u.totals.count)}</td>
              <td class="num">{co2.value} <span class="muted small">{co2.unit}</span></td>
            {:else}
              <td
                class="num muted"
                colspan="2"
                title="L'employé n'a pas activé le partage identifié (ADR-0015)"
              >
                — partage non activé
              </td>
            {/if}
          </tr>
        {/each}
      {/if}
    </tbody>
  </table>
</div>

<style>
  .head {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    margin-bottom: var(--sp-5);
    gap: var(--sp-4);
  }
  .num {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
  .name {
    color: var(--ivory);
    font-weight: 500;
  }
  .small {
    font-size: var(--fs-caption);
  }
</style>
