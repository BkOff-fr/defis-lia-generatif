import { defineConfig } from 'vite';
import { crx } from '@crxjs/vite-plugin';
import manifest from './manifest.json';

// Configuration Vite extension MV3 — voir ADR-0005.
// Build → dist/, package signé → web-ext si Firefox, CRX si Chrome.

export default defineConfig({
  plugins: [
    // @ts-expect-error CRXJS typings strict
    crx({ manifest })
  ],
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    sourcemap: false,
    minify: 'esbuild',
    // Cible NF-04 : extension ≤ 500 Ko
    chunkSizeWarningLimit: 500,
    rollupOptions: {
      output: {
        chunkFileNames: 'assets/[name]-[hash].js',
        entryFileNames: 'assets/[name]-[hash].js'
      }
    }
  }
});
