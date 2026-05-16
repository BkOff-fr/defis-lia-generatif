<script lang="ts">
  interface Props {
    eyebrow: string;
    value: string;
    unit?: string;
    hint?: string;
  }

  let { eyebrow, value, unit, hint }: Props = $props();
</script>

<div class="metric-card">
  <div class="eyebrow">{eyebrow}</div>
  <div>
    <span class="value">{value}</span>
    {#if unit}<span class="unit">{unit}</span>{/if}
  </div>
  {#if hint}<div class="hint muted">{hint}</div>{/if}
</div>

<style>
  .hint {
    font-size: var(--fs-caption);
    margin-top: var(--sp-2);
  }
</style>
