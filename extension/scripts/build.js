// Sobr.ia extension — orchestrateur de build (C27.1).
//
// Lance Vite en 2 passes :
//   1. main : popup + options + service-worker (ES modules, HTML entries)
//   2. iife : 3 content scripts en format IIFE (contrainte MV3)
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
    // Watch mode : seule la passe `main` est vraiment utile en dev (popup/options HMR).
    // Les content scripts en watch nécessitent un rebuild manuel — acceptable v0.6.0.
    console.log(`[build] watch mode (${targetLabel}) — passe main uniquement`);
    await runVite({ SOBRIA_KIND: 'main' }, ['--watch']);
    return;
  }

  const totalPasses = 1 + CONTENT_SCRIPTS.length;
  console.log(
    `[build] (${targetLabel}) 1/${totalPasses} passe main (popup + options + service-worker)`
  );
  await runVite({ SOBRIA_KIND: 'main' });

  // Rollup interdit IIFE multi-entry → 1 passe par content script.
  for (let i = 0; i < CONTENT_SCRIPTS.length; i++) {
    const cs = CONTENT_SCRIPTS[i];
    const step = i + 2;
    console.log(`[build] (${targetLabel}) ${step}/${totalPasses} passe iife (content-${cs})`);
    await runVite({ SOBRIA_KIND: 'iife', SOBRIA_CONTENT: cs });
  }

  console.log(`[build] (${targetLabel}) terminé.`);
}

main().catch((err) => {
  console.error('[build] échec :', err.message);
  process.exit(1);
});
