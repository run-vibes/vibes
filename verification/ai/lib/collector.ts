/**
 * Artifact Collector for AI Verification
 *
 * Collects verification artifacts (snapshots, checkpoints, videos) based on
 * annotation references from story criteria.
 */

import * as fs from 'node:fs/promises';
import * as path from 'node:path';
import type { Annotation, AnnotationType } from './parser.js';

/**
 * A collected artifact with its type, path, and data
 */
export interface CollectedArtifact {
  type: 'image' | 'video';
  path: string;
  data: Buffer;
}

/**
 * Image file extensions we recognize
 */
const IMAGE_EXTENSIONS = ['.png', '.jpg', '.jpeg'];

/**
 * Video file extensions we recognize
 */
const VIDEO_EXTENSIONS = ['.webm', '.mp4'];

/**
 * Map annotation types to their artifact directories
 */
const ARTIFACT_DIRS: Record<AnnotationType, string[]> = {
  snapshot: ['snapshots'],
  checkpoint: ['checkpoints'],
  video: ['videos/cli', 'videos/web'],
};

/**
 * Infer the artifact type from file extension
 */
function inferArtifactType(filePath: string): 'image' | 'video' {
  const ext = path.extname(filePath).toLowerCase();
  if (VIDEO_EXTENSIONS.includes(ext)) {
    return 'video';
  }
  return 'image';
}

/**
 * Find files in a directory matching a name pattern with allowed extensions
 *
 * @param dir - Directory to search in
 * @param name - Base name to match (without extension)
 * @param extensions - Allowed file extensions
 * @param matchPrefix - If true, match files starting with name (for checkpoints)
 * @returns Array of matching file paths, sorted alphabetically
 */
async function findMatchingFiles(
  dir: string,
  name: string,
  extensions: string[],
  matchPrefix: boolean = false
): Promise<string[]> {
  try {
    const entries = await fs.readdir(dir, { withFileTypes: true });
    const matches: string[] = [];

    for (const entry of entries) {
      if (!entry.isFile()) continue;

      const ext = path.extname(entry.name).toLowerCase();
      if (!extensions.includes(ext)) continue;

      const baseName = path.basename(entry.name, ext);

      if (matchPrefix) {
        // For checkpoints: match files starting with the name
        if (baseName.startsWith(name)) {
          matches.push(path.join(dir, entry.name));
        }
      } else {
        // For snapshots/videos: exact name match
        if (baseName === name) {
          matches.push(path.join(dir, entry.name));
        }
      }
    }

    // Sort alphabetically for consistent ordering
    return matches.sort();
  } catch {
    // Directory doesn't exist or isn't readable
    return [];
  }
}

/**
 * Read a file and create a CollectedArtifact
 */
async function readArtifact(filePath: string): Promise<CollectedArtifact> {
  const data = await fs.readFile(filePath);
  return {
    type: inferArtifactType(filePath),
    path: filePath,
    data,
  };
}

/**
 * Collect a single artifact based on an annotation.
 *
 * For snapshots: looks for an exact name match in snapshots/
 * For videos: looks for an exact name match in videos/cli/ or videos/web/
 * For checkpoints: returns the first matching file (use collectArtifacts for all)
 *
 * @param annotation - The annotation specifying the artifact to collect
 * @param verificationDir - Base verification directory (defaults to cwd/verification)
 * @returns The collected artifact, or null if not found
 */
export async function collectArtifact(
  annotation: Annotation,
  verificationDir: string = path.join(process.cwd(), 'verification')
): Promise<CollectedArtifact | null> {
  const dirs = ARTIFACT_DIRS[annotation.type];
  const extensions =
    annotation.type === 'video' ? VIDEO_EXTENSIONS : IMAGE_EXTENSIONS;
  const matchPrefix = annotation.type === 'checkpoint';

  for (const dir of dirs) {
    const fullDir = path.join(verificationDir, dir);
    const matches = await findMatchingFiles(
      fullDir,
      annotation.name,
      extensions,
      matchPrefix
    );

    if (matches.length > 0) {
      return readArtifact(matches[0]);
    }
  }

  return null;
}

/**
 * Collect all artifacts matching an annotation.
 *
 * For snapshots: returns array with single matching file
 * For videos: returns array with single matching file
 * For checkpoints: returns all files matching the name prefix, sorted
 *
 * @param annotation - The annotation specifying the artifacts to collect
 * @param verificationDir - Base verification directory (defaults to cwd/verification)
 * @returns Array of collected artifacts (empty if none found)
 */
export async function collectArtifacts(
  annotation: Annotation,
  verificationDir: string = path.join(process.cwd(), 'verification')
): Promise<CollectedArtifact[]> {
  const dirs = ARTIFACT_DIRS[annotation.type];
  const extensions =
    annotation.type === 'video' ? VIDEO_EXTENSIONS : IMAGE_EXTENSIONS;
  const matchPrefix = annotation.type === 'checkpoint';

  const allMatches: string[] = [];

  for (const dir of dirs) {
    const fullDir = path.join(verificationDir, dir);
    const matches = await findMatchingFiles(
      fullDir,
      annotation.name,
      extensions,
      matchPrefix
    );
    allMatches.push(...matches);
  }

  // Sort all matches for consistent ordering
  allMatches.sort();

  // Read all matching files
  const artifacts: CollectedArtifact[] = [];
  for (const filePath of allMatches) {
    artifacts.push(await readArtifact(filePath));
  }

  return artifacts;
}
