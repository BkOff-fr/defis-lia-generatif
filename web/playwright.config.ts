import { defineConfig, devices } from '@playwright/test';

/**
 * Configuration Playwright pour Sobr.ia.
 *
 * Démarre `npm run dev` (Vite SvelteKit) automatiquement avant les tests
 * et attend que le port 5173 réponde. On teste ici uniquement le « contrat
 * no-mock » (l'app servie hors Tauri doit refuser de mocker et signaler
 * `tauri_unavailable`). Les e2e métier nécessitant l'IPC réel (validation
 * Estimer end-to-end, Journal, etc.) tournent dans une suite séparée sur
 * `cargo tauri dev` — chantier C09.5 / CI dédiée.
 */
export default defineConfig({
  testDir: './tests',
  fullyParallel: false,
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
