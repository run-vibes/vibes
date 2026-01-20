#!/usr/bin/env npx tsx
/**
 * Learning Show Command
 *
 * Shows full details of a specific learning.
 *
 * Usage:
 *   npx tsx learn-show.ts <path>
 *
 * Path format:
 *   scope/story/id     - e.g., coherence-verification/05-learnings-capture/CHORE0202/L001
 *   story/id           - e.g., FEAT0199/L001 (searches all scopes)
 *   id                 - e.g., L001 (shows all learnings with that ID)
 *
 * Examples:
 *   npx tsx learn-show.ts coherence-verification/05-learnings-capture/CHORE0202/L001
 *   npx tsx learn-show.ts FEAT0199/L001
 *   npx tsx learn-show.ts L001
 */

import * as path from 'node:path';
import { scanAllLearnings, type Learning } from './lib/learnings.js';

/**
 * Parse the learning path into components
 */
function parseLearningPath(learningPath: string): {
  scope?: string;
  storyId?: string;
  learningId?: string;
} {
  const parts = learningPath.split('/');

  // Single part: just learning ID (L001)
  if (parts.length === 1) {
    return { learningId: parts[0] };
  }

  // Two parts: story/id (FEAT0199/L001)
  if (parts.length === 2) {
    return { storyId: parts[0], learningId: parts[1] };
  }

  // Four parts: epic/milestone/story/id
  if (parts.length === 4) {
    return {
      scope: `${parts[0]}/${parts[1]}`,
      storyId: parts[2],
      learningId: parts[3],
    };
  }

  // Three parts could be: milestone/story/id or epic/milestone/story (missing id)
  if (parts.length === 3) {
    // If last part looks like a learning ID (L001, ML001)
    if (/^[LM]L?\d+$/i.test(parts[2])) {
      return {
        scope: parts[0], // Could be just milestone
        storyId: parts[1],
        learningId: parts[2],
      };
    }
    // Otherwise treat as scope/story with no ID
    return {
      scope: `${parts[0]}/${parts[1]}`,
      storyId: parts[2],
    };
  }

  return { learningId: learningPath };
}

/**
 * Extract story ID from source path
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
 * Match a learning against the search criteria
 */
function matchesLearning(
  learning: Learning,
  criteria: { scope?: string; storyId?: string; learningId?: string }
): boolean {
  // Match learning ID
  if (criteria.learningId && learning.id.toLowerCase() !== criteria.learningId.toLowerCase()) {
    return false;
  }

  // Match story ID
  if (criteria.storyId) {
    const storyId = extractStoryId(learning.source);
    if (storyId.toLowerCase() !== criteria.storyId.toLowerCase()) {
      return false;
    }
  }

  // Match scope
  if (criteria.scope) {
    const learningScope = learning.scope || '';
    // Allow partial match (e.g., "05-learnings" matches "coherence-verification/05-learnings-capture")
    if (!learningScope.toLowerCase().includes(criteria.scope.toLowerCase())) {
      return false;
    }
  }

  return true;
}

/**
 * Print a learning in full detail
 */
function printLearning(learning: Learning): void {
  const storyId = extractStoryId(learning.source);

  console.log('');
  console.log('═'.repeat(80));
  console.log(`  ${learning.id}: ${learning.title}`);
  console.log('═'.repeat(80));
  console.log('');

  console.log(`Source:     ${learning.sourceType}`);
  if (learning.sourceType === 'story') {
    console.log(`Story:      ${storyId}`);
  }
  if (learning.scope) {
    console.log(`Scope:      ${learning.scope}`);
  }
  console.log(`Category:   ${learning.category}`);
  console.log(`Status:     ${learning.applied ? 'applied' : 'pending'}`);
  console.log('');

  console.log('─'.repeat(80));
  console.log('Context:');
  console.log('─'.repeat(80));
  console.log(learning.context || '(none)');
  console.log('');

  console.log('─'.repeat(80));
  console.log('Insight:');
  console.log('─'.repeat(80));
  // Parse and format the insight which may contain "What went well:", etc.
  const insight = learning.insight;
  if (insight.includes('**What went well:**')) {
    // Split by bullet separator
    const parts = insight.split(' • ');
    for (const part of parts) {
      console.log(part.trim());
    }
  } else {
    console.log(insight);
  }
  console.log('');

  console.log('─'.repeat(80));
  console.log('Suggested Action:');
  console.log('─'.repeat(80));
  console.log(learning.suggestedAction || '(none)');
  console.log('');

  console.log('─'.repeat(80));
  console.log('Applies To:');
  console.log('─'.repeat(80));
  console.log(learning.appliesTo || '(to be determined)');
  console.log('');

  if (learning.applied) {
    console.log('─'.repeat(80));
    console.log('Applied:');
    console.log('─'.repeat(80));
    console.log(learning.applied);
    console.log('');
  }

  console.log(`File: ${learning.source}`);
  console.log('');
}

/**
 * Main entry point
 */
async function main(): Promise<void> {
  const args = process.argv.slice(2);

  if (args.length === 0) {
    console.log('Usage: just learn show <path>');
    console.log('');
    console.log('Path format:');
    console.log('  scope/story/id  - e.g., coherence-verification/05-learnings-capture/CHORE0202/L001');
    console.log('  story/id        - e.g., FEAT0199/L001');
    console.log('  id              - e.g., L001 (shows all with that ID)');
    process.exit(1);
  }

  const learningPath = args[0];
  const criteria = parseLearningPath(learningPath);
  const baseDir = process.cwd();

  // Scan all learnings
  const learnings = await scanAllLearnings(baseDir);

  // Filter by criteria
  const matches = learnings.filter(l => matchesLearning(l, criteria));

  if (matches.length === 0) {
    console.log(`No learnings found matching: ${learningPath}`);
    console.log('');
    console.log('Available learnings:');
    for (const l of learnings.slice(0, 10)) {
      const storyId = extractStoryId(l.source);
      console.log(`  ${l.scope ? l.scope + '/' : ''}${storyId}/${l.id}`);
    }
    if (learnings.length > 10) {
      console.log(`  ... and ${learnings.length - 10} more`);
    }
    process.exit(1);
  }

  // Print all matches
  for (const learning of matches) {
    printLearning(learning);
  }

  if (matches.length > 1) {
    console.log(`Found ${matches.length} learnings matching: ${learningPath}`);
  }
}

// Run main
main().catch((error) => {
  console.error('Error:', error instanceof Error ? error.message : String(error));
  process.exit(1);
});
