/**
 * Visual regression tests for Card component
 *
 * Tests the Card component across its variants to catch
 * styling regressions in the design system.
 */
import { test, expect } from '../fixtures/vibes.js';

test.describe('Card Component Visual Regression', () => {
  test('Card Default Variant', async ({ page, serverUrl }) => {
    // Navigate to dashboard overview which uses Card with crt variant
    await page.goto(`${serverUrl}/groove/dashboard/overview`);
    await page.waitForTimeout(500);

    // Capture a card on the page (TrendCard uses Card variant="crt")
    const trendCard = page.locator('.trend-card').first();
    await expect(trendCard).toHaveScreenshot('card-crt-trend.png');
  });

  test('Card CRT Variant - Health', async ({ page, serverUrl }) => {
    await page.goto(`${serverUrl}/groove/dashboard/overview`);
    await page.waitForTimeout(500);

    // HealthCard uses Card variant="crt"
    const healthCard = page.locator('[data-testid="status-indicator"]').locator('..');
    if (await healthCard.count() > 0) {
      await expect(healthCard.first()).toHaveScreenshot('card-crt-health.png');
    }
  });

  test('Card CRT Variant - Learnings', async ({ page, serverUrl }) => {
    await page.goto(`${serverUrl}/groove/dashboard/overview`);
    await page.waitForTimeout(500);

    // Find a card with "Learnings" title
    const learningsCard = page.locator('.dashboard-card').filter({ hasText: 'Total' }).first();
    if (await learningsCard.count() > 0) {
      await expect(learningsCard).toHaveScreenshot('card-crt-learnings.png');
    }
  });

  test('Card Grid Layout', async ({ page, serverUrl }) => {
    await page.goto(`${serverUrl}/groove/dashboard/overview`);
    await page.waitForTimeout(500);

    // Capture the full overview grid to verify cards align consistently
    const overviewGrid = page.locator('.dashboard-overview');
    if (await overviewGrid.count() > 0) {
      await expect(overviewGrid).toHaveScreenshot('card-grid-layout.png');
    }
  });

  test('Dashboard OpenWorld Cards', async ({ page, serverUrl }) => {
    await page.goto(`${serverUrl}/groove/dashboard/open-world`);
    await page.waitForTimeout(500);

    // Capture the open world dashboard which has multiple Card variants
    await expect(page).toHaveScreenshot('card-openworld-dashboard.png');
  });
});
