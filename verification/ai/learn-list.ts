#!/usr/bin/env npx tsx
/**
 * Learning List Command
 *
 * Lists all learnings from stories, milestones, and ad-hoc files with status.
 *
 * Usage:
 *   npx tsx learn-list.ts [--pending] [--category <cat>]
 *
 * Options:
 *   --pending          Only show pending (unapplied) learnings
 *   --category <cat>   Filter by category
 *
 * Examples:
 *   npx tsx learn-list.ts
 *   npx tsx learn-list.ts --pending
 *   npx tsx learn-list.ts --category code
 *   npx tsx learn-list.ts --pending --category arch
 */

import * as path from 'node:path';
import {
  scanAllLearnings,
  getLearningsSummary,
  type Learning,
  type LearningSourceType,
} from './lib/learnings.js';

/**
 * Parse command line arguments
 */
function parseArgs(args: string[]): { pendingOnly: boolean; categoryFilter: string | null } {
  const relevantArgs = args.slice(2); // Skip node and script path

  const pendingOnly = relevantArgs.includes('--pending');

  const categoryIndex = relevantArgs.indexOf('--category');
  const categoryFilter = categoryIndex >= 0 && categoryIndex + 1 < relevantArgs.length
    ? relevantArgs[categoryIndex + 1]
    : null;

  return { pendingOnly, categoryFilter };
}

/**
 * Determine status of a learning
 */
function getStatus(learning: Learning): 'applied' | 'pending' {
  return learning.applied && learning.applied.trim() ? 'applied' : 'pending';
}

/**
 * Truncate text to fit in table column
 */
function truncate(text: string, maxLength: number): string {
  if (text.length <= maxLength) {
    return text;
  }
  return text.substring(0, maxLength - 3) + '...';
}

/**
 * Pad string to specified length
 */
function padEnd(text: string, length: number): string {
  if (text.length >= length) {
    return text;
  }
  return text + ' '.repeat(length - text.length);
}

/**
 * Format learning ID based on source type
 */
function formatId(learning: Learning): string {
  if (learning.sourceType === 'adhoc') {
    // For ad-hoc learnings, use filename-based ID
    const filename = path.basename(learning.source, '.md');
    return filename;
  }
  return learning.id;
}

/**
 * Extract story ID from filename
 * [FEAT][0208]-name.md -> "FEAT0208"
 */
function extractStoryId(sourcePath: string): string {
  const filename = path.basename(sourcePath, '.md');
  const match = filename.match(/\[(\w+)\]\[(\d+)\]/);
  if (match) {
    return `${match[1]}${match[2]}`;
  }
  return filename;
}

/**
 * Extract source identifier (story ID, milestone name, or filename)
 */
function extractSourceId(learning: Learning): string {
  if (learning.sourceType === 'story') {
    return extractStoryId(learning.source);
  }

  if (learning.sourceType === 'milestone') {
    // Extract just milestone name from path like .../epics/epic-name/milestones/01-name/LEARNINGS.md
    const parts = learning.source.split(path.sep);
    const epicsIndex = parts.indexOf('epics');
    if (epicsIndex >= 0 && parts.length > epicsIndex + 3) {
      return parts[epicsIndex + 3]; // milestone name
    }
    return path.basename(path.dirname(learning.source));
  }

  if (learning.sourceType === 'adhoc') {
    return path.basename(learning.source, '.md');
  }

  return path.basename(learning.source, '.md');
}

/**
 * Extract scope (epic for milestones, epic/milestone for stories)
 */
function extractScope(learning: Learning): string {
  if (learning.sourceType === 'milestone') {
    // Extract epic name from path for milestone learnings
    const parts = learning.source.split(path.sep);
    const epicsIndex = parts.indexOf('epics');
    if (epicsIndex >= 0 && parts.length > epicsIndex + 1) {
      return parts[epicsIndex + 1]; // epic name
    }
    return '-';
  }

  if (learning.scope) {
    // Return full scope (e.g., "coherence-verification/04-ai-assisted-verification")
    return learning.scope;
  }
  return '-';
}

/**
 * Print a table of learnings
 */
function printTable(learnings: Learning[], title: string): void {
  if (learnings.length === 0) {
    console.log(`\n--- ${title} (0) ---`);
    console.log('No learnings found.');
    return;
  }

  console.log(`\n--- ${title} (${learnings.length}) ---`);

  // Determine column widths and headers based on source type
  const sourceType = learnings[0]?.sourceType;
  const isAdhoc = sourceType === 'adhoc';
  const isMilestone = sourceType === 'milestone';

  const idColWidth = isAdhoc ? 20 : 6;
  const sourceColWidth = isMilestone ? 28 : 12;
  const scopeColWidth = isMilestone ? 24 : 48;
  const titleColWidth = 26;
  const categoryColWidth = 10;
  const statusColWidth = 8;

  // Column header varies by source type
  const sourceHeader = isMilestone ? 'Milestone' : (isAdhoc ? 'File' : 'Story');
  const scopeHeader = isMilestone ? 'Epic' : 'Scope';

  // Header
  const header = [
    padEnd('ID', idColWidth),
    padEnd(sourceHeader, sourceColWidth),
    padEnd(scopeHeader, scopeColWidth),
    padEnd('Title', titleColWidth),
    padEnd('Category', categoryColWidth),
    padEnd('Status', statusColWidth),
  ].join(' | ');

  const separator = [
    '-'.repeat(idColWidth),
    '-'.repeat(sourceColWidth),
    '-'.repeat(scopeColWidth),
    '-'.repeat(titleColWidth),
    '-'.repeat(categoryColWidth),
    '-'.repeat(statusColWidth),
  ].join('-+-');

  console.log(`| ${header} |`);
  console.log(`|-${separator}-|`);

  // Rows
  for (const learning of learnings) {
    const id = truncate(formatId(learning), idColWidth);
    const source = truncate(extractSourceId(learning), sourceColWidth);
    const scope = truncate(extractScope(learning), scopeColWidth);
    const titleText = truncate(learning.title || '(untitled)', titleColWidth);
    const category = truncate(learning.category, categoryColWidth);
    const status = getStatus(learning);

    const row = [
      padEnd(id, idColWidth),
      padEnd(source, sourceColWidth),
      padEnd(scope, scopeColWidth),
      padEnd(titleText, titleColWidth),
      padEnd(category, categoryColWidth),
      padEnd(status, statusColWidth),
    ].join(' | ');

    console.log(`| ${row} |`);
  }
}

/**
 * Main entry point
 */
async function main(): Promise<void> {
  const { pendingOnly, categoryFilter } = parseArgs(process.argv);
  const baseDir = process.cwd();

  // Scan all learnings
  let learnings = await scanAllLearnings(baseDir);

  // Apply filters
  if (pendingOnly) {
    learnings = learnings.filter(l => !l.applied || !l.applied.trim());
  }

  if (categoryFilter) {
    learnings = learnings.filter(
      l => l.category.toLowerCase() === categoryFilter.toLowerCase()
    );
  }

  // Get summary stats (before filtering for accurate totals)
  const allLearnings = await scanAllLearnings(baseDir);
  const summary = getLearningsSummary(allLearnings);

  // Print header
  console.log('=== Learnings Summary ===');
  console.log('');

  // Show filter status if any filters are active
  const filters: string[] = [];
  if (pendingOnly) filters.push('pending only');
  if (categoryFilter) filters.push(`category: ${categoryFilter}`);

  if (filters.length > 0) {
    console.log(`Filters: ${filters.join(', ')}`);
    console.log(`Showing: ${learnings.length} of ${summary.total} learnings`);
  } else {
    console.log(`Total: ${summary.total} learnings (${summary.pending} pending, ${summary.applied} applied)`);
  }

  // Group by source type
  const storyLearnings = learnings.filter(l => l.sourceType === 'story');
  const milestoneLearnings = learnings.filter(l => l.sourceType === 'milestone');
  const adhocLearnings = learnings.filter(l => l.sourceType === 'adhoc');

  // Print tables
  printTable(storyLearnings, 'Story Learnings');
  printTable(milestoneLearnings, 'Milestone Learnings');
  printTable(adhocLearnings, 'Ad-hoc Learnings');

  console.log('');
}

// Run main
main().catch((error) => {
  console.error('Error:', error instanceof Error ? error.message : String(error));
  process.exit(1);
});
