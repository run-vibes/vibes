import { test, expect } from '../fixtures/vibes.js';

test('assessment page loads with heading', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/assessment`);

  await expect(page.getByRole('heading', { name: 'ASSESSMENT', exact: true })).toBeVisible();
});

test('assessment page has groove navigation tabs', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/assessment`);

  // Should have nav tabs for Security and Assessment
  await expect(page.getByRole('link', { name: 'Security' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'Assessment' })).toBeVisible();
});

test('assessment tab is active on assessment page', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/assessment`);

  const assessmentTab = page.getByRole('link', { name: 'Assessment' });
  await expect(assessmentTab).toHaveClass(/active/);
});

test('security tab navigates to security page', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/assessment`);

  const securityTab = page.getByRole('link', { name: 'Security' });
  await securityTab.click();

  // Should navigate to /groove (security/quarantine page)
  await expect(page).toHaveURL(`${serverUrl}/groove`);
});

test('assessment shows connection status', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/assessment`);

  // Should show Connected badge when WebSocket connects
  await expect(page.getByText('Connected')).toBeVisible({ timeout: 5000 });
});

test('assessment has tier filter chips', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/assessment`);

  // Should have filter chips for assessment tiers
  await expect(page.getByRole('button', { name: 'lightweight' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'medium' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'heavy' })).toBeVisible();
});

test('assessment has search input', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/assessment`);

  await expect(page.getByPlaceholder('Search events...')).toBeVisible();
});

test('tier filter chips toggle active state', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/assessment`);

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
  await page.goto(`${serverUrl}/groove/assessment`);

  // Stream panel should be visible with title
  await expect(page.getByText('Assessment Events')).toBeVisible();
});

test('search input accepts text', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/assessment`);

  const searchInput = page.getByPlaceholder('Search events...');
  await searchInput.fill('test query');

  await expect(searchInput).toHaveValue('test query');
});

test('multiple tier filter chips can be active simultaneously', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/assessment`);

  const lightweightChip = page.getByRole('button', { name: 'lightweight' });
  const mediumChip = page.getByRole('button', { name: 'medium' });

  // Activate both
  await lightweightChip.click();
  await mediumChip.click();

  await expect(lightweightChip).toHaveClass(/active/);
  await expect(mediumChip).toHaveClass(/active/);
});

test('assessment shows LIVE indicator when connected', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/assessment`);

  // Wait for connection
  await expect(page.getByText('Connected')).toBeVisible({ timeout: 5000 });

  // Should show LIVE in the stream view
  await expect(page.getByText('LIVE')).toBeVisible();
});

test('jump to latest button is hidden when following', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/groove/assessment`);

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

  // Should navigate to assessment page
  await expect(page).toHaveURL(`${serverUrl}/groove/assessment`);
  await expect(page.getByText('Assessment Events')).toBeVisible();
});
