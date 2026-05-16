// SvelteKit + adapter-static. Le binaire `sobria-team-aggregator` embarque
// `build/` via `rust-embed` et le sert à la racine. Fallback `index.html`
// pour permettre le routage SPA client-side. Pas de prerender (toutes les
// pages dépendent de l'auth, qui est runtime).

import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      pages: 'build',
      assets: 'build',
      fallback: 'index.html',
      precompress: false,
      strict: true
    }),
    alias: {
      $lib: './src/lib',
      $components: './src/lib/components',
      $charts: './src/lib/charts'
    }
  },
  compilerOptions: {
    runes: true
  }
};

export default config;
