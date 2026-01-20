#!/usr/bin/env npx tsx
/**
 * Learning Propagation Engine
 *
 * AI-powered tool for applying learnings to target files.
 *
 * Usage:
 *   npx tsx learn-apply.ts [--model "provider:model"]
 *
 * Examples:
 *   npx tsx learn-apply.ts
 *   npx tsx learn-apply.ts --model "claude:claude-sonnet-4-20250514"
 */

import * as fs from 'node:fs/promises';
import * as path from 'node:path';
import * as readline from 'node:readline';
import {
  scanAllLearnings,
  filterUnappliedLearnings,
  parseAppliesTo,
  markLearningAsApplied,
  type Learning,
} from './lib/learnings.js';
import { getModelConfig, loadConfig, type ModelConfig } from './lib/router.js';
import Anthropic from '@anthropic-ai/sdk';
import { Ollama } from 'ollama';

/**
 * AI-suggested change to a file
 */
interface SuggestedChange {
  description: string;
  oldText: string;
  newText: string;
}

/**
 * AI analysis result for applying a learning
 */
interface ApplyAnalysis {
  shouldChange: boolean;
  reason: string;
  changes: SuggestedChange[];
}

/**
 * Parse command line arguments
 */
function parseArgs(args: string[]): { modelOverride?: string } {
  const relevantArgs = args.slice(2); // Skip node and script path

  let modelOverride: string | undefined;

  // Look for --model flag
  const modelIndex = relevantArgs.indexOf('--model');
  if (modelIndex !== -1 && modelIndex + 1 < relevantArgs.length) {
    modelOverride = relevantArgs[modelIndex + 1];
  }

  return { modelOverride };
}

/**
 * Create readline interface for user input
 */
function createReadline(): readline.Interface {
  return readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });
}

/**
 * Prompt user for input
 */
async function prompt(rl: readline.Interface, question: string): Promise<string> {
  return new Promise((resolve) => {
    rl.question(question, (answer) => {
      resolve(answer.trim().toLowerCase());
    });
  });
}

/**
 * Build the prompt for AI to suggest changes
 */
function buildApplyPrompt(learning: Learning, filePath: string, fileContent: string): string {
  return `You are helping apply a learning to improve a codebase.

## Learning
**ID:** ${learning.id}
**Title:** ${learning.title}
**Category:** ${learning.category}
**Context:** ${learning.context}
**Insight:** ${learning.insight}
**Suggested Action:** ${learning.suggestedAction}

## Target File: ${filePath}
\`\`\`
${fileContent}
\`\`\`

Based on this learning, suggest specific changes to improve the target file.
The changes should incorporate the insight from the learning.

Respond in JSON only (no markdown code blocks):
{
  "shouldChange": true or false,
  "reason": "Why this change helps (or why no change needed)",
  "changes": [
    {
      "description": "What this change does",
      "oldText": "exact text to find in the file",
      "newText": "replacement text"
    }
  ]
}

Guidelines:
- Only suggest changes that directly relate to the learning's insight
- The oldText must be an exact substring found in the file
- If no changes are needed, set shouldChange to false and changes to []
- Keep changes focused and minimal`;
}

/**
 * Parse the AI response into an ApplyAnalysis
 */
function parseApplyResponse(response: string): ApplyAnalysis {
  // Try to extract JSON from markdown code blocks if present
  let jsonStr = response.trim();

  // Remove markdown code blocks if present
  const jsonBlockMatch = jsonStr.match(/```(?:json)?\s*([\s\S]*?)```/);
  if (jsonBlockMatch) {
    jsonStr = jsonBlockMatch[1].trim();
  }

  let parsed: unknown;
  try {
    parsed = JSON.parse(jsonStr);
  } catch {
    throw new Error(`Failed to parse AI response as JSON: ${response.substring(0, 200)}...`);
  }

  if (typeof parsed !== 'object' || parsed === null) {
    throw new Error(`Expected JSON object, got: ${typeof parsed}`);
  }

  const obj = parsed as Record<string, unknown>;

  // Validate required fields
  if (typeof obj.shouldChange !== 'boolean') {
    throw new Error(`Invalid shouldChange field: ${obj.shouldChange}`);
  }

  if (typeof obj.reason !== 'string') {
    throw new Error(`Missing or invalid reason field`);
  }

  if (!Array.isArray(obj.changes)) {
    throw new Error(`Missing or invalid changes field`);
  }

  // Validate each change
  const changes: SuggestedChange[] = [];
  for (const change of obj.changes) {
    if (typeof change !== 'object' || change === null) {
      continue;
    }
    const c = change as Record<string, unknown>;
    if (typeof c.description === 'string' &&
        typeof c.oldText === 'string' &&
        typeof c.newText === 'string') {
      changes.push({
        description: c.description,
        oldText: c.oldText,
        newText: c.newText,
      });
    }
  }

  return {
    shouldChange: obj.shouldChange,
    reason: obj.reason,
    changes,
  };
}

/**
 * Analyze a learning against a file using Claude
 */
async function analyzeWithClaude(
  learning: Learning,
  filePath: string,
  fileContent: string,
  model: string
): Promise<ApplyAnalysis> {
  if (!process.env.ANTHROPIC_API_KEY) {
    throw new Error('ANTHROPIC_API_KEY environment variable not set');
  }

  const anthropic = new Anthropic();
  const prompt = buildApplyPrompt(learning, filePath, fileContent);

  const response = await anthropic.messages.create({
    model,
    max_tokens: 4096,
    messages: [
      {
        role: 'user',
        content: prompt,
      },
    ],
  });

  const textBlock = response.content.find((block) => block.type === 'text');
  if (!textBlock || textBlock.type !== 'text') {
    throw new Error('No text response from Claude');
  }

  return parseApplyResponse(textBlock.text);
}

/**
 * Analyze a learning against a file using Ollama
 */
async function analyzeWithOllama(
  learning: Learning,
  filePath: string,
  fileContent: string,
  model: string
): Promise<ApplyAnalysis> {
  const ollama = new Ollama();
  const prompt = buildApplyPrompt(learning, filePath, fileContent);

  const response = await ollama.generate({
    model,
    prompt,
    stream: false,
  });

  return parseApplyResponse(response.response);
}

/**
 * Analyze a learning against a file using the configured model
 */
async function analyzeFile(
  learning: Learning,
  filePath: string,
  fileContent: string,
  modelConfig: ModelConfig
): Promise<ApplyAnalysis> {
  if (modelConfig.provider === 'claude') {
    return analyzeWithClaude(learning, filePath, fileContent, modelConfig.model);
  } else {
    return analyzeWithOllama(learning, filePath, fileContent, modelConfig.model);
  }
}

/**
 * Generate a simple diff display for a change
 */
function formatDiff(filePath: string, change: SuggestedChange): string {
  const lines: string[] = [];
  lines.push(`--- a/${filePath}`);
  lines.push(`+++ b/${filePath}`);
  lines.push('@@ change @@');

  // Show old text with - prefix
  for (const line of change.oldText.split('\n')) {
    lines.push(`- ${line}`);
  }

  // Show new text with + prefix
  for (const line of change.newText.split('\n')) {
    lines.push(`+ ${line}`);
  }

  return lines.join('\n');
}

/**
 * Apply a change to file content
 */
function applyChange(content: string, change: SuggestedChange): string | null {
  if (!content.includes(change.oldText)) {
    return null; // Change cannot be applied
  }
  return content.replace(change.oldText, change.newText);
}

/**
 * Resolve a file pattern to actual file paths
 */
async function resolveFilePath(pattern: string, baseDir: string): Promise<string | null> {
  // Try direct path first
  const directPath = path.join(baseDir, pattern);
  try {
    await fs.access(directPath);
    return directPath;
  } catch {
    // Not a direct path
  }

  // For now, we only support direct file paths, not globs
  // A more sophisticated implementation could use glob matching
  return null;
}

/**
 * Process a single learning
 */
async function processLearning(
  learning: Learning,
  modelConfig: ModelConfig,
  rl: readline.Interface,
  baseDir: string
): Promise<{ applied: boolean; targets: string[] }> {
  console.log('');
  console.log('='.repeat(60));
  console.log(`=== Learning: ${learning.id} - ${learning.title} ===`);
  console.log('='.repeat(60));
  console.log(`Source: ${path.relative(baseDir, learning.source)} (${learning.sourceType})`);
  console.log(`Category: ${learning.category}`);
  console.log(`Insight: ${learning.insight}`);
  console.log(`Suggested Action: ${learning.suggestedAction}`);
  console.log(`Applies To: ${learning.appliesTo}`);
  console.log('');

  // Parse target files
  const targetPatterns = parseAppliesTo(learning.appliesTo);
  if (targetPatterns.length === 0) {
    console.log('No specific targets defined. Skipping.');
    const answer = await prompt(rl, 'Mark as reviewed anyway? [y/n]: ');
    if (answer === 'y' || answer === 'yes') {
      return { applied: true, targets: ['reviewed'] };
    }
    return { applied: false, targets: [] };
  }

  const appliedTargets: string[] = [];

  for (const pattern of targetPatterns) {
    const filePath = await resolveFilePath(pattern, baseDir);
    if (!filePath) {
      console.log(`Target not found: ${pattern}`);
      continue;
    }

    console.log(`\nAnalyzing: ${pattern}`);

    let fileContent: string;
    try {
      fileContent = await fs.readFile(filePath, 'utf-8');
    } catch (error) {
      console.log(`  Cannot read file: ${error instanceof Error ? error.message : String(error)}`);
      continue;
    }

    // Get AI analysis
    let analysis: ApplyAnalysis;
    try {
      process.stdout.write('  Analyzing with AI...');
      analysis = await analyzeFile(learning, pattern, fileContent, modelConfig);
      console.log(' done');
    } catch (error) {
      console.log(` error: ${error instanceof Error ? error.message : String(error)}`);
      continue;
    }

    if (!analysis.shouldChange || analysis.changes.length === 0) {
      console.log(`  No changes needed: ${analysis.reason}`);
      continue;
    }

    console.log(`  Reason: ${analysis.reason}`);
    console.log(`  ${analysis.changes.length} suggested change(s):`);

    // Process each suggested change
    let currentContent = fileContent;
    let fileModified = false;

    for (let i = 0; i < analysis.changes.length; i++) {
      const change = analysis.changes[i];
      console.log('');
      console.log(`  Change ${i + 1}: ${change.description}`);
      console.log('');
      console.log(formatDiff(pattern, change).split('\n').map(l => '    ' + l).join('\n'));
      console.log('');

      const answer = await prompt(rl, '  [a]ccept / [r]eject / [s]kip learning / [q]uit? ');

      if (answer === 'q' || answer === 'quit') {
        console.log('Quitting...');
        process.exit(0);
      }

      if (answer === 's' || answer === 'skip') {
        console.log('  Skipping this learning');
        return { applied: false, targets: [] };
      }

      if (answer === 'a' || answer === 'accept') {
        const newContent = applyChange(currentContent, change);
        if (newContent === null) {
          console.log('  Could not apply change (text not found in file)');
        } else {
          currentContent = newContent;
          fileModified = true;
          console.log('  Change accepted');
        }
      } else {
        console.log('  Change rejected');
      }
    }

    // Write modified content if any changes were accepted
    if (fileModified) {
      try {
        await fs.writeFile(filePath, currentContent, 'utf-8');
        console.log(`  File updated: ${pattern}`);
        appliedTargets.push(pattern);
      } catch (error) {
        console.log(`  Failed to write file: ${error instanceof Error ? error.message : String(error)}`);
      }
    }
  }

  return {
    applied: appliedTargets.length > 0,
    targets: appliedTargets,
  };
}

/**
 * Main entry point
 */
async function main(): Promise<void> {
  const args = parseArgs(process.argv);
  const baseDir = process.cwd();

  // Setup paths
  const configPath = path.join(baseDir, 'verification/config.toml');

  // Get model configuration
  let modelConfig: ModelConfig;
  try {
    modelConfig = getModelConfig(configPath, args.modelOverride);
    console.log(`Using model: ${modelConfig.provider}:${modelConfig.model}`);
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }

  // Scan all learnings
  console.log('Scanning for learnings...');
  const allLearnings = await scanAllLearnings(baseDir);
  console.log(`Found ${allLearnings.length} total learnings`);

  // Filter to unapplied
  const pendingLearnings = filterUnappliedLearnings(allLearnings);
  console.log(`${pendingLearnings.length} pending (unapplied)`);

  if (pendingLearnings.length === 0) {
    console.log('');
    console.log('No pending learnings to apply.');
    return;
  }

  // Create readline interface
  const rl = createReadline();

  try {
    let appliedCount = 0;
    let skippedCount = 0;

    for (const learning of pendingLearnings) {
      const result = await processLearning(learning, modelConfig, rl, baseDir);

      if (result.applied && result.targets.length > 0) {
        // Mark learning as applied
        try {
          await markLearningAsApplied(learning, result.targets);
          console.log(`  Learning ${learning.id} marked as applied`);
          appliedCount++;
        } catch (error) {
          console.log(`  Warning: Could not mark learning as applied: ${error instanceof Error ? error.message : String(error)}`);
        }
      } else {
        skippedCount++;
      }
    }

    // Summary
    console.log('');
    console.log('='.repeat(60));
    console.log('=== Summary ===');
    console.log('='.repeat(60));
    console.log(`Total pending: ${pendingLearnings.length}`);
    console.log(`Applied: ${appliedCount}`);
    console.log(`Skipped: ${skippedCount}`);

  } finally {
    rl.close();
  }
}

// Run main
main().catch((error) => {
  console.error('Fatal error:', error);
  process.exit(1);
});
