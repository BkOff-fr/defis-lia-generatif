import { expect, test } from '@playwright/test';

/**
 * C25 — Régression layout immersif de l'écran Datacenters Europe (M12).
 *
 * Le chantier C25 a transformé `/datacenters` en mise en page immersive :
 *   - `.dc-route`           → conteneur de route `position: relative; overflow: hidden`
 *                            (gated sur `tauriAvailable && datacenters.length > 0`).
 *   - `.dc-map-fill`        → wrapper carte `position: absolute; inset: 0`.
 *   - `.dc-filters-overlay` → overlay filtres flottant top-left `position: absolute`.
 *   - `.dc-drill-overlay`   → overlay drill-down monté UNIQUEMENT si selectedDc
 *                            ou selectedCountry est défini (cf. C25 B3, plus de
 *                            placeholder « Cliquez un marker »).
 *
 * En mode vite-only (cette suite), le runtime Tauri est absent. La gating
 * `tauriAvailable && datacenters.length > 0` empêche `.dc-route` d'être
 * montée — c'est exactement le contrat « no-mock » (CLAUDE.md §13). Les
 * assertions structurelles sur le layout immersif vivent dans la suite
 * Tauri dédiée (C09.5).
 *
 * Cette suite garantit donc :
 *   1. La dégradation gracieuse est propre : bannière `tauri_unavailable`
 *      visible, empty-shell visible.
 *   2. Aucune trace du layout immersif n'apparaît hors Tauri (aucun
 *      `.dc-route`, `.dc-map-fill`, `.dc-filters-overlay`, `.dc-drill-overlay`).
 *   3. Le drill-overlay n'est jamais monté tant qu'aucune sélection n'a eu
 *      lieu (régression B3 — supprime le placeholder vide historique).
 *
 * Le bloc `test.skip` documente le contrat immersif réel (CSS computed)
 * qui sera vérifié dès que la suite Tauri sera branchée.
 */

test.describe('C25 /datacenters immersive layout', () => {
  test('vite-only : dégradation gracieuse, aucun layout immersif monté', async ({ page }) => {
    await page.goto('/datacenters');

    // 1. Coque rendue (hero h1 toujours visible quel que soit le contexte).
    await expect(
      page.getByRole('heading', { name: /Où tournent.*physiquement.*vos prompts/i })
    ).toBeVisible();

    // 2. Bannière tauri_unavailable (prouve que la route a bootstrappé et a
    //    refusé de mocker la carte).
    const banner = page.getByRole('alert');
    await expect(banner).toBeVisible();
    await expect(banner).toContainText(/Application non lancée via Tauri/);

    // 3. Empty-shell pédagogique visible (branche `!bootstrapping && (!tauri || !data)`).
    await expect(page.getByRole('heading', { name: /Carte indisponible/i })).toBeVisible();

    // 4. Aucun élément du layout immersif ne doit fuiter en vite-only.
    //    Toute la mise en page C25 B1-B3 est gated sur la présence d'un
    //    runtime Tauri + un dataset chargé.
    await expect(page.locator('.dc-route')).toHaveCount(0);
    await expect(page.locator('.dc-map-fill')).toHaveCount(0);
    await expect(page.locator('.dc-filters-overlay')).toHaveCount(0);

    // 5. Régression C25 B3 : tant qu'aucune sélection n'a eu lieu, le
    //    drill-overlay n'est JAMAIS monté (plus de placeholder vide).
    await expect(page.locator('.dc-drill-overlay')).toHaveCount(0);
  });

  // Contrat immersif complet — vérifiable uniquement avec un runtime Tauri
  // qui fournit les 28 datacenters via `list_datacenters`. À brancher sur la
  // suite C09.5 (cargo tauri dev) dès qu'elle existe.
  //
  // Garanties attendues côté Tauri :
  //   - `.dc-route` présent, `getComputedStyle().position === 'relative'`,
  //     `overflow === 'hidden'`.
  //   - `.dc-map-fill` présent, `position === 'absolute'`, `inset: 0`.
  //   - `.dc-filters-overlay` présent, `position === 'absolute'`,
  //     `top: 16px`, `left: 16px`, z-index élevé.
  //   - `.dc-drill-overlay` absent au mount (aucune sélection initiale).
  //   - Au clic sur un marker Leaflet : `.dc-drill-overlay` apparaît,
  //     contient `<DatacenterDrillDown>` (et un `aria-label` close).
  //   - Au clic sur une country layer : `.dc-drill-overlay` contient
  //     `<CountryDrillDown>`.
  //   - Carte Leaflet `.leaflet-container` rendue et remplit `.dc-map-fill`.
  test.skip('tauri-only : layout immersif respecte le contrat CSS C25', async () => {
    // Voir briefs/chantiers/C25-*.md pour le plan complet.
  });
});
