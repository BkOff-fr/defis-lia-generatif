import { test, expect } from '@playwright/test';

// Skeleton — meaningful Lighthouse score check is wired in C33.7 with
// playwright-lighthouse or @lhci/cli. For now we just smoke-test the
// homepage responds 200 and contains the headline.
test('homepage responds 200 with expected content', async ({ page }) => {
  const response = await page.goto('/');
  expect(response?.status()).toBe(200);
  await expect(page.locator('h2').first()).toBeVisible();
});
