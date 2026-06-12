import { expect, test } from '@playwright/test';

/**
 * Contrat « démo » du DatacenterPicker (C25 A14 / C36).
 *
 * Le picker est un combobox accessible (role="combobox", trigger
 * aria-label="Choisir un datacenter", listbox id="dc-picker-listbox")
 * monté à deux endroits :
 *   • Estimer (`/`) — dans le Composer. Depuis C36, le composer se rend
 *     aussi hors Tauri (modèles servis par la démo) : le picker y est
 *     donc PRÉSENT en vite-only.
 *   • Simuler (`/simuler`) — rendu inconditionnellement ; le catalogue
 *     `list_datacenters` est couvert par la démo, le listbox expose donc
 *     les 28 datacenters des fixtures en plus du sentinel « Aucun choisi ».
 *
 * Ces tests vérouillent :
 *   1. Le picker est rendu (fermé) sur `/` dans le Composer démo.
 *   2. Le trigger « Choisir un datacenter » est visible sur `/simuler`.
 *   3. Le trigger affiche « Aucun choisi » par défaut (aucune sélection).
 *   4. Au clic, le panel listbox s'ouvre et expose un champ recherche.
 *   5. Le sentinel « Aucun choisi » est rendu comme option role="option".
 *   6. Le catalogue démo peuple le listbox (options datacenter + groupes pays).
 *   7. La touche Escape referme le panel.
 *
 * Le flux end-to-end avec persistance (pick → estimate → refresh → picker
 * pré-rempli) reste Tauri-only : `set_app_preferences` démo n'est
 * persistée qu'en mémoire (perdue au reload) — voir test.skip en bas.
 */

test.describe('C25 DatacenterPicker', () => {
  test('Estimer (/) : picker présent dans le Composer démo, fermé par défaut', async ({ page }) => {
    await page.goto('/');

    // Sanity check du contexte : bannière démo, composer monté, pas d'erreur.
    await expect(page.locator('aside.demo-banner')).toBeVisible();
    await expect(page.locator('form.composer')).toBeVisible();
    await expect(page.getByRole('alert')).toHaveCount(0);

    // Le picker est rendu dans le Composer (catalogue démo), panel fermé.
    await expect(page.getByRole('button', { name: 'Choisir un datacenter' })).toBeVisible();
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

  test('Simuler (/simuler) : le catalogue démo peuple le listbox', async ({ page }) => {
    await page.goto('/simuler');

    await page.getByRole('button', { name: 'Choisir un datacenter' }).click();
    const listbox = page.locator('#dc-picker-listbox');
    await expect(listbox).toBeVisible();

    // C36 : `list_datacenters` est couvert par la démo — le listbox expose
    // les datacenters des fixtures en plus du sentinel « Aucun choisi ».
    const options = listbox.getByRole('option');
    expect(await options.count()).toBeGreaterThan(1);
    await expect(options.first()).toContainText(/Aucun choisi/);

    // Les groupes pays (role="separator") sont rendus pour le catalogue.
    expect(await listbox.getByRole('separator').count()).toBeGreaterThan(0);
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
   * La persistance des préférences nécessite le runtime Tauri : en démo,
   * `set_app_preferences` n'écrit qu'en mémoire (état perdu au reload). Ce
   * flux est exécuté dans la suite Tauri dédiée (chantier CI C09.5).
   */
  test.skip('Tauri only — open, search, pick, estimate, refresh, picker pre-filled', async () => {
    // Implémenté dans la suite Tauri (cargo tauri dev) — voir brief C25 A14.
  });
});
