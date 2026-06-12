import { expect, test } from '@playwright/test';

/**
 * Contrat « démo » de l'écran Référentiel modèles (M9 / C18 / C36).
 *
 * M9 lit le registre embarqué via `list_models` et `get_model_detail` —
 * deux commandes couvertes par la démo web : le catalogue affiché est le
 * référentiel réel sérialisé en fixtures (`src/lib/demo/fixtures`).
 *
 * Garanties :
 *   1. La coque est rendue (hero h1 + sub).
 *   2. Bannière « Mode démo » visible, aucune bannière d'erreur, pas
 *      d'empty shell « Référentiel indisponible ».
 *   3. La grille rend les cards modèles (catalogue complet).
 *   4. Cliquer une card ouvre le drawer détail (role="dialog") —
 *      `get_model_detail` est servi par la démo.
 *   5. Le lien Méthodologie reste accessible.
 */

test('Référentiel modèles : sert le catalogue des fixtures hors contexte Tauri', async ({
  page
}) => {
  await page.goto('/modeles');

  await expect(page).toHaveTitle(/Bibliothèque de modèles/);
  await expect(
    page.getByRole('heading', { name: /chiffres derrière.*chaque modèle/i })
  ).toBeVisible();

  // Mode démo : bannière layout, pas d'erreur, pas d'empty shell.
  await expect(page.locator('aside.demo-banner')).toBeVisible();
  const grid = page.getByRole('grid');
  await expect(grid).toBeVisible();
  await expect(page.getByRole('alert')).toHaveCount(0);
  await expect(page.getByRole('heading', { name: /Référentiel indisponible/i })).toHaveCount(0);

  // Le catalogue est peuplé (les fixtures embarquent tout le référentiel).
  const rows = grid.getByRole('row');
  expect(await rows.count()).toBeGreaterThanOrEqual(10);

  // Drawer détail : get_model_detail est couvert par la démo.
  await rows.first().click();
  await expect(page.getByRole('dialog')).toBeVisible();

  // Méthodologie OK
  await expect(page.locator('.topbar a[href="/methodo"]')).toBeVisible();
});

test('C42 — ancienne URL /m9 redirige vers /modeles (query préservée)', async ({ page }) => {
  await page.goto('/m9?focus=claude-haiku-4-5');
  await page.waitForURL('**/modeles**', { timeout: 8000 });
  expect(page.url()).toContain('/modeles');
  expect(page.url()).toContain('focus=claude-haiku-4-5');
});
