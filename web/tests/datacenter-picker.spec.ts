import { expect, test } from '@playwright/test';

/**
 * Contrat « no-mock » du DatacenterPicker (C25 A14).
 *
 * Le picker est un combobox accessible (role="combobox", trigger
 * aria-label="Choisir un datacenter", listbox id="dc-picker-listbox")
 * monté à deux endroits :
 *   • Estimer (`/`) — dans le Composer, gaté par `tauriAvailable && models.length > 0`.
 *     Donc côté Vite-only, il est ABSENT (contrat no-mock).
 *   • Simuler (`/simuler`) — rendu inconditionnellement avec un catalogue vide.
 *     Donc côté Vite-only, il est PRÉSENT mais sans aucune option de datacenter
 *     (seul le sentinel « Aucun choisi » apparaît dans le listbox).
 *
 * Ces tests vérouillent :
 *   1. Le picker n'est PAS rendu sur `/` hors Tauri (pas de mock dans Composer).
 *   2. Le trigger « Choisir un datacenter » est visible sur `/simuler`.
 *   3. Le trigger affiche « Aucun choisi » par défaut (aucune sélection).
 *   4. Au clic, le panel listbox s'ouvre et expose un champ recherche.
 *   5. Le sentinel « Aucun choisi » est rendu comme option role="option" dans le listbox.
 *   6. Avec un catalogue vide (Vite-only), aucun groupe pays / option de datacenter
 *      n'est rendu en plus du sentinel.
 *   7. La touche Escape referme le panel.
 *
 * Le flux complet end-to-end (open → recherche → pick → estimate → rafraîchir →
 * picker pré-rempli depuis le store de préférences) nécessite Tauri (catalogue
 * réel + IPC `estimate`) ; il est marqué `test.skip` ci-dessous et tournera
 * dans la suite Tauri dédiée (chantier CI C09.5).
 */

test.describe('C25 DatacenterPicker', () => {
  test('Estimer (/) : picker absent hors Tauri (Composer non rendu)', async ({ page }) => {
    await page.goto('/');

    // La bannière tauri_unavailable doit être présente — sanity check du contexte.
    // On attend d'abord la visibilité (auto-wait du bootstrap) avant le contenu.
    const banner = page.getByRole('alert');
    await expect(banner).toBeVisible();
    await expect(banner).toContainText(/Application non lancée via Tauri/);

    // Composer absent → DatacenterPicker absent (le trigger n'a aucun fallback).
    await expect(page.getByRole('button', { name: 'Choisir un datacenter' })).toHaveCount(0);
    await expect(page.locator('#dc-picker-listbox')).toHaveCount(0);
  });

  test('Simuler (/simuler) : trigger visible avec « Aucun choisi » par défaut', async ({
    page
  }) => {
    await page.goto('/simuler');

    const trigger = page.getByRole('button', { name: 'Choisir un datacenter' });
    await expect(trigger).toBeVisible();
    // Le trigger porte le label de section + l'état par défaut.
    await expect(trigger).toContainText(/Datacenter/);
    await expect(trigger).toContainText(/Aucun choisi/);
    await expect(trigger).toContainText(/L'estimation utilise vos PUE\/IF par défaut/);

    // Picker fermé : pas de listbox dans le DOM.
    await expect(page.locator('#dc-picker-listbox')).toHaveCount(0);
  });

  test('Simuler (/simuler) : clic ouvre le listbox + champ recherche', async ({ page }) => {
    await page.goto('/simuler');

    const trigger = page.getByRole('button', { name: 'Choisir un datacenter' });
    await trigger.click();

    // Le panel listbox est rendu (id stable, role="listbox").
    const listbox = page.locator('#dc-picker-listbox');
    await expect(listbox).toBeVisible();
    await expect(listbox).toHaveAttribute('role', 'listbox');

    // Le combobox parent reflète l'état ouvert. On le localise via son
    // `aria-controls` (id stable du listbox) pour ne pas tomber sur le
    // <select> natif du baseline modèle (qui a un rôle combobox implicite).
    const combobox = page.locator('[aria-controls="dc-picker-listbox"]');
    await expect(combobox).toHaveAttribute('aria-expanded', 'true');

    // Le champ recherche est focusable et porte le bon placeholder.
    const search = page.getByPlaceholder(/Rechercher.*nom.*ville.*opérateur.*pays/);
    await expect(search).toBeVisible();

    // Sentinel « Aucun choisi » présent dans la liste comme option a11y.
    const none = listbox.getByRole('option', { name: /Aucun choisi/ });
    await expect(none).toBeVisible();
    await expect(none).toContainText(/Utilise les paramètres par défaut/);
  });

  test('Simuler (/simuler) : catalogue vide hors Tauri → seul le sentinel est listé', async ({
    page
  }) => {
    await page.goto('/simuler');

    await page.getByRole('button', { name: 'Choisir un datacenter' }).click();
    const listbox = page.locator('#dc-picker-listbox');
    await expect(listbox).toBeVisible();

    // Le catalogue Tauri n'est pas joignable : aucune option "datacenter" réelle
    // ne doit apparaître. Seul le sentinel « Aucun choisi » est listé.
    const options = listbox.getByRole('option');
    await expect(options).toHaveCount(1);
    await expect(options.first()).toContainText(/Aucun choisi/);

    // Aucun groupe pays (role="separator") rendu non plus.
    await expect(listbox.getByRole('separator')).toHaveCount(0);
  });

  test('Simuler (/simuler) : Escape referme le panel', async ({ page }) => {
    await page.goto('/simuler');

    const trigger = page.getByRole('button', { name: 'Choisir un datacenter' });
    await trigger.click();
    await expect(page.locator('#dc-picker-listbox')).toBeVisible();

    await page.keyboard.press('Escape');
    await expect(page.locator('#dc-picker-listbox')).toHaveCount(0);
    await expect(page.locator('[aria-controls="dc-picker-listbox"]')).toHaveAttribute(
      'aria-expanded',
      'false'
    );
  });

  /**
   * Flux end-to-end complet : ouvrir le picker, rechercher, sélectionner un
   * datacenter du catalogue Gold, lancer une estimation, rafraîchir la page et
   * vérifier que la sélection est pré-remplie depuis le store de préférences.
   *
   * Ce flux dépend de l'IPC Tauri (`list_datacenters`, `estimate`,
   * `get_preferences`) et du catalogue réel — il est exécuté dans la suite
   * Tauri dédiée (chantier CI C09.5), pas dans la CI Vite no-mock.
   */
  test.skip('Tauri only — open, search, pick, estimate, refresh, picker pre-filled', async () => {
    // Implémenté dans la suite Tauri (cargo tauri dev) — voir brief C25 A14.
  });
});
