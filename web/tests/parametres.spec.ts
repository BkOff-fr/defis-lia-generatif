import { expect, test } from '@playwright/test';

/**
 * Contrat « no-mock » de l'écran Paramètres.
 *
 * Paramètres affiche le runtime via `meta_info()` IPC. Hors Tauri, le
 * runtime n'existe pas — on vérifie :
 *   1. La coque est rendue (hero h1).
 *   2. La bannière `tauri_unavailable` s'affiche.
 *   3. La section Runtime n'est PAS peuplée (pas de chemin filesystem
 *      affiché — sinon c'est qu'on a mocké).
 *   4. La section « Préférences à venir » est rendue (statique).
 */

test('Paramètres : section Runtime vide hors Tauri (no-mock)', async ({ page }) => {
  await page.goto('/parametres');

  await expect(page).toHaveTitle(/Paramètres/);
  await expect(
    page.getByRole('heading', { name: /Vos.*paramètres.*moteur Sobr\.ia/i })
  ).toBeVisible();

  const banner = page.getByRole('alert');
  await expect(banner).toBeVisible();
  await expect(banner).toContainText(/Application non lancée via Tauri/);

  // Aucune valeur runtime affichée
  await expect(page.locator('.runtime-grid')).toHaveCount(0);

  // Section préférences à venir présente
  await expect(page.getByRole('heading', { name: /Préférences utilisateur/ })).toBeVisible();
});

/**
 * Smoke tests des 4 stubs « coming soon » :
 * la page sert, le titre contient l'EFs prévue, le composant ComingSoon
 * rend le bloc IPC attendus.
 */
test.describe('Stubs (M4 / M9-M12 / M10 / M6)', () => {
  for (const r of [
    { path: '/simuler', module: 'M4', titleMatch: /5 ans/ },
    { path: '/importer', module: 'M10', titleMatch: /journal d.usage entreprise/i },
    { path: '/territoire', module: 'M9 / M12', titleMatch: /Cartographie IRIS/ },
    { path: '/exporter', module: 'M6', titleMatch: /rapport.*sourcé/i }
  ]) {
    test(`${r.path} — coque + IPC attendus`, async ({ page }) => {
      const response = await page.goto(r.path);
      expect(response?.status()).toBe(200);
      await expect(page.getByText(`Module ${r.module} · en chantier`)).toBeVisible();
      await expect(page.getByRole('heading', { name: r.titleMatch })).toBeVisible();
      await expect(page.getByText('IPC attendus')).toBeVisible();
    });
  }
});
