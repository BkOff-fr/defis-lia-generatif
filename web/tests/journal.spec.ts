import { expect, test } from '@playwright/test';

/**
 * Contrat « no-mock » de l'écran Journal d'audit.
 *
 * Mêmes garanties que `estimate.spec.ts` :
 *   1. Hors Tauri, la coque éditoriale est rendue (titre + h1 « Toutes vos
 *      estimations »).
 *   2. La bannière `tauri_unavailable` s'affiche avec un message d'action.
 *   3. La toolbar (Vérifier la chaîne / Exporter NDJSON) n'est PAS rendue
 *      — son absence est la garantie qu'on ne falsifie pas un ledger côté
 *      front.
 *   4. La bannière RGPD (mention « ledger jamais envoyé sur Internet ») est
 *      visible — c'est un engagement permanent, pas un nice-to-have.
 *
 * Cf. CLAUDE.md §13 (pas de mock côté front) + DoD étape D.
 */

test('Journal : refuse de servir un ledger mocké hors contexte Tauri', async ({ page }) => {
  await page.goto('/journal');

  // 1. Coque rendue
  await expect(page).toHaveTitle(/Journal d'audit/);
  await expect(
    page.getByRole('heading', { name: /Toutes vos estimations.*vérifiables/i })
  ).toBeVisible();

  // 2. Bannière tauri_unavailable
  const banner = page.getByRole('alert');
  await expect(banner).toBeVisible();
  await expect(banner).toContainText(/Application non lancée via Tauri/);
  await expect(banner).toContainText(/cargo run -p sobria-app/);

  // 3. Toolbar non rendue (verrouille le contrat no-mock)
  await expect(page.getByRole('button', { name: /Vérifier la chaîne/ })).toHaveCount(0);
  await expect(page.getByRole('button', { name: /Exporter NDJSON/ })).toHaveCount(0);

  // 4. Bandeau RGPD permanent
  await expect(page.getByText(/Le ledger n'est jamais envoyé sur Internet/)).toBeVisible();
});

/**
 * Lien depuis l'écran Estimer (Signature → `/journal?focus=N`) :
 * on charge la cible directement avec un focus, on vérifie que la page
 * répond et conserve l'état attendu (encore en `tauri_unavailable`, donc
 * pas de drawer ouvert — mais l'URL est bien acceptée sans 404 SvelteKit).
 */
test('Journal : accepte ?focus=N depuis le lien Signature', async ({ page }) => {
  const response = await page.goto('/journal?focus=42');
  expect(response?.status()).toBe(200);
  await expect(
    page.getByRole('heading', { name: /Toutes vos estimations.*vérifiables/i })
  ).toBeVisible();
});
