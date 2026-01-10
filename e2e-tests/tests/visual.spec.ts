/**
 * Visual regression tests for key pages
 *
 * These tests capture screenshots and compare them against baselines
 * to catch unintended UI changes.
 */
import { test, expect } from '../fixtures/vibes.js';

test.describe('Visual Regression', () => {
  test.describe('Dashboard Pages', () => {
    test('Dashboard Overview', async ({ page, serverUrl }) => {
      await page.goto(`${serverUrl}/groove/dashboard/overview`);
      // Wait for content to load
      await page.waitForSelector('.dashboard-overview', { timeout: 10000 }).catch(() => {
        // Fallback if specific class doesn't exist
      });
      await page.waitForTimeout(500); // Allow animations to settle
      await expect(page).toHaveScreenshot('dashboard-overview.png');
    });

    test('Dashboard Strategy', async ({ page, serverUrl }) => {
      await page.goto(`${serverUrl}/groove/dashboard/strategy`);
      await page.waitForTimeout(500);
      await expect(page).toHaveScreenshot('dashboard-strategy.png');
    });

    test('Dashboard Health', async ({ page, serverUrl }) => {
      await page.goto(`${serverUrl}/groove/dashboard/health`);
      await page.waitForTimeout(500);
      await expect(page).toHaveScreenshot('dashboard-health.png');
    });

    test('Dashboard Learnings', async ({ page, serverUrl }) => {
      await page.goto(`${serverUrl}/groove/dashboard/learnings`);
      await page.waitForTimeout(500);
      await expect(page).toHaveScreenshot('dashboard-learnings.png');
    });

    test('Dashboard Attribution', async ({ page, serverUrl }) => {
      await page.goto(`${serverUrl}/groove/dashboard/attribution`);
      await page.waitForTimeout(500);
      await expect(page).toHaveScreenshot('dashboard-attribution.png');
    });
  });

  test.describe('Main Pages', () => {
    test('Sessions Page', async ({ page, serverUrl }) => {
      await page.goto(`${serverUrl}/sessions`);
      await page.waitForTimeout(500);
      await expect(page).toHaveScreenshot('sessions.png');
    });

    test('Firehose Page', async ({ page, serverUrl }) => {
      await page.goto(`${serverUrl}/firehose`);
      await page.waitForTimeout(500);
      await expect(page).toHaveScreenshot('firehose.png');
    });

    test('Settings Page', async ({ page, serverUrl }) => {
      await page.goto(`${serverUrl}/settings`);
      await page.waitForTimeout(500);
      await expect(page).toHaveScreenshot('settings.png');
    });
  });

  test.describe('Assessment Pages', () => {
    test('Assessment Status', async ({ page, serverUrl }) => {
      await page.goto(`${serverUrl}/groove/assessment/status`);
      await page.waitForTimeout(500);
      await expect(page).toHaveScreenshot('assessment-status.png');
    });

    test('Assessment History', async ({ page, serverUrl }) => {
      await page.goto(`${serverUrl}/groove/assessment/history`);
      await page.waitForTimeout(500);
      await expect(page).toHaveScreenshot('assessment-history.png');
    });
  });
});
