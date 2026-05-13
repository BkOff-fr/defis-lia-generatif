import { expect, test } from '@playwright/test';

/**
 * Contrat « no-mock » de l'écran Empreinte projet (M17).
 *
 * M17 consomme 6 commandes IPC (`list_projects`, `get_project`,
 * `create_project`, `update_project`, `delete_project`,
 * `generate_project_datasheet`) qui lisent / écrivent dans
 * `referentiel.sqlite` et combinent avec le ledger d'audit pour générer
 * un datasheet Gebru au format JSON-LD.
 *
 * Hors Tauri, la coque pédagogique reste rendue (hero + bannière +
 * empty state) mais aucun projet mocké n'est servi : pas de liste, pas
 * de datasheet, le bouton « Nouveau projet » est désactivé.
 *
 * Garanties :
 *   1. Title + hero h1 + sub visibles.
 *   2. Bannière `tauri_unavailable` mentionne `cargo run -p sobria-app`.
 *   3. Liste des projets vide → empty state avec mention « Gebru ».
 *   4. Bouton « Nouveau projet » disabled.
 *   5. Aucune card de projet rendue.
 *   6. Panel droit affiche le placeholder (pas de datasheet rendu).
 *   7. Le lien Méthodologie reste accessible.
 */

test('Empreinte projet : refuse de servir des projets ou un datasheet mocké hors Tauri', async ({
  page
}) => {
  await page.goto('/m17');

  await expect(page).toHaveTitle(/Empreinte projet/);

  // Hero h1 (italic "publie")
  await expect(
    page.getByRole('heading', { name: /Documente.*publie.*reproduis/i })
  ).toBeVisible();

  // Bannière tauri_unavailable
  const banner = page.getByRole('alert');
  await expect(banner).toBeVisible();
  await expect(banner).toContainText(/Application non lancée via Tauri/);
  await expect(banner).toContainText(/cargo run -p sobria-app/);

  // Bouton "Nouveau projet" présent mais désactivé hors Tauri.
  const newBtn = page.getByRole('button', { name: /Nouveau projet/i });
  await expect(newBtn).toBeVisible();
  await expect(newBtn).toBeDisabled();

  // Empty state explicite avec mention Gebru.
  await expect(page.locator('.empty-state')).toBeVisible();
  await expect(page.locator('.empty-state')).toContainText(/Gebru/);

  // Aucune card projet rendue (rien dans la liste).
  await expect(page.locator('.project-card')).toHaveCount(0);

  // Panel droit affiche le placeholder de sélection (pas de datasheet).
  await expect(page.locator('.placeholder')).toBeVisible();
  await expect(page.locator('.placeholder')).toContainText(/Sélectionne ou crée un projet/i);

  // Aucune section Gebru rendue (pas de datasheet généré).
  await expect(page.locator('.gebru-card')).toHaveCount(0);

  // Lien méthodologie accessible.
  await expect(page.locator('.topbar a[href="/methodo"]')).toBeVisible();
});
