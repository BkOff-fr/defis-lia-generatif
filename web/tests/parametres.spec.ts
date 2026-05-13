import { expect, test } from '@playwright/test';

/**
 * Contrat « no-mock » de l'écran Paramètres (C10 — ADR-0010).
 *
 * Paramètres affiche : persona, modules par catégorie, modules disponibles,
 * refaire onboarding + langue. Hors Tauri, l'IPC `get_app_preferences` et
 * `meta_info` échouent — on vérifie :
 *   1. La coque est rendue (hero h1, sections persona / modules).
 *   2. Les bannières `tauri_unavailable` s'affichent.
 *   3. Les contrôles d'écriture (boutons persona, toggles modules, langue)
 *      sont disabled — pas de mock, pas de fallback.
 *   4. La section Runtime n'est PAS peuplée (pas de chemin filesystem
 *      affiché — sinon c'est qu'on a mocké).
 */

test('Paramètres : sections persona + modules + runtime vide hors Tauri', async ({ page }) => {
  await page.goto('/parametres');

  await expect(page).toHaveTitle(/Paramètres/);
  await expect(
    page.getByRole('heading', { name: /Vos.*paramètres.*moteur Sobr\.ia/i })
  ).toBeVisible();

  // Bannière tauri_unavailable
  const banners = page.getByRole('alert');
  await expect(banners.first()).toBeVisible();
  await expect(banners.first()).toContainText(/Application non lancée via Tauri/);

  // Les 5 sections principales sont rendues (titres présents)
  await expect(page.getByRole('heading', { name: /Persona courant/ })).toBeVisible();
  await expect(page.getByRole('heading', { name: /Modules activés/ })).toBeVisible();
  await expect(page.getByRole('heading', { name: /Modules disponibles/ })).toBeVisible();
  await expect(page.getByRole('heading', { name: /Réinitialiser/ })).toBeVisible();
  await expect(page.getByRole('heading', { name: /^Runtime$/ })).toBeVisible();

  // Les 5 personas sont proposés (boutons disabled hors Tauri)
  for (const p of ['student', 'pro_tech', 'enterprise', 'public_sector', 'researcher']) {
    const btn = page.locator(`button[data-persona="${p}"]`);
    await expect(btn).toBeVisible();
    await expect(btn).toBeDisabled();
  }

  // Bouton « Refaire l'onboarding » disabled hors Tauri
  await expect(page.locator('[data-action="redo-onboarding"]')).toBeDisabled();

  // Aucune valeur runtime affichée (pas de mock IPC)
  await expect(page.locator('.runtime-grid')).toHaveCount(0);
});

/**
 * Smoke tests des stubs « coming soon ». Note : /simuler porte désormais
 * le moduleId M13 (et non M4 réservé, cf. ADR-0010 + brief C10).
 */
test.describe('Stubs (M13 / M9-M12 / M10 / M6)', () => {
  for (const r of [
    { path: '/simuler', module: 'M13', titleMatch: /7 leviers/ },
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
