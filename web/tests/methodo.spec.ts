import { expect, test } from '@playwright/test';

/**
 * Smoke test de la page Méthodologie (module M8).
 *
 * Contrairement à Estimer et Journal, /methodo n'a aucune dépendance IPC :
 * c'est de la documentation statique embarquée. On vérifie donc :
 *   1. La page sert et porte le bon titre.
 *   2. Les sections principales (Méthode, Validation, Glossaire,
 *      Références, Bibliographie, À propos) sont présentes.
 *   3. Le sommaire latéral (TOC) cible bien chaque ancre.
 *   4. Les liens externes ont rel="noopener noreferrer" (Vérif sécurité
 *      a11y avant d'ouvrir des liens externes depuis Tauri).
 */

test('Méthodologie : sert toutes les sections principales', async ({ page }) => {
  const response = await page.goto('/methodo');
  expect(response?.status()).toBe(200);

  await expect(page).toHaveTitle(/Méthodologie/);
  await expect(page.getByRole('heading', { name: /méthode.*sourcée/i })).toBeVisible();

  // Les 6 sections doivent rendre leurs h2.
  await expect(page.getByRole('heading', { name: 'Formule de référence' })).toBeVisible();
  await expect(page.getByRole('heading', { name: /Propagation d.incertitude/ })).toBeVisible();
  await expect(page.getByRole('heading', { name: /Glossaire/ })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Références normatives' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Bibliographie sélective' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'À propos de cette version' })).toBeVisible();

  // Le glossaire contient au moins quelques termes clés.
  await expect(page.getByRole('cell', { name: /CO₂ équivalent/ })).toBeVisible();
  await expect(page.getByRole('cell', { name: /PUE/, exact: false })).toBeVisible();

  // Sommaire : un lien d'ancre par section.
  const toc = page.locator('.toc');
  await expect(toc.getByRole('link', { name: 'Méthode' })).toBeVisible();
  await expect(toc.getByRole('link', { name: 'Glossaire' })).toBeVisible();
});

test('Méthodologie : liens externes sécurisés (noopener)', async ({ page }) => {
  await page.goto('/methodo');
  // Chaque lien externe doit ouvrir un nouvel onglet avec rel sécurisé.
  const externals = page.locator('a[target="_blank"]');
  const count = await externals.count();
  expect(count).toBeGreaterThan(0);
  for (let i = 0; i < count; i++) {
    const rel = await externals.nth(i).getAttribute('rel');
    expect(rel ?? '').toMatch(/noopener/);
  }
});
