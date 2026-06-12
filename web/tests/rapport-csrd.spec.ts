import { expect, test } from '@playwright/test';

/**
 * Contrat « démo » de l'écran Rapport CSRD/AGEC (M22 / C36).
 *
 * M22 consomme l'IPC `export_csrd_report` qui lit le ledger d'audit Rust
 * et génère un PDF + JSON-LD PROV-O — commande NON couverte par la démo
 * web. La page ne bootstrape aucun IPC : hors Tauri, le formulaire est
 * rendu et utilisable (validation client), mais la génération elle-même
 * nécessite l'application de bureau (dialogue natif + ledger local) —
 * aucun PDF mocké ne doit jamais être produit.
 *
 * Garanties :
 *   1. La coque est rendue (hero h1 + sub) + bannière « Mode démo ».
 *   2. C42 : bannière « app de bureau requise » au chargement + Générer désactivé.
 *   3. Les 4 champs du formulaire SONT rendus et éditables (coque
 *      éducative, validation côté client).
 *   4. Le bouton "Choisir le dossier puis générer" suit la validation du
 *      formulaire (désactivé tant que la raison sociale est vide).
 *   5. Aucune card de succès (pas de PDF mocké).
 *   6. Le lien Méthodologie reste accessible.
 */

test('Rapport CSRD/AGEC : formulaire rendu, génération réservée à l’app de bureau', async ({
  page
}) => {
  await page.goto('/rapport-csrd');

  await expect(page).toHaveTitle(/Rapport réglementaire \(CSRD\/AGEC\)/);
  await expect(
    page.getByRole('heading', { name: /Un rapport.*conforme.*prêt à signer/i })
  ).toBeVisible();

  // Mode démo : bannière layout, pas de bannière d'erreur au boot.
  await expect(page.locator('aside.demo-banner')).toBeVisible();

  // Les 4 champs sont rendus et éditables (validation purement client).
  const orgInput = page.locator('input[type="text"]').first();
  await expect(orgInput).toBeVisible();
  await expect(orgInput).toBeEnabled();
  await expect(page.locator('input[type="date"]').first()).toBeEnabled();
  await expect(page.locator('select')).toBeEnabled();
  // C42 — bannière « Application de bureau requise » dès le chargement
  // (la génération écrit sur disque) ; le bouton Générer est désactivé.
  const alert = page.getByRole('alert').first();
  await expect(alert).toBeVisible();
  await expect(alert).toContainText(/Application de bureau requise/);
  await expect(alert).not.toContainText(/cargo/);
  await expect(page.locator('button[type=submit]')).toBeDisabled();

  // Bouton principal : désactivé tant que le formulaire est incomplet,
  // activé dès que la raison sociale est posée (les dates sont préremplies).
  const generateBtn = page.getByRole('button', { name: /Choisir le dossier puis générer/i });
  await expect(generateBtn).toBeDisabled();
  await orgInput.fill('ACME SAS');
  await expect(generateBtn).toBeEnabled();

  // Aucune success card (aucun PDF mocké, jamais).
  await expect(page.locator('.success')).toHaveCount(0);

  // Méthodologie OK
  await expect(page.locator('.topbar a[href="/methodo"]')).toBeVisible();
});
