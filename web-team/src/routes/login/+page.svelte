<script lang="ts">
  import { goto } from '$app/navigation';
  import { auth } from '$lib/auth.svelte';
  import { browserFingerprint } from '$lib/format';
  import { ApiError } from '$lib/api';

  type Mode = 'admin-login' | 'user-login' | 'enroll';

  let mode = $state<Mode>('admin-login');
  let username = $state('admin');
  let password = $state('');
  let code = $state('');
  let displayName = $state('');
  let error = $state('');
  let busy = $state(false);
  let createdCodesNotice = $state('');

  async function submit(e: Event) {
    e.preventDefault();
    error = '';
    busy = true;
    try {
      if (mode === 'admin-login') {
        await auth.login(username, password, 'admin');
        goto('/admin/dashboard');
      } else if (mode === 'user-login') {
        await auth.login(username, password, 'user');
        goto('/user/dashboard');
      } else {
        const fp = browserFingerprint();
        await auth.enroll(code, password, fp, displayName || undefined);
        goto('/user/dashboard');
      }
    } catch (e) {
      if (e instanceof ApiError) {
        error = e.message;
      } else {
        error = String(e);
      }
    } finally {
      busy = false;
    }
  }
</script>

<div class="wrap">
  <div class="card login">
    <div class="logo">Sobr.ia</div>
    <h2>Mode Équipe</h2>
    <p class="muted">
      Serveur self-hosted — vos données restent chez votre organisation. Agrégats d'équipe
      k-anonymes : aucune surveillance individuelle sans consentement explicite du salarié.
    </p>

    <div class="tabs" role="tablist">
      <button
        type="button"
        class:active={mode === 'admin-login'}
        onclick={() => (mode = 'admin-login')}>Admin</button
      >
      <button
        type="button"
        class:active={mode === 'user-login'}
        onclick={() => (mode = 'user-login')}>Employé</button
      >
      <button type="button" class:active={mode === 'enroll'} onclick={() => (mode = 'enroll')}>
        S'enrôler
      </button>
    </div>

    <form onsubmit={submit}>
      {#if mode === 'admin-login'}
        <label for="username">Nom d'utilisateur admin</label>
        <input id="username" type="text" autocomplete="username" bind:value={username} required />
        <label for="password">Mot de passe</label>
        <input
          id="password"
          type="password"
          autocomplete="current-password"
          bind:value={password}
          required
        />
      {:else if mode === 'user-login'}
        <label for="fingerprint">Identifiant employé (fingerprint)</label>
        <input
          id="fingerprint"
          type="text"
          autocomplete="username"
          bind:value={username}
          required
        />
        <p class="hint muted">
          Si vous vous êtes enrôlé depuis ce navigateur, votre fingerprint est :
          <code>{browserFingerprint()}</code>
        </p>
        <label for="password">Mot de passe</label>
        <input
          id="password"
          type="password"
          autocomplete="current-password"
          bind:value={password}
          required
        />
      {:else}
        <label for="code">Code d'enrôlement (12 chiffres)</label>
        <input
          id="code"
          type="text"
          inputmode="numeric"
          pattern="\d{'{12}'}"
          minlength="12"
          maxlength="12"
          bind:value={code}
          required
        />
        <label for="displayName">Nom (optionnel)</label>
        <input id="displayName" type="text" bind:value={displayName} />
        <label for="password">Choisir un mot de passe (≥ 8 caractères)</label>
        <input
          id="password"
          type="password"
          autocomplete="new-password"
          minlength="8"
          bind:value={password}
          required
        />
        <p class="hint muted">
          Votre fingerprint généré pour ce navigateur :
          <code>{browserFingerprint()}</code>
        </p>
      {/if}

      {#if error}<p class="error">{error}</p>{/if}
      {#if createdCodesNotice}<p class="muted">{createdCodesNotice}</p>{/if}

      <button type="submit" class="primary" disabled={busy}>
        {#if busy}…{:else if mode === 'enroll'}M'enrôler{:else}Se connecter{/if}
      </button>
    </form>
  </div>
</div>

<style>
  .wrap {
    min-height: 80vh;
    display: grid;
    place-items: center;
  }
  .login {
    max-width: 460px;
    width: 100%;
  }
  .logo {
    font-family: var(--font-display);
    font-size: 28px;
    color: var(--lime);
    margin-bottom: var(--sp-1);
  }
  h2 {
    margin-bottom: var(--sp-2);
  }
  .tabs {
    display: flex;
    gap: var(--sp-1);
    margin: var(--sp-4) 0 var(--sp-5);
    background: var(--surface);
    padding: 4px;
    border-radius: var(--r-sm);
    border: 1px solid var(--border);
  }
  .tabs button {
    flex: 1;
    background: transparent;
    border: none;
    color: var(--ivory-2);
    padding: var(--sp-2);
    border-radius: var(--r-sm);
  }
  .tabs button.active {
    background: var(--surface-2);
    color: var(--ivory);
  }
  form {
    display: flex;
    flex-direction: column;
    gap: var(--sp-3);
  }
  form button.primary {
    margin-top: var(--sp-3);
  }
  .hint {
    font-size: var(--fs-caption);
  }
</style>
