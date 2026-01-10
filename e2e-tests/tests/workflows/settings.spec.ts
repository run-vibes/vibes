/**
 * Settings workflow test
 *
 * Tests settings page interactions.
 * Video recording captures toggle behaviors.
 */
import { test, expect } from '../../fixtures/vibes.js';

test.describe('Settings Workflow', () => {
  test('settings page loads all sections', async ({ page, serverUrl }) => {
    await page.goto(`${serverUrl}/settings`);
    await expect(page).toHaveURL(/\/settings/);

    // Verify key sections are visible
    await expect(page.locator('text=APPEARANCE')).toBeVisible();
    await expect(page.locator('text=GROOVE')).toBeVisible();
    await page.waitForTimeout(300);
  });

  test('toggles theme setting', async ({ page, serverUrl }) => {
    await page.goto(`${serverUrl}/settings`);
    await page.waitForTimeout(300);

    // Find theme toggle buttons
    const lightButton = page.locator('button:has-text("Light")').first();
    const darkButton = page.locator('button:has-text("Dark")').first();

    // Click Light theme
    if (await lightButton.isVisible()) {
      await lightButton.click();
      await page.waitForTimeout(500);
    }

    // Click Dark theme back
    if (await darkButton.isVisible()) {
      await darkButton.click();
      await page.waitForTimeout(500);
    }
  });

  test('toggles CRT effects', async ({ page, serverUrl }) => {
    await page.goto(`${serverUrl}/settings`);
    await page.waitForTimeout(300);

    // Find CRT effects toggle in the APPEARANCE section
    const crtSection = page.locator('text=CRT Effects').first();
    if (await crtSection.isVisible()) {
      // Find the On/Off buttons near the CRT Effects label
      const onButton = page.locator('button:has-text("On")').first();
      const offButton = page.locator('button:has-text("Off")').first();

      if (await offButton.isVisible()) {
        await offButton.click();
        await page.waitForTimeout(300);
      }

      if (await onButton.isVisible()) {
        await onButton.click();
        await page.waitForTimeout(300);
      }
    }
  });

  test('toggles learning indicator setting', async ({ page, serverUrl }) => {
    await page.goto(`${serverUrl}/settings`);
    await page.waitForTimeout(300);

    // Scroll to GROOVE section
    await page.locator('text=Learning Indicator').scrollIntoViewIfNeeded();
    await page.waitForTimeout(300);

    // Find the learning indicator toggle
    // The buttons are in a row after "Learning Indicator"
    const grooveSection = page.locator('.settings-panel:has-text("GROOVE")');

    if (await grooveSection.isVisible()) {
      // Toggle on
      const onButtons = grooveSection.locator('button:has-text("On")');
      if (await onButtons.first().isVisible()) {
        await onButtons.first().click();
        await page.waitForTimeout(300);
      }

      // Toggle off
      const offButtons = grooveSection.locator('button:has-text("Off")');
      if (await offButtons.first().isVisible()) {
        await offButtons.first().click();
        await page.waitForTimeout(300);
      }
    }
  });
});
