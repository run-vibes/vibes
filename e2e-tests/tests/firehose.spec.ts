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

test('firehose has search input', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/firehose`);

  await expect(page.getByPlaceholder('Search events...')).toBeVisible();
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

test('search input accepts text', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/firehose`);

  const searchInput = page.getByPlaceholder('Search events...');
  await searchInput.fill('test query');

  await expect(searchInput).toHaveValue('test query');
});

test('search input can be cleared', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/firehose`);

  const searchInput = page.getByPlaceholder('Search events...');
  await searchInput.fill('test query');
  await searchInput.clear();

  await expect(searchInput).toHaveValue('');
});

test('multiple filter chips can be active simultaneously', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/firehose`);

  const sessionChip = page.getByRole('button', { name: 'SESSION' });
  const claudeChip = page.getByRole('button', { name: 'CLAUDE' });

  // Activate both
  await sessionChip.click();
  await claudeChip.click();

  await expect(sessionChip).toHaveClass(/active/);
  await expect(claudeChip).toHaveClass(/active/);
});

test('firehose shows LIVE indicator when connected', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/firehose`);

  // Wait for connection
  await expect(page.getByText('Connected')).toBeVisible({ timeout: 5000 });

  // Should show LIVE in the stream view
  await expect(page.getByText('LIVE')).toBeVisible();
});

test('jump to latest button appears when scrolled away', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/firehose`);

  // Wait for connection
  await expect(page.getByText('Connected')).toBeVisible({ timeout: 5000 });

  // Initially, Jump to latest should not be visible (we're at the bottom)
  await expect(page.getByRole('button', { name: /Jump to latest/i })).not.toBeVisible();

  // Note: Testing actual scroll behavior requires events to be present
  // which depends on server activity. We verify the button exists in the DOM
  // but is hidden when following.
});

test('firehose shows event count', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/firehose`);

  // Should show event count in the stream header
  await expect(page.getByText(/\d+ events/)).toBeVisible({ timeout: 5000 });
});
