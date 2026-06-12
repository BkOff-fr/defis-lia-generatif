<script lang="ts">
  // C41 — Terme technique avec définition en tooltip accessible
  // (hover + focus clavier) et lien vers le glossaire complet (M8).
  import { LEXIQUE, type TermKey } from '$lib/lexique';

  type Props = {
    k: TermKey;
    children?: import('svelte').Snippet;
  };
  let { k, children }: Props = $props();
  // $derived : suit k si le terme change (et calme state_referenced_locally).
  const suffix = Math.random().toString(36).slice(2, 7);
  const tipId = $derived(`term-tip-${k}-${suffix}`);
</script>

<span class="term">
  <a class="term-anchor" href="/methodo#glossaire" aria-describedby={tipId}>
    {@render children?.()}
  </a>
  <span class="term-tip" role="tooltip" id={tipId}>
    {LEXIQUE[k]}
    <span class="term-more">Glossaire complet →</span>
  </span>
</span>

<style>
  .term {
    position: relative;
    display: inline-block;
  }
  .term-anchor {
    color: inherit;
    border-bottom: 1px dotted var(--ivory-3);
    cursor: help;
  }
  .term-anchor:hover,
  .term-anchor:focus-visible {
    border-bottom-color: var(--lime);
  }
  .term-tip {
    position: absolute;
    bottom: calc(100% + 8px);
    /* Ancrage gauche : le tip s'étend vers la droite — jamais coupé par le
       bord gauche du bloc (cas P5–P95 dans la pill du ResultBlock). */
    left: 0;
    width: max-content;
    max-width: 280px;
    padding: 10px 12px;
    background: var(--ink-3);
    border: 1px solid var(--border-hi);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-pop);
    font: 400 var(--fs-caption) / var(--lh-caption) var(--font-ui);
    color: var(--ivory-2);
    text-transform: none;
    letter-spacing: normal;
    text-align: left;
    opacity: 0;
    pointer-events: none;
    transition: opacity var(--dur-fast) var(--ease);
    z-index: 30;
  }
  .term:hover .term-tip,
  .term:focus-within .term-tip {
    opacity: 1;
  }
  .term-more {
    display: block;
    margin-top: 6px;
    color: var(--lime);
  }
</style>
