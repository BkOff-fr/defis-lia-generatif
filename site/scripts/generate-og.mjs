#!/usr/bin/env node
/**
 * Génère public/og-image.png (1200×630) depuis le SVG inline ci-dessous.
 * Lancé à la main : `node scripts/generate-og.mjs` (ou en prebuild si besoin).
 */
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import sharp from 'sharp';

const __dirname = dirname(fileURLToPath(import.meta.url));
const OUT = resolve(__dirname, '..', 'public', 'og-image.png');

const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1200 630">
  <defs>
    <radialGradient id="bg" cx="20%" cy="30%" r="80%">
      <stop offset="0%" stop-color="#131815" />
      <stop offset="60%" stop-color="#0e1310" />
      <stop offset="100%" stop-color="#0a0d0b" />
    </radialGradient>
    <radialGradient id="glow" cx="50%" cy="50%" r="50%">
      <stop offset="0%" stop-color="#c5f04a" stop-opacity="0.25" />
      <stop offset="100%" stop-color="#c5f04a" stop-opacity="0" />
    </radialGradient>
  </defs>

  <rect width="1200" height="630" fill="url(#bg)" />

  <!-- Globe glow (right side) -->
  <circle cx="950" cy="315" r="280" fill="url(#glow)" />

  <!-- Stylized globe : ring + 28 dots (Europe pattern) -->
  <g transform="translate(950, 315)">
    <circle cx="0" cy="0" r="180" fill="#0e1310" stroke="#c5f04a" stroke-opacity="0.35" stroke-width="1.5"/>
    ${(() => {
      const dots = [];
      // Pattern of dots ~mimicking the Europe datacenter spread on the globe fallback
      const positions = [
        [-30, -90],
        [10, -85],
        [50, -80],
        [-50, -30],
        [-10, -25],
        [30, -20],
        [70, -25],
        [-70, 20],
        [-25, 25],
        [15, 30],
        [55, 25],
        [95, 20],
        [-50, 75],
        [-5, 80],
        [40, 75],
        [80, 80],
        [-20, 120],
        [25, 125],
        [65, 120],
        [-80, -50],
        [-90, 40],
        [105, -50],
        [105, 40],
        [-90, 90],
        [105, 90],
        [0, -130],
        [55, -125],
        [80, 135],
      ];
      for (const [x, y] of positions) {
        dots.push(`<circle cx="${x}" cy="${y}" r="3.5" fill="#c5f04a" opacity="0.85"/>`);
      }
      return dots.join('');
    })()}
  </g>

  <!-- Left text block -->
  <g transform="translate(80, 200)">
    <text x="0" y="0" font-family="Inter, -apple-system, system-ui, sans-serif" font-size="22" font-weight="500" letter-spacing="6" fill="#c5f04a">
      SOBR.IA
    </text>
    <text x="0" y="80" font-family="Cormorant Garamond, Georgia, serif" font-style="italic" font-size="78" fill="#f0ece3">
      L'empreinte de vos
    </text>
    <text x="0" y="170" font-family="Cormorant Garamond, Georgia, serif" font-style="italic" font-size="78" fill="#f0ece3">
      prompts IA, <tspan fill="#c5f04a">mesurée</tspan>.
    </text>
    <text x="0" y="240" font-family="Inter, -apple-system, system-ui, sans-serif" font-size="22" fill="#b8b4ac">
      En local. Sans inscription. Sans cloud Sobr.ia.
    </text>
    <text x="0" y="310" font-family="Inter, -apple-system, system-ui, sans-serif" font-size="16" font-weight="500" letter-spacing="1.5" fill="#72706a">
      AFNOR SPEC 2314 · EcoLogits · Open source · Candidat data.gouv.fr
    </text>
  </g>
</svg>`;

await sharp(Buffer.from(svg)).png({ compressionLevel: 9 }).toFile(OUT);

console.log(`✓ Generated ${OUT}`);
