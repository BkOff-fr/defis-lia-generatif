import { expect, test } from '@playwright/test';

/**
 * Contrat « no-mock » de l'écran Dashboard personnel (M15).
 *
 * M15 consomme l'IPC `get_dashboard_summary` qui lit le ledger d'audit Rust et
 * renvoie des agrégats P50 par période. Hors Tauri, la coque pédagogique reste
 * rendue (header + switch périodes désactivés + bannière) mais aucune métrique
 * mockée n'est servie : pas de valeurs numériques, pas de chart, pas de top.
 *
 * Garanties :
 *   1. La coque est rendue (hero h1 + sub).
 *   2. Bannière `tauri_unavailable` avec mention `cargo run -p sobria-app`.
 *   3. Le switch de périodes contient bien les 5 tabs (rôle tablist).
 *   4. Tous les tabs sont disabled hors Tauri.
 *   5. Aucune barre du chart n'est rendue (pas de données → pas de SVG bars).
 *   6. Aucune entrée dans le top modèles (top-list absente ou vide).
 *   7. Le lien Méthodologie reste accessible.
 */

test('Dashboard personnel : refuse de servir des métriques mockées hors contexte Tauri', async ({
  page
}) => {
  await page.goto('/m15');

  await expect(page).toHaveTitle(/Tableau de bord personnel/);
  await expect(
    page.getByRole('heading', { name: /Ton.*empreinte IA.*p[ée]riode apr[èe]s p[ée]riode/i })
  ).toBeVisible();

  // Bannière tauri_unavailable
  const banner = page.getByRole('alert');
  await expect(banner).toBeVisible();
  await expect(banner).toContainText(/Application non lancée via Tauri/);
  await expect(banner).toContainText(/cargo run -p sobria-app/);

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

  // Tous les tabs disabled hors Tauri (aucune sélection possible).
  for (let i = 0; i < 5; i += 1) {
    await expect(tabs.nth(i)).toBeDisabled();
  }

  // Aucune barre rendue (pas de daily_series côté front).
  await expect(page.locator('rect.bar')).toHaveCount(0);

  // Aucune entrée dans le top.
  await expect(page.locator('.top-list li')).toHaveCount(0);

  // Lien méthodologie accessible.
  await expect(page.locator('.topbar a[href="/methodo"]')).toBeVisible();
});
