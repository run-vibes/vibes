/**
 * Verification Tier 3: Video Capture
 *
 * Records video walkthroughs of key user flows for verification.
 * Videos are saved to verification/videos/web/.
 *
 * Run via: just verify videos
 */
import { test as base, expect } from '../fixtures/vibes.js';
import * as fs from 'fs';
import * as path from 'path';

const projectRoot = path.resolve(import.meta.dirname, '../..');
const outputDir = path.join(projectRoot, 'verification/videos/web');

// Ensure output directory exists
fs.mkdirSync(outputDir, { recursive: true });

// Extend the test to always record video
const test = base.extend({
  // Override context to force video recording
  context: async ({ context }, use) => {
    await use(context);
  },
});

// Override test config for video recording
test.use({
  video: {
    mode: 'on',
    size: { width: 1280, height: 720 },
  },
});

test.describe('Verification Videos', () => {
  test('groove-dashboard-walkthrough', async ({ page, serverUrl }) => {
    // Navigate through the main Groove dashboard
    await page.goto(`${serverUrl}/groove/status`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);

    // Visit learnings
    await page.goto(`${serverUrl}/groove/learnings`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);

    // Visit strategy
    await page.goto(`${serverUrl}/groove/strategy`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);

    // Visit trends
    await page.goto(`${serverUrl}/groove/trends`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);

    // Visit stream
    await page.goto(`${serverUrl}/groove/stream`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(2000); // Let some events flow

    expect(page.url()).toContain('/groove/stream');
  });

  test('sessions-walkthrough', async ({ page, serverUrl }) => {
    // Navigate to sessions
    await page.goto(`${serverUrl}/sessions`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1500);

    // Try to click on a session if one exists
    const sessionRow = page.locator('.session-row, [data-testid="session-row"], tr a').first();
    if (await sessionRow.isVisible().catch(() => false)) {
      await sessionRow.click();
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1500);
    }

    // Navigate to firehose
    await page.goto(`${serverUrl}/firehose`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1500);

    expect(page.url()).toContain('/firehose');
  });

  test('settings-walkthrough', async ({ page, serverUrl }) => {
    // Navigate to settings
    await page.goto(`${serverUrl}/settings`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1500);

    // Navigate to models
    await page.goto(`${serverUrl}/models`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1500);

    // Navigate to agents
    await page.goto(`${serverUrl}/agents`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1500);

    // Navigate to traces
    await page.goto(`${serverUrl}/traces`);
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1500);

    expect(page.url()).toContain('/traces');
  });
});

// After all tests, copy videos to verification directory
test.afterEach(async ({ page }, testInfo) => {
  // Get video path from test attachments
  const video = testInfo.attachments.find(a => a.name === 'video');
  if (video?.path) {
    const destName = `${testInfo.title.replace(/[^a-zA-Z0-9-]/g, '-')}.webm`;
    const destPath = path.join(outputDir, destName);

    // Copy video to verification directory
    try {
      fs.copyFileSync(video.path, destPath);
      console.log(`[video] Saved: ${destName}`);
    } catch (err) {
      console.log(`[video] Could not copy video: ${err}`);
    }
  }
});
