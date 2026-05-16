// Sobr.ia extension — synchronisation des logos (mutualisés avec web/ et le design system).
//
// Source de vérité :
//   - sobr-ia-design-system/project/assets/logo-mark.svg     → mark complet (gradients)
//   - sobr-ia-design-system/project/assets/logo.svg          → mark + wordmark (référence)
//   - web/static/favicon.svg                                  → mark simplifié (lisible à 16 px)
//   - web/static/apple-touch-icon.svg                         → mark sur fond ink (tuile app)
//
// L'extension reçoit des copies sous extension/src/assets/icons/. Le manifest MV3 référence
// les SVG directement (supporté Chrome 88+ et Firefox 1+), donc plus de rastérisation
// PNG — cohérent avec la frugalité du projet (CLAUDE.md §8).

import { copyFileSync, mkdirSync, existsSync, statSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const extensionRoot = resolve(__dirname, '..');
const repoRoot = resolve(extensionRoot, '..');

const dstDir = resolve(extensionRoot, 'src/assets/icons');
if (!existsSync(dstDir)) mkdirSync(dstDir, { recursive: true });

const sources = [
  {
    src: resolve(repoRoot, 'web/static/favicon.svg'),
    dst: resolve(dstDir, 'favicon.svg'),
    role: 'mark simplifié (16 px)'
  },
  {
    src: resolve(repoRoot, 'web/static/apple-touch-icon.svg'),
    dst: resolve(dstDir, 'tile.svg'),
    role: 'tuile app icon (sur fond ink)'
  },
  {
    src: resolve(repoRoot, 'sobr-ia-design-system/project/assets/logo-mark.svg'),
    dst: resolve(dstDir, 'mark.svg'),
    role: 'mark complet (gradients lime)'
  },
  {
    src: resolve(repoRoot, 'sobr-ia-design-system/project/assets/logo.svg'),
    dst: resolve(dstDir, 'wordmark.svg'),
    role: 'mark + wordmark horizontal (référence)'
  }
];

let missing = 0;
console.log('[sync-logos] mutualisation des logos depuis sources canoniques :');
for (const { src, dst, role } of sources) {
  if (!existsSync(src)) {
    console.error(
      `  ✗ ${src.replace(repoRoot + '\\', '').replace(repoRoot + '/', '')} introuvable`
    );
    missing++;
    continue;
  }
  copyFileSync(src, dst);
  const size = statSync(dst).size;
  const relSrc = src.replace(repoRoot + '\\', '').replace(repoRoot + '/', '');
  const relDst = dst.replace(extensionRoot + '\\', '').replace(extensionRoot + '/', '');
  console.log(`  ✓ ${relSrc}`);
  console.log(`    → ${relDst} (${size} octets, ${role})`);
}

if (missing > 0) {
  console.error(`\n[sync-logos] ${missing} source(s) introuvable(s) — sync incomplète.`);
  process.exit(1);
}

console.log('\n[sync-logos] terminé.');
