<script lang="ts">
  import type { Component } from 'svelte';

  interface Props {
    icon: Component;
    title: string;
    tagline: string;
    href: string;
  }

  let { icon: Icon, title, tagline, href }: Props = $props();
</script>

<a
  {href}
  class="block rounded-lg border border-ivory-4/40 bg-ink-2 p-6 transition-colors hover:border-lime-signature/60 group"
>
  <div class="flex items-center gap-3">
    <Icon size={28} class="text-lime-signature transition-colors group-hover:text-lime-deep" />
    <h3 class="font-sans text-lg text-ivory">{title}</h3>
  </div>
  <p class="mt-3 text-sm text-ivory-2 leading-relaxed">{tagline}</p>
  <p
    class="mt-4 text-xs font-sans uppercase tracking-wider text-ivory-3 group-hover:text-lime-signature transition-colors"
  >
    Voir le guide →
  </p>
</a>
