import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],
  build: {
    // Sourcemaps off : on embarque le bundle dans le binaire Rust, on
    // veut le bundle le plus compact possible.
    sourcemap: false,
    // Inline tout ce qui est <8 KB (réduit la fragmentation du embed).
    assetsInlineLimit: 8192
  },
  server: {
    port: 5174,
    proxy: {
      // En dev, proxy vers le binaire local sur :8443 (HTTPS auto-signé).
      // L'utilisateur lance `cargo run -p sobria-team-aggregator -- serve`
      // en parallèle de `npm run dev`.
      '/api': {
        target: 'https://localhost:8443',
        secure: false,
        changeOrigin: true
      }
    }
  }
});
