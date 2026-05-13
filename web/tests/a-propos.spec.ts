import { expect, test } from '@playwright/test';

/**
 * Smoke test de la page À propos / Crédits (module M14).
 *
 * Comme /methodo, la page est essentiellement statique (le bloc « État
 * technique » dépend de l'IPC `meta_info` mais affiche un message clair
 * hors Tauri sans bloquer le reste du contenu). On vérifie donc :
 *   1. La route répond et porte le bon titre.
 *   2. Les sections principales (Méthodologie, Licences, Sources,
 *      Contributeurs, Mentions légales, État technique) sont présentes.
 *   3. Les liens externes ont rel="noopener noreferrer".
 *   4. Hors Tauri, la section « État technique » signale clairement
 *      l'indisponibilité de l'IPC plutôt que de planter.
 */

test('À propos : charge et présente toutes les sections', async ({ page }) => {
  const response = await page.goto('/a-propos');
  expect(response?.status()).toBe(200);

  await expect(page).toHaveTitle(/À propos/);

  // Logo + mission visibles.
  await expect(page.getByRole('heading', { level: 1, name: 'Sobr.ia' })).toBeVisible();
  await expect(page.getByText(/Candidat au défi data\.gouv\.fr/)).toBeVisible();

  // Les 6 sections doivent rendre leurs h2.
  await expect(page.getByRole('heading', { name: 'Méthodologie' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Licences' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Sources des données' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Contributeurs' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Mentions légales' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'État technique' })).toBeVisible();

  // Quelques contenus clés.
  await expect(page.getByText(/AFNOR SPEC 2314/).first()).toBeVisible();
  await expect(page.getByText('Etalab 2.0')).toBeVisible();
  await expect(page.getByText('Thibault')).toBeVisible();
  await expect(page.getByText(/Aucune donnée envoyée à un serveur externe/)).toBeVisible();
});

test('À propos : liens externes sécurisés (noopener)', async ({ page }) => {
  await page.goto('/a-propos');
  const externals = page.locator('a[target="_blank"]');
  const count = await externals.count();
  expect(count).toBeGreaterThan(0);
  for (let i = 0; i < count; i++) {
    const rel = await externals.nth(i).getAttribute('rel');
    expect(rel ?? '').toMatch(/noopener/);
  }
});

test("À propos : hors Tauri, état technique signale l'indisponibilité IPC", async ({ page }) => {
  await page.goto('/a-propos');
  // Le bloc « État technique » doit afficher le code IPC explicite.
  await expect(page.getByText('tauri_unavailable')).toBeVisible();
});
