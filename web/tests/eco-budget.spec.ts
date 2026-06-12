import { expect, test } from '@playwright/test';

/**
 * Contrat « démo » de l'écran Eco-budget personnel (M25 — C19 / C36).
 *
 * M25 consomme les IPC `list_personal_goals`, `set_personal_goal`,
 * `delete_personal_goal`, `get_budget_status` exposés par sobria-app, et
 * stocke les objectifs dans `referentiel.sqlite`. Aucune n'est couverte
 * par la démo web : le suivi se fait depuis le ledger local de
 * l'application de bureau. Hors Tauri, le formulaire reste rendu
 * (éducatif) mais le chargement des objectifs échoue proprement avec la
 * bannière « Application de bureau requise » — aucun objectif mocké.
 *
 * Garanties :
 *   1. La coque est rendue (hero h1 + sub) + bannière « Mode démo ».
 *   2. Bannière « Application de bureau requise » orientée application
 *      de bureau (plus de mention `cargo run`).
 *   3. Les 3 champs du formulaire sont rendus (indicateur, période, valeur).
 *   4. Le bouton "Enregistrer l'objectif" reste désactivé (formulaire vide).
 *   5. L'unité automatique affichée est gCO₂eq par défaut (indicator=co2eq).
 *   6. Aucune liste d'objectifs (pas de mock budget servi).
 *   7. La note méthodologique ISO 8601 / P50 est visible.
 *   8. Le lien Méthodologie reste accessible.
 */

test('Eco-budget M25 : objectifs réservés à l’application de bureau', async ({ page }) => {
  await page.goto('/eco-budget');

  await expect(page).toHaveTitle(/Objectifs.*habitudes/);
  await expect(
    page.getByRole('heading', { name: /Pose un.*budget.*suis l'impact/i })
  ).toBeVisible();
  await expect(page.locator('aside.demo-banner')).toBeVisible();

  // Bannière « Application de bureau requise » (chargement des objectifs
  // rejeté — list_personal_goals non couvert par la démo).
  const banner = page.getByRole('alert').first();
  await expect(banner).toBeVisible();
  await expect(banner).toContainText(/Application de bureau requise/);
  await expect(banner).toContainText(/application de bureau/);
  await expect(banner).not.toContainText(/cargo run/);

  // Les 3 champs sont rendus
  const indicatorSelect = page.locator('select').first();
  const periodSelect = page.locator('select').nth(1);
  const valueInput = page.locator('input[type="number"]').first();
  await expect(indicatorSelect).toBeVisible();
  await expect(periodSelect).toBeVisible();
  await expect(valueInput).toBeVisible();

  // L'unité automatique gCO₂eq est affichée (état initial : indicator=co2eq).
  await expect(page.locator('.unit-badge')).toContainText(/gCO₂eq/);

  // Bouton principal désactivé tant que le formulaire est vide.
  await expect(page.getByRole('button', { name: /Enregistrer l'objectif/i })).toBeDisabled();

  // Aucune liste d'objectifs servie : la zone liste affiche l'erreur de
  // chargement (pas d'état vide trompeur, pas de mock budget).
  await expect(page.locator('.budget-list')).toHaveCount(0);
  await expect(page.locator('.budget-item')).toHaveCount(0);

  // Note méthodologique présente
  const methNote = page.locator('.meth-note');
  await expect(methNote).toContainText(/ISO 8601/);
  await expect(methNote).toContainText(/P50/);

  // Méthodologie OK (icon-btn dans la topbar)
  await expect(page.locator('.topbar a[href="/methodo"]')).toBeVisible();
});
