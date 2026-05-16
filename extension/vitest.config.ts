// Sobr.ia extension — Vitest config (C27.1).
// Pas de tests encore en C27.1 (arrivent en C27.2 — parité moteur JS vs Rust).
// passWithNoTests évite de faire échouer la CI tant que les tests ne sont pas en place.

import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    environment: 'happy-dom',
    globals: true,
    include: ['tests/unit/**/*.spec.ts', 'tests/unit/**/*.test.ts'],
    exclude: ['tests/e2e/**', 'node_modules/**', 'dist/**', 'dist-firefox/**'],
    passWithNoTests: true,
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html', 'lcov'],
      include: ['src/**/*.ts'],
      exclude: ['src/**/*.spec.ts', 'src/**/*.test.ts', 'src/assets/**']
    }
  }
});
