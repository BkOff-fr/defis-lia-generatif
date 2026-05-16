<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { auth } from '$lib/auth.svelte';
  import Nav from '$lib/components/Nav.svelte';
  import '../app.css';

  let { children } = $props();

  const PUBLIC_PATHS = ['/login', '/enroll'];

  onMount(async () => {
    await auth.hydrate();
    const pathname = $page.url.pathname;
    if (!auth.loggedIn && !PUBLIC_PATHS.includes(pathname)) {
      goto('/login');
    } else if (auth.loggedIn && pathname === '/login') {
      gotoRoleHome();
    } else if (pathname === '/' && auth.loggedIn) {
      gotoRoleHome();
    } else if (pathname === '/') {
      goto('/login');
    }
  });

  function gotoRoleHome() {
    if (auth.role === 'admin') goto('/admin/dashboard');
    else goto('/user/dashboard');
  }

  const showChrome = $derived(auth.loggedIn && !PUBLIC_PATHS.includes($page.url.pathname));
</script>

{#if auth.loading}
  <div class="boot">
    <span>Chargement…</span>
  </div>
{:else}
  {#if showChrome}
    <Nav />
  {/if}
  <main class:embedded={showChrome}>
    {@render children()}
  </main>
{/if}

<style>
  .boot {
    min-height: 100vh;
    display: grid;
    place-items: center;
    color: var(--ivory-3);
  }
  main {
    padding: var(--sp-6);
    max-width: 1200px;
    margin: 0 auto;
  }
  main.embedded {
    padding-top: var(--sp-5);
  }
</style>
