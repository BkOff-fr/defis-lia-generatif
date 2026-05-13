import { expect, test } from '@playwright/test';

/**
 * Contrat « no-mock » de l'écran Simuler (M13).
 *
 * Le simulateur consomme `simulate_scenarios` via Tauri IPC — hors Tauri on
 * doit refuser de servir une simulation factice. Le moteur Monte-Carlo
 * tourne UNIQUEMENT côté Rust (cf. CLAUDE.md §13 + brief C11).
 *
 * Particularité M13 (cf. brief C11 §4) : la coque du panneau de leviers est
 * rendue même hors Tauri pour que l'utilisateur visualise l'écran avant
 * d'installer l'app, MAIS aucun verdict CO₂eq n'est calculé — on affiche un
 * placeholder explicite (« lance cargo run pour activer le moteur »).
 *
 * Garanties :
 *   1. La coque est rendue (hero h1 + sub).
 *   2. Bannière `tauri_unavailable` avec mention `cargo run`.
 *   3. Les 7 sliders sont rendus (mais leurs deltas ne calculent rien).
 *   4. Aucun verdict CO₂eq fake — on doit voir le placeholder invitant à
 *      lancer Tauri à la place du bloc Verdict.
 *   5. Le lien Méthodologie reste accessible.
 */

test('Simuler : rend la coque + sliders mais refuse tout verdict mocké', async ({ page }) => {
  await page.goto('/simuler');

  await expect(page).toHaveTitle(/Simulateur/);
  await expect(
    page.getByRole('heading', { name: /Et si on changeait.*un seul levier/i })
  ).toBeVisible();

  // Bannière tauri_unavailable
  const banner = page.getByRole('alert');
  await expect(banner).toBeVisible();
  await expect(banner).toContainText(/Application non lancée via Tauri/);
  await expect(banner).toContainText(/cargo run -p sobria-app/);

  // Les sliders SONT rendus (PUE, mix, embodied, WUE → au moins 4 ranges).
  // Le contrat no-mock ne nous prive pas de la coque éducative.
  const ranges = page.locator('input[type="range"]');
  await expect(ranges.first()).toBeVisible();
  expect(await ranges.count()).toBeGreaterThanOrEqual(4);

  // Le panneau de leviers est étiqueté pour les lecteurs d'écran.
  await expect(page.getByLabel('Panneau des leviers')).toBeVisible();

  // PAS de verdict calculé — on voit le placeholder explicite.
  await expect(page.getByText(/cargo run -p sobria-app/).last()).toBeVisible();

  // Méthodologie OK
  await expect(page.locator('.topbar a[href="/methodo"]')).toBeVisible();
});
