import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

// Configuration Vite — Sobr.ia
// Optimisée pour Tauri (port fixe, pas de HMR network), build minimal.

export default defineConfig({
  plugins: [sveltekit()],

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
