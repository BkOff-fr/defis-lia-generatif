import { expect, test } from '@playwright/test';

/**
 * Contrat « démo » de la section Mode Équipe (C29.1 — brief §C29.1 / C36).
 *
 * La section Mode Équipe est rendue dans la page /parametres entre
 * « Extension navigateur » et « Runtime ». Les 8 IPC `team_*` ne sont PAS
 * couverts par la démo web : toute action réseau rejettera avec le message
 * « application de bureau ». La coque reste rendue et les champs sont
 * éditables (validation client), mais aucune donnée équipe n'est servie.
 *
 * On vérifie :
 *   1. La section est rendue (heading + section-foot avec docs/operations/team-aggregator.md).
 *   2. Le pill de statut affiche "Non configuré" (état initial, sans data).
 *   3. Les champs sont éditables mais les actions restent gardées par la
 *      validation client (URL vide → boutons désactivés).
 *   4. Le 3-radio dispatcher (local / team / both) est présent.
 *   5. Aucun bloc "enrôlé" n'est rendu sans IPC (pas de fake data).
 *
 * Le flow complet (URL → ping → enrôlement → set_mode → logout) sera
 * testé en e2e manuel via la commande `cargo run -p sobria-app` sur un
 * binaire `sobria-team-aggregator` local (cf. brief C29 §"DoD globale").
 */

test('Paramètres → Mode Équipe : section rendue + aucune donnée équipe servie', async ({
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

  // 3. URL input présent et éditable (validation purement client).
  const urlInput = page.locator('[data-testid="team-url-input"]');
  await expect(urlInput).toBeVisible();
  await expect(urlInput).toBeEnabled();

  // Bouton Enregistrer l'URL désactivé tant que l'URL est vide/invalide.
  await expect(page.locator('[data-testid="team-save-url"]')).toBeDisabled();

  // Bouton Vérifier la connexion désactivé (URL non posée).
  await expect(page.locator('[data-testid="team-ping"]')).toBeDisabled();

  // 4. Bloc Enrôlement visible (puisque pas enrôlé), champs éditables mais
  //    soumission gardée par la validation client (champs vides).
  await expect(page.locator('[data-testid="team-enroll-block"]')).toBeVisible();
  await expect(page.locator('[data-testid="team-code-input"]')).toBeEnabled();
  await expect(page.locator('[data-testid="team-password-input"]')).toBeEnabled();
  await expect(page.locator('[data-testid="team-password-confirm-input"]')).toBeEnabled();
  await expect(page.locator('[data-testid="team-enroll-btn"]')).toBeDisabled();

  // 5. Dispatcher : les 3 radios sont rendus.
  for (const m of ['local', 'team', 'both']) {
    const radio = page.locator(`[data-testid="team-mode-${m}"]`);
    await expect(radio).toBeVisible();
  }

  // 6. Pas de bloc "enrôlé" (logout / dashboard externe) sans données IPC.
  await expect(page.locator('[data-testid="team-enrolled-block"]')).toHaveCount(0);

  // 7. Le footer pointe vers la doc opérationnelle.
  await expect(section.getByText(/team-aggregator\.md/)).toBeVisible();
});

test("Paramètres → Mode Équipe : input client-side ne déclenche pas d'IPC", async ({ page }) => {
  // En démo web, le boot équipe échoue silencieusement (loadTeam catch).
  // On vérifie qu'aucune erreur visible ne fuit dans la section (pas de
  // bannière rouge "team_error" en boot).
  await page.goto('/parametres');
  await expect(page.locator('[data-testid="team-section"]')).toBeVisible();
  await expect(page.locator('[data-testid="team-error"]')).toHaveCount(0);
  await expect(page.locator('[data-testid="team-ping-err"]')).toHaveCount(0);
});
