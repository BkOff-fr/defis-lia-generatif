import { expect, test } from '@playwright/test';

/**
 * Contrat « no-mock » de l'écran Référentiel modèles (M9 / C18).
 *
 * M9 lit le registre Rust embarqué via `list_models` et `get_model_detail`.
 * Hors runtime Tauri : aucune card ne s'affiche, le drawer reste fermé,
 * et l'utilisateur est explicitement redirigé vers `cargo run -p sobria-app`
 * (cf. CLAUDE.md §13 : pas de mock, pas de fallback).
 *
 * Garanties :
 *   1. La coque est rendue (hero h1 + sub).
 *   2. Bannière `tauri_unavailable` avec mention `cargo run -p sobria-app`.
 *   3. AUCUNE card modèle (grille vide).
 *   4. AUCUN drawer ouvert (pas de role="dialog").
 *   5. L'empty shell explicative est visible.
 *   6. Le lien Méthodologie reste accessible.
 */

test('Référentiel modèles : refuse de servir un catalogue mocké hors contexte Tauri', async ({
  page
}) => {
  await page.goto('/m9');

  await expect(page).toHaveTitle(/Référentiel modèles/);
  await expect(
    page.getByRole('heading', { name: /chiffres derrière.*chaque modèle/i })
  ).toBeVisible();

  // Bannière tauri_unavailable
  const banner = page.getByRole('alert');
  await expect(banner).toBeVisible();
  await expect(banner).toContainText(/Application non lancée via Tauri/);
  await expect(banner).toContainText(/cargo run -p sobria-app/);

  // Pas de grille rendue (role="grid")
  await expect(page.getByRole('grid')).toHaveCount(0);
  // Pas de drawer ouvert (role="dialog")
  await expect(page.getByRole('dialog')).toHaveCount(0);

  // Empty shell visible
  await expect(page.getByRole('heading', { name: /Référentiel indisponible/i })).toBeVisible();

  // Méthodologie OK
  await expect(page.locator('.topbar a[href="/methodo"]')).toBeVisible();
});
