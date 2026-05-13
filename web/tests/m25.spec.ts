import { expect, test } from '@playwright/test';

/**
 * Contrat « no-mock » de l'écran Eco-budget personnel (M25 — C19).
 *
 * M25 consomme les IPC `list_personal_goals`, `set_personal_goal`,
 * `delete_personal_goal`, `get_budget_status` exposés par sobria-app, et
 * stocke les objectifs dans `referentiel.sqlite`. Hors Tauri, le formulaire
 * reste rendu (éducatif) mais les inputs/boutons sont désactivés et aucune
 * donnée mockée n'est servie. L'état vide affiche la copie ciblée.
 *
 * Garanties :
 *   1. La coque est rendue (hero h1 + sub).
 *   2. Bannière `tauri_unavailable` avec mention `cargo run -p sobria-app`.
 *   3. Les 3 champs du formulaire sont rendus (indicateur, période, valeur).
 *   4. Les inputs/select sont disabled hors Tauri.
 *   5. Le bouton "Enregistrer l'objectif" est disabled.
 *   6. L'unité automatique affichée est gCO₂eq par défaut (indicator=co2eq).
 *   7. L'état vide est affiché (aucun objectif chargeable).
 *   8. Aucune liste d'objectifs (pas de mock budget servi).
 *   9. La note méthodologique ISO 8601 / P50 est visible.
 *  10. Le lien Méthodologie reste accessible.
 */

test('Eco-budget M25 : refuse de servir des objectifs mockés hors Tauri', async ({ page }) => {
  await page.goto('/m25');

  await expect(page).toHaveTitle(/Objectifs.*habitudes/);
  await expect(
    page.getByRole('heading', { name: /Pose un.*budget.*suis l'impact/i })
  ).toBeVisible();

  // Bannière tauri_unavailable
  const banner = page.getByRole('alert').first();
  await expect(banner).toBeVisible();
  await expect(banner).toContainText(/Application non lancée via Tauri/);
  await expect(banner).toContainText(/cargo run -p sobria-app/);

  // Les 3 champs sont rendus
  const indicatorSelect = page.locator('select').first();
  const periodSelect = page.locator('select').nth(1);
  const valueInput = page.locator('input[type="number"]').first();
  await expect(indicatorSelect).toBeVisible();
  await expect(periodSelect).toBeVisible();
  await expect(valueInput).toBeVisible();

  // Tous les inputs disabled hors Tauri
  await expect(indicatorSelect).toBeDisabled();
  await expect(periodSelect).toBeDisabled();
  await expect(valueInput).toBeDisabled();

  // L'unité automatique gCO₂eq est affichée (état initial : indicator=co2eq).
  await expect(page.locator('.unit-badge')).toContainText(/gCO₂eq/);

  // Bouton principal disabled
  await expect(page.getByRole('button', { name: /Enregistrer l'objectif/i })).toBeDisabled();

  // État vide : aucun objectif servi hors Tauri (charge IPC court-circuitée)
  // → l'illustration + le texte « Aucun objectif défini » s'affichent.
  await expect(page.locator('.empty')).toBeVisible();
  await expect(page.locator('.empty-text')).toContainText(/Aucun objectif défini/);
  await expect(page.locator('.empty-text')).toContainText(/budget mensuel CO₂eq/);

  // Aucune liste d'objectifs servie
  await expect(page.locator('.budget-list')).toHaveCount(0);
  await expect(page.locator('.budget-item')).toHaveCount(0);

  // Note méthodologique présente
  const methNote = page.locator('.meth-note');
  await expect(methNote).toContainText(/ISO 8601/);
  await expect(methNote).toContainText(/P50/);

  // Méthodologie OK (icon-btn dans la topbar)
  await expect(page.locator('.topbar a[href="/methodo"]')).toBeVisible();
});
