<script lang="ts">
  interface Props {
    as?: 'div' | 'article' | 'section';
    children: import('svelte').Snippet;
  }

  let { as = 'div', children }: Props = $props();
</script>

<svelte:element
  this={as}
  class="rounded-lg border border-ivory-4/40 bg-ink-2 p-6 transition-colors hover:border-lime-signature/40"
>
  {@render children()}
</svelte:element>
