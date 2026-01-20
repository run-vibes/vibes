import { describe, test, expect } from 'vitest';
import {
  parseStoryFile,
  findStoryFile,
  parseAcceptanceCriteria,
  parseAnnotation,
  type ParsedStory,
  type Criterion,
} from './parser.js';

describe('Story Parser', () => {
  describe('parseAnnotation', () => {
    test('parses basic annotation', () => {
      const result = parseAnnotation('<!-- verify: snapshot:sessions -->');
      expect(result).toEqual({
        type: 'snapshot',
        name: 'sessions',
        hint: undefined,
      });
    });

    test('parses annotation with hint', () => {
      const result = parseAnnotation('<!-- verify: snapshot:sessions | should show 3 sessions -->');
      expect(result).toEqual({
        type: 'snapshot',
        name: 'sessions',
        hint: 'should show 3 sessions',
      });
    });

    test('parses checkpoint annotation', () => {
      const result = parseAnnotation('<!-- verify: checkpoint:navigate-groove-dashboard -->');
      expect(result).toEqual({
        type: 'checkpoint',
        name: 'navigate-groove-dashboard',
        hint: undefined,
      });
    });

    test('parses video annotation', () => {
      const result = parseAnnotation('<!-- verify: video:dashboard-walkthrough | should navigate all tabs -->');
      expect(result).toEqual({
        type: 'video',
        name: 'dashboard-walkthrough',
        hint: 'should navigate all tabs',
      });
    });

    test('returns undefined for non-verify comments', () => {
      const result = parseAnnotation('<!-- this is a regular comment -->');
      expect(result).toBeUndefined();
    });

    test('returns undefined for invalid annotation format', () => {
      const result = parseAnnotation('<!-- verify: invalid -->');
      expect(result).toBeUndefined();
    });

    test('handles extra whitespace in annotation', () => {
      const result = parseAnnotation('<!--  verify:  snapshot:sessions  |  should show stuff  -->');
      expect(result).toEqual({
        type: 'snapshot',
        name: 'sessions',
        hint: 'should show stuff',
      });
    });
  });

  describe('parseAcceptanceCriteria', () => {
    test('extracts criteria from markdown', () => {
      const markdown = `
## Acceptance Criteria

- [ ] First criterion
- [ ] Second criterion
- [ ] Third criterion

## Other Section
`;
      const result = parseAcceptanceCriteria(markdown);
      expect(result).toHaveLength(3);
      expect(result[0].text).toBe('First criterion');
      expect(result[1].text).toBe('Second criterion');
      expect(result[2].text).toBe('Third criterion');
    });

    test('extracts criteria with annotations', () => {
      const markdown = `
## Acceptance Criteria

- [ ] Sessions page displays list <!-- verify: snapshot:sessions -->
- [ ] User can click button <!-- verify: checkpoint:click-flow | button should be visible -->
- [ ] Third criterion without annotation

## Other Section
`;
      const result = parseAcceptanceCriteria(markdown);
      expect(result).toHaveLength(3);

      expect(result[0].text).toBe('Sessions page displays list');
      expect(result[0].annotation).toEqual({
        type: 'snapshot',
        name: 'sessions',
        hint: undefined,
      });

      expect(result[1].text).toBe('User can click button');
      expect(result[1].annotation).toEqual({
        type: 'checkpoint',
        name: 'click-flow',
        hint: 'button should be visible',
      });

      expect(result[2].text).toBe('Third criterion without annotation');
      expect(result[2].annotation).toBeUndefined();
    });

    test('handles checked criteria', () => {
      const markdown = `
## Acceptance Criteria

- [x] Already done
- [ ] Not done yet
`;
      const result = parseAcceptanceCriteria(markdown);
      expect(result).toHaveLength(2);
      expect(result[0].text).toBe('Already done');
      expect(result[1].text).toBe('Not done yet');
    });

    test('returns empty array for missing acceptance criteria section', () => {
      const markdown = `
# Story Title

## Summary

Some summary text

## Tasks

- Task 1
- Task 2
`;
      const result = parseAcceptanceCriteria(markdown);
      expect(result).toEqual([]);
    });

    test('stops at next heading', () => {
      const markdown = `
## Acceptance Criteria

- [ ] First criterion
- [ ] Second criterion

## Implementation Notes

- [ ] This is not a criterion
`;
      const result = parseAcceptanceCriteria(markdown);
      expect(result).toHaveLength(2);
      expect(result[0].text).toBe('First criterion');
      expect(result[1].text).toBe('Second criterion');
    });

    test('handles multi-line criteria text', () => {
      const markdown = `
## Acceptance Criteria

- [ ] First criterion that spans
  multiple lines should be combined
- [ ] Second criterion
`;
      const result = parseAcceptanceCriteria(markdown);
      expect(result).toHaveLength(2);
      // Should only include the first line of the criterion
      expect(result[0].text).toBe('First criterion that spans');
    });
  });

  describe('parseStoryFile', () => {
    test('extracts frontmatter and criteria', () => {
      const content = `---
id: FEAT0109
title: Board generator grouped layout
type: feat
status: done
priority: high
scope: coherence-verification/02-epic-based-project-hierarchy
depends: []
estimate: 2h
created: 2026-01-17
---

# Board generator grouped layout

## Summary

Some summary text.

## Acceptance Criteria

- [ ] README shows In Progress, Backlog, Icebox, Epics, Done sections
- [ ] Epics section groups milestones under H3 headers <!-- verify: snapshot:board-readme | should show grouped layout -->
- [ ] All links resolve correctly

## Completion

After all acceptance criteria are met...
`;
      const result = parseStoryFile(content);
      expect(result.id).toBe('FEAT0109');
      expect(result.title).toBe('Board generator grouped layout');
      expect(result.scope).toBe('coherence-verification/02-epic-based-project-hierarchy');
      expect(result.criteria).toHaveLength(3);
      expect(result.criteria[1].annotation).toEqual({
        type: 'snapshot',
        name: 'board-readme',
        hint: 'should show grouped layout',
      });
    });

    test('handles story without acceptance criteria', () => {
      const content = `---
id: CHORE0001
title: Some chore
type: chore
status: backlog
scope: test/scope
---

# Some chore

## Summary

Just some task without acceptance criteria.
`;
      const result = parseStoryFile(content);
      expect(result.id).toBe('CHORE0001');
      expect(result.title).toBe('Some chore');
      expect(result.scope).toBe('test/scope');
      expect(result.criteria).toEqual([]);
    });

    test('handles missing scope with default', () => {
      const content = `---
id: FEAT0001
title: Some feature
type: feat
status: backlog
---

# Some feature

## Acceptance Criteria

- [ ] Something works
`;
      const result = parseStoryFile(content);
      expect(result.id).toBe('FEAT0001');
      expect(result.scope).toBe('unknown');
    });
  });

  describe('findStoryFile', () => {
    test('generates correct pattern for FEAT story', () => {
      const pattern = findStoryFile('FEAT0109');
      expect(pattern.typeCode).toBe('FEAT');
      expect(pattern.number).toBe('0109');
      expect(pattern.globPattern).toContain('[FEAT][0109]');
    });

    test('generates correct pattern for CHORE story', () => {
      const pattern = findStoryFile('CHORE0146');
      expect(pattern.typeCode).toBe('CHORE');
      expect(pattern.number).toBe('0146');
      expect(pattern.globPattern).toContain('[CHORE][0146]');
    });

    test('generates correct pattern for BUG story', () => {
      const pattern = findStoryFile('BUG0001');
      expect(pattern.typeCode).toBe('BUG');
      expect(pattern.number).toBe('0001');
      expect(pattern.globPattern).toContain('[BUG][0001]');
    });

    test('handles lowercase input', () => {
      const pattern = findStoryFile('feat0109');
      expect(pattern.typeCode).toBe('FEAT');
      expect(pattern.number).toBe('0109');
    });

    test('throws for invalid story ID format', () => {
      expect(() => findStoryFile('INVALID')).toThrow();
      expect(() => findStoryFile('FEAT')).toThrow();
      expect(() => findStoryFile('0109')).toThrow();
      expect(() => findStoryFile('')).toThrow();
    });
  });
});
