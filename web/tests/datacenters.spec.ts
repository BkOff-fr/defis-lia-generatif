import { expect, test } from '@playwright/test';

/**
 * Contrat « no-mock » de l'écran Datacenters Europe (M12).
 *
 * M12 charge les 28 datacenters européens embarqués dans le binaire via les
 * commandes IPC `list_datacenters` et `aggregate_datacenters_by_country`. Le
 * dataset est statique (include_str! au build) mais la carte ne doit pas
 * s'afficher en l'absence de runtime Tauri (cf. CLAUDE.md §13).
 *
 * Garanties :
 *   1. La coque est rendue (hero h1 + sub).
 *   2. Bannière `tauri_unavailable` avec mention `cargo run -p sobria-app`.
 *   3. AUCUN container Leaflet (`.leaflet-container`) visible.
 *   4. AUCUN drill-down rendu (pas de carte → pas de markers cliquables).
 *   5. L'empty shell explicative est visible.
 *   6. Le lien Méthodologie reste accessible.
 */

test('Datacenters Europe : refuse de servir une carte mockée hors contexte Tauri', async ({
  page
}) => {
  await page.goto('/datacenters');

  await expect(page).toHaveTitle(/Datacenters Europe/);
  await expect(
    page.getByRole('heading', { name: /Où tournent.*physiquement.*vos prompts/i })
  ).toBeVisible();

  // Bannière tauri_unavailable
  const banner = page.getByRole('alert');
  await expect(banner).toBeVisible();
  await expect(banner).toContainText(/Application non lancée via Tauri/);
  await expect(banner).toContainText(/cargo run -p sobria-app/);

  // Pas de carte Leaflet
  await expect(page.locator('.leaflet-container')).toHaveCount(0);

  // Empty shell visible
  await expect(page.getByRole('heading', { name: /Carte indisponible/i })).toBeVisible();

  // Méthodologie OK
  await expect(page.locator('.topbar a[href="/methodo"]')).toBeVisible();
});
