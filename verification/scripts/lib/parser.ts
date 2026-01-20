/**
 * Story Parser for AI Verification
 *
 * Extracts acceptance criteria and verification annotations from story markdown files.
 */

/**
 * Annotation types that can be used to link criteria to artifacts
 */
export type AnnotationType = 'snapshot' | 'checkpoint' | 'video';

/**
 * A verification annotation parsed from a criterion
 */
export interface Annotation {
  type: AnnotationType;
  name: string;
  hint?: string;
}

/**
 * A single acceptance criterion from a story
 */
export interface Criterion {
  text: string;
  annotation?: Annotation;
}

/**
 * A parsed story with extracted metadata and criteria
 */
export interface ParsedStory {
  id: string;
  title: string;
  scope: string;
  criteria: Criterion[];
}

/**
 * Pattern information for finding a story file by ID
 */
export interface StoryPattern {
  typeCode: string;
  number: string;
  globPattern: string;
}

/**
 * Parse a verify annotation from a comment string.
 *
 * Format: `<!-- verify: type:name -->` or `<!-- verify: type:name | hint -->`
 *
 * @param comment - The HTML comment string to parse
 * @returns The parsed annotation or undefined if not a verify annotation
 */
export function parseAnnotation(comment: string): Annotation | undefined {
  // Match: <!-- verify: type:name --> or <!-- verify: type:name | hint -->
  const match = comment.match(
    /<!--\s*verify:\s*(\w+):([^\s|>]+)(?:\s*\|\s*([^>]+?))?\s*-->/
  );

  if (!match) {
    return undefined;
  }

  const [, typeStr, name, hint] = match;
  const type = typeStr.toLowerCase();

  // Validate annotation type
  if (type !== 'snapshot' && type !== 'checkpoint' && type !== 'video') {
    return undefined;
  }

  return {
    type: type as AnnotationType,
    name: name.trim(),
    hint: hint?.trim(),
  };
}

/**
 * Parse acceptance criteria from markdown content.
 *
 * Looks for a `## Acceptance Criteria` section and extracts all list items.
 * Stops at the next heading or end of content.
 *
 * @param markdown - The full markdown content
 * @returns Array of parsed criteria
 */
export function parseAcceptanceCriteria(markdown: string): Criterion[] {
  const criteria: Criterion[] = [];

  // Find the Acceptance Criteria section
  const sectionMatch = markdown.match(/## Acceptance Criteria\s*\n([\s\S]*?)(?=\n##\s|\n---\s|$)/i);
  if (!sectionMatch) {
    return criteria;
  }

  const sectionContent = sectionMatch[1];

  // Match list items: - [ ] or - [x] followed by text
  const listItemRegex = /^-\s*\[[x ]\]\s*(.+?)(?:\s*(<!--[^>]+-->))?$/gm;

  let match;
  while ((match = listItemRegex.exec(sectionContent)) !== null) {
    const [, textWithPossibleAnnotation, explicitAnnotation] = match;

    // Check if annotation is inline (not captured separately)
    let text = textWithPossibleAnnotation.trim();
    let annotationStr = explicitAnnotation;

    // If no explicit annotation captured, check for inline annotation in text
    const inlineAnnotationMatch = text.match(/^(.+?)\s*(<!--.+-->)$/);
    if (inlineAnnotationMatch && !annotationStr) {
      text = inlineAnnotationMatch[1].trim();
      annotationStr = inlineAnnotationMatch[2];
    }

    const criterion: Criterion = { text };

    if (annotationStr) {
      const annotation = parseAnnotation(annotationStr);
      if (annotation) {
        criterion.annotation = annotation;
      }
    }

    criteria.push(criterion);
  }

  return criteria;
}

/**
 * Parse YAML frontmatter from markdown content.
 *
 * @param content - The full file content
 * @returns Object with frontmatter fields
 */
function parseFrontmatter(content: string): Record<string, string> {
  const frontmatter: Record<string, string> = {};

  const match = content.match(/^---\s*\n([\s\S]*?)\n---/);
  if (!match) {
    return frontmatter;
  }

  const yamlContent = match[1];
  const lines = yamlContent.split('\n');

  for (const line of lines) {
    const colonIndex = line.indexOf(':');
    if (colonIndex > 0) {
      const key = line.slice(0, colonIndex).trim();
      let value = line.slice(colonIndex + 1).trim();
      // Remove surrounding quotes if present
      if ((value.startsWith('"') && value.endsWith('"')) ||
          (value.startsWith("'") && value.endsWith("'"))) {
        value = value.slice(1, -1);
      }
      frontmatter[key] = value;
    }
  }

  return frontmatter;
}

/**
 * Parse a story file's content into a structured format.
 *
 * Extracts frontmatter (id, title, scope) and acceptance criteria.
 *
 * @param content - The full story file content
 * @returns Parsed story object
 */
export function parseStoryFile(content: string): ParsedStory {
  const frontmatter = parseFrontmatter(content);
  const criteria = parseAcceptanceCriteria(content);

  return {
    id: frontmatter.id || 'unknown',
    title: frontmatter.title || 'Untitled',
    scope: frontmatter.scope || 'unknown',
    criteria,
  };
}

/**
 * Generate the file pattern for finding a story by ID.
 *
 * Story files follow the pattern: [TYPE][NNNN]-name.md
 * e.g., FEAT0109 -> [FEAT][0109]-*.md
 *
 * @param storyId - The story ID (e.g., "FEAT0109", "CHORE0146")
 * @returns Pattern information for locating the story file
 * @throws Error if the story ID format is invalid
 */
export function findStoryFile(storyId: string): StoryPattern {
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

  return {
    typeCode,
    number: paddedNumber,
    globPattern: `docs/board/stages/**/stories/[${typeCode}][${paddedNumber}]*.md`,
  };
}
