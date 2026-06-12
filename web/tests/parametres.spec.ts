import { expect, test } from '@playwright/test';

/**
 * Contrat « démo » de l'écran Paramètres (C10 — ADR-0010 / C36).
 *
 * Paramètres affiche : persona, modules par catégorie, modules disponibles,
 * refaire onboarding + langue, référentiel, extension, mode équipe,
 * runtime. La page est PARTIELLEMENT couverte par la démo web :
 * `meta_info`, `list_methodologies`, `get_referentiel_status` et
 * `get/set_app_preferences` répondent, mais le bootstrap groupé inclut les
 * commandes pairing (extension navigateur) NON couvertes — la page affiche
 * donc la bannière « Application de bureau requise ».
 *
 * On vérifie :
 *   1. La coque est rendue (hero h1, sections persona / modules) +
 *      bannière « Mode démo ».
 *   2. La bannière « Application de bureau requise » s'affiche, orientée
 *      application de bureau (plus de mention `cargo run`).
 *   3. Les contrôles de préférences (personas, refaire onboarding) sont
 *      actifs — `set_app_preferences` est couvert par la démo.
 *   4. La section Runtime n'est PAS peuplée (le bootstrap groupé a échoué
 *      avant d'assigner `meta` — aucun chemin filesystem inventé).
 */

test('Paramètres : sections persona/modules + runtime démo, pairing en bandeau (C41)', async ({
  page
}) => {
  await page.goto('/parametres');

  await expect(page).toHaveTitle(/Paramètres/);
  await expect(
    page.getByRole('heading', { name: /Vos.*paramètres.*moteur Sobr\.ia/i })
  ).toBeVisible();
  await expect(page.locator('aside.demo-banner')).toBeVisible();

  // C41 — plus de bannière globale : le rejet pairing (desktop-only) est
  // confiné à sa section, les sections couvertes par la démo s'affichent.
  await expect(page.getByRole('alert')).toHaveCount(0);
  const pairingMsg = page.locator('.reload-msg').filter({ hasText: /application de bureau/i });
  await expect(pairingMsg.first()).toBeVisible();
  await expect(pairingMsg.first()).not.toContainText(/cargo run/);

  // Les 5 sections principales sont rendues (titres présents)
  await expect(page.getByRole('heading', { name: /Persona courant/ })).toBeVisible();
  await expect(page.getByRole('heading', { name: /Modules activés/ })).toBeVisible();
  await expect(page.getByRole('heading', { name: /Modules disponibles/ })).toBeVisible();
  await expect(page.getByRole('heading', { name: /Réinitialiser/ })).toBeVisible();
  await expect(page.getByRole('heading', { name: /^Runtime$/ })).toBeVisible();

  // Les 5 personas sont proposés et actifs (set_app_preferences couvert
  // par la démo — préférences en mémoire le temps de la session).
  for (const p of ['student', 'pro_tech', 'enterprise', 'public_sector', 'researcher']) {
    const btn = page.locator(`button[data-persona="${p}"]`);
    await expect(btn).toBeVisible();
    await expect(btn).toBeEnabled();
  }

  // Bouton « Refaire l'onboarding » actif (préférences démo disponibles).
  await expect(page.locator('[data-action="redo-onboarding"]')).toBeEnabled();

  // C41 — bootstrap scindé : les sections couvertes par la démo s'affichent
  // (runtime + méthodologies + référentiel), seul le pairing remonte le
  // bandeau « Application de bureau requise » (vérifié plus haut).
  await expect(page.locator('.runtime-grid')).toBeVisible();
  await expect(page.locator('.runtime-grid')).toContainText(/démo web/);
});

// NOTE : les anciens smoke tests des stubs /importer (M10) et /exporter (M6)
// ont été retirés — ces routes n'existent plus (modules listés « À venir en
// v1.1+ » dans Paramètres). /simuler (M13) et /territoire (M20) ont leurs
// propres contrats démo dans `tests/simuler.spec.ts` / `tests/territoire.spec.ts`.
