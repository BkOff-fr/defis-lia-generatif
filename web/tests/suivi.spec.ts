import { expect, test } from '@playwright/test';

/**
 * Contrat « démo » de l'écran Dashboard personnel (M15 / C36).
 *
 * M15 consomme l'IPC `get_dashboard_summary` — couverte par la démo web :
 * les agrégats sont dérivés arithmétiquement des P50 du moteur (fixtures
 * seed 42), aucun chiffre inventé côté front. Hors Tauri, le dashboard se
 * rend donc avec des données d'exemple.
 *
 * Garanties :
 *   1. La coque est rendue (hero h1 + sub).
 *   2. Bannière « Mode démo » visible, aucune bannière d'erreur.
 *   3. Le switch de périodes contient bien les 5 tabs (rôle tablist).
 *   4. Les tabs sont actifs et le switch de période fonctionne.
 *   5. Le chart rend ses barres (daily_series servie par la démo).
 *   6. Le top modèles est peuplé.
 *   7. Le lien Méthodologie reste accessible.
 */

test('Dashboard personnel : sert les métriques des fixtures hors contexte Tauri', async ({
  page
}) => {
  await page.goto('/suivi');

  await expect(page).toHaveTitle(/Tableau de bord personnel/);
  await expect(
    page.getByRole('heading', { name: /Votre empreinte IA.*p[ée]riode apr[èe]s p[ée]riode/i })
  ).toBeVisible();

  // Mode démo : bannière layout, pas d'erreur tauri_unavailable.
  await expect(page.locator('aside.demo-banner')).toBeVisible();

  // Tablist visible avec les 5 périodes attendues
  const tablist = page.getByRole('tablist', { name: /Sélection de la période/i });
  await expect(tablist).toBeVisible();
  const tabs = tablist.getByRole('tab');
  await expect(tabs).toHaveCount(5);
  await expect(tabs.nth(0)).toHaveText(/Aujourd'hui/);
  await expect(tabs.nth(1)).toHaveText(/7 derniers jours/);
  await expect(tabs.nth(2)).toHaveText(/Ce mois-ci/);
  await expect(tabs.nth(3)).toHaveText(/Mois précédent/);
  await expect(tabs.nth(4)).toHaveText(/Cette année/);

  // Les métriques se chargent sans erreur.
  await expect(page.locator('rect.bar').first()).toBeVisible();
  await expect(page.getByRole('alert')).toHaveCount(0);

  // Tous les tabs sont actifs (le backend démo répond) et le switch marche.
  for (let i = 0; i < 5; i += 1) {
    await expect(tabs.nth(i)).toBeEnabled();
  }
  await tabs.nth(1).click();
  await expect(tabs.nth(1)).toHaveAttribute('aria-selected', 'true');

  // Chart + top modèles peuplés depuis les fixtures.
  expect(await page.locator('rect.bar').count()).toBeGreaterThan(0);
  expect(await page.locator('.top-list li').count()).toBeGreaterThan(0);

  // Lien méthodologie accessible.
  await expect(page.locator('.topbar a[href="/methodo"]')).toBeVisible();
});
