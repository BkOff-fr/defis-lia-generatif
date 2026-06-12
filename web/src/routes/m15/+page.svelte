<script lang="ts">
  // C42 — ancienne URL « /m15 » : redirection permanente côté client vers
  // « /suivi » (slugs parlants ; les codes modules restent internes).
  // Adapter-static SPA : pas de redirect serveur possible, on remplace
  // l'entrée d'historique (pas de retour piégé sur la page morte).
  import { onMount } from 'svelte';
  onMount(() => {
    const { search, hash } = window.location;
    window.location.replace(`/suivi${search}${hash}`);
  });
</script>

<p class="redirect-note">Redirection vers <a href="/suivi">/suivi</a>…</p>

<style>
  .redirect-note {
    padding: 48px;
    color: var(--ivory-2);
    font: 400 var(--fs-body) / var(--lh-body) var(--font-ui);
  }
</style>
