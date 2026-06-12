<script lang="ts">
  // C42 — ancienne URL « /m17 » : redirection permanente côté client vers
  // « /datasheets » (slugs parlants ; les codes modules restent internes).
  // Adapter-static SPA : pas de redirect serveur possible, on remplace
  // l'entrée d'historique (pas de retour piégé sur la page morte).
  import { onMount } from 'svelte';
  onMount(() => {
    const { search, hash } = window.location;
    window.location.replace(`/datasheets${search}${hash}`);
  });
</script>

<p class="redirect-note">Redirection vers <a href="/datasheets">/datasheets</a>…</p>

<style>
  .redirect-note {
    padding: 48px;
    color: var(--ivory-2);
    font: 400 var(--fs-body) / var(--lh-body) var(--font-ui);
  }
</style>
