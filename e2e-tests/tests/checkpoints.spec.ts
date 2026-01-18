/**
 * Verification Tier 2: Checkpoint Capture
 *
 * Captures screenshots at key interaction points defined in verification/checkpoints.json.
 * These checkpoints document user flows through the application.
 *
 * Run via: just verify checkpoints
 */
import { test, expect } from '../fixtures/vibes.js';
import * as fs from 'fs';
import * as path from 'path';

const projectRoot = path.resolve(import.meta.dirname, '../..');
const checkpointsConfigPath = path.join(projectRoot, 'verification/checkpoints.json');
const outputDir = path.join(projectRoot, 'verification/checkpoints');

interface CheckpointStep {
  action: 'goto' | 'click' | 'wait' | 'type' | 'scroll';
  url?: string;
  selector?: string;
  duration?: number;
  text?: string;
  screenshot: string;
  optional?: boolean;
}

interface CheckpointDefinition {
  name: string;
  description?: string;
  steps: CheckpointStep[];
}

interface CheckpointsConfig {
  baseUrl: string;
  outputDir: string;
  checkpoints: CheckpointDefinition[];
}

// Load checkpoint definitions
function loadCheckpointsConfig(): CheckpointsConfig {
  if (!fs.existsSync(checkpointsConfigPath)) {
    throw new Error(`Checkpoints config not found: ${checkpointsConfigPath}`);
  }
  return JSON.parse(fs.readFileSync(checkpointsConfigPath, 'utf-8'));
}

const config = loadCheckpointsConfig();

// Ensure output directory exists
fs.mkdirSync(outputDir, { recursive: true });

test.describe('Verification Checkpoints', () => {
  for (const checkpoint of config.checkpoints) {
    test(`checkpoint: ${checkpoint.name}`, async ({ page, serverUrl }) => {
      console.log(`[checkpoint] Starting: ${checkpoint.name}`);

      for (const step of checkpoint.steps) {
        try {
          switch (step.action) {
            case 'goto':
              await page.goto(`${serverUrl}${step.url}`);
              await page.waitForLoadState('networkidle');
              break;

            case 'click':
              if (step.selector) {
                // Try multiple selectors (comma-separated fallbacks)
                const selectors = step.selector.split(',').map(s => s.trim());
                let clicked = false;

                for (const selector of selectors) {
                  try {
                    const element = await page.waitForSelector(selector, { timeout: 3000 });
                    if (element) {
                      await element.click();
                      clicked = true;
                      break;
                    }
                  } catch {
                    // Try next selector
                  }
                }

                if (!clicked && !step.optional) {
                  throw new Error(`Could not click any of: ${step.selector}`);
                }
              }
              break;

            case 'wait':
              if (step.duration) {
                await page.waitForTimeout(step.duration);
              }
              break;

            case 'type':
              if (step.selector && step.text) {
                await page.fill(step.selector, step.text);
              }
              break;

            case 'scroll':
              if (step.selector) {
                await page.locator(step.selector).scrollIntoViewIfNeeded();
              } else {
                await page.evaluate(() => window.scrollBy(0, 300));
              }
              break;
          }

          // Allow UI to settle
          await page.waitForTimeout(300);

          // Take screenshot at this checkpoint step
          const screenshotPath = path.join(outputDir, `${checkpoint.name}-${step.screenshot}.png`);
          await page.screenshot({
            path: screenshotPath,
            fullPage: false,
          });

          console.log(`[checkpoint] Captured: ${checkpoint.name}-${step.screenshot}.png`);

        } catch (error) {
          if (step.optional) {
            console.log(`[checkpoint] Optional step skipped: ${step.action} - ${error}`);
            // Still try to take a screenshot
            const screenshotPath = path.join(outputDir, `${checkpoint.name}-${step.screenshot}-skipped.png`);
            await page.screenshot({ path: screenshotPath, fullPage: false });
          } else {
            throw error;
          }
        }
      }

      // Verify at least one screenshot was created
      const files = fs.readdirSync(outputDir).filter(f => f.startsWith(checkpoint.name));
      expect(files.length).toBeGreaterThan(0);
    });
  }
});
