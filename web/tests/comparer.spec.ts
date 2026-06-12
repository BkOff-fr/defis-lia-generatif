import { expect, test } from '@playwright/test';

/**
 * Contrat « démo » de l'écran Comparer (C36).
 *
 * Le comparateur fan-out N estimations en parallèle via
 * `estimate_for_comparison` — commande couverte par la démo web : chaque
 * cellule de la matrice est le point de grille le plus proche précalculé
 * par le moteur Rust (seed 42).
 *
 * Garanties :
 *   1. La coque est rendue (hero + sub).
 *   2. Bannière « Mode démo » visible, aucune bannière d'erreur.
 *   3. Le sélecteur de modèles et le bouton « Comparer » sont rendus.
 *   4. Lancer la comparaison produit un verdict CO₂eq réel.
 *   5. Le lien Méthodologie reste accessible.
 */

test('Comparer : sert une matrice issue des fixtures du moteur hors Tauri', async ({ page }) => {
  await page.goto('/comparer');

  await expect(page).toHaveTitle(/Comparer/);
  await expect(page.getByRole('heading', { name: /Le bon LLM.*pour le bon usage/i })).toBeVisible();

  // Mode démo : bannière layout présente, pas d'erreur tauri_unavailable.
  await expect(page.locator('aside.demo-banner')).toBeVisible();
  await expect(page.locator('.model-chips')).toBeVisible();
  await expect(page.getByRole('alert')).toHaveCount(0);

  // Bouton « Comparer » actif (sélection par défaut servie par list_models).
  const compareBtn = page.getByRole('button', { name: /^Comparer$/ });
  await expect(compareBtn).toBeEnabled();

  // La comparaison aboutit : verdict + profils détaillés.
  await compareBtn.click();
  const verdict = page.locator('section.verdict');
  await expect(verdict).toBeVisible();
  await expect(verdict).toContainText(/CO₂eq/);
  await expect(page.locator('section.cards-section')).toBeVisible();

  // Lien Méthodologie OK
  await expect(page.locator('.topbar a[href="/methodo"]')).toBeVisible();
});
