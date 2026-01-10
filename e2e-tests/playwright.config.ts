import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  timeout: 60000,
  retries: 1,

  // Snapshot settings for visual regression
  snapshotDir: './snapshots',
  snapshotPathTemplate: '{snapshotDir}/{testFilePath}/{arg}{ext}',
  expect: {
    toHaveScreenshot: {
      maxDiffPixelRatio: 0.01, // Allow 1% pixel difference
      threshold: 0.2, // Per-pixel color threshold
    },
  },

  use: {
    headless: true,
    video: 'on-first-retry',
    trace: 'on-first-retry',
    viewport: { width: 1280, height: 720 },
  },

  projects: [
    {
      name: 'chromium',
      use: { browserName: 'chromium' },
    },
  ],
});
