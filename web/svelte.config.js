// Configuration SvelteKit — Sobr.ia
// Adapter static : on génère du HTML/JS pur que Tauri embarque (ADR-0001 + ADR-0002).

import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      pages: 'build',
      assets: 'build',
      fallback: 'index.html', // SPA mode pour Tauri
      precompress: false,
      strict: true
    }),
    alias: {
      '$lib': './src/lib',
      '$components': './src/lib/components',
      '$stores': './src/lib/stores',
      '$ipc': './src/lib/ipc'
    },
    typescript: {
      config: (config) => ({ ...config, compilerOptions: { ...config.compilerOptions, strict: true } })
    }
  },
  compilerOptions: {
    runes: true // Svelte 5 runes (stores typés modernes)
  }
};

export default config;
