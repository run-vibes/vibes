/**
 * Learning Scanner for AI-Powered Propagation
 *
 * Scans and parses learnings from stories, milestones, and ad-hoc files.
 */

import * as fs from 'node:fs/promises';
import * as path from 'node:path';

/**
 * Learning source types
 */
export type LearningSourceType = 'story' | 'milestone' | 'adhoc';

/**
 * A structured learning entry
 */
export interface Learning {
  id: string;                    // L001, ML001, or filename slug
  source: string;                // File path where learning lives
  sourceType: LearningSourceType;
  scope?: string;                // For stories: epic/milestone scope from frontmatter
  title: string;
  category: string;
  context: string;
  insight: string;
  suggestedAction: string;
  appliesTo: string;
  applied: string;               // Empty if not applied
}

/**
 * Result of applying a learning
 */
export interface ApplyResult {
  learning: Learning;
  targetFile: string;
  changes: Array<{
    description: string;
    oldText: string;
    newText: string;
  }>;
  applied: boolean;
  reason?: string;
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
 * Parse a learning table from markdown content.
 *
 * Learning tables have this format:
 * ```markdown
 * ### L001: Title
 *
 * | Field | Value |
 * |-------|-------|
 * | **Category** | value |
 * | **Context** | value |
 * | **Insight** | value |
 * | **Suggested Action** | value |
 * | **Applies To** | value |
 * | **Applied** | value |
 * ```
 */
function parseLearningTable(content: string, headerMatch: RegExpMatchArray): Learning | null {
  const id = headerMatch[1];
  const title = headerMatch[2]?.trim() || '';

  // Find the table that follows this header
  const headerEnd = (headerMatch.index ?? 0) + headerMatch[0].length;
  const remainingContent = content.slice(headerEnd);

  // Extract table rows
  const tableMatch = remainingContent.match(/\|[^|]+\|[^|]+\|/g);
  if (!tableMatch) {
    return null;
  }

  // Parse table rows into a map
  const fields: Record<string, string> = {};
  for (const row of tableMatch) {
    // Skip header and separator rows
    if (row.includes('---') || row.includes('Field')) {
      continue;
    }

    // Extract field name and value
    const match = row.match(/\|\s*\*\*(\w+(?:\s+\w+)?)\*\*\s*\|\s*([^|]*)\|/);
    if (match) {
      const fieldName = match[1].toLowerCase().replace(/\s+/g, '');
      const value = match[2].trim();
      fields[fieldName] = value;
    }
  }

  // Return null if required fields are missing
  if (!fields.insight) {
    return null;
  }

  return {
    id,
    source: '', // Will be set by caller
    sourceType: 'story', // Will be set by caller
    title,
    category: fields.category || 'unknown',
    context: fields.context || '',
    insight: fields.insight || '',
    suggestedAction: fields.suggestedaction || '',
    appliesTo: fields.appliesto || '',
    applied: fields.applied || '',
  };
}

/**
 * Parse all learnings from a markdown file content.
 */
function parseLearningsFromContent(
  content: string,
  source: string,
  sourceType: LearningSourceType
): Learning[] {
  const learnings: Learning[] = [];

  // Match learning headers: ### L001: Title or ### ML001: Title
  const headerRegex = /^###\s+(L\d+|ML\d+):\s*(.*)$/gm;

  let match;
  while ((match = headerRegex.exec(content)) !== null) {
    const learning = parseLearningTable(content, match);
    if (learning) {
      learning.source = source;
      learning.sourceType = sourceType;
      learnings.push(learning);
    }
  }

  return learnings;
}

/**
 * Extract scope from story frontmatter
 */
function extractScopeFromFrontmatter(content: string): string | undefined {
  const scopeMatch = content.match(/^scope:\s*(.+)$/m);
  return scopeMatch ? scopeMatch[1].trim() : undefined;
}

/**
 * Scan story files in done stage for learnings.
 */
async function scanStoryLearnings(baseDir: string): Promise<Learning[]> {
  const learnings: Learning[] = [];
  const storiesDir = path.join(baseDir, 'docs/board/stages/done/stories');

  const files = await findMarkdownFiles(storiesDir);

  for (const file of files) {
    try {
      const content = await fs.readFile(file, 'utf-8');

      // Check if file has a Learnings section
      const learningSectionMatch = content.match(/## Learnings\s*\n([\s\S]*?)(?=\n##\s|\n---\s|$)/i);
      if (learningSectionMatch) {
        const sectionContent = learningSectionMatch[1];
        const storyLearnings = parseLearningsFromContent(sectionContent, file, 'story');

        // Extract scope from frontmatter and attach to learnings
        const scope = extractScopeFromFrontmatter(content);
        for (const learning of storyLearnings) {
          learning.scope = scope;
        }

        learnings.push(...storyLearnings);
      }
    } catch {
      // Skip files that can't be read
    }
  }

  return learnings;
}

/**
 * Scan milestone LEARNINGS.md files.
 */
async function scanMilestoneLearnings(baseDir: string): Promise<Learning[]> {
  const learnings: Learning[] = [];
  const epicsDir = path.join(baseDir, 'docs/board/epics');

  try {
    const epicEntries = await fs.readdir(epicsDir, { withFileTypes: true });

    for (const epicEntry of epicEntries) {
      if (!epicEntry.isDirectory()) continue;

      const milestonesDir = path.join(epicsDir, epicEntry.name, 'milestones');

      try {
        const milestoneEntries = await fs.readdir(milestonesDir, { withFileTypes: true });

        for (const milestoneEntry of milestoneEntries) {
          if (!milestoneEntry.isDirectory()) continue;

          const learningsFile = path.join(milestonesDir, milestoneEntry.name, 'LEARNINGS.md');

          try {
            const content = await fs.readFile(learningsFile, 'utf-8');
            const milestoneLearnings = parseLearningsFromContent(content, learningsFile, 'milestone');
            learnings.push(...milestoneLearnings);
          } catch {
            // LEARNINGS.md doesn't exist for this milestone
          }
        }
      } catch {
        // Milestones directory doesn't exist
      }
    }
  } catch {
    // Epics directory doesn't exist
  }

  return learnings;
}

/**
 * Scan ad-hoc learnings from docs/learnings/.
 */
async function scanAdhocLearnings(baseDir: string): Promise<Learning[]> {
  const learnings: Learning[] = [];
  const learningsDir = path.join(baseDir, 'docs/learnings');

  try {
    const files = await findMarkdownFiles(learningsDir);

    for (const file of files) {
      try {
        const content = await fs.readFile(file, 'utf-8');
        const adhocLearnings = parseLearningsFromContent(content, file, 'adhoc');
        learnings.push(...adhocLearnings);
      } catch {
        // Skip files that can't be read
      }
    }
  } catch {
    // Learnings directory doesn't exist
  }

  return learnings;
}

/**
 * Scan all learning sources and return all learnings.
 */
export async function scanAllLearnings(baseDir: string = process.cwd()): Promise<Learning[]> {
  const [storyLearnings, milestoneLearnings, adhocLearnings] = await Promise.all([
    scanStoryLearnings(baseDir),
    scanMilestoneLearnings(baseDir),
    scanAdhocLearnings(baseDir),
  ]);

  return [...storyLearnings, ...milestoneLearnings, ...adhocLearnings];
}

/**
 * Filter learnings to only unapplied ones.
 */
export function filterUnappliedLearnings(learnings: Learning[]): Learning[] {
  return learnings.filter(learning => !learning.applied || learning.applied.trim() === '');
}

/**
 * Parse the "Applies To" field to extract target file patterns.
 *
 * Examples:
 * - "CLAUDE.md" -> ["CLAUDE.md"]
 * - "tests/*.ts" -> ["tests/*.ts"]
 * - "CLAUDE.md, CONVENTIONS.md" -> ["CLAUDE.md", "CONVENTIONS.md"]
 * - "To be determined" -> []
 */
export function parseAppliesTo(appliesTo: string): string[] {
  if (!appliesTo || appliesTo.toLowerCase().includes('to be determined')) {
    return [];
  }

  // Split by comma and clean up
  return appliesTo
    .split(/[,;]/)
    .map(s => s.trim())
    .filter(s => s.length > 0 && !s.toLowerCase().includes('none'));
}

/**
 * Update a learning's "Applied" field in its source file.
 */
export async function markLearningAsApplied(
  learning: Learning,
  appliedTargets: string[]
): Promise<void> {
  const content = await fs.readFile(learning.source, 'utf-8');

  // Find the learning by ID and update its Applied field
  const date = new Date().toISOString().split('T')[0];
  const appliedValue = appliedTargets.map(t => `${t} (${date})`).join(', ');

  // Build regex to find the Applied row for this specific learning
  // We need to find the learning header first, then find the Applied row within its table
  const learningHeaderPattern = new RegExp(
    `(###\\s+${escapeRegex(learning.id)}:[\\s\\S]*?\\|\\s*\\*\\*Applied\\*\\*\\s*\\|)\\s*([^|]*)\\|`,
    'm'
  );

  const match = content.match(learningHeaderPattern);
  if (!match) {
    throw new Error(`Could not find Applied field for learning ${learning.id} in ${learning.source}`);
  }

  // Replace just the Applied value
  const newContent = content.replace(
    learningHeaderPattern,
    `$1 ${appliedValue} |`
  );

  await fs.writeFile(learning.source, newContent, 'utf-8');
}

/**
 * Escape special regex characters in a string.
 */
function escapeRegex(str: string): string {
  return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

/**
 * Get a summary of learning counts by source type.
 */
export function getLearningsSummary(learnings: Learning[]): {
  total: number;
  byType: Record<LearningSourceType, number>;
  applied: number;
  pending: number;
} {
  const byType: Record<LearningSourceType, number> = {
    story: 0,
    milestone: 0,
    adhoc: 0,
  };

  let applied = 0;

  for (const learning of learnings) {
    byType[learning.sourceType]++;
    if (learning.applied && learning.applied.trim()) {
      applied++;
    }
  }

  return {
    total: learnings.length,
    byType,
    applied,
    pending: learnings.length - applied,
  };
}
