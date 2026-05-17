import { expect, test } from '@playwright/test';

/**
 * Contrat « no-mock » du wizard d'onboarding (C10 — ADR-0010).
 *
 * Ces tests vérifient le parcours UI du wizard sans dépendre de l'IPC
 * Tauri (cf. estimate.spec.ts et CLAUDE.md §13). En contexte navigateur :
 *  - le store `preferences` reste en mode `loaded: false` ;
 *  - les gardes de route ne se déclenchent pas (préférences inconnues) ;
 *  - l'IPC `set_app_preferences` échoue avec `tauri_unavailable` à l'étape
 *    « Terminer » — c'est ce qu'on vérifie pour le contrat no-mock.
 *
 * Les e2e métier avec IPC réelle (persona persisté, rail filtré, switch
 * de bundle) tournent dans la suite `cargo tauri dev` (chantier dédié,
 * cf. C09.5 / C10.5).
 */

// ─── Test 1 — Splash → intro → persona picker → bundle Étudiant pré-coché ──────
test('Onboarding : splash → intro → persona picker → bundle Étudiant pré-coché à 5 modules', async ({
  page
}) => {
  await page.goto('/onboarding');

  // Étape 1 — Splash
  await expect(page).toHaveTitle(/Bienvenue/);
  await expect(page.getByText(/Mesurez la sobriété de votre IA générative/i)).toBeVisible();
  const continueBtn = page.getByRole('button', { name: /^Continuer/ });
  await expect(continueBtn).toBeVisible();
  await continueBtn.click();

  // C32.2 — Étape 2 « Sobr.ia en 30 secondes » avant le persona picker.
  await expect(page.getByRole('heading', { name: /Sobr\.ia en 30 secondes/ })).toBeVisible();
  await page.locator('[data-action="continue-intro"]').click();

  // Étape 3 — Persona picker (5 cartes)
  await expect(page.getByRole('heading', { name: /Vous êtes/ })).toBeVisible();
  for (const p of ['student', 'pro_tech', 'enterprise', 'public_sector', 'researcher']) {
    await expect(page.locator(`[data-persona="${p}"]`)).toBeVisible();
  }

  // Sélectionner Étudiant·e
  await page.locator('[data-persona="student"]').click();

  // Étape 4 — Bundle pré-coché : 5 modules pour le persona Student
  // (mirror sobria_core::Persona::Student::default_modules — voir
  // crates/sobria-core/src/preferences.rs ; C32.1 retrait M14).
  await expect(page.getByRole('heading', { name: /Voici votre première sélection/ })).toBeVisible();

  const expectedStudent = ['m1', 'm8', 'm13', 'm15', 'm25'];
  // On vérifie que chaque module du bundle est représenté ET coché.
  for (const m of expectedStudent) {
    const row = page
      .locator(`label[data-checked]`)
      .filter({ has: page.locator(`input[data-module="${m}"]`) });
    await expect(row).toHaveAttribute('data-checked', 'true');
  }
  await expect(page.getByText(/^5 modules sélectionnés/)).toBeVisible();
});

// ─── Test 2 — Étape Bundle « + Plus de modules » révèle les autres ────────
test('Onboarding : « + Plus de modules » dévoile les modules hors bundle', async ({ page }) => {
  await page.goto('/onboarding');

  // Skip splash → intro (C32.2)
  await page.getByRole('button', { name: /^Continuer/ }).click();
  // C32.2 — Passer l'étape « 30 secondes » via le bouton dédié.
  await page.locator('[data-action="skip-intro"]').click();
  // Choix « à la carte »
  await page.getByRole('button', { name: /choisir à la carte/i }).click();

  // À la carte : aucun module pré-coché — le toggle « plus de modules »
  // n'est PAS visible (on a déjà la liste complète en haut). On vérifie
  // donc que tous les modules sont là, non cochés.
  await expect(page.getByText(/0 module sélectionné/)).toBeVisible();
  const m22 = page.locator('input[data-module="m22"]');
  await expect(m22).toHaveCount(1);

  // Retour étape 2 puis Entreprise → la collapsable doit cette fois exister.
  await page.getByRole('button', { name: /^Précédent$/ }).click();
  await page.locator('[data-persona="enterprise"]').click();

  // Bundle Entreprise (8 modules — C32.1 retrait M14, aligné sur Rust).
  await expect(page.getByText(/^8 modules sélectionnés/)).toBeVisible();
  // M22 (CSRD) est dans le bundle Entreprise — coché.
  await expect(
    page.locator('label[data-checked="true"]', { has: page.locator('input[data-module="m22"]') })
  ).toHaveCount(1);

  // « + Plus de modules disponibles » présent (24 - 8 = 16 restants).
  const moreToggle = page.getByRole('button', { name: /Plus de modules/i });
  await expect(moreToggle).toBeVisible();
  await moreToggle.click();
  // Après dépliage, M14 (À propos, retiré de TOUS les bundles en C32.1) est
  // visible non coché.
  await expect(
    page.locator('label[data-checked="false"]', { has: page.locator('input[data-module="m14"]') })
  ).toHaveCount(1);
});

// ─── Test 3 — Tentative « Terminer » hors Tauri → erreur claire ────────
test("Onboarding : « Terminer » hors Tauri affiche l'erreur tauri_unavailable", async ({
  page
}) => {
  await page.goto('/onboarding');

  // Splash → Intro (C32.2) → Persona
  await page.getByRole('button', { name: /^Continuer/ }).click();
  await page.locator('[data-action="continue-intro"]').click();
  await page.locator('[data-persona="student"]').click();
  // Étape Bundle → Ready
  await page.getByRole('button', { name: /^Continuer/ }).click();
  await expect(page.getByRole('heading', { name: /C'est parti/ })).toBeVisible();

  // Clic « Terminer » sans contexte Tauri → bannière d'erreur affichée,
  // pas de redirection. Le contrat no-mock est respecté : aucune
  // persistance silencieuse, l'utilisateur sait pourquoi.
  await page.locator('[data-action="finish"]').click();
  const banner = page.getByRole('alert');
  await expect(banner).toBeVisible({ timeout: 5000 });
  await expect(banner).toContainText(/Échec de l.enregistrement|cargo run -p sobria-app|Tauri/i);

  // L'URL reste sur /onboarding (pas de goto('/') prématuré).
  await expect(page).toHaveURL(/\/onboarding/);
});

// ─── Test 4 — Garde de route M13 + bandeau /?disabled=m13 ──────────────
test('Garde de route : /?disabled=m13 affiche le bandeau « activer dans Paramètres »', async ({
  page
}) => {
  // On simule directement ce que ferait la garde de /simuler quand
  // l'utilisateur n'a pas M13 dans son bundle. Hors Tauri, la garde elle-
  // même ne se déclenche pas (preferences.loaded=false), mais l'URL
  // posée doit bien afficher le bandeau coral.
  await page.goto('/?disabled=m13');

  const banner = page.locator('[data-disabled-module="m13"]').first();
  await expect(banner).toBeVisible();
  await expect(banner).toContainText(/M13/);
  await expect(banner).toContainText(/Simulateur/);
  await expect(banner.getByRole('link', { name: /Activer dans Paramètres/ })).toBeVisible();
});
