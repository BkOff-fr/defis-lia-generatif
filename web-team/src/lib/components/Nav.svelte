<script lang="ts">
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { auth } from '$lib/auth.svelte';

  interface Link {
    href: string;
    label: string;
  }

  const adminLinks: Link[] = [
    { href: '/admin/dashboard', label: 'Dashboard' },
    { href: '/admin/codes', label: 'Codes' },
    { href: '/admin/users', label: 'Employés' }
  ];
  const userLinks: Link[] = [{ href: '/user/dashboard', label: 'Mon usage' }];

  const links = $derived(auth.role === 'admin' ? adminLinks : userLinks);

  function logout() {
    auth.logout();
    goto('/login');
  }
</script>

<header>
  <div class="brand">
    <span class="logo">Sobr.ia</span>
    <span class="sub">Mode Équipe</span>
  </div>
  <nav>
    {#each links as l (l.href)}
      <a href={l.href} class:active={$page.url.pathname === l.href}>{l.label}</a>
    {/each}
  </nav>
  <div class="user">
    <span class="badge {auth.role === 'admin' ? 'lime' : 'amber'}">{auth.role ?? '—'}</span>
    <button class="ghost" onclick={logout}>Déconnexion</button>
  </div>
</header>

<style>
  header {
    display: flex;
    align-items: center;
    gap: var(--sp-6);
    padding: var(--sp-4) var(--sp-6);
    border-bottom: 1px solid var(--border);
    background: var(--ink-2);
  }
  .brand {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .logo {
    font-family: var(--font-display);
    font-size: 22px;
    color: var(--lime);
  }
  .sub {
    font-size: var(--fs-eyebrow);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--ivory-3);
  }
  nav {
    display: flex;
    gap: var(--sp-4);
    flex: 1;
  }
  nav a {
    color: var(--ivory-2);
    padding: var(--sp-2) var(--sp-3);
    border-radius: var(--r-sm);
    transition: background var(--dur) var(--ease);
  }
  nav a:hover {
    background: var(--surface);
    text-decoration: none;
  }
  nav a.active {
    color: var(--lime);
    background: var(--lime-soft);
  }
  .user {
    display: flex;
    align-items: center;
    gap: var(--sp-3);
  }
</style>
