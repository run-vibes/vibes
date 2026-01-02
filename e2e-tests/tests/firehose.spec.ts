import { test, expect } from '../fixtures/vibes.js';

test('firehose page loads with heading', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/firehose`);

  await expect(page.getByRole('heading', { name: 'Firehose' })).toBeVisible();
});

test('firehose shows connection status', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/firehose`);

  // Should show Connected badge when WebSocket connects
  await expect(page.getByText('Connected')).toBeVisible({ timeout: 5000 });
});

test('firehose has event type filter chips', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/firehose`);

  // Should have filter chips for event types
  await expect(page.getByRole('button', { name: 'SESSION' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'CLAUDE' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'TOOL' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'HOOK' })).toBeVisible();
});

test('firehose has session filter input', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/firehose`);

  await expect(page.getByPlaceholder('Filter by session...')).toBeVisible();
});

test('filter chips toggle active state', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/firehose`);

  const sessionChip = page.getByRole('button', { name: 'SESSION' });

  // Initially not active
  await expect(sessionChip).not.toHaveClass(/active/);

  // Click to activate
  await sessionChip.click();
  await expect(sessionChip).toHaveClass(/active/);

  // Click again to deactivate
  await sessionChip.click();
  await expect(sessionChip).not.toHaveClass(/active/);
});

test('firehose shows event stream panel', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/firehose`);

  // Stream panel should be visible with title
  await expect(page.getByText('Event Stream')).toBeVisible();
});
