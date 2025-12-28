import { test, expect } from '../fixtures/vibes.js';

test('server starts and serves web UI', async ({ page, serverUrl }) => {
  await page.goto(serverUrl);

  // The page should load without error
  await expect(page).toHaveTitle(/.*/);

  // Body should be visible
  await expect(page.locator('body')).toBeVisible();
});

test('server health endpoint returns ok', async ({ serverUrl }) => {
  const response = await fetch(`${serverUrl}/health`);
  expect(response.ok).toBe(true);
});
