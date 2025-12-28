import { test, expect } from '../fixtures/vibes.js';
import { waitForSession } from '../helpers/cli.js';

test('session detail page loads for existing session', async ({ page, serverUrl, cli }) => {
  // Start CLI session
  const cliProc = cli('claude', '--session-name', 'web-input-test');

  // Wait for session via API
  const session = await waitForSession(serverUrl, 'web-input-test');

  // Open session in browser
  await page.goto(`${serverUrl}/claude/${session.id}`);

  // Page should load without error
  await expect(page.locator('body')).toBeVisible();

  cliProc.kill();
});

test('history page loads', async ({ page, serverUrl }) => {
  await page.goto(`${serverUrl}/history`);

  // History page should load
  await expect(page.locator('body')).toBeVisible();
});

test('API returns sessions list', async ({ serverUrl, cli }) => {
  // Start a CLI session
  const cliProc = cli('claude', '--session-name', 'api-test');

  // Wait for session via API
  const session = await waitForSession(serverUrl, 'api-test');
  expect(session.name).toBe('api-test');

  // Verify full API response
  const response = await fetch(`${serverUrl}/api/claude/sessions`);
  const data = await response.json();

  expect(data.sessions).toBeDefined();
  expect(data.sessions.length).toBeGreaterThan(0);

  cliProc.kill();
});
