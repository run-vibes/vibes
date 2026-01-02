import { test, expect } from '../fixtures/vibes.js';

test('sessions page loads with heading', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/sessions`);

  await expect(page.getByRole('heading', { name: 'Sessions' })).toBeVisible();
});

test('sessions page shows empty state when no sessions', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/sessions`);

  // Should show empty state message
  await expect(page.getByText('No active sessions.')).toBeVisible();
});

test('sessions page has New Session button', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/sessions`);

  await expect(page.getByRole('button', { name: 'New Session' })).toBeVisible();
});

test('sessions page shows connection status', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/sessions`);

  // Connection status should be visible (connected, connecting, or disconnected)
  await expect(page.locator('.connection-status')).toBeVisible();
});
