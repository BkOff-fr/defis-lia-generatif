import { expect, test } from '@playwright/test';

/**
 * Contrat « no-mock » de la section Mode Équipe (C29.1 — brief §C29.1).
 *
 * La section Mode Équipe est rendue dans la page /parametres entre
 * « Extension navigateur » et « Runtime ». Hors Tauri, les 8 IPC `team_*`
 * sont indisponibles — on vérifie :
 *   1. La section est rendue (heading + section-foot avec docs/operations/team-aggregator.md).
 *   2. Le pill de statut affiche "Non configuré" (état initial, sans data).
 *   3. Les contrôles d'écriture (URL input, mots de passe, boutons) sont
 *      disabled hors Tauri — pas de mock, pas de fallback (CLAUDE.md §13).
 *   4. Le 3-radio dispatcher (local / team / both) est présent et disabled.
 *   5. Aucun bloc "enrôlé" n'est rendu sans IPC (pas de fake data).
 *
 * Le flow complet (URL → ping → enrôlement → set_mode → logout) sera
 * testé en e2e manuel via la commande `cargo run -p sobria-app` sur un
 * binaire `sobria-team-aggregator` local (cf. brief C29 §"DoD globale").
 */

test('Paramètres → Mode Équipe : section rendue + contrôles disabled hors Tauri', async ({
  page
}) => {
  await page.goto('/parametres');

  // 1. La section est présente avec son heading FR.
  const section = page.locator('[data-testid="team-section"]');
  await expect(section).toBeVisible();
  await expect(section.getByRole('heading', { name: /Mode Équipe self-hosted/i })).toBeVisible();
  await expect(section.getByText(/8 IPC.*team_/)).toBeVisible();

  // 2. Pill de statut = "Non configuré" (état initial sans IPC).
  const pill = page.locator('[data-testid="team-status-pill"]');
  await expect(pill).toBeVisible();
  await expect(pill).toContainText(/Non configuré/i);

  // 3. URL input présent ET disabled hors Tauri.
  const urlInput = page.locator('[data-testid="team-url-input"]');
  await expect(urlInput).toBeVisible();
  await expect(urlInput).toBeDisabled();

  // Bouton Enregistrer l'URL disabled (Tauri OFF + URL vide).
  await expect(page.locator('[data-testid="team-save-url"]')).toBeDisabled();

  // Bouton Vérifier la connexion disabled (Tauri OFF + URL non posée).
  await expect(page.locator('[data-testid="team-ping"]')).toBeDisabled();

  // Toggle "Accepter les certificats auto-signés" disabled hors Tauri.
  await expect(page.locator('[data-testid="team-accept-cert"]')).toBeDisabled();

  // 4. Bloc Enrôlement visible (puisque pas enrôlé) avec champs disabled.
  await expect(page.locator('[data-testid="team-enroll-block"]')).toBeVisible();
  await expect(page.locator('[data-testid="team-code-input"]')).toBeDisabled();
  await expect(page.locator('[data-testid="team-password-input"]')).toBeDisabled();
  await expect(page.locator('[data-testid="team-password-confirm-input"]')).toBeDisabled();
  await expect(page.locator('[data-testid="team-enroll-btn"]')).toBeDisabled();

  // 5. Dispatcher : les 3 radios sont rendus et disabled hors Tauri.
  for (const m of ['local', 'team', 'both']) {
    const radio = page.locator(`[data-testid="team-mode-${m}"]`);
    await expect(radio).toBeVisible();
    await expect(radio).toBeDisabled();
  }

  // 6. Pas de bloc "enrôlé" (logout / dashboard externe) sans données IPC.
  await expect(page.locator('[data-testid="team-enrolled-block"]')).toHaveCount(0);

  // 7. Le footer pointe vers la doc opérationnelle.
  await expect(section.getByText(/team-aggregator\.md/)).toBeVisible();
});

test("Paramètres → Mode Équipe : input client-side ne déclenche pas d'IPC", async ({ page }) => {
  // Hors Tauri les inputs sont disabled. On vérifie qu'aucune erreur visible
  // ne fuit dans la section (pas de bannière rouge "team_error" en boot).
  await page.goto('/parametres');
  await expect(page.locator('[data-testid="team-section"]')).toBeVisible();
  await expect(page.locator('[data-testid="team-error"]')).toHaveCount(0);
  await expect(page.locator('[data-testid="team-ping-err"]')).toHaveCount(0);
});
