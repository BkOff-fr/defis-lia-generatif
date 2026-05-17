<script lang="ts">
  interface Props {
    href?: string;
    variant?: 'primary' | 'secondary' | 'ghost';
    size?: 'sm' | 'md' | 'lg';
    type?: 'button' | 'submit';
    disabled?: boolean;
    onclick?: (e: MouseEvent) => void;
    children: import('svelte').Snippet;
  }

  let {
    href,
    variant = 'primary',
    size = 'md',
    type = 'button',
    disabled = false,
    onclick,
    children,
  }: Props = $props();

  const base =
    'inline-flex items-center justify-center gap-2 rounded-md font-sans font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed';
  const sizes = {
    sm: 'px-3 py-1.5 text-sm',
    md: 'px-4 py-2 text-sm',
    lg: 'px-6 py-3 text-base',
  };
  const variants = {
    primary: 'bg-lime-signature text-ink hover:bg-lime-deep',
    secondary:
      'border border-ivory-4 text-ivory hover:border-lime-signature hover:text-lime-signature',
    ghost: 'text-ivory-2 hover:text-ivory',
  };

  const cls = $derived(`${base} ${sizes[size]} ${variants[variant]}`);
</script>

{#if href}
  <a {href} class={cls} aria-disabled={disabled}>
    {@render children()}
  </a>
{:else}
  <button {type} {onclick} {disabled} class={cls}>
    {@render children()}
  </button>
{/if}
