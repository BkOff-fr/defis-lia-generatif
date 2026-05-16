<script lang="ts">
  // SVG line chart minimaliste — pas de d3/Plot, pour rester frugal sur
  // la taille embedded. Suffit pour la viz mensuelle/quotidienne.

  interface Point {
    label: string;
    value: number;
  }

  interface Props {
    points: Point[];
    color?: string;
    height?: number;
    valueFormat?: (v: number) => string;
  }

  let { points, color = 'var(--lime)', height = 220, valueFormat }: Props = $props();

  const width = 720;
  const padding = { top: 16, right: 16, bottom: 32, left: 56 };

  const innerW = $derived(width - padding.left - padding.right);
  const innerH = $derived(height - padding.top - padding.bottom);

  const max = $derived(points.length === 0 ? 1 : Math.max(...points.map((p) => p.value), 0.0001));

  function x(i: number): number {
    if (points.length <= 1) return padding.left + innerW / 2;
    return padding.left + (i / (points.length - 1)) * innerW;
  }

  function y(v: number): number {
    return padding.top + innerH - (v / max) * innerH;
  }

  const path = $derived(
    points.map((p, i) => `${i === 0 ? 'M' : 'L'}${x(i).toFixed(1)},${y(p.value).toFixed(1)}`).join(' ')
  );

  const area = $derived(
    points.length === 0
      ? ''
      : `${path} L${x(points.length - 1).toFixed(1)},${(padding.top + innerH).toFixed(1)} L${x(0).toFixed(1)},${(padding.top + innerH).toFixed(1)} Z`
  );

  const yTicks = $derived([0, 0.25, 0.5, 0.75, 1].map((r) => max * r));

  function fmt(v: number): string {
    return valueFormat ? valueFormat(v) : v.toFixed(1);
  }
</script>

<svg viewBox="0 0 {width} {height}" preserveAspectRatio="xMidYMid meet" role="img" aria-label="Évolution temporelle">
  <!-- Grille horizontale -->
  {#each yTicks as t}
    <line
      x1={padding.left}
      x2={padding.left + innerW}
      y1={y(t)}
      y2={y(t)}
      stroke="var(--plot-grid)"
      stroke-width="1"
    />
    <text x={padding.left - 8} y={y(t)} dy="0.32em" text-anchor="end" fill="var(--plot-axis)" font-size="10">
      {fmt(t)}
    </text>
  {/each}

  {#if points.length > 0}
    <path d={area} fill={color} fill-opacity="0.12" />
    <path d={path} fill="none" stroke={color} stroke-width="2" stroke-linejoin="round" />
    {#each points as p, i}
      <circle cx={x(i)} cy={y(p.value)} r="3" fill={color} />
    {/each}
  {:else}
    <text
      x={padding.left + innerW / 2}
      y={padding.top + innerH / 2}
      text-anchor="middle"
      fill="var(--ivory-3)"
      font-size="13"
    >
      Aucune donnée sur la fenêtre.
    </text>
  {/if}

  <!-- Axe X : labels espacés selon le nombre de points -->
  {#each points as p, i}
    {#if points.length <= 12 || i % Math.ceil(points.length / 8) === 0}
      <text
        x={x(i)}
        y={padding.top + innerH + 16}
        text-anchor="middle"
        fill="var(--plot-axis)"
        font-size="10"
      >
        {p.label}
      </text>
    {/if}
  {/each}
</svg>
