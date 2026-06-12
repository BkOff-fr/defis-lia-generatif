import { readFileSync } from 'node:fs';

import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

// Version unique, lue depuis package.json au build (synchronisée sur le
// workspace Cargo par la release). Injectée en constante compile-time —
// évite d'importer package.json au runtime (interdit par `server.fs.allow`
// de SvelteKit en dev, cf. C37).
const pkg = JSON.parse(readFileSync(new URL('./package.json', import.meta.url), 'utf-8')) as {
  version: string;
};

// Configuration Vite — Sobr.ia
// Optimisée pour Tauri (port fixe, pas de HMR network), build minimal.

export default defineConfig({
  plugins: [sveltekit()],

  define: {
    __APP_VERSION__: JSON.stringify(pkg.version)
  },

  // Tauri attend un port fixe pour le hot reload
  server: {
    port: 5173,
    strictPort: true,
    host: process.env.TAURI_DEV_HOST || false,
    hmr: process.env.TAURI_DEV_HOST
      ? { protocol: 'ws', host: process.env.TAURI_DEV_HOST, port: 1421 }
      : undefined,
    watch: {
      ignored: ['**/src-tauri/**']
    }
  },

  // Variables d'environnement préfixées TAURI_ exposées au frontend
  envPrefix: ['VITE_', 'TAURI_'],

  build: {
    target: process.env.TAURI_PLATFORM === 'windows' ? 'chrome105' : 'safari13',
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
    // Cible frugalité : bundle final ≤ 200 Ko gzip
    chunkSizeWarningLimit: 200
  }
});
