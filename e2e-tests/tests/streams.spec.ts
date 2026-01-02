import { test, expect } from '../fixtures/vibes.js';

test('homepage loads with Streams heading', async ({ page, serverUrl }) => {
  await page.goto(serverUrl);

  await expect(page.getByRole('heading', { name: 'Streams' })).toBeVisible();
});

test('homepage shows status badges', async ({ page, serverUrl }) => {
  await page.goto(serverUrl);

  // Status bar items should be visible
  await expect(page.locator('.status-item').filter({ hasText: 'Server' })).toBeVisible();
  await expect(page.locator('.status-item').filter({ hasText: 'Firehose' })).toBeVisible();
  await expect(page.locator('.status-item').filter({ hasText: 'Tunnel' })).toBeVisible();
});

test('homepage has navigation cards', async ({ page, serverUrl }) => {
  await page.goto(serverUrl);

  // Should have cards linking to subpages
  await expect(page.getByRole('link', { name: /Firehose/i })).toBeVisible();
  await expect(page.getByRole('link', { name: /Debug Console/i })).toBeVisible();
  await expect(page.getByRole('link', { name: /Sessions/i })).toBeVisible();
});

test('firehose card links to firehose page', async ({ page, serverUrl }) => {
  await page.goto(serverUrl);

  await page.getByRole('link', { name: /Firehose/i }).click();

  await expect(page).toHaveURL(/\/firehose/);
});

test('sessions card links to sessions page', async ({ page, serverUrl }) => {
  await page.goto(serverUrl);

  await page.getByRole('link', { name: /Sessions/i }).click();

  await expect(page).toHaveURL(/\/sessions/);
});
