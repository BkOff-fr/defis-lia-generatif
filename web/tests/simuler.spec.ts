import { expect, test } from '@playwright/test';

/**
 * Contrat « démo » de l'écran Simuler (M13 / C36).
 *
 * Le simulateur consomme `simulate_scenarios` via Tauri IPC — commande NON
 * couverte par la démo web : le moteur Monte-Carlo de simulation tourne
 * UNIQUEMENT côté Rust (cf. CLAUDE.md §13 + brief C11). En revanche, la
 * coque se nourrit de la démo : `list_models` (baseline) et
 * `list_datacenters` (picker) sont servis par les fixtures.
 *
 * Garanties :
 *   1. La coque est rendue (hero h1 + sub) + bannière « Mode démo ».
 *   2. Le baseline est peuplé (modèles servis par la démo).
 *   3. Les sliders sont rendus (PUE, mix, embodied, WUE → ≥ 4 ranges).
 *   4. La simulation auto-déclenchée échoue proprement : bannière
 *      « Application de bureau requise » orientée application de bureau
 *      (plus de mention `cargo run`).
 *   5. Aucun verdict CO₂eq fake — le bloc Verdict reste sur son
 *      placeholder, aucun composant Verdict monté.
 *   6. Le lien Méthodologie reste accessible.
 */

test('Simuler : coque démo + refus de tout verdict hors application de bureau', async ({
  page
}) => {
  await page.goto('/simuler');

  await expect(page).toHaveTitle(/Simulateur/);
  await expect(
    page.getByRole('heading', { name: /Et si on changeait.*un seul levier/i })
  ).toBeVisible();
  await expect(page.locator('aside.demo-banner')).toBeVisible();

  // Le baseline est peuplé depuis les fixtures (list_models couvert).
  const baselineSelect = page.locator('.baseline-card select').first();
  await expect(baselineSelect).toBeVisible();
  expect(await baselineSelect.locator('option').count()).toBeGreaterThan(1);

  // Les sliders SONT rendus (PUE, mix, embodied, WUE → au moins 4 ranges).
  const ranges = page.locator('input[type="range"]');
  await expect(ranges.first()).toBeVisible();
  expect(await ranges.count()).toBeGreaterThanOrEqual(4);

  // Le panneau de leviers est étiqueté pour les lecteurs d'écran.
  await expect(page.getByLabel('Panneau des leviers')).toBeVisible();

  // La simulation auto-déclenchée rejette : bannière « Application de
  // bureau requise » (simulate_scenarios non couvert par la démo).
  const banner = page.getByRole('alert');
  await expect(banner).toBeVisible();
  await expect(banner).toContainText(/Application de bureau requise/);
  await expect(banner).toContainText(/application de bureau/);
  await expect(banner).not.toContainText(/cargo run/);

  // PAS de verdict calculé — le bloc droit reste sur son placeholder.
  await expect(page.getByText(/Préparation du baseline/)).toBeVisible();

  // Méthodologie OK
  await expect(page.locator('.topbar a[href="/methodo"]')).toBeVisible();
});
