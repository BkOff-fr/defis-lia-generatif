import { expect, test } from '@playwright/test';

/**
 * Contrat « démo » de l'écran Estimer (C36).
 *
 * Hors Tauri (i.e. via `npm run dev` dans un navigateur), le mode démo web
 * sert des fixtures précalculées par le VRAI moteur Rust (seed 42 — cf.
 * `src/lib/demo/index.ts`). L'app doit donc :
 *  1. rendre la coque éditoriale (hero, rail) ;
 *  2. afficher la bannière `aside.demo-banner` (« Mode démo ») du layout,
 *     et AUCUNE bannière d'erreur `tauri_unavailable` ;
 *  3. rendre le composer (modèles servis par `list_models`) ;
 *  4. servir une estimation réelle : `estimate_prompt` renvoie le point de
 *     grille le plus proche calculé par `sobria-estimator`.
 *
 * CLAUDE.md §13 reste respecté : aucune valeur inventée côté front — la
 * démo ne fait que servir des résultats du moteur Monte-Carlo.
 */

test('Estimer : sert la démo précalculée hors contexte Tauri', async ({ page }) => {
  await page.goto('/');

  // 1. La coque éditoriale est bien rendue (titre + sous-titre).
  await expect(page).toHaveTitle(/Sobr\.ia/);
  await expect(page.getByRole('heading', { name: /poids carbone.*requête.*LLM/i })).toBeVisible();

  // 2. Bannière « Mode démo » visible, composer monté, aucune erreur.
  const demoBanner = page.locator('aside.demo-banner');
  await expect(demoBanner).toBeVisible();
  await expect(demoBanner).toContainText(/Mode démo/);
  await expect(page.locator('form.composer')).toBeVisible();
  await expect(page.getByRole('alert')).toHaveCount(0);

  // 3. Le composer est rendu avec son bouton de soumission actif.
  const submit = page.getByRole('button', { name: /Estimer l'impact/ });
  await expect(submit).toBeVisible();
  await expect(submit).toBeEnabled();

  // 4. Une estimation aboutit à un vrai résultat (point de grille le plus
  //    proche servi par les fixtures du moteur — pas une erreur).
  await page
    .locator('form.composer textarea')
    .fill('Explique-moi la photosynthèse en trois phrases.');
  await submit.click();
  const result = page.locator('section.result-block');

  // C40 — boucle « Réduire » : des alternatives plus sobres sont proposées,
  // réestimées par le moteur (fixtures démo) sur la même requête.
  const reduce = page.locator('section.reduce-suggestions');
  await expect(reduce).toBeVisible({ timeout: 8000 });
  // Deux issues légitimes : des alternatives plus sobres (cartes avec delta %)
  // ou « déjà l'un des plus sobres » si le modèle courant est en bas du
  // catalogue (cas du défaut GPT-4o mini).
  const outcome = reduce.locator('.reduce-card, .reduce-done').first();
  await expect(outcome).toBeVisible({ timeout: 8000 });
  if (await reduce.locator('.reduce-card').count()) {
    await expect(reduce.locator('.reduce-delta').first()).toContainText('%');
  }
  await expect(result).toBeVisible();
  await expect(result).toContainText(/CO₂eq/);

  // 5. Le rail C39 : 5 essentiels visibles, le reste derrière « Plus ».
  await expect(page.locator('nav a[href="/comparer"]').first()).toBeVisible();
  await expect(page.locator('nav a[href="/methodo"]')).toHaveCount(0);
  await page.click('nav button.rail-more');
  await expect(page.locator('nav a[href="/methodo"]').first()).toBeVisible();
});
