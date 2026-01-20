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
 * Print a table of learnings
 */
function printTable(learnings: Learning[], title: string): void {
  if (learnings.length === 0) {
    console.log(`\n--- ${title} (0) ---`);
    console.log('No learnings found.');
    return;
  }

  console.log(`\n--- ${title} (${learnings.length}) ---`);

  // Determine column widths based on content
  const isAdhoc = learnings[0]?.sourceType === 'adhoc';
  const idColWidth = isAdhoc ? 28 : 6;
  const titleColWidth = 30;
  const categoryColWidth = 10;
  const statusColWidth = 8;
  const appliesToColWidth = 25;

  // Header
  const header = [
    padEnd('ID', idColWidth),
    padEnd('Title', titleColWidth),
    padEnd('Category', categoryColWidth),
    padEnd('Status', statusColWidth),
    padEnd('Applies To', appliesToColWidth),
  ].join(' | ');

  const separator = [
    '-'.repeat(idColWidth),
    '-'.repeat(titleColWidth),
    '-'.repeat(categoryColWidth),
    '-'.repeat(statusColWidth),
    '-'.repeat(appliesToColWidth),
  ].join('-+-');

  console.log(`| ${header} |`);
  console.log(`|-${separator}-|`);

  // Rows
  for (const learning of learnings) {
    const id = truncate(formatId(learning), idColWidth);
    const title = truncate(learning.title || '(untitled)', titleColWidth);
    const category = truncate(learning.category, categoryColWidth);
    const status = getStatus(learning);
    const appliesTo = truncate(learning.appliesTo || '-', appliesToColWidth);

    const row = [
      padEnd(id, idColWidth),
      padEnd(title, titleColWidth),
      padEnd(category, categoryColWidth),
      padEnd(status, statusColWidth),
      padEnd(appliesTo, appliesToColWidth),
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
