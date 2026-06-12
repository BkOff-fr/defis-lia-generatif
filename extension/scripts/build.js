// Sobr.ia extension — orchestrateur de build (C27.1).
//
// Lance Vite en 5 passes :
//   1. main : popup + options (ES modules, HTML entries)
//   2. sw   : service-worker IIFE monolithique (contrainte MV3 Chrome)
//   3–5. iife : 3 content scripts (1 passe chacun)
//
// Usage :
//   node scripts/build.js              # build Chrome → dist/
//   node scripts/build.js --firefox    # build Firefox → dist-firefox/
//   node scripts/build.js --watch      # watch mode (passe main uniquement)

import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const __dirname = dirname(fileURLToPath(import.meta.url));
const extensionRoot = resolve(__dirname, '..');

const args = process.argv.slice(2);
const isFirefox = args.includes('--firefox');
const isWatch = args.includes('--watch');

const baseEnv = isFirefox ? { SOBRIA_TARGET: 'firefox' } : { SOBRIA_TARGET: 'chrome' };
const targetLabel = isFirefox ? 'Firefox' : 'Chrome';

function runVite(extraEnv, viteArgs = []) {
  return new Promise((resolvePromise, rejectPromise) => {
    const npx = process.platform === 'win32' ? 'npx.cmd' : 'npx';
    const child = spawn(npx, ['vite', 'build', ...viteArgs], {
      cwd: extensionRoot,
      stdio: 'inherit',
      env: { ...process.env, ...baseEnv, ...extraEnv },
      shell: process.platform === 'win32'
    });
    child.on('error', rejectPromise);
    child.on('exit', (code) => {
      if (code === 0) resolvePromise();
      else rejectPromise(new Error(`vite build exited with code ${code}`));
    });
  });
}

const CONTENT_SCRIPTS = ['chatgpt', 'claude', 'le-chat'];

async function main() {
  if (isWatch) {
    console.log(`[build] watch mode (${targetLabel}) — build complet initial puis watch popup/options`);
    await runVite({ SOBRIA_KIND: 'main' });
    await runVite({ SOBRIA_KIND: 'sw' });
    for (const cs of CONTENT_SCRIPTS) {
      await runVite({ SOBRIA_KIND: 'iife', SOBRIA_CONTENT: cs });
    }
    await runVite({ SOBRIA_KIND: 'main', SOBRIA_WATCH: '1' }, ['--watch']);
    return;
  }

  const totalPasses = 2 + CONTENT_SCRIPTS.length;
  console.log(`[build] (${targetLabel}) 1/${totalPasses} passe main (popup + options)`);
  await runVite({ SOBRIA_KIND: 'main' });

  console.log(`[build] (${targetLabel}) 2/${totalPasses} passe sw (service-worker IIFE)`);
  await runVite({ SOBRIA_KIND: 'sw' });

  // Rollup interdit IIFE multi-entry → 1 passe par content script.
  for (let i = 0; i < CONTENT_SCRIPTS.length; i++) {
    const cs = CONTENT_SCRIPTS[i];
    const step = i + 3;
    console.log(`[build] (${targetLabel}) ${step}/${totalPasses} passe iife (content-${cs})`);
    await runVite({ SOBRIA_KIND: 'iife', SOBRIA_CONTENT: cs });
  }

  console.log(`[build] (${targetLabel}) terminé.`);
}

main().catch((err) => {
  console.error('[build] échec :', err.message);
  process.exit(1);
});
