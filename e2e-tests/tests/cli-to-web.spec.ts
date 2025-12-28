import { test, expect } from '../fixtures/vibes.js';
import { waitForSession, waitForPrompt, collectOutput } from '../helpers/cli.js';

test('CLI session appears in Web UI session list', async ({ page, serverUrl, cli }) => {
  // Start CLI session
  const cliProc = cli('claude', '--session-name', 'mirror-test');

  // Wait for session to appear via API
  const session = await waitForSession(serverUrl, 'mirror-test');
  expect(session.id).toBeTruthy();

  // Open sessions page in browser
  await page.goto(`${serverUrl}/claude`);

  // Should see the session in the list
  await expect(page.getByText('mirror-test')).toBeVisible({ timeout: 5000 });

  cliProc.kill();
});

test('can navigate to session detail page', async ({ page, serverUrl, cli }) => {
  // Start CLI session
  const cliProc = cli('claude', '--session-name', 'detail-test');

  // Wait for session via API
  const session = await waitForSession(serverUrl, 'detail-test');

  // Navigate directly to session
  await page.goto(`${serverUrl}/claude/${session.id}`);

  // Page should load (session detail view)
  await expect(page.locator('body')).toBeVisible();

  cliProc.kill();
});
