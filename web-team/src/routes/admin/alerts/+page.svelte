<script lang="ts">
  import { onMount } from 'svelte';
  import { apiDelete, apiGet, apiPost, ApiError } from '$lib/api';
  import { formatDateTime } from '$lib/format';

  // C29.4 — page admin "Alertes seuils".
  //
  // Routes consommées :
  // - GET    /api/v1/admin/alerts          → liste actifs+désactivés
  // - POST   /api/v1/admin/alerts          → crée un seuil
  // - DELETE /api/v1/admin/alerts/:id      → soft delete (disabled_at)
  // - GET    /api/v1/admin/alerts/triggers → historique (50 derniers)

  type Scope = 'user' | 'team';
  type Period = 'daily' | 'weekly' | 'monthly';
  type NotifyKind = 'webhook' | 'email' | 'log_only';

  interface Threshold {
    id: string;
    scope: Scope;
    target_id: string | null;
    period: Period;
    gco2eq_max: number;
    notify_kind: NotifyKind;
    notify_target: string | null;
    created_by_admin_id: string;
    created_at: string;
    disabled_at: string | null;
  }

  interface TriggerRow {
    id: string;
    threshold_id: string;
    period_start: string;
    period_end: string;
    observed_gco2eq: number;
    triggered_at: string;
    notified_at: string | null;
    notify_error: string | null;
  }

  interface UserRow {
    id: string;
    display_name: string | null;
    fingerprint: string;
  }

  // ── State ──────────────────────────────────────────────────────────────
  let thresholds = $state<Threshold[]>([]);
  let triggers = $state<TriggerRow[]>([]);
  let users = $state<UserRow[]>([]);
  let loading = $state(true);
  let error = $state('');
  let creating = $state(false);

  // Form
  let scope = $state<Scope>('team');
  let targetUserId = $state<string>('');
  let period = $state<Period>('daily');
  let gco2eqMax = $state<number>(100);
  let notifyKind = $state<NotifyKind>('log_only');
  let notifyTarget = $state<string>('');

  async function load() {
    loading = true;
    error = '';
    try {
      const [tList, tTriggers, uList] = await Promise.all([
        apiGet<{ thresholds: Threshold[] }>('/api/v1/admin/alerts'),
        apiGet<{ triggers: TriggerRow[] }>('/api/v1/admin/alerts/triggers?limit=50'),
        apiGet<{ users: UserRow[] }>('/api/v1/admin/users')
      ]);
      thresholds = tList.thresholds;
      triggers = tTriggers.triggers;
      users = uList.users;
    } catch (e) {
      error = e instanceof ApiError ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  onMount(load);

  async function createThreshold() {
    creating = true;
    error = '';
    try {
      const body: Record<string, unknown> = {
        scope,
        period,
        gco2eq_max: gco2eqMax,
        notify_kind: notifyKind
      };
      if (scope === 'user') {
        if (!targetUserId) {
          error = 'Sélectionnez un utilisateur';
          creating = false;
          return;
        }
        body.target_id = targetUserId;
      }
      if (notifyKind !== 'log_only') {
        if (!notifyTarget.trim()) {
          error = notifyKind === 'webhook' ? 'URL webhook requise' : 'Adresse email requise';
          creating = false;
          return;
        }
        body.notify_target = notifyTarget.trim();
      }
      await apiPost('/api/v1/admin/alerts', body);
      // Reset form + reload list
      gco2eqMax = 100;
      notifyTarget = '';
      await load();
    } catch (e) {
      error = e instanceof ApiError ? e.message : String(e);
    } finally {
      creating = false;
    }
  }

  async function disableThreshold(id: string) {
    if (!confirm('Désactiver ce seuil ? Les alertes en cours ne seront pas annulées.')) return;
    error = '';
    try {
      await apiDelete(`/api/v1/admin/alerts/${id}`);
      await load();
    } catch (e) {
      error = e instanceof ApiError ? e.message : String(e);
    }
  }

  function thresholdLabel(t: Threshold): string {
    const scopeLabel = t.scope === 'team' ? 'Équipe' : 'User';
    const target =
      t.scope === 'user'
        ? (users.find((u) => u.id === t.target_id)?.display_name ??
          users.find((u) => u.id === t.target_id)?.fingerprint ??
          t.target_id?.slice(0, 8) ??
          '—')
        : '';
    return target ? `${scopeLabel} · ${target}` : scopeLabel;
  }
</script>

<section class="head">
  <div>
    <h2>Alertes seuils</h2>
    <p class="muted">
      Définissez un plafond de gCO₂eq par période (jour / semaine / mois) pour un utilisateur ou
      pour toute l'équipe. Lorsque la consommation observée dépasse le seuil, une notification est
      envoyée (webhook, email ou simple log). Un seul déclenchement par période est garanti.
    </p>
  </div>
</section>

{#if error}
  <p class="error" role="alert" data-testid="alerts-error">{error}</p>
{/if}

<div class="card">
  <h3>Créer une alerte</h3>
  <div class="form-grid">
    <div class="row">
      <span class="form-label">Portée</span>
      <label><input type="radio" bind:group={scope} value="team" /> Toute l'équipe</label>
      <label><input type="radio" bind:group={scope} value="user" /> Utilisateur</label>
    </div>

    {#if scope === 'user'}
      <div class="row">
        <label for="alert-user">Utilisateur</label>
        <select id="alert-user" bind:value={targetUserId}>
          <option value="" disabled>— sélectionner —</option>
          {#each users as u (u.id)}
            <option value={u.id}>{u.display_name ?? u.fingerprint} ({u.id.slice(0, 8)}…)</option>
          {/each}
        </select>
      </div>
    {/if}

    <div class="row">
      <span class="form-label">Période</span>
      <label><input type="radio" bind:group={period} value="daily" /> Jour</label>
      <label><input type="radio" bind:group={period} value="weekly" /> Semaine</label>
      <label><input type="radio" bind:group={period} value="monthly" /> Mois</label>
    </div>

    <div class="row">
      <label for="alert-gco2eq">gCO₂eq max</label>
      <input id="alert-gco2eq" type="number" min="0.01" step="0.01" bind:value={gco2eqMax} />
    </div>

    <div class="row">
      <span class="form-label">Notification</span>
      <label><input type="radio" bind:group={notifyKind} value="log_only" /> Log uniquement</label>
      <label><input type="radio" bind:group={notifyKind} value="webhook" /> Webhook</label>
      <label><input type="radio" bind:group={notifyKind} value="email" /> Email</label>
    </div>

    {#if notifyKind === 'webhook'}
      <div class="row">
        <label for="alert-webhook">URL webhook</label>
        <input
          id="alert-webhook"
          type="url"
          placeholder="https://hooks.example.com/sobria"
          bind:value={notifyTarget}
        />
      </div>
    {:else if notifyKind === 'email'}
      <div class="row">
        <label for="alert-email">Adresse email</label>
        <input
          id="alert-email"
          type="email"
          placeholder="ops@example.org"
          bind:value={notifyTarget}
        />
        <span class="hint">
          SMTP doit être configuré (table <code>config</code> : <code>smtp_url</code> +
          <code>smtp_from</code>) — sinon fallback log uniquement.
        </span>
      </div>
    {/if}

    <div class="actions">
      <button class="primary" onclick={createThreshold} disabled={creating}>
        {#if creating}Création…{:else}Créer l'alerte{/if}
      </button>
    </div>
  </div>
</div>

<div class="card">
  <h3>Seuils définis</h3>
  {#if loading}
    <p class="muted">Chargement…</p>
  {:else if thresholds.length === 0}
    <p class="muted">Aucun seuil défini pour l'instant.</p>
  {:else}
    <table data-testid="alerts-thresholds-table">
      <thead>
        <tr>
          <th>Portée</th>
          <th>Période</th>
          <th>Max</th>
          <th>Notif.</th>
          <th>Cible</th>
          <th>Créé</th>
          <th>État</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        {#each thresholds as t (t.id)}
          <tr class:disabled={!!t.disabled_at}>
            <td>{thresholdLabel(t)}</td>
            <td>{t.period}</td>
            <td><code>{t.gco2eq_max.toFixed(2)} g</code></td>
            <td>{t.notify_kind}</td>
            <td class="truncate">{t.notify_target ?? '—'}</td>
            <td>{formatDateTime(t.created_at)}</td>
            <td>
              {#if t.disabled_at}
                <span class="badge muted">désactivé</span>
              {:else}
                <span class="badge lime">actif</span>
              {/if}
            </td>
            <td>
              {#if !t.disabled_at}
                <button class="danger" onclick={() => disableThreshold(t.id)}> Désactiver </button>
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<div class="card">
  <h3>Historique des déclenchements (50 derniers)</h3>
  {#if loading}
    <p class="muted">Chargement…</p>
  {:else if triggers.length === 0}
    <p class="muted">Aucun déclenchement enregistré.</p>
  {:else}
    <table data-testid="alerts-triggers-table">
      <thead>
        <tr>
          <th>Déclenché</th>
          <th>Seuil</th>
          <th>Période</th>
          <th>Observé</th>
          <th>Notif.</th>
        </tr>
      </thead>
      <tbody>
        {#each triggers as tr (tr.id)}
          <tr>
            <td>{formatDateTime(tr.triggered_at)}</td>
            <td><code>{tr.threshold_id.slice(0, 8)}…</code></td>
            <td>{formatDateTime(tr.period_start)} → {formatDateTime(tr.period_end)}</td>
            <td><code>{tr.observed_gco2eq.toFixed(2)} g</code></td>
            <td>
              {#if tr.notified_at && !tr.notify_error}
                <span class="badge lime">OK</span>
              {:else if tr.notify_error}
                <span class="badge amber" title={tr.notify_error}>erreur</span>
              {:else}
                <span class="badge muted">en cours</span>
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<style>
  .head {
    margin-bottom: var(--sp-5);
  }
  .card {
    margin-bottom: var(--sp-5);
  }
  .form-grid {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--sp-3);
  }
  .row label {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    margin: 0;
  }
  .form-label {
    min-width: 100px;
    color: var(--ivory-3);
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .hint {
    font-size: 12px;
    color: var(--ivory-3);
  }
  .actions {
    margin-top: var(--sp-3);
  }
  table {
    width: 100%;
    border-collapse: collapse;
  }
  th,
  td {
    text-align: left;
    padding: var(--sp-2) var(--sp-3);
    border-bottom: 1px solid var(--border);
  }
  tr.disabled {
    opacity: 0.55;
  }
  .truncate {
    max-width: 320px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .error {
    background: rgba(255, 128, 144, 0.08);
    border: 1px solid rgba(255, 128, 144, 0.3);
    color: var(--coral, #ff8090);
    padding: var(--sp-3) var(--sp-4);
    border-radius: var(--radius-sm, 6px);
    margin-bottom: var(--sp-4);
  }
  .badge {
    display: inline-block;
    padding: 2px 8px;
    border-radius: 999px;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .badge.lime {
    background: rgba(197, 240, 74, 0.1);
    color: var(--lime, #c5f04a);
  }
  .badge.amber {
    background: rgba(255, 178, 92, 0.1);
    color: var(--amber, #ffb25c);
  }
  .badge.muted {
    background: rgba(255, 255, 255, 0.05);
    color: var(--ivory-3, rgba(244, 240, 232, 0.6));
  }
</style>
