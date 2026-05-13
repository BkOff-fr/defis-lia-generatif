import { expect, test } from '@playwright/test';

/**
 * Contrat « no-mock » de l'écran Workbench.
 *
 * Mêmes garanties que les autres écrans :
 *   1. Hors Tauri, la coque est rendue (hero h1 + sub).
 *   2. Bannière `tauri_unavailable` visible avec mention `cargo run`.
 *   3. La barre de filtres et la table NE sont PAS rendues — sinon on
 *      laisserait croire que l'utilisateur peut explorer un référentiel
 *      mocké.
 *   4. Le lien Méthodologie reste accessible (la coque marche sans IPC).
 */

test('Workbench : refuse de servir un référentiel mocké hors contexte Tauri', async ({ page }) => {
  await page.goto('/workbench');

  await expect(page).toHaveTitle(/Workbench/);
  await expect(
    page.getByRole('heading', { name: /Tous les modèles que.*Sobr\.ia.*sait estimer/i })
  ).toBeVisible();

  // Bannière tauri_unavailable
  const banner = page.getByRole('alert');
  await expect(banner).toBeVisible();
  await expect(banner).toContainText(/Application non lancée via Tauri/);
  await expect(banner).toContainText(/cargo run -p sobria-app/);

  // Filtres et table non rendus
  await expect(page.getByPlaceholder(/Rechercher un modèle/)).toHaveCount(0);
  await expect(page.locator('table.models-table')).toHaveCount(0);

  // Le lien Méthodologie reste accessible depuis la topbar de la page
  await expect(page.locator('.topbar a[href="/methodo"]')).toBeVisible();
});
