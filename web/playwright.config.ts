import { defineConfig, devices } from '@playwright/test';

/**
 * Configuration Playwright pour Sobr.ia.
 *
 * Démarre `npm run dev` (Vite SvelteKit) automatiquement avant les tests
 * et attend que le port 5173 réponde. On teste ici le « contrat démo »
 * (chantier C37) : hors Tauri, les commandes IPC couvertes servent des
 * fixtures précalculées par le moteur Rust lui-même (`src/lib/demo`,
 * seed 42) — jamais de données inventées côté TypeScript — et les
 * commandes non couvertes rejettent `tauri_unavailable` avec un message
 * orienté « application de bureau ». L'application de bureau, elle,
 * n'active jamais la démo (gating `!isTauriContext()`). Les e2e métier
 * nécessitant l'IPC réel (Journal/ledger, simulation, exports) tournent
 * dans une suite séparée sur `cargo tauri dev` — chantier C09.5 / CI
 * dédiée.
 */
export default defineConfig({
  testDir: './tests',
  fullyParallel: false,
  // Un seul worker : les tests se partagent un unique dev server Vite, et
  // plusieurs onglets concurrents font occasionnellement timeout au mount
  // (Vite re-optimize sur changement de deps). Sequentiel = stable.
  workers: 1,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  reporter: 'list',
  use: {
    baseURL: 'http://localhost:5173',
    trace: 'on-first-retry',
    headless: true
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] }
    }
  ],
  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:5173',
    reuseExistingServer: !process.env.CI,
    timeout: 60_000,
    stdout: 'ignore',
    stderr: 'pipe'
  }
});
