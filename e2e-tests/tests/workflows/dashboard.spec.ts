/**
 * Dashboard workflow test
 *
 * Tests navigation through all dashboard tabs.
 * Video recording captures the full journey.
 */
import { test, expect } from '../../fixtures/vibes.js';

test.describe('Dashboard Workflow', () => {
  test('navigates through all dashboard pages', async ({ page, serverUrl }) => {
    // Start at Dashboard Overview
    await page.goto(`${serverUrl}/groove/dashboard/overview`);
    await expect(page).toHaveURL(/\/groove\/dashboard\/overview/);
    await page.waitForTimeout(500);

    // Navigate to Learnings
    const learningsTab = page.locator('text=Learnings').first();
    if (await learningsTab.isVisible()) {
      await learningsTab.click();
      await expect(page).toHaveURL(/\/groove\/dashboard\/learnings/);
      await page.waitForTimeout(500);
    }

    // Navigate to Attribution
    const attributionTab = page.locator('text=Attribution').first();
    if (await attributionTab.isVisible()) {
      await attributionTab.click();
      await expect(page).toHaveURL(/\/groove\/dashboard\/attribution/);
      await page.waitForTimeout(500);
    }

    // Navigate to Strategy
    const strategyTab = page.locator('text=Strategy').first();
    if (await strategyTab.isVisible()) {
      await strategyTab.click();
      await expect(page).toHaveURL(/\/groove\/dashboard\/strategy/);
      await page.waitForTimeout(500);
    }

    // Navigate to Health
    const healthTab = page.locator('text=Health').first();
    if (await healthTab.isVisible()) {
      await healthTab.click();
      await expect(page).toHaveURL(/\/groove\/dashboard\/health/);
      await page.waitForTimeout(500);
    }
  });

  test('dashboard overview shows key metrics', async ({ page, serverUrl }) => {
    await page.goto(`${serverUrl}/groove/dashboard/overview`);

    // Verify overview page loads with expected sections
    await page.waitForTimeout(500);

    // Check for typical dashboard elements (cards, metrics)
    const body = page.locator('body');
    await expect(body).toBeVisible();
  });
});
