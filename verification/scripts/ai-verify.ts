#!/usr/bin/env npx tsx
/**
 * AI Verification Script
 *
 * Main entry point for running AI-assisted verification on story acceptance criteria.
 *
 * Usage:
 *   npx tsx ai-verify.ts <story-id> [--model "provider:model"]
 *
 * Examples:
 *   npx tsx ai-verify.ts FEAT0109
 *   npx tsx ai-verify.ts FEAT0109 --model "ollama:llava:34b"
 *   npx tsx ai-verify.ts FEAT0109 --model "claude:claude-sonnet-4-20250514"
 */

import * as fs from 'node:fs/promises';
import * as path from 'node:path';
import { parseStoryFile, type Criterion, type ParsedStory } from './lib/parser.js';
import { collectArtifact, type CollectedArtifact } from './lib/collector.js';
import { analyze, getModelConfig, ModelError, type Verdict, type ModelConfig } from './lib/router.js';
import { generateAndSaveReport, type CriterionResult } from './lib/report.js';

/**
 * Parse command line arguments
 */
function parseArgs(args: string[]): { storyId: string; modelOverride?: string } {
  const relevantArgs = args.slice(2); // Skip node and script path

  if (relevantArgs.length === 0) {
    throw new Error('Usage: ai-verify.ts <story-id> [--model "provider:model"]');
  }

  const storyId = relevantArgs[0];
  let modelOverride: string | undefined;

  // Look for --model flag
  const modelIndex = relevantArgs.indexOf('--model');
  if (modelIndex !== -1 && modelIndex + 1 < relevantArgs.length) {
    modelOverride = relevantArgs[modelIndex + 1];
  }

  return { storyId, modelOverride };
}

/**
 * Recursively find all markdown files in a directory
 */
async function findMarkdownFiles(dir: string): Promise<string[]> {
  const files: string[] = [];

  try {
    const entries = await fs.readdir(dir, { withFileTypes: true });

    for (const entry of entries) {
      const fullPath = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        files.push(...(await findMarkdownFiles(fullPath)));
      } else if (entry.isFile() && entry.name.endsWith('.md')) {
        files.push(fullPath);
      }
    }
  } catch {
    // Directory doesn't exist or isn't readable
  }

  return files;
}

/**
 * Find the actual story file path by searching for the story ID pattern.
 *
 * Story files are named like [FEAT][0201]-name.md where FEAT0201 is the story ID.
 * We use file searching instead of glob because the brackets are literal in filenames.
 */
async function findStoryFilePath(storyId: string): Promise<string> {
  // Normalize to uppercase
  const normalized = storyId.toUpperCase();

  // Match pattern: TYPE + NUMBER (e.g., FEAT0109, CHORE0146, BUG0001)
  const match = normalized.match(/^([A-Z]+)(\d+)$/);
  if (!match) {
    throw new Error(
      `Invalid story ID format: "${storyId}". Expected format like FEAT0109, CHORE0146, or BUG0001`
    );
  }

  const [, typeCode, number] = match;
  // Pad number to 4 digits
  const paddedNumber = number.padStart(4, '0');

  // Pattern to match in filename: [FEAT][0201]
  const filenamePattern = `[${typeCode}][${paddedNumber}]`;

  // Search through docs/board/stages for story files
  const stagesDir = path.join(process.cwd(), 'docs/board/stages');
  const allFiles = await findMarkdownFiles(stagesDir);

  // Find files that match the pattern
  const matches = allFiles.filter((filePath) => {
    const filename = path.basename(filePath);
    return filename.includes(filenamePattern);
  });

  if (matches.length === 0) {
    throw new Error(`Story ${storyId} not found`);
  }

  // Return the first match (there should typically only be one)
  return matches[0];
}

/**
 * Load and parse a story file
 */
async function loadStory(filePath: string): Promise<ParsedStory> {
  const content = await fs.readFile(filePath, 'utf-8');
  return parseStoryFile(content);
}

/**
 * Verify a single criterion against its artifact
 */
async function verifyCriterion(
  criterion: Criterion,
  modelOverride: string | undefined,
  configPath: string,
  verificationDir: string
): Promise<CriterionResult> {
  // If no annotation, we can't collect an artifact
  if (!criterion.annotation) {
    return {
      criterion,
      artifact: null,
      verdict: null,
    };
  }

  // Try to collect the artifact
  const artifact = await collectArtifact(criterion.annotation, verificationDir);

  if (!artifact) {
    return {
      criterion,
      artifact: null,
      verdict: null,
    };
  }

  // Analyze the artifact
  try {
    const verdict = await analyze(artifact, criterion, modelOverride, configPath);
    return {
      criterion,
      artifact,
      verdict,
    };
  } catch (error) {
    // For model errors, we still return a result but with no verdict
    // This allows the report to show the error rather than stopping
    if (error instanceof ModelError) {
      console.error(`  Error analyzing criterion: ${error.message}`);
    } else {
      console.error(`  Unexpected error: ${error instanceof Error ? error.message : String(error)}`);
    }
    return {
      criterion,
      artifact,
      verdict: null,
    };
  }
}

/**
 * Print a summary of the verification results
 */
function printSummary(results: CriterionResult[], story: ParsedStory): void {
  let passed = 0;
  let failed = 0;
  let needsReview = 0;

  for (const result of results) {
    if (!result.verdict) {
      needsReview++;
    } else if (result.verdict.verdict === 'pass' && result.verdict.confidence >= 80) {
      passed++;
    } else if (result.verdict.verdict === 'fail' && result.verdict.confidence >= 80) {
      failed++;
    } else {
      needsReview++;
    }
  }

  console.log('');
  console.log('=== Summary ===');
  console.log(`Story: ${story.id} - ${story.title}`);
  console.log(`Total criteria: ${results.length}`);
  console.log(`  Pass: ${passed}`);
  console.log(`  Fail: ${failed}`);
  console.log(`  Needs Review: ${needsReview}`);
}

/**
 * Main entry point
 */
async function main(): Promise<void> {
  // Parse arguments
  let args: { storyId: string; modelOverride?: string };
  try {
    args = parseArgs(process.argv);
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }

  const { storyId, modelOverride } = args;

  // Find and load the story
  console.log(`Finding story: ${storyId}`);
  let storyFilePath: string;
  let story: ParsedStory;

  try {
    storyFilePath = await findStoryFilePath(storyId);
    console.log(`Found: ${storyFilePath}`);
    story = await loadStory(storyFilePath);
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }

  // Check for acceptance criteria
  if (story.criteria.length === 0) {
    console.error(`No acceptance criteria found in ${storyId}`);
    process.exit(1);
  }

  console.log(`Found ${story.criteria.length} acceptance criteria`);

  // Setup paths
  const verificationDir = path.resolve(process.cwd(), 'verification');
  const configPath = path.join(verificationDir, 'config.toml');

  // Get model configuration
  let model: ModelConfig;
  try {
    model = getModelConfig(configPath, modelOverride);
    console.log(`Using model: ${model.provider}:${model.model}`);
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }

  // Verify each criterion
  console.log('');
  console.log('Verifying criteria...');

  const results: CriterionResult[] = [];

  for (let i = 0; i < story.criteria.length; i++) {
    const criterion = story.criteria[i];
    const num = i + 1;

    // Show progress
    if (criterion.annotation) {
      console.log(`  [${num}/${story.criteria.length}] ${criterion.text.substring(0, 50)}...`);
      console.log(`      Artifact: ${criterion.annotation.type}:${criterion.annotation.name}`);
    } else {
      console.log(`  [${num}/${story.criteria.length}] ${criterion.text.substring(0, 50)}... (no annotation)`);
    }

    const result = await verifyCriterion(criterion, modelOverride, configPath, verificationDir);
    results.push(result);

    // Show result status
    if (!result.artifact && criterion.annotation) {
      console.log(`      -> Artifact not found`);
    } else if (result.verdict) {
      console.log(`      -> ${result.verdict.verdict} (${result.verdict.confidence}%)`);
    } else if (!criterion.annotation) {
      console.log(`      -> Skipped (no annotation)`);
    } else {
      console.log(`      -> Analysis failed`);
    }
  }

  // Generate and save report
  console.log('');
  console.log('Generating report...');

  try {
    const reportPath = await generateAndSaveReport(
      story,
      results,
      model,
      configPath,
      verificationDir
    );
    console.log(`Report saved: ${reportPath}`);
  } catch (error) {
    console.error(`Failed to generate report: ${error instanceof Error ? error.message : String(error)}`);
    process.exit(1);
  }

  // Print summary
  printSummary(results, story);
}

// Run main
main().catch((error) => {
  console.error('Fatal error:', error);
  process.exit(1);
});
