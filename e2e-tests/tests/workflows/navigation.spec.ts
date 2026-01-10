/**
 * Navigation workflow test
 *
 * Tests core navigation between main sections.
 * Video recording captures the full journey.
 */
import { test, expect } from '../../fixtures/vibes.js';

test.describe('Navigation Workflow', () => {
  test('navigates through main sections', async ({ page, serverUrl }) => {
    // Start at home
    await page.goto(serverUrl);
    await expect(page.locator('body')).toBeVisible();

    // Navigate to Sessions
    await page.click('text=SESSIONS');
    await expect(page).toHaveURL(/\/sessions/);
    await page.waitForTimeout(300);

    // Navigate to Firehose
    await page.click('text=FIREHOSE');
    await expect(page).toHaveURL(/\/firehose/);
    await page.waitForTimeout(300);

    // Navigate to Groove
    await page.click('text=GROOVE');
    await expect(page).toHaveURL(/\/groove/);
    await page.waitForTimeout(300);
  });

  test('navigates groove subnav sections', async ({ page, serverUrl }) => {
    // Go to Groove first
    await page.goto(`${serverUrl}/groove`);
    await page.waitForTimeout(300);

    // Click Security (default groove page)
    const securityLink = page.locator('text=Security').first();
    if (await securityLink.isVisible()) {
      await securityLink.click();
      await page.waitForTimeout(300);
    }

    // Click Assessment
    const assessmentLink = page.locator('text=Assessment').first();
    if (await assessmentLink.isVisible()) {
      await assessmentLink.click();
      await expect(page).toHaveURL(/\/groove\/assessment/);
      await page.waitForTimeout(300);
    }

    // Click Dashboard
    const dashboardLink = page.locator('text=Dashboard').first();
    if (await dashboardLink.isVisible()) {
      await dashboardLink.click();
      await expect(page).toHaveURL(/\/groove\/dashboard/);
      await page.waitForTimeout(300);
    }
  });
});
