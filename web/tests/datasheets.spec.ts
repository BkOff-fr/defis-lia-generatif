import { expect, test } from '@playwright/test';

/**
 * Contrat « démo » de l'écran Empreinte projet (M17 / C36).
 *
 * M17 consomme 6 commandes IPC (`list_projects`, `get_project`,
 * `create_project`, `update_project`, `delete_project`,
 * `generate_project_datasheet`) qui lisent / écrivent dans
 * `referentiel.sqlite` et combinent avec le ledger d'audit pour générer
 * un datasheet Gebru au format JSON-LD. Aucune n'est couverte par la
 * démo web (état local de l'application de bureau).
 *
 * Hors Tauri, la coque pédagogique reste rendue (hero + bannières +
 * empty state) mais aucun projet mocké n'est servi : pas de liste, pas
 * de datasheet.
 *
 * Garanties :
 *   1. Title + hero h1 + sub visibles + bannière « Mode démo ».
 *   2. Bannière « Application de bureau requise » orientée application
 *      de bureau (plus de mention `cargo run`).
 *   3. Liste des projets vide → empty state « Aucun projet ».
 *   4. Aucune card de projet rendue.
 *   5. Panel droit affiche le placeholder (pas de datasheet rendu).
 *   6. Le lien Méthodologie reste accessible.
 */

test('Empreinte projet : projets et datasheets réservés à l’application de bureau', async ({
  page
}) => {
  await page.goto('/datasheets');

  await expect(page).toHaveTitle(/Datasheet scientifique/);

  // Hero h1 (italic "publie")
  await expect(page.getByRole('heading', { name: /Documente.*publie.*reproduis/i })).toBeVisible();
  await expect(page.locator('aside.demo-banner')).toBeVisible();

  // Bannière « Application de bureau requise » (list_projects rejeté).
  const banner = page.getByRole('alert');
  await expect(banner).toBeVisible();
  await expect(banner).toContainText(/Application de bureau requise/);
  await expect(banner).toContainText(/application de bureau/);
  await expect(banner).not.toContainText(/cargo run/);

  // C42 — Bouton "Nouveau projet" rendu mais DÉSACTIVÉ hors Tauri (action
  // ledger desktop-only), avec explication au survol. Il échouait tard avant
  // que l'IPC `create_project` n'est pas joignable (application de bureau).
  const newProject = page.getByRole('button', { name: /Nouveau projet/i });
  await expect(newProject).toBeVisible();
  await expect(newProject).toBeDisabled();
  await expect(newProject).toHaveAttribute('title', /application de bureau/i);

  // Empty state explicite : aucun projet servi par la démo.
  await expect(page.locator('.empty-state')).toBeVisible();
  await expect(page.locator('.empty-state')).toContainText(/Aucun projet/);
  await expect(page.locator('.empty-state')).not.toContainText(/cargo run/);

  // Gebru est référencé dans l'eyebrow du hero (méthodo visible quel que
  // soit le contexte) — c'est le signal "format académique" promis dans le brief.
  await expect(page.locator('.hero-eyebrow')).toContainText(/Gebru/);

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
