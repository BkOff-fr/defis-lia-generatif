import { expect, test } from '@playwright/test';

/**
 * C25/C36 — Layout immersif de l'écran Datacenters Europe (M12).
 *
 * Le chantier C25 a transformé `/datacenters` en mise en page immersive :
 *   - `.dc-route`           → conteneur de route `position: relative; overflow: hidden`
 *                            (gated sur `backendAvailable && datacenters.length > 0`).
 *   - `.dc-map-fill`        → wrapper carte `position: absolute; inset: 0`.
 *   - `.dc-filters-overlay` → overlay filtres flottant top-left `position: absolute`.
 *   - `.dc-drill-overlay`   → overlay drill-down monté UNIQUEMENT si selectedDc
 *                            ou selectedCountry est défini (cf. C25 B3, plus de
 *                            placeholder « Cliquez un marker »).
 *
 * Depuis C36, le mode démo web sert `list_datacenters` /
 * `aggregate_datacenters_by_country` hors Tauri : le layout immersif est
 * donc monté aussi en vite-only et son contrat CSS est vérifiable ici.
 *
 * Cette suite garantit :
 *   1. Bannière « Mode démo » visible, aucune bannière d'erreur, pas
 *      d'empty-shell.
 *   2. Le layout immersif est monté : `.dc-route`, `.dc-map-fill`,
 *      `.dc-filters-overlay`, carte Leaflet.
 *   3. Le contrat CSS C25 B1 est respecté (positions calculées).
 *   4. Régression B3 : le drill-overlay n'est jamais monté tant
 *      qu'aucune sélection n'a eu lieu.
 */

test.describe('C25 /datacenters immersive layout', () => {
  test('mode démo : layout immersif monté, drill-overlay absent sans sélection', async ({
    page
  }) => {
    await page.goto('/datacenters');

    // 1. Coque rendue + mode démo sans erreur ni empty-shell.
    await expect(
      page.getByRole('heading', { name: /Où tournent.*physiquement.*vos prompts/i })
    ).toBeVisible();
    await expect(page.locator('aside.demo-banner')).toBeVisible();
    await expect(page.locator('.dc-route')).toBeVisible();
    await expect(page.getByRole('alert')).toHaveCount(0);
    await expect(page.getByRole('heading', { name: /Carte indisponible/i })).toHaveCount(0);

    // 2. Layout immersif monté avec la carte Leaflet.
    await expect(page.locator('.dc-map-fill')).toBeVisible();
    await expect(page.locator('.dc-filters-overlay')).toBeVisible();
    await expect(page.locator('.dc-map-fill .leaflet-container')).toBeVisible();

    // 3. Contrat CSS C25 B1 (computed styles).
    expect(
      await page.locator('.dc-route').evaluate((el) => {
        const s = getComputedStyle(el);
        return { position: s.position, overflow: s.overflow };
      })
    ).toEqual({ position: 'relative', overflow: 'hidden' });
    expect(await page.locator('.dc-map-fill').evaluate((el) => getComputedStyle(el).position)).toBe(
      'absolute'
    );
    expect(
      await page.locator('.dc-filters-overlay').evaluate((el) => getComputedStyle(el).position)
    ).toBe('absolute');

    // 4. Régression C25 B3 : tant qu'aucune sélection n'a eu lieu, le
    //    drill-overlay n'est JAMAIS monté (plus de placeholder vide).
    await expect(page.locator('.dc-drill-overlay')).toHaveCount(0);
  });

  // Drill-down complet (clic marker → fiche détaillée, clic pays →
  // CountryDrillDown) : la fiche datacenter dépend de
  // `get_datacenter_detail` (profil 24 h), volontairement NON couverte par
  // la démo (`desktopOnly`). L'interaction est donc vérifiée dans la suite
  // Tauri dédiée (C09.5) avec le runtime réel.
  test.skip('tauri-only : drill-down marker/pays alimenté par get_datacenter_detail', async () => {
    // Voir briefs/chantiers/C25-*.md pour le plan complet.
  });
});
