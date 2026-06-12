import { expect, test } from '@playwright/test';

/**
 * Contrat « démo » de l'écran Journal d'audit (C36).
 *
 * Le ledger d'audit (chaîné SHA-256) est strictement local : aucune
 * commande `*_audit_*` n'est couverte par la démo web. Hors Tauri :
 *   1. La coque éditoriale est rendue (titre + h1 « Toutes vos
 *      estimations ») et la bannière « Mode démo » du layout est visible.
 *   2. Une bannière d'avertissement « Application de bureau requise »
 *      s'affiche — le message oriente vers l'application de bureau (plus
 *      aucune mention de commande développeur type `cargo run`).
 *   3. Aucune entrée de ledger n'est servie (tableau vide) — on ne
 *      falsifie jamais un ledger côté front.
 *   4. La bannière RGPD (« ledger jamais envoyé sur Internet ») reste
 *      visible — engagement permanent.
 */

test('Journal : ledger réservé à l’application de bureau, pas de données démo', async ({
  page
}) => {
  await page.goto('/journal');

  // 1. Coque rendue + mode démo signalé par le layout.
  await expect(page).toHaveTitle(/Journal d'audit/);
  await expect(
    page.getByRole('heading', { name: /Toutes vos estimations.*vérifiables/i })
  ).toBeVisible();
  await expect(page.locator('aside.demo-banner')).toBeVisible();

  // 2. Bannière « Application de bureau requise » (nouveau libellé C36).
  const banner = page.getByRole('alert');
  await expect(banner).toBeVisible();
  await expect(banner).toContainText(/Application de bureau requise/);
  await expect(banner).toContainText(/application de bureau/);
  await expect(banner).not.toContainText(/cargo run/);

  // 3. Aucune entrée mockée : le tableau affiche l'état vide.
  await expect(page.getByText(/Aucune entrée pour cette page/)).toBeVisible();

  // 4. Bandeau RGPD permanent
  await expect(page.getByText(/Le ledger n'est jamais envoyé sur Internet/)).toBeVisible();
});

/**
 * Lien depuis l'écran Estimer (Signature → `/journal?focus=N`) :
 * on charge la cible directement avec un focus, on vérifie que la page
 * répond et conserve l'état attendu (ledger indisponible hors app de
 * bureau, donc pas de drawer ouvert — mais l'URL est bien acceptée sans
 * 404 SvelteKit).
 */
test('Journal : accepte ?focus=N depuis le lien Signature', async ({ page }) => {
  const response = await page.goto('/journal?focus=42');
  expect(response?.status()).toBe(200);
  await expect(
    page.getByRole('heading', { name: /Toutes vos estimations.*vérifiables/i })
  ).toBeVisible();
});
