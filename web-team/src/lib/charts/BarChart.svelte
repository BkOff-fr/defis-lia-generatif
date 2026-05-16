<script lang="ts">
  // Bar chart horizontal — top modèles / top users. SVG natif.

  interface Bar {
    label: string;
    value: number;
    detail?: string;
  }

  interface Props {
    bars: Bar[];
    color?: string;
    valueFormat?: (v: number) => string;
  }

  let { bars, color = 'var(--lime)', valueFormat }: Props = $props();

  const max = $derived(bars.length === 0 ? 1 : Math.max(...bars.map((b) => b.value), 0.0001));
  const rowHeight = 32;
  const labelWidth = 200;
  const valueWidth = 100;
  const barWidthMax = 320;
  const width = labelWidth + barWidthMax + valueWidth;
  const height = $derived(Math.max(80, bars.length * rowHeight + 16));

  function fmt(v: number): string {
    return valueFormat ? valueFormat(v) : v.toFixed(2);
  }
</script>

<svg viewBox="0 0 {width} {height}" preserveAspectRatio="xMidYMin meet" role="img" aria-label="Top barres">
  {#if bars.length === 0}
    <text
      x={width / 2}
      y={height / 2}
      text-anchor="middle"
      fill="var(--ivory-3)"
      font-size="13"
    >
      Aucune donnée.
    </text>
  {:else}
    {#each bars as b, i}
      <g transform="translate(0, {i * rowHeight + 8})">
        <text x={labelWidth - 12} y={rowHeight / 2} dy="0.32em" text-anchor="end" fill="var(--ivory-2)" font-size="12">
          {b.label}
        </text>
        <rect
          x={labelWidth}
          y={4}
          width={(b.value / max) * barWidthMax}
          height={rowHeight - 12}
          rx={3}
          fill={color}
          fill-opacity="0.85"
        />
        <text x={labelWidth + (b.value / max) * barWidthMax + 8} y={rowHeight / 2} dy="0.32em" fill="var(--ivory)" font-size="12">
          {fmt(b.value)}{b.detail ? ` · ${b.detail}` : ''}
        </text>
      </g>
    {/each}
  {/if}
</svg>
