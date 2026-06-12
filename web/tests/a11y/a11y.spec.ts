// Audit axe-core des pages clés en mode démo (job CI « a11y »).
//
// Périmètre v1 : violations `critical` + `serious` uniquement — le bruit
// `moderate`/`minor` (contrastes décoratifs, landmarks dupliqués…) se
// traite au fil de l'eau, pas en gate bloquant. Étendre la sévérité quand
// le socle est stable.
import { test, expect } from '@playwright/test';
import { injectAxe, getViolations } from 'axe-playwright';

const PAGES = [
  { path: '/', name: 'accueil' },
  { path: '/modeles', name: 'catalogue modèles' },
  { path: '/comparer', name: 'comparateur' },
  { path: '/parametres', name: 'paramètres' },
  { path: '/methodologies', name: 'méthodologies' }
];

for (const { path, name } of PAGES) {
  test(`axe — ${name} (${path}) sans violation critical/serious`, async ({ page }) => {
    await page.goto(path);
    await page.waitForLoadState('networkidle');
    await injectAxe(page);
    const violations = await getViolations(page, undefined, {
      runOnly: { type: 'tag', values: ['wcag2a', 'wcag2aa'] }
    });
    const blocking = violations.filter((v) => v.impact === 'critical' || v.impact === 'serious');
    const resume = blocking
      .map((v) => `[${v.impact}] ${v.id} : ${v.help} (${v.nodes.length} nœud·s)`)
      .join('\n');
    expect(blocking, `Violations axe sur ${path} :\n${resume}`).toEqual([]);
  });
}
