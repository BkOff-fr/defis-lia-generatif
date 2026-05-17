<script lang="ts">
  import { onMount } from 'svelte';

  let open = $state(false);
  let containerEl: HTMLDivElement | null = $state(null);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let pagefindUI: any = null;

  async function loadPagefindAssets(): Promise<void> {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    if ((window as any).PagefindUI) return;
    if (!document.querySelector('link[data-pagefind-css]')) {
      const link = document.createElement('link');
      link.rel = 'stylesheet';
      link.href = '/pagefind/pagefind-ui.css';
      link.setAttribute('data-pagefind-css', 'true');
      document.head.appendChild(link);
    }
    await new Promise<void>((resolve, reject) => {
      const existing = document.querySelector('script[data-pagefind-js]');
      if (existing) {
        existing.addEventListener('load', () => resolve(), { once: true });
        existing.addEventListener('error', () => reject(new Error('pagefind load failed')), {
          once: true,
        });
        return;
      }
      const script = document.createElement('script');
      script.src = '/pagefind/pagefind-ui.js';
      script.setAttribute('data-pagefind-js', 'true');
      script.onload = () => resolve();
      script.onerror = () => reject(new Error('pagefind load failed'));
      document.head.appendChild(script);
    });
  }

  async function ensurePagefind() {
    if (pagefindUI || !containerEl) return;
    try {
      await loadPagefindAssets();
    } catch {
      console.warn('Pagefind UI non disponible — exécutez `npm run build` pour générer l’index.');
      return;
    }
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const Ctor = (window as any).PagefindUI;
    if (typeof Ctor === 'function') {
      pagefindUI = new Ctor({
        element: containerEl,
        showSubResults: true,
        translations: {
          placeholder: 'Rechercher dans la documentation…',
          clear_search: 'Effacer',
          load_more: 'Voir plus',
          search_label: 'Rechercher',
          filters_label: 'Filtres',
          zero_results: 'Aucun résultat pour [SEARCH_TERM]',
          many_results: '[COUNT] résultats pour [SEARCH_TERM]',
          one_result: '1 résultat pour [SEARCH_TERM]',
          alt_search: 'Aucun résultat pour [SEARCH_TERM]. Essayez [DIFFERENT_TERM] ?',
          search_suggestion: 'Aucun résultat pour [SEARCH_TERM]. Suggestions :',
          searching: 'Recherche…',
        },
      });
    }
  }

  function toggle() {
    open = !open;
    if (open) ensurePagefind();
  }

  onMount(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && open) open = false;
      if ((e.key === '/' || (e.metaKey && e.key === 'k')) && !open) {
        const tag = (document.activeElement?.tagName ?? '').toLowerCase();
        if (tag === 'input' || tag === 'textarea') return;
        e.preventDefault();
        toggle();
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  });
</script>

<button
  type="button"
  onclick={toggle}
  aria-label="Rechercher dans la documentation"
  class="inline-flex items-center gap-2 rounded-md border border-ivory-4/40 px-3 py-1.5 text-xs text-ivory-2 hover:border-lime-signature/40 hover:text-ivory transition-colors"
>
  <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
    <path
      d="M11.742 10.344a6.5 6.5 0 1 0-1.397 1.398h-.001c.03.04.062.078.098.115l3.85 3.85a1 1 0 0 0 1.415-1.414l-3.85-3.85a1.007 1.007 0 0 0-.115-.1zM12 6.5a5.5 5.5 0 1 1-11 0 5.5 5.5 0 0 1 11 0z"
    />
  </svg>
  <span class="hidden sm:inline">Rechercher</span>
  <kbd
    class="ml-1 hidden md:inline px-1.5 py-0.5 rounded border border-ivory-4/40 font-mono text-[10px]"
    >/</kbd
  >
</button>

{#if open}
  <div
    role="dialog"
    aria-modal="true"
    aria-label="Recherche documentation"
    class="fixed inset-0 z-[100] bg-ink/80 backdrop-blur-sm flex items-start justify-center pt-24 px-4"
    onclick={(e) => e.target === e.currentTarget && (open = false)}
    onkeydown={(e) => e.key === 'Escape' && (open = false)}
    tabindex="-1"
  >
    <div class="bg-ink-2 border border-ivory-4/40 rounded-lg p-4 w-full max-w-2xl shadow-2xl">
      <div bind:this={containerEl} class="pagefind-host"></div>
      <p class="mt-3 text-xs text-ivory-3">
        Astuce : appuyez sur <kbd class="px-1.5 py-0.5 rounded border border-ivory-4/40 font-mono"
          >Esc</kbd
        > pour fermer.
      </p>
    </div>
  </div>
{/if}

<style>
  :global(.pagefind-ui) {
    --pagefind-ui-scale: 0.9;
    --pagefind-ui-primary: #c5f04a;
    --pagefind-ui-text: #f0ece3;
    --pagefind-ui-background: #131815;
    --pagefind-ui-border: rgba(70, 68, 63, 0.4);
    --pagefind-ui-tag: rgba(255, 255, 255, 0.07);
    --pagefind-ui-border-width: 1px;
    --pagefind-ui-border-radius: 8px;
    --pagefind-ui-image-border-radius: 6px;
    --pagefind-ui-font: var(--font-sans);
  }
</style>
