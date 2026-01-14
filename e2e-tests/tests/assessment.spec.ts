import { test, expect } from '../fixtures/vibes.js';

test('status page loads with dashboard cards', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/status`);

  // Status page shows dashboard overview with cards
  // Wait for any card to be visible (Learnings, Attribution, Health, Strategy)
  await expect(page.getByText('Learnings').first()).toBeVisible({ timeout: 10000 });
});

test('status page has groove navigation tabs', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/status`);

  // Should have nav tabs in subnav (new flat structure)
  await expect(page.getByRole('link', { name: 'Status' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'Learnings' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'Security' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'Stream' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'Strategy' })).toBeVisible();
});

test('status tab is active on status page', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/status`);

  const statusTab = page.getByRole('link', { name: 'Status' });
  await expect(statusTab).toHaveClass(/active/);
});

test('security tab navigates to security page', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/status`);

  const securityTab = page.getByRole('link', { name: 'Security' });
  await securityTab.click();

  // Should navigate to /groove/security
  await expect(page).toHaveURL(`${serverUrl}/groove/security`);
});

test('assessment shows connection status', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/stream`);

  // Should show Connected badge when WebSocket connects
  await expect(page.getByText('Connected')).toBeVisible({ timeout: 5000 });
});

test('assessment has tier filter chips', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/stream`);

  // Should have filter chips for assessment tiers
  await expect(page.getByRole('button', { name: 'lightweight' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'medium' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'heavy' })).toBeVisible();
});

test('assessment has search input', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/stream`);

  await expect(page.getByPlaceholder('Search events...')).toBeVisible();
});

test('tier filter chips toggle active state', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/stream`);

  const lightweightChip = page.getByRole('button', { name: 'lightweight' });

  // Initially not active
  await expect(lightweightChip).not.toHaveClass(/active/);

  // Click to activate
  await lightweightChip.click();
  await expect(lightweightChip).toHaveClass(/active/);

  // Click again to deactivate
  await lightweightChip.click();
  await expect(lightweightChip).not.toHaveClass(/active/);
});

test('assessment shows event stream panel', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/stream`);

  // Stream panel should be visible with title
  await expect(page.getByText('Assessment Events')).toBeVisible();
});

test('search input accepts text', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/stream`);

  const searchInput = page.getByPlaceholder('Search events...');
  await searchInput.fill('test query');

  await expect(searchInput).toHaveValue('test query');
});

test('multiple tier filter chips can be active simultaneously', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/stream`);

  const lightweightChip = page.getByRole('button', { name: 'lightweight' });
  const mediumChip = page.getByRole('button', { name: 'medium' });

  // Activate both
  await lightweightChip.click();
  await mediumChip.click();

  await expect(lightweightChip).toHaveClass(/active/);
  await expect(mediumChip).toHaveClass(/active/);
});

test('assessment shows LIVE indicator when connected', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/stream`);

  // Wait for connection
  await expect(page.getByText('Connected')).toBeVisible({ timeout: 5000 });

  // Should show LIVE in the stream view
  await expect(page.getByText('LIVE')).toBeVisible();
});

test('jump to latest button is hidden when following', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/stream`);

  // Wait for connection
  await expect(page.getByText('Connected')).toBeVisible({ timeout: 5000 });

  // Initially, Jump to latest should not be visible (we're following)
  await expect(page.getByRole('button', { name: /Jump to latest/i })).not.toBeVisible();
});

test('groove navigation works from security to status', async ({ page, serverUrl }) => {
  // Start on security page
  await page.goto(`${serverUrl}/groove/security`);

  // Click Status tab
  const statusTab = page.getByRole('link', { name: 'Status' });
  await statusTab.click();

  // Should navigate to status page (dashboard overview)
  await expect(page).toHaveURL(`${serverUrl}/groove/status`);
  await expect(page.getByText('Learnings').first()).toBeVisible();
});
