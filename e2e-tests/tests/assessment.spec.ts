import { test, expect } from '../fixtures/vibes.js';

test('assessment status page loads with content', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/assessment/status`);

  // Status page shows either STATUS heading (when processor initialized)
  // or Error heading (when processor not yet initialized)
  // Both are valid states for the status page
  const statusHeading = page.getByRole('heading', { name: 'STATUS' });
  const errorHeading = page.getByRole('heading', { name: 'Error' });

  await expect(statusHeading.or(errorHeading)).toBeVisible({ timeout: 10000 });
});

test('assessment page has groove navigation tabs', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/assessment/status`);

  // Should have nav tabs for Security and Assessment in subnav
  await expect(page.getByRole('link', { name: 'Security' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'Assessment' })).toBeVisible();
});

test('assessment tab is active on assessment page', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/assessment/status`);

  const assessmentTab = page.getByRole('link', { name: 'Assessment' });
  await expect(assessmentTab).toHaveClass(/active/);
});

test('security tab navigates to security page', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/assessment/status`);

  const securityTab = page.getByRole('link', { name: 'Security' });
  await securityTab.click();

  // Should navigate to /groove (security/quarantine page)
  await expect(page).toHaveURL(`${serverUrl}/groove`);
});

test('assessment shows connection status', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/assessment/stream`);

  // Should show Connected badge when WebSocket connects
  await expect(page.getByText('Connected')).toBeVisible({ timeout: 5000 });
});

test('assessment has tier filter chips', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/assessment/stream`);

  // Should have filter chips for assessment tiers
  await expect(page.getByRole('button', { name: 'lightweight' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'medium' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'heavy' })).toBeVisible();
});

test('assessment has search input', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/assessment/stream`);

  await expect(page.getByPlaceholder('Search events...')).toBeVisible();
});

test('tier filter chips toggle active state', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/assessment/stream`);

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
  await page.goto(`${serverUrl}/groove/assessment/stream`);

  // Stream panel should be visible with title
  await expect(page.getByText('Assessment Events')).toBeVisible();
});

test('search input accepts text', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/assessment/stream`);

  const searchInput = page.getByPlaceholder('Search events...');
  await searchInput.fill('test query');

  await expect(searchInput).toHaveValue('test query');
});

test('multiple tier filter chips can be active simultaneously', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/assessment/stream`);

  const lightweightChip = page.getByRole('button', { name: 'lightweight' });
  const mediumChip = page.getByRole('button', { name: 'medium' });

  // Activate both
  await lightweightChip.click();
  await mediumChip.click();

  await expect(lightweightChip).toHaveClass(/active/);
  await expect(mediumChip).toHaveClass(/active/);
});

test('assessment shows LIVE indicator when connected', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/assessment/stream`);

  // Wait for connection
  await expect(page.getByText('Connected')).toBeVisible({ timeout: 5000 });

  // Should show LIVE in the stream view
  await expect(page.getByText('LIVE')).toBeVisible();
});

test('jump to latest button is hidden when following', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/assessment/stream`);

  // Wait for connection
  await expect(page.getByText('Connected')).toBeVisible({ timeout: 5000 });

  // Initially, Jump to latest should not be visible (we're following)
  await expect(page.getByRole('button', { name: /Jump to latest/i })).not.toBeVisible();
});

test('groove navigation works from security to assessment', async ({ page, serverUrl }) => {
  // Start on security page
  await page.goto(`${serverUrl}/groove`);

  // Click Assessment tab
  const assessmentTab = page.getByRole('link', { name: 'Assessment' });
  await assessmentTab.click();

  // Should navigate to assessment status page (default)
  await expect(page).toHaveURL(`${serverUrl}/groove/assessment/status`);
  await expect(page.getByRole('heading', { name: 'STATUS' })).toBeVisible();
});
