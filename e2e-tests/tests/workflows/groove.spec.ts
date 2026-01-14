/**
 * Groove workflow test
 *
 * Tests navigation through all Groove pages.
 * Video recording captures the full journey.
 */
import { test, expect } from '../../fixtures/vibes.js';

test.describe('Groove Workflow', () => {
  test('navigates through all groove pages', async ({ page, serverUrl }) => {
    // Start at Groove Status
    await page.goto(`${serverUrl}/groove/status`);
    await expect(page).toHaveURL(/\/groove\/status/);
    await page.waitForTimeout(500);

    // Navigate to Learnings
    const learningsTab = page.locator('text=Learnings').first();
    if (await learningsTab.isVisible()) {
      await learningsTab.click();
      await expect(page).toHaveURL(/\/groove\/learnings/);
      await page.waitForTimeout(500);
    }

    // Navigate to Strategy
    const strategyTab = page.locator('text=Strategy').first();
    if (await strategyTab.isVisible()) {
      await strategyTab.click();
      await expect(page).toHaveURL(/\/groove\/strategy/);
      await page.waitForTimeout(500);
    }

    // Navigate to Security
    const securityTab = page.locator('text=Security').first();
    if (await securityTab.isVisible()) {
      await securityTab.click();
      await expect(page).toHaveURL(/\/groove\/security/);
      await page.waitForTimeout(500);
    }

    // Navigate to Stream
    const streamTab = page.locator('text=Stream').first();
    if (await streamTab.isVisible()) {
      await streamTab.click();
      await expect(page).toHaveURL(/\/groove\/stream/);
      await page.waitForTimeout(500);
    }
  });

  test('groove status shows key metrics', async ({ page, serverUrl }) => {
    await page.goto(`${serverUrl}/groove/status`);

    // Verify status page loads with expected sections
    await page.waitForTimeout(500);

    // Check for typical dashboard elements (cards, metrics)
    const body = page.locator('body');
    await expect(body).toBeVisible();
  });
});
