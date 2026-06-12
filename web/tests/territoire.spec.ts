import { expect, test } from '@playwright/test';

/**
 * Contrat « démo » de l'écran Territoire FR (M20 / C36).
 *
 * M20 charge 200 sites industriels (RTE IRIS), 13 agrégats régionaux et un
 * Sankey énergétique national via 4 commandes IPC — aucune n'est couverte
 * par la démo web (datasets locaux du backend Rust). Hors Tauri :
 * bannière « Application de bureau requise », aucune carte ni Sankey.
 *
 * Garanties :
 *   1. La coque est rendue (hero h1 + sub) + bannière « Mode démo ».
 *   2. Bannière d'avertissement « Application de bureau requise » dont le
 *      message oriente vers l'application de bureau (plus de `cargo run`).
 *   3. AUCUN container Leaflet visible (pas de markers mockés).
 *   4. AUCUN SVG Sankey rendu (pas de flux mockés).
 *   5. L'empty shell explicative est visible.
 *   6. Le lien Méthodologie reste accessible.
 */

test('Territoire FR : réservé à l’application de bureau, pas de carte démo', async ({ page }) => {
  await page.goto('/territoire');

  await expect(page).toHaveTitle(/Territoire France/);
  await expect(page.getByRole('heading', { name: /angle.*territorial.*français/i })).toBeVisible();
  await expect(page.locator('aside.demo-banner')).toBeVisible();

  // Bannière « Application de bureau requise » (nouveau libellé C36).
  const banner = page.getByRole('alert');
  await expect(banner).toBeVisible();
  await expect(banner).toContainText(/Application de bureau requise/);
  await expect(banner).toContainText(/application de bureau/);
  await expect(banner).not.toContainText(/cargo run/);

  // Pas de carte Leaflet — le container `.leaflet-container` n'apparaît
  // que si le composant TerritoireMap a été monté avec des données réelles.
  await expect(page.locator('.leaflet-container')).toHaveCount(0);

  // Pas de Sankey — `<section class="sankey-card">` n'est rendu que si
  // l'IPC sankey_fr_data a réussi.
  await expect(page.locator('.sankey-card')).toHaveCount(0);

  // L'empty shell explicative doit être visible
  await expect(page.getByRole('heading', { name: /Carte et Sankey indisponibles/i })).toBeVisible();

  // Méthodologie OK
  await expect(page.locator('.topbar a[href="/methodo"]')).toBeVisible();
});
