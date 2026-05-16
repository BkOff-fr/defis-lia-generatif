import { expect, test } from '@playwright/test';

/**
 * Contrat « no-mock » de l'écran Rapport CSRD/AGEC (M22).
 *
 * M22 consomme l'IPC `export_csrd_report` qui lit le ledger d'audit Rust et
 * génère un PDF + JSON-LD PROV-O. Hors Tauri, le formulaire reste visible
 * (éducatif) mais les inputs sont désactivés et le bouton de génération est
 * inaccessible — aucun PDF mocké ne doit jamais être produit.
 *
 * Garanties :
 *   1. La coque est rendue (hero h1 + sub).
 *   2. Bannière `tauri_unavailable` avec mention `cargo run -p sobria-app`.
 *   3. Les 4 champs du formulaire SONT rendus (la coque éducative est OK).
 *   4. Les inputs sont disabled hors Tauri (pas de soumission possible).
 *   5. Le bouton "Choisir le dossier puis générer" est disabled.
 *   6. Aucune card de succès (pas de PDF mocké).
 *   7. Le lien Méthodologie reste accessible.
 */

test('Rapport CSRD/AGEC : refuse de servir un PDF mocké hors contexte Tauri', async ({ page }) => {
  await page.goto('/rapport-csrd');

  await expect(page).toHaveTitle(/Rapport réglementaire \(CSRD\/AGEC\)/);
  await expect(
    page.getByRole('heading', { name: /Un rapport.*conforme.*prêt à signer/i })
  ).toBeVisible();

  // Bannière tauri_unavailable
  const banner = page.getByRole('alert');
  await expect(banner).toBeVisible();
  await expect(banner).toContainText(/Application non lancée via Tauri/);
  await expect(banner).toContainText(/cargo run -p sobria-app/);

  // Les 4 champs sont rendus
  await expect(page.locator('input[type="text"]').first()).toBeVisible();
  await expect(page.locator('input[type="date"]').first()).toBeVisible();
  await expect(page.locator('select')).toBeVisible();

  // Tous les inputs disabled hors Tauri
  await expect(page.locator('input[type="text"]').first()).toBeDisabled();
  await expect(page.locator('input[type="date"]').first()).toBeDisabled();
  await expect(page.locator('select')).toBeDisabled();

  // Bouton principal disabled
  await expect(
    page.getByRole('button', { name: /Choisir le dossier puis générer/i })
  ).toBeDisabled();

  // Aucune success card
  await expect(page.locator('.success')).toHaveCount(0);

  // Méthodologie OK
  await expect(page.locator('.topbar a[href="/methodo"]')).toBeVisible();
});
