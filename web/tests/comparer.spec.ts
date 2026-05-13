import { expect, test } from '@playwright/test';

/**
 * Contrat « no-mock » de l'écran Comparer.
 *
 * Le comparateur fan-out N estimations en parallèle via `estimatePrompt` —
 * hors Tauri, on doit refuser de servir une matrice mockée.
 *
 * Garanties :
 *   1. La coque est rendue (hero + sub).
 *   2. Bannière `tauri_unavailable` avec mention `cargo run`.
 *   3. Sélecteur de modèles et bouton « Comparer » ne sont PAS rendus.
 *   4. Le lien Méthodologie reste accessible.
 */

test('Comparer : refuse de servir une matrice mockée hors contexte Tauri', async ({ page }) => {
  await page.goto('/comparer');

  await expect(page).toHaveTitle(/Comparer/);
  await expect(page.getByRole('heading', { name: /Le bon LLM.*pour le bon usage/i })).toBeVisible();

  const banner = page.getByRole('alert');
  await expect(banner).toBeVisible();
  await expect(banner).toContainText(/Application non lancée via Tauri/);
  await expect(banner).toContainText(/cargo run -p sobria-app/);

  // Sélecteur de modèles et bouton « Comparer » absents
  await expect(page.getByRole('button', { name: /^Comparer$/ })).toHaveCount(0);
  await expect(page.locator('.model-chips')).toHaveCount(0);

  // Lien Méthodologie OK
  await expect(page.locator('.topbar a[href="/methodo"]')).toBeVisible();
});
