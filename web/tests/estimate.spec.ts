import { expect, test } from '@playwright/test';

/**
 * Contrat « no-mock » de l'écran Estimer.
 *
 * Hors Tauri (i.e. via `npm run dev` dans un navigateur), l'app doit :
 *  1. rendre la coque éditoriale (hero, rail, footer signature désactivé) ;
 *  2. afficher la bannière `tauri_unavailable` avec un message d'action
 *     clair pointant vers `cargo run -p sobria-app` ;
 *  3. NE PAS afficher le composer (sélecteur modèle + textarea + bouton
 *     Estimer) — sinon on laisserait croire que l'utilisateur peut estimer
 *     alors que le moteur Rust n'est pas joignable.
 *
 * Ce test verrouille le principe CLAUDE.md §13 : « pas de mock, pas de
 * fallback, pas de données factices ». Toute régression qui ré-introduit
 * un mock côté front fera tomber cette spec.
 */

test('Estimer : refuse de mocker hors contexte Tauri', async ({ page }) => {
  await page.goto('/');

  // 1. La coque éditoriale est bien rendue (titre + sous-titre).
  await expect(page).toHaveTitle(/Sobr\.ia/);
  await expect(page.getByRole('heading', { name: /poids carbone.*requête.*LLM/i })).toBeVisible();

  // 2. Bannière `tauri_unavailable` avec le message d'action attendu.
  const banner = page.getByRole('alert');
  await expect(banner).toBeVisible();
  await expect(banner).toContainText(/Application non lancée via Tauri/);
  await expect(banner).toContainText(/cargo run -p sobria-app|cargo tauri dev/);

  // 3. Le composer (form d'estimation) n'est PAS rendu — son absence est
  //    la garantie du contrat no-mock.
  await expect(page.locator('form.composer')).toHaveCount(0);
  await expect(page.getByRole('button', { name: /Estimer l'impact/ })).toHaveCount(0);

  // 4. Le rail de navigation reste actif (la coque doit fonctionner même
  //    sans IPC pour que l'utilisateur lise Méthodologie, etc.).
  await expect(page.getByRole('link', { name: 'Méthodologie', exact: true })).toBeVisible();
});
