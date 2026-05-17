import { test, expect } from '@playwright/test';
import { injectAxe, checkA11y } from 'axe-playwright';

test('homepage has no critical a11y violations', async ({ page }) => {
  await page.goto('/');
  await injectAxe(page);
  await checkA11y(page, undefined, {
    detailedReport: true,
    detailedReportOptions: { html: false },
    axeOptions: {
      runOnly: { type: 'tag', values: ['wcag2a', 'wcag2aa'] },
    },
  });
  await expect(page).toHaveTitle(/Sobr\.ia/);
  await expect(page.getByRole('heading', { level: 1 })).toContainText(/empreinte/i);
});
