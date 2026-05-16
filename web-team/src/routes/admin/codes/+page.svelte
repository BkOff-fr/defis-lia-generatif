<script lang="ts">
  import { onMount } from 'svelte';
  import { apiDelete, apiGet, apiPost, ApiError } from '$lib/api';
  import { formatDateTime } from '$lib/format';

  // /admin/codes ne renvoie pas la liste côté API pour C28.3 — c'est CLI-only.
  // En attendant un GET admin (C28.5 si besoin), on garde la liste des codes
  // créés dans la session (in-memory) pour pouvoir les révoquer depuis l'UI.

  interface CreatedCode {
    id: string;
    code: string;
    expires_at: string;
  }

  let count = $state(10);
  let ttlDays = $state(7);
  let creating = $state(false);
  let error = $state('');
  let lastBatch = $state<CreatedCode[]>([]);

  async function create() {
    creating = true;
    error = '';
    try {
      const resp = await apiPost<{ codes: CreatedCode[] }>('/api/v1/admin/codes', {
        count,
        ttl_days: ttlDays
      });
      lastBatch = resp.codes;
    } catch (e) {
      error = e instanceof ApiError ? e.message : String(e);
    } finally {
      creating = false;
    }
  }

  async function revoke(id: string) {
    error = '';
    try {
      await apiDelete<{ revoked: boolean }>(`/api/v1/admin/codes/${id}`);
      lastBatch = lastBatch.filter((c) => c.id !== id);
    } catch (e) {
      error = e instanceof ApiError ? e.message : String(e);
    }
  }

  async function copyAll() {
    const lines = lastBatch.map((c, i) => `${i + 1}. ${c.code}`).join('\n');
    try {
      await navigator.clipboard.writeText(lines);
    } catch {
      // pas de clipboard : on ignore
    }
  }
</script>

<section class="head">
  <div>
    <h2>Codes d'enrôlement</h2>
    <p class="muted">
      Chaque code single-use 12 chiffres permet à un employé de s'enrôler depuis son
      navigateur ou son app Sobr.ia. Argon2id PHC en base, donc <strong>impossible</strong>
      d'afficher un code une fois la session fermée.
    </p>
  </div>
</section>

<div class="card create-card">
  <h3>Créer un lot</h3>
  <div class="grid">
    <div>
      <label for="count">Nombre de codes</label>
      <input id="count" type="number" min="1" max="500" bind:value={count} />
    </div>
    <div>
      <label for="ttl">TTL (jours)</label>
      <input id="ttl" type="number" min="1" max="365" bind:value={ttlDays} />
    </div>
    <button class="primary" onclick={create} disabled={creating}>
      {#if creating}Création…{:else}Générer{/if}
    </button>
  </div>
  {#if error}<p class="error">{error}</p>{/if}
</div>

{#if lastBatch.length > 0}
  <div class="card warning">
    <div class="row">
      <h3>Lot fraîchement créé ({lastBatch.length})</h3>
      <button onclick={copyAll}>Copier la liste</button>
    </div>
    <p class="muted">
      ⚠️ Ces codes ne seront plus affichés en clair une fois cette page rechargée.
      Distribuez-les par un canal sûr (gestionnaire de mots de passe, mail chiffré).
    </p>
    <table>
      <thead>
        <tr>
          <th>#</th>
          <th>Code</th>
          <th>Expire le</th>
          <th>Action</th>
        </tr>
      </thead>
      <tbody>
        {#each lastBatch as c, i}
          <tr>
            <td>{i + 1}</td>
            <td><code class="big">{c.code}</code></td>
            <td>{formatDateTime(c.expires_at)}</td>
            <td>
              <button class="danger" onclick={() => revoke(c.id)}>Révoquer</button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}

<style>
  .head {
    margin-bottom: var(--sp-5);
  }
  .create-card {
    margin-bottom: var(--sp-5);
  }
  .grid {
    display: grid;
    grid-template-columns: 160px 160px auto;
    gap: var(--sp-4);
    align-items: end;
  }
  .row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--sp-3);
  }
  .warning {
    border-color: rgba(245, 183, 105, 0.4);
    box-shadow: 0 0 0 1px rgba(245, 183, 105, 0.15);
  }
  code.big {
    font-size: 16px;
    letter-spacing: 0.08em;
    color: var(--lime);
  }
</style>
