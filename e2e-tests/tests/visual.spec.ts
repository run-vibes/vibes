/**
 * Visual regression tests for key pages
 *
 * These tests capture screenshots and compare them against baselines
 * to catch unintended UI changes.
 */
import { test, expect } from '../fixtures/vibes.js';

test.describe('Visual Regression', () => {
  test.describe('Groove Pages', () => {
    test('Status (Overview)', async ({ page, serverUrl }) => {
      await page.goto(`${serverUrl}/groove/status`);
      // Wait for content to load
      await page.waitForSelector('.dashboard-page', { timeout: 10000 }).catch(() => {
        // Fallback if specific class doesn't exist
      });
      await page.waitForTimeout(500); // Allow animations to settle
      await expect(page).toHaveScreenshot('groove-status.png');
    });

    test('Strategy', async ({ page, serverUrl }) => {
      await page.goto(`${serverUrl}/groove/strategy`);
      await page.waitForTimeout(500);
      await expect(page).toHaveScreenshot('groove-strategy.png');
    });

    test('Learnings', async ({ page, serverUrl }) => {
      await page.goto(`${serverUrl}/groove/learnings`);
      await page.waitForTimeout(500);
      await expect(page).toHaveScreenshot('groove-learnings.png');
    });

    test('Security', async ({ page, serverUrl }) => {
      await page.goto(`${serverUrl}/groove/security`);
      await page.waitForTimeout(500);
      await expect(page).toHaveScreenshot('groove-security.png');
    });

    test('Trends', async ({ page, serverUrl }) => {
      await page.goto(`${serverUrl}/groove/trends`);
      await page.waitForTimeout(500);
      await expect(page).toHaveScreenshot('groove-trends.png');
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

  test.describe('Groove Stream & History', () => {
    test('Stream', async ({ page, serverUrl }) => {
      await page.goto(`${serverUrl}/groove/stream`);
      await page.waitForTimeout(500);
      await expect(page).toHaveScreenshot('groove-stream.png');
    });

    test('History', async ({ page, serverUrl }) => {
      await page.goto(`${serverUrl}/groove/history`);
      await page.waitForTimeout(500);
      await expect(page).toHaveScreenshot('groove-history.png');
    });
  });
});
