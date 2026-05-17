<script lang="ts">
  interface Props {
    tone?: 'lime' | 'amber' | 'coral' | 'neutral';
    children: import('svelte').Snippet;
  }

  let { tone = 'neutral', children }: Props = $props();

  const tones = {
    lime: 'bg-lime-signature/15 text-lime-signature border-lime-signature/30',
    amber: 'bg-amber/15 text-amber border-amber/30',
    coral: 'bg-coral/15 text-coral border-coral/30',
    neutral: 'bg-ivory-4/15 text-ivory-2 border-ivory-4/40',
  };
</script>

<span
  class={`inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-sans ${tones[tone]}`}
>
  {@render children()}
</span>
