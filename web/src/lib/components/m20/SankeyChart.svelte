<script lang="ts" module>
  // Sankey énergétique national — composant dataviz transverse (cf. CDC §4.3).
  //
  // SVG manuel, pas de lib externe (frugalité, voir CDC §8). Layout :
  //   - N layers déduits du champ `layer` des nodes (DAG strict).
  //   - Chaque node = rectangle de hauteur proportionnelle à value_twh.
  //   - Chaque link = path cubic Bezier entre layer i et layer i+k.
  //   - Couleurs sémantiques selon l'id du node (renouv / nucléaire / fossile / autre).

  export type SemanticTone =
    | 'renewable'
    | 'nuclear'
    | 'fossil'
    | 'transfer'
    | 'consumer'
    | 'neutral';

  const RENEWABLE_KEYS = [
    'hydro',
    'eolien',
    'solaire',
    'bioenergies',
    'wind',
    'solar',
    'hydraulique'
  ];
  const NUCLEAR_KEYS = ['nucle', 'nuclear'];
  const FOSSIL_KEYS = ['gaz', 'charbon', 'fioul', 'coal', 'gas', 'oil'];
  const TRANSFER_KEYS = ['import', 'export', 'pompage', 'echanges', 'echange'];

  export function inferTone(idOrLabel: string): SemanticTone {
    const k = idOrLabel.toLowerCase();
    if (NUCLEAR_KEYS.some((kw) => k.includes(kw))) return 'nuclear';
    if (FOSSIL_KEYS.some((kw) => k.includes(kw))) return 'fossil';
    if (RENEWABLE_KEYS.some((kw) => k.includes(kw))) return 'renewable';
    if (TRANSFER_KEYS.some((kw) => k.includes(kw))) return 'transfer';
    return 'neutral';
  }
</script>

<script lang="ts">
  import { Activity, ExternalLink } from '@lucide/svelte';
  import type { SankeyDataDto, SankeyNodeDto } from '$lib/api';

  type Props = {
    data: SankeyDataDto;
    fetchedAt?: string | null;
  };
  const { data, fetchedAt = null }: Props = $props();

  // ─── Layout SVG ─────────────────────────────────────────────────────────
  //
  // Largeur 800, hauteur 380, padding latéral 20px pour les labels.

  const SVG_W = 800;
  const SVG_H = 380;
  const PAD_X = 24;
  const PAD_Y = 20;
  const NODE_W = 14;
  const NODE_GAP_Y = 12;

  type LayoutNode = SankeyNodeDto & {
    x: number;
    y: number;
    height: number;
    tone: SemanticTone;
    outOffset: number; // running offset for outgoing links from top of node
    inOffset: number; // running offset for incoming links from top of node
  };

  type LayoutLink = {
    source: LayoutNode;
    target: LayoutNode;
    value_twh: number;
    thickness: number;
    path: string;
  };

  const layout = $derived.by<{ nodes: LayoutNode[]; links: LayoutLink[] }>(() => {
    if (data.nodes.length === 0) return { nodes: [], links: [] };

    // 1) Group nodes by layer
    const layerMap = new Map<number, SankeyNodeDto[]>();
    for (const n of data.nodes) {
      const arr = layerMap.get(n.layer) ?? [];
      arr.push(n);
      layerMap.set(n.layer, arr);
    }
    const sortedLayers = [...layerMap.keys()].sort((a, b) => a - b);
    const numLayers = sortedLayers.length;

    // 2) Échelle de valeur — par layer le total varie, on prend le max comme référence
    //    pour éviter qu'une layer dominante n'écrase les autres.
    const layerTotals = sortedLayers.map((l) =>
      (layerMap.get(l) ?? []).reduce((s, n) => s + n.value_twh, 0)
    );
    const maxLayerTotal = Math.max(...layerTotals);
    if (maxLayerTotal <= 0) return { nodes: [], links: [] };

    const availableH = SVG_H - 2 * PAD_Y;
    const nodesPerLayer = sortedLayers.map((l) => (layerMap.get(l) ?? []).length);
    const maxNodesInLayer = Math.max(...nodesPerLayer);
    const totalGapH = (maxNodesInLayer - 1) * NODE_GAP_Y;
    const scaleH = (availableH - totalGapH) / maxLayerTotal;

    // 3) X positions
    const usableW = SVG_W - 2 * PAD_X - NODE_W;
    const layerX: number[] = [];
    if (numLayers === 1) layerX.push(PAD_X + usableW / 2);
    else for (let i = 0; i < numLayers; i++) layerX.push(PAD_X + (i / (numLayers - 1)) * usableW);

    // 4) Position des nodes dans chaque layer
    const layoutNodes: LayoutNode[] = [];
    const nodeById = new Map<string, LayoutNode>();
    for (let li = 0; li < numLayers; li++) {
      const layerNodes = layerMap.get(sortedLayers[li] ?? 0) ?? [];
      // Sort by value desc pour empiler les gros en haut
      const sortedNodes = [...layerNodes].sort((a, b) => b.value_twh - a.value_twh);
      const layerSum = sortedNodes.reduce((s, n) => s + n.value_twh, 0);
      const gapTotal = (sortedNodes.length - 1) * NODE_GAP_Y;
      const layerH = layerSum * scaleH + gapTotal;
      let y = PAD_Y + (availableH - layerH) / 2;
      for (const n of sortedNodes) {
        const h = Math.max(1, n.value_twh * scaleH);
        const ln: LayoutNode = {
          ...n,
          x: layerX[li] ?? PAD_X,
          y,
          height: h,
          tone: inferTone(`${n.id} ${n.label}`),
          outOffset: 0,
          inOffset: 0
        };
        layoutNodes.push(ln);
        nodeById.set(n.id, ln);
        y += h + NODE_GAP_Y;
      }
    }

    // 5) Build links + paths
    const layoutLinks: LayoutLink[] = [];
    // Sort links by source.y then target.y for cleaner stacking.
    const sortedLinks = [...data.links].sort((a, b) => {
      const sa = nodeById.get(a.source);
      const sb = nodeById.get(b.source);
      const ta = nodeById.get(a.target);
      const tb = nodeById.get(b.target);
      if (!sa || !sb || !ta || !tb) return 0;
      return sa.y - sb.y || ta.y - tb.y;
    });

    for (const lk of sortedLinks) {
      const s = nodeById.get(lk.source);
      const t = nodeById.get(lk.target);
      if (!s || !t) continue;
      const thickness = Math.max(1, lk.value_twh * scaleH);
      const sx = s.x + NODE_W;
      const sy = s.y + s.outOffset + thickness / 2;
      const tx = t.x;
      const ty = t.y + t.inOffset + thickness / 2;
      // Cubic Bezier path avec control points horizontaux mid-point
      const cx = (sx + tx) / 2;
      const path = `M ${sx} ${sy} C ${cx} ${sy}, ${cx} ${ty}, ${tx} ${ty}`;
      layoutLinks.push({
        source: s,
        target: t,
        value_twh: lk.value_twh,
        thickness,
        path
      });
      s.outOffset += thickness;
      t.inOffset += thickness;
    }

    return { nodes: layoutNodes, links: layoutLinks };
  });

  // Couleurs par sémantique
  const TONE_COLORS: Record<SemanticTone, { stroke: string; fill: string }> = {
    nuclear: { stroke: '#f5b769', fill: 'rgba(245, 183, 105, 0.7)' }, // amber
    renewable: { stroke: '#c5f04a', fill: 'rgba(197, 240, 74, 0.7)' }, // lime
    fossil: { stroke: '#a0998c', fill: 'rgba(160, 153, 140, 0.5)' }, // gray
    transfer: { stroke: '#7eb6ff', fill: 'rgba(126, 182, 255, 0.55)' }, // blue
    consumer: { stroke: '#f0ece3', fill: 'rgba(240, 236, 227, 0.45)' }, // ivory
    neutral: { stroke: '#b8b4ac', fill: 'rgba(184, 180, 172, 0.45)' }
  };

  function fmtTwh(value: number, digits = 1): string {
    if (!Number.isFinite(value)) return '—';
    return new Intl.NumberFormat('fr-FR', { maximumFractionDigits: digits }).format(value);
  }
  function fmtPct(value: number, total: number): string {
    if (total <= 0) return '—';
    return `${((value / total) * 100).toFixed(1)}%`;
  }

  let hoveredLink = $state<LayoutLink | null>(null);

  // Fallback table ARIA (a11y).
  const sortedNodesByValue = $derived([...data.nodes].sort((a, b) => b.value_twh - a.value_twh));
</script>

<section class="sankey-card">
  <header class="sh">
    <div class="sh-l">
      <span class="ico"><Activity size={14} strokeWidth={1.8} /></span>
      <div>
        <div class="eyebrow">Sankey énergétique national</div>
        <h3>Production → usages · {data.year}</h3>
      </div>
    </div>
    <div class="sh-r">
      <span class="total mono">
        {fmtTwh(data.total_production_twh, 1)}<span class="u">TWh</span>
      </span>
    </div>
  </header>

  <div class="svg-wrap">
    <svg
      viewBox="0 0 {SVG_W} {SVG_H}"
      preserveAspectRatio="xMidYMid meet"
      role="img"
      aria-label="Sankey énergétique national"
    >
      <!-- Links (sous les nodes) -->
      {#each layout.links as lk (lk.source.id + '→' + lk.target.id)}
        <path
          d={lk.path}
          stroke={TONE_COLORS[lk.source.tone].fill}
          stroke-width={lk.thickness}
          fill="none"
          opacity={hoveredLink && hoveredLink !== lk ? 0.18 : 0.5}
          role="img"
          aria-label="{lk.source.label} vers {lk.target.label} : {fmtTwh(lk.value_twh, 2)} TWh"
          onmouseenter={() => (hoveredLink = lk)}
          onmouseleave={() => (hoveredLink = null)}
          style="transition: opacity 200ms"
        />
      {/each}

      <!-- Nodes -->
      {#each layout.nodes as n (n.id)}
        <g>
          <rect
            x={n.x}
            y={n.y}
            width={NODE_W}
            height={n.height}
            fill={TONE_COLORS[n.tone].stroke}
            opacity="0.9"
            rx="2"
          />
          <text
            x={n.x + NODE_W + 6}
            y={n.y + n.height / 2 + 3}
            fill="#f0ece3"
            font-family="Geist, system-ui, sans-serif"
            font-size="10"
            font-weight="500"
          >
            {n.label}
          </text>
          <text
            x={n.x + NODE_W + 6}
            y={n.y + n.height / 2 + 14}
            fill="#72706a"
            font-family="JetBrains Mono, monospace"
            font-size="8"
          >
            {fmtTwh(n.value_twh, 1)} TWh
          </text>
        </g>
      {/each}
    </svg>

    {#if hoveredLink}
      <div class="tooltip">
        <strong>{hoveredLink.source.label}</strong>
        <span class="arrow">→</span>
        <strong>{hoveredLink.target.label}</strong>
        <span class="value mono">
          {fmtTwh(hoveredLink.value_twh, 2)} TWh ·
          {fmtPct(hoveredLink.value_twh, data.total_production_twh)}
        </span>
      </div>
    {/if}
  </div>

  <!-- Legend -->
  <div class="legend">
    <span class="lg-item">
      <span class="lg-swatch" style="background: {TONE_COLORS.nuclear.fill}"></span> Nucléaire
    </span>
    <span class="lg-item">
      <span class="lg-swatch" style="background: {TONE_COLORS.renewable.fill}"></span> Renouvelables
    </span>
    <span class="lg-item">
      <span class="lg-swatch" style="background: {TONE_COLORS.fossil.fill}"></span> Fossiles
    </span>
    <span class="lg-item">
      <span class="lg-swatch" style="background: {TONE_COLORS.transfer.fill}"></span> Import / export
    </span>
  </div>

  <!-- Fallback table ARIA (a11y) — visible only via screen reader. -->
  <table class="sr-only" aria-label="Décomposition Sankey (table accessibilité)">
    <caption>Production électrique nationale FR — {data.year}</caption>
    <thead>
      <tr>
        <th>Source</th>
        <th>Layer</th>
        <th>Valeur TWh</th>
        <th>Part %</th>
      </tr>
    </thead>
    <tbody>
      {#each sortedNodesByValue as n (n.id)}
        <tr>
          <td>{n.label}</td>
          <td>{n.layer}</td>
          <td>{fmtTwh(n.value_twh, 2)}</td>
          <td>{fmtPct(n.value_twh, data.total_production_twh)}</td>
        </tr>
      {/each}
    </tbody>
  </table>

  <footer class="sfoot mono">
    Source :
    <a href={data.source_url} target="_blank" rel="noopener noreferrer">
      RTE eco2mix {data.year}
      <ExternalLink size={9} strokeWidth={2} />
    </a>
    <span class="sha">SHA-256 {data.source_sha256.slice(0, 16)}…</span>
    {#if fetchedAt}
      <span class="fetched">fetched {fetchedAt}</span>
    {/if}
  </footer>
</section>

<style>
  .sankey-card {
    padding: 22px 24px 18px;
    background: rgba(255, 255, 255, 0.015);
    border: 1px solid var(--border);
    border-radius: var(--radius-xl);
  }

  .sh {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 14px;
    margin-bottom: 14px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border);
  }
  .sh-l {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .sh-l .ico {
    display: inline-grid;
    place-items: center;
    width: 28px;
    height: 28px;
    background: var(--lime-soft);
    border: 1px solid rgba(197, 240, 74, 0.3);
    border-radius: 8px;
    color: var(--lime);
  }
  .eyebrow {
    font: 500 10px/1 var(--font-ui);
    text-transform: uppercase;
    letter-spacing: 0.14em;
    color: var(--ivory-3);
    margin-bottom: 4px;
  }
  h3 {
    font: 400 22px/1.15 var(--font-display);
    font-style: italic;
    color: var(--ivory);
    letter-spacing: -0.01em;
    margin: 0;
  }
  .total {
    font: 400 30px/1 var(--font-display);
    font-style: italic;
    color: var(--lime);
    letter-spacing: -0.01em;
  }
  .total .u {
    font: 400 12px/1 var(--font-ui);
    font-style: normal;
    color: var(--ivory-3);
    margin-left: 5px;
  }

  .svg-wrap {
    position: relative;
    margin-bottom: 12px;
  }
  .svg-wrap svg {
    width: 100%;
    height: auto;
    display: block;
  }
  .tooltip {
    position: absolute;
    bottom: 8px;
    left: 50%;
    transform: translateX(-50%);
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 7px 12px;
    background: rgba(10, 13, 11, 0.92);
    border: 1px solid rgba(197, 240, 74, 0.3);
    border-radius: 999px;
    font: 400 11px/1 var(--font-ui);
    color: var(--ivory);
    pointer-events: none;
    white-space: nowrap;
  }
  .tooltip strong {
    color: var(--ivory);
    font-weight: 600;
  }
  .tooltip .arrow {
    color: var(--ivory-3);
  }
  .tooltip .value {
    color: var(--lime);
    font-weight: 600;
  }

  .legend {
    display: flex;
    flex-wrap: wrap;
    gap: 14px;
    padding: 10px 0;
    border-top: 1px solid var(--border);
    border-bottom: 1px solid var(--border);
    margin-bottom: 12px;
  }
  .lg-item {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font: 400 11px/1 var(--font-ui);
    color: var(--ivory-2);
  }
  .lg-swatch {
    display: inline-block;
    width: 14px;
    height: 8px;
    border-radius: 2px;
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }

  .sfoot {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    align-items: center;
    font: 400 10px/1.4 var(--font-mono);
    color: var(--ivory-3);
  }
  .sfoot a {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    color: var(--lime);
    text-decoration: none;
    border-bottom: 1px dashed rgba(197, 240, 74, 0.3);
  }
  .sfoot a:hover {
    color: var(--ivory);
  }
  .sha {
    color: var(--ivory-4);
  }
  .fetched {
    color: var(--ivory-4);
  }

  .mono {
    font-family: var(--font-mono);
  }
</style>
