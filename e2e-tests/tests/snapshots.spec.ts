/**
 * Verification Tier 1: Snapshot Capture
 *
 * Captures screenshots of key pages defined in verification/snapshots.json.
 * These snapshots serve as visual documentation of the current UI state.
 *
 * Run via: just verify snapshots
 */
import { test, expect } from '../fixtures/vibes.js';
import * as fs from 'fs';
import * as path from 'path';

const projectRoot = path.resolve(import.meta.dirname, '../..');
const snapshotsConfigPath = path.join(projectRoot, 'verification/snapshots.json');
const outputDir = path.join(projectRoot, 'verification/snapshots');

interface SnapshotDefinition {
  name: string;
  url: string;
  description?: string;
  waitFor?: string;
}

interface SnapshotsConfig {
  baseUrl: string;
  outputDir: string;
  snapshots: SnapshotDefinition[];
}

// Load snapshot definitions
function loadSnapshotsConfig(): SnapshotsConfig {
  if (!fs.existsSync(snapshotsConfigPath)) {
    throw new Error(`Snapshots config not found: ${snapshotsConfigPath}`);
  }
  return JSON.parse(fs.readFileSync(snapshotsConfigPath, 'utf-8'));
}

const config = loadSnapshotsConfig();

// Ensure output directory exists
fs.mkdirSync(outputDir, { recursive: true });

test.describe('Verification Snapshots', () => {
  for (const snapshot of config.snapshots) {
    test(`capture ${snapshot.name}`, async ({ page, serverUrl }) => {
      // Navigate to the page
      await page.goto(`${serverUrl}${snapshot.url}`);

      // Wait for page to be ready
      await page.waitForLoadState('networkidle');

      // If a specific selector is defined, wait for it
      if (snapshot.waitFor) {
        await page.waitForSelector(snapshot.waitFor, { timeout: 10000 }).catch(() => {
          console.log(`[snapshot] Selector not found: ${snapshot.waitFor}, continuing anyway`);
        });
      }

      // Allow animations to settle
      await page.waitForTimeout(500);

      // Take screenshot
      const screenshotPath = path.join(outputDir, `${snapshot.name}.png`);
      await page.screenshot({
        path: screenshotPath,
        fullPage: false,
      });

      console.log(`[snapshot] Captured: ${snapshot.name}.png`);

      // Verify file was created
      expect(fs.existsSync(screenshotPath)).toBe(true);
    });
  }
});
