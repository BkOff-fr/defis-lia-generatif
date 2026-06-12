import { expect, test } from '@playwright/test';

/**
 * Contrat « démo » de l'écran Datacenters Europe (M12 / C36).
 *
 * M12 charge les 28 datacenters européens via `list_datacenters` et
 * `aggregate_datacenters_by_country` — deux commandes couvertes par la
 * démo web (le dataset embarqué est servi tel quel en fixtures). La carte
 * Leaflet se monte donc aussi hors Tauri.
 *
 * Garanties :
 *   1. La coque est rendue (hero h1 + sub).
 *   2. Bannière « Mode démo » visible, aucune bannière d'erreur, pas
 *      d'empty shell « Carte indisponible ».
 *   3. Le container Leaflet est monté (datacenters servis par la démo).
 *   4. Le lien Méthodologie reste accessible.
 */

test('Datacenters Europe : sert la carte depuis les fixtures hors contexte Tauri', async ({
  page
}) => {
  await page.goto('/datacenters');

  await expect(page).toHaveTitle(/Datacenters Europe/);
  await expect(
    page.getByRole('heading', { name: /Où tournent.*physiquement.*vos prompts/i })
  ).toBeVisible();

  // Mode démo : bannière layout, pas d'erreur tauri_unavailable.
  await expect(page.locator('aside.demo-banner')).toBeVisible();

  // La carte Leaflet est montée avec le catalogue des fixtures.
  await expect(page.locator('.leaflet-container')).toBeVisible();
  await expect(page.getByRole('alert')).toHaveCount(0);

  // Pas d'empty shell (réservé au cas « aucune donnée »).
  await expect(page.getByRole('heading', { name: /Carte indisponible/i })).toHaveCount(0);

  // Méthodologie OK
  await expect(page.locator('.topbar a[href="/methodo"]')).toBeVisible();
});
