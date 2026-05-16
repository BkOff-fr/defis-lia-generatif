<script lang="ts">
  // Donut chart — répartition méthodologie afnor_sobria vs ecologits.
  // SVG natif. Deux à 5 slices max, palette lime/amber/coral/violet/blue.

  interface Slice {
    label: string;
    value: number;
  }

  interface Props {
    slices: Slice[];
    size?: number;
  }

  let { slices, size = 220 }: Props = $props();

  const colors = ['var(--lime)', 'var(--amber)', 'var(--coral)', 'var(--violet)', 'var(--blue)'];
  const total = $derived(slices.reduce((acc, s) => acc + s.value, 0));
  const radius = size / 2 - 8;
  const inner = radius * 0.6;

  function arc(start: number, end: number): string {
    const cx = size / 2;
    const cy = size / 2;
    const sx = cx + radius * Math.cos(start);
    const sy = cy + radius * Math.sin(start);
    const ex = cx + radius * Math.cos(end);
    const ey = cy + radius * Math.sin(end);
    const isx = cx + inner * Math.cos(end);
    const isy = cy + inner * Math.sin(end);
    const iex = cx + inner * Math.cos(start);
    const iey = cy + inner * Math.sin(start);
    const large = end - start > Math.PI ? 1 : 0;
    return `M${sx.toFixed(2)},${sy.toFixed(2)}
            A${radius},${radius} 0 ${large} 1 ${ex.toFixed(2)},${ey.toFixed(2)}
            L${isx.toFixed(2)},${isy.toFixed(2)}
            A${inner},${inner} 0 ${large} 0 ${iex.toFixed(2)},${iey.toFixed(2)}
            Z`;
  }

  const arcs = $derived.by(() => {
    if (total <= 0) return [];
    let acc = -Math.PI / 2;
    return slices.map((s) => {
      const start = acc;
      const span = (s.value / total) * Math.PI * 2;
      acc += span;
      return { label: s.label, value: s.value, share: s.value / total, d: arc(start, start + span) };
    });
  });
</script>

<div class="donut">
  <svg viewBox="0 0 {size} {size}" role="img" aria-label="Donut chart">
    {#if total <= 0}
      <text x={size / 2} y={size / 2} text-anchor="middle" fill="var(--ivory-3)" font-size="13">
        Aucune donnée.
      </text>
    {:else}
      {#each arcs as a, i}
        <path d={a.d} fill={colors[i % colors.length]} fill-opacity="0.9" />
      {/each}
    {/if}
  </svg>
  <ul class="legend">
    {#each arcs as a, i}
      <li>
        <span class="swatch" style="background: {colors[i % colors.length]}"></span>
        <span class="label">{a.label}</span>
        <span class="pct">{(a.share * 100).toFixed(0)}%</span>
      </li>
    {/each}
  </ul>
</div>

<style>
  .donut {
    display: flex;
    align-items: center;
    gap: var(--sp-5);
  }
  .legend {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
  }
  .legend li {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    font-size: var(--fs-body);
  }
  .swatch {
    display: inline-block;
    width: 12px;
    height: 12px;
    border-radius: 3px;
  }
  .label {
    color: var(--ivory);
  }
  .pct {
    color: var(--ivory-3);
    font-variant-numeric: tabular-nums;
    margin-left: var(--sp-2);
  }
</style>
