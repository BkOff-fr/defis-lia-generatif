import { expect, test } from '@playwright/test';

/**
 * Contrat « no-mock » de l'écran Simuler (M13).
 *
 * Le simulateur consomme `simulate_scenarios` via Tauri IPC — hors Tauri on
 * doit refuser de servir une simulation factice. Le moteur Monte-Carlo
 * tourne UNIQUEMENT côté Rust (cf. CLAUDE.md §13 + brief C11).
 *
 * Garanties :
 *   1. La coque est rendue (hero h1 + sub).
 *   2. Bannière `tauri_unavailable` avec mention `cargo run`.
 *   3. Les 7 leviers / la baseline NE sont PAS rendus — sinon on
 *      laisserait croire à l'utilisateur qu'il peut bouger un curseur
 *      avec une simulation mockée.
 *   4. Le lien Méthodologie reste accessible.
 */

test('Simuler : refuse de servir une simulation mockée hors contexte Tauri', async ({ page }) => {
  await page.goto('/simuler');

  await expect(page).toHaveTitle(/Simulateur/);
  await expect(
    page.getByRole('heading', { name: /Et si on changeait.*un seul levier/i })
  ).toBeVisible();

  // Bannière tauri_unavailable
  const banner = page.getByRole('alert');
  await expect(banner).toBeVisible();
  await expect(banner).toContainText(/Application non lancée via Tauri/);
  await expect(banner).toContainText(/cargo run -p sobria-app/);

  // Aucun lever / aucune baseline rendus
  await expect(page.getByLabel('Panneau des leviers')).toHaveCount(0);
  await expect(page.locator('input[type="range"]')).toHaveCount(0);
  await expect(page.locator('select.b-select')).toHaveCount(0);

  // Méthodologie OK
  await expect(page.locator('.topbar a[href="/methodo"]')).toBeVisible();
});
