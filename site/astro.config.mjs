// @ts-check
import { defineConfig } from 'astro/config';
import svelte from '@astrojs/svelte';
import mdx from '@astrojs/mdx';
import tailwindcss from '@tailwindcss/vite';

// https://astro.build/config
export default defineConfig({
  site: 'https://sobria.brilliantstudio.co',
  output: 'static',
  trailingSlash: 'ignore',
  prefetch: { prefetchAll: false, defaultStrategy: 'hover' },
  integrations: [svelte(), mdx()],
  vite: {
    plugins: [tailwindcss()],
  },
  server: { port: 4321, host: false },
  build: { inlineStylesheets: 'auto' },
});
