import { test, expect } from '../fixtures/vibes.js';

test('shows empty state when no sessions', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/claude`);

  // Should show empty state message with New Session button
  await expect(page.getByText('No active sessions')).toBeVisible();
  await expect(page.getByRole('button', { name: 'New Session' })).toBeVisible();
});

test('New Session button creates a session and navigates to it', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/claude`);

  // Click New Session button
  await page.getByRole('button', { name: 'New Session' }).click();

  // Should navigate to the session detail page
  await expect(page).toHaveURL(/\/claude\/[a-f0-9-]+/);
});

test('sessions page loads without error', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/claude`);

  // Page should load with Sessions header
  await expect(page.getByRole('heading', { name: 'Claude Sessions' })).toBeVisible();
});

test('CLI-created session appears in session list', async ({ page, serverUrl, cli }) => {
  // Start CLI session in background
  const cliProc = cli('claude', '--session-name', 'e2e-flow-test');

  // Poll the API until a session appears
  let sessionFound = false;
  for (let i = 0; i < 30; i++) {
    await new Promise(r => setTimeout(r, 500));
    try {
      const response = await fetch(`${serverUrl}/api/claude/sessions`);
      const data = await response.json();
      if (data.sessions && data.sessions.length > 0) {
        sessionFound = true;
        break;
      }
    } catch {
      // Server might not be ready yet
    }
  }

  expect(sessionFound).toBe(true);

  // Now check the web UI shows it
  await page.goto(`${serverUrl}/claude`);
  await expect(page.getByText('e2e-flow-test')).toBeVisible({ timeout: 10000 });

  cliProc.kill();
});
