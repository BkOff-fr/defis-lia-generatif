<script lang="ts">
  // Bannière du mode démo web (C36). Affichée uniquement hors contexte
  // Tauri (jamais dans l'application de bureau) — cf. `$lib/demo`.
  // Transparence méthodologique : les chiffres affichés sortent du vrai
  // moteur (fixtures seed 42) mais sur des requêtes d'exemple.
</script>

<aside class="demo-banner" role="status" aria-label="Mode démonstration">
  <span class="demo-dot" aria-hidden="true"></span>
  <span class="demo-text">
    <strong>Mode démo</strong> — données d'exemple précalculées par le moteur Monte-Carlo (seed 42).
    Pour mesurer vos propres usages,
    <a
      href="https://github.com/BkOff-fr/defis-lia-generatif/releases"
      target="_blank"
      rel="noopener noreferrer">installez l'application de bureau</a
    >.
  </span>
</aside>

<style>
  .demo-banner {
    position: sticky;
    top: 0;
    z-index: 10;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 20px;
    background: rgba(245, 183, 105, 0.1);
    border-bottom: 1px solid rgba(245, 183, 105, 0.25);
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
  }
  .demo-dot {
    flex: none;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--amber);
    box-shadow: 0 0 8px rgba(245, 183, 105, 0.6);
  }
  .demo-text {
    font: 500 var(--fs-caption) / var(--lh-caption) var(--font-ui);
    color: var(--ivory-2);
  }
  .demo-text strong {
    color: var(--amber);
    font-weight: 600;
  }
  .demo-text a {
    color: var(--amber);
    border-bottom-color: rgba(245, 183, 105, 0.4);
  }
  .demo-text a:hover {
    border-bottom-color: var(--amber);
  }
</style>
