import { describe, test, expect, beforeAll, afterAll } from 'vitest';
import * as fs from 'node:fs/promises';
import * as path from 'node:path';
import {
  collectArtifact,
  collectArtifacts,
  type CollectedArtifact,
} from './collector.js';
import type { Annotation } from './parser.js';

// Test fixtures directory
const TEST_FIXTURES_DIR = path.join(import.meta.dirname, '__fixtures__');
const SNAPSHOTS_DIR = path.join(TEST_FIXTURES_DIR, 'snapshots');
const CHECKPOINTS_DIR = path.join(TEST_FIXTURES_DIR, 'checkpoints');
const VIDEOS_CLI_DIR = path.join(TEST_FIXTURES_DIR, 'videos', 'cli');
const VIDEOS_WEB_DIR = path.join(TEST_FIXTURES_DIR, 'videos', 'web');

// Create test fixtures before running tests
async function createTestFixtures() {
  // Create directories
  await fs.mkdir(SNAPSHOTS_DIR, { recursive: true });
  await fs.mkdir(CHECKPOINTS_DIR, { recursive: true });
  await fs.mkdir(VIDEOS_CLI_DIR, { recursive: true });
  await fs.mkdir(VIDEOS_WEB_DIR, { recursive: true });

  // Create test snapshot (minimal PNG header)
  const pngHeader = Buffer.from([
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a,
  ]);
  await fs.writeFile(path.join(SNAPSHOTS_DIR, 'test-snapshot.png'), pngHeader);

  // Create checkpoint images (numbered sequence)
  await fs.writeFile(
    path.join(CHECKPOINTS_DIR, 'test-checkpoint-01-step-one.png'),
    pngHeader
  );
  await fs.writeFile(
    path.join(CHECKPOINTS_DIR, 'test-checkpoint-02-step-two.png'),
    pngHeader
  );
  await fs.writeFile(
    path.join(CHECKPOINTS_DIR, 'test-checkpoint-03-step-three.png'),
    pngHeader
  );

  // Create test video (minimal webm header - EBML signature)
  const webmHeader = Buffer.from([
    0x1a, 0x45, 0xdf, 0xa3, 0x93, 0x42, 0x82, 0x88,
  ]);
  await fs.writeFile(path.join(VIDEOS_CLI_DIR, 'test-cli-video.webm'), webmHeader);
  await fs.writeFile(path.join(VIDEOS_WEB_DIR, 'test-web-video.webm'), webmHeader);
}

// Clean up test fixtures after tests
async function cleanupTestFixtures() {
  await fs.rm(TEST_FIXTURES_DIR, { recursive: true, force: true });
}

describe('Artifact Collector', () => {
  beforeAll(async () => {
    await createTestFixtures();
  });

  afterAll(async () => {
    await cleanupTestFixtures();
  });

  describe('collectArtifact', () => {
    test('collects a snapshot by name', async () => {
      const annotation: Annotation = {
        type: 'snapshot',
        name: 'test-snapshot',
      };

      const result = await collectArtifact(annotation, TEST_FIXTURES_DIR);

      expect(result).not.toBeNull();
      expect(result!.type).toBe('image');
      expect(result!.path).toContain('test-snapshot.png');
      expect(result!.data).toBeInstanceOf(Buffer);
      expect(result!.data.length).toBeGreaterThan(0);
    });

    test('collects checkpoint images matching name prefix', async () => {
      const annotation: Annotation = {
        type: 'checkpoint',
        name: 'test-checkpoint',
      };

      const result = await collectArtifacts(annotation, TEST_FIXTURES_DIR);

      expect(result).toHaveLength(3);
      expect(result[0].type).toBe('image');
      expect(result[0].path).toContain('test-checkpoint-01');
      expect(result[1].path).toContain('test-checkpoint-02');
      expect(result[2].path).toContain('test-checkpoint-03');
    });

    test('collects a CLI video by name', async () => {
      const annotation: Annotation = {
        type: 'video',
        name: 'test-cli-video',
      };

      const result = await collectArtifact(annotation, TEST_FIXTURES_DIR);

      expect(result).not.toBeNull();
      expect(result!.type).toBe('video');
      expect(result!.path).toContain('test-cli-video.webm');
      expect(result!.data).toBeInstanceOf(Buffer);
    });

    test('collects a web video by name', async () => {
      const annotation: Annotation = {
        type: 'video',
        name: 'test-web-video',
      };

      const result = await collectArtifact(annotation, TEST_FIXTURES_DIR);

      expect(result).not.toBeNull();
      expect(result!.type).toBe('video');
      expect(result!.path).toContain('test-web-video.webm');
    });

    test('returns null for missing snapshot', async () => {
      const annotation: Annotation = {
        type: 'snapshot',
        name: 'nonexistent-snapshot',
      };

      const result = await collectArtifact(annotation, TEST_FIXTURES_DIR);

      expect(result).toBeNull();
    });

    test('returns null for missing video', async () => {
      const annotation: Annotation = {
        type: 'video',
        name: 'nonexistent-video',
      };

      const result = await collectArtifact(annotation, TEST_FIXTURES_DIR);

      expect(result).toBeNull();
    });

    test('returns empty array for missing checkpoint', async () => {
      const annotation: Annotation = {
        type: 'checkpoint',
        name: 'nonexistent-checkpoint',
      };

      const result = await collectArtifacts(annotation, TEST_FIXTURES_DIR);

      expect(result).toEqual([]);
    });
  });

  describe('collectArtifacts', () => {
    test('returns array with single artifact for snapshot', async () => {
      const annotation: Annotation = {
        type: 'snapshot',
        name: 'test-snapshot',
      };

      const result = await collectArtifacts(annotation, TEST_FIXTURES_DIR);

      expect(result).toHaveLength(1);
      expect(result[0].type).toBe('image');
    });

    test('returns multiple artifacts for checkpoint sequence', async () => {
      const annotation: Annotation = {
        type: 'checkpoint',
        name: 'test-checkpoint',
      };

      const result = await collectArtifacts(annotation, TEST_FIXTURES_DIR);

      expect(result).toHaveLength(3);
      // Should be sorted by filename
      expect(result[0].path).toContain('-01-');
      expect(result[1].path).toContain('-02-');
      expect(result[2].path).toContain('-03-');
    });

    test('returns array with single artifact for video', async () => {
      const annotation: Annotation = {
        type: 'video',
        name: 'test-cli-video',
      };

      const result = await collectArtifacts(annotation, TEST_FIXTURES_DIR);

      expect(result).toHaveLength(1);
      expect(result[0].type).toBe('video');
    });
  });

  describe('artifact type inference', () => {
    test('infers image type for .png files', async () => {
      const annotation: Annotation = {
        type: 'snapshot',
        name: 'test-snapshot',
      };

      const result = await collectArtifact(annotation, TEST_FIXTURES_DIR);

      expect(result!.type).toBe('image');
    });

    test('infers video type for .webm files', async () => {
      const annotation: Annotation = {
        type: 'video',
        name: 'test-cli-video',
      };

      const result = await collectArtifact(annotation, TEST_FIXTURES_DIR);

      expect(result!.type).toBe('video');
    });
  });

  describe('integration with real verification artifacts', () => {
    // These tests use actual verification artifacts if they exist
    const REAL_VERIFICATION_DIR = path.resolve(
      import.meta.dirname,
      '../../..'
    );

    test('can collect real snapshot if exists', async () => {
      const annotation: Annotation = {
        type: 'snapshot',
        name: 'sessions',
      };

      const result = await collectArtifact(annotation, REAL_VERIFICATION_DIR);

      // This test passes whether or not real artifacts exist
      if (result) {
        expect(result.type).toBe('image');
        expect(result.path).toContain('sessions.png');
      }
    });

    test('can collect real checkpoint sequence if exists', async () => {
      const annotation: Annotation = {
        type: 'checkpoint',
        name: 'navigate-groove-dashboard',
      };

      const result = await collectArtifacts(annotation, REAL_VERIFICATION_DIR);

      // This test passes whether or not real artifacts exist
      if (result.length > 0) {
        expect(result.every((a) => a.type === 'image')).toBe(true);
        expect(result[0].path).toContain('navigate-groove-dashboard');
      }
    });

    test('can collect real video if exists', async () => {
      const annotation: Annotation = {
        type: 'video',
        name: 'dashboard-walkthrough',
      };

      const result = await collectArtifact(annotation, REAL_VERIFICATION_DIR);

      // This test passes whether or not real artifacts exist
      if (result) {
        expect(result.type).toBe('video');
        expect(result.path).toContain('dashboard-walkthrough');
      }
    });
  });
});
