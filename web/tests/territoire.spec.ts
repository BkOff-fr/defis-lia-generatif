import { expect, test } from '@playwright/test';

/**
 * Contrat « no-mock » de l'écran Territoire FR (M20).
 *
 * M20 charge 200 sites industriels (RTE IRIS), 13 agrégats régionaux et un
 * Sankey énergétique national depuis les datasets ODRÉ stockés localement
 * — toujours via 4 commandes IPC (cf. CLAUDE.md §13). Hors Tauri, aucune
 * carte / aucun Sankey ne doit être rendu.
 *
 * Garanties :
 *   1. La coque est rendue (hero h1 + sub).
 *   2. Bannière `tauri_unavailable` avec mention `cargo run -p sobria-app`.
 *   3. AUCUN container Leaflet visible (pas de markers mockés).
 *   4. AUCUN SVG Sankey rendu (pas de flux mockés).
 *   5. Le lien Méthodologie reste accessible.
 */

test('Territoire FR : refuse de servir une carte mockée hors contexte Tauri', async ({ page }) => {
  await page.goto('/territoire');

  await expect(page).toHaveTitle(/Territoire France/);
  await expect(
    page.getByRole('heading', { name: /angle.*territorial.*français/i })
  ).toBeVisible();

  // Bannière tauri_unavailable
  const banner = page.getByRole('alert');
  await expect(banner).toBeVisible();
  await expect(banner).toContainText(/Application non lancée via Tauri/);
  await expect(banner).toContainText(/cargo run -p sobria-app/);

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
