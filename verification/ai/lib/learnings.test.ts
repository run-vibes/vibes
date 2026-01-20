import { describe, test, expect } from 'vitest';
import {
  filterUnappliedLearnings,
  parseAppliesTo,
  getLearningsSummary,
  type Learning,
} from './learnings.js';

describe('Learning Scanner', () => {
  describe('filterUnappliedLearnings', () => {
    test('filters out applied learnings', () => {
      const learnings: Learning[] = [
        {
          id: 'L001',
          source: '/test/story.md',
          sourceType: 'story',
          title: 'Applied learning',
          category: 'code',
          context: 'Some context',
          insight: 'Some insight',
          suggestedAction: 'Do something',
          appliesTo: 'CLAUDE.md',
          applied: 'CLAUDE.md (2026-01-20)',
        },
        {
          id: 'L002',
          source: '/test/story.md',
          sourceType: 'story',
          title: 'Unapplied learning',
          category: 'process',
          context: 'Some context',
          insight: 'Another insight',
          suggestedAction: 'Do something else',
          appliesTo: 'CONVENTIONS.md',
          applied: '',
        },
      ];

      const result = filterUnappliedLearnings(learnings);
      expect(result).toHaveLength(1);
      expect(result[0].id).toBe('L002');
    });

    test('handles empty applied field with whitespace', () => {
      const learnings: Learning[] = [
        {
          id: 'L001',
          source: '/test/story.md',
          sourceType: 'story',
          title: 'Test',
          category: 'code',
          context: 'Context',
          insight: 'Insight',
          suggestedAction: 'Action',
          appliesTo: 'file.md',
          applied: '   ',
        },
      ];

      const result = filterUnappliedLearnings(learnings);
      expect(result).toHaveLength(1);
    });

    test('returns empty array when all learnings are applied', () => {
      const learnings: Learning[] = [
        {
          id: 'L001',
          source: '/test/story.md',
          sourceType: 'story',
          title: 'Applied',
          category: 'code',
          context: 'Context',
          insight: 'Insight',
          suggestedAction: 'Action',
          appliesTo: 'file.md',
          applied: 'file.md (2026-01-20)',
        },
      ];

      const result = filterUnappliedLearnings(learnings);
      expect(result).toHaveLength(0);
    });
  });

  describe('parseAppliesTo', () => {
    test('parses single file', () => {
      const result = parseAppliesTo('CLAUDE.md');
      expect(result).toEqual(['CLAUDE.md']);
    });

    test('parses comma-separated files', () => {
      const result = parseAppliesTo('CLAUDE.md, CONVENTIONS.md');
      expect(result).toEqual(['CLAUDE.md', 'CONVENTIONS.md']);
    });

    test('parses semicolon-separated files', () => {
      const result = parseAppliesTo('CLAUDE.md; CONVENTIONS.md');
      expect(result).toEqual(['CLAUDE.md', 'CONVENTIONS.md']);
    });

    test('returns empty for "To be determined"', () => {
      const result = parseAppliesTo('To be determined');
      expect(result).toEqual([]);
    });

    test('returns empty for empty string', () => {
      const result = parseAppliesTo('');
      expect(result).toEqual([]);
    });

    test('filters out "none"', () => {
      const result = parseAppliesTo('None');
      expect(result).toEqual([]);
    });

    test('handles glob patterns', () => {
      const result = parseAppliesTo('tests/*.ts');
      expect(result).toEqual(['tests/*.ts']);
    });

    test('trims whitespace', () => {
      const result = parseAppliesTo('  CLAUDE.md  ,  CONVENTIONS.md  ');
      expect(result).toEqual(['CLAUDE.md', 'CONVENTIONS.md']);
    });
  });

  describe('getLearningsSummary', () => {
    test('calculates correct summary', () => {
      const learnings: Learning[] = [
        {
          id: 'L001',
          source: '/test/story1.md',
          sourceType: 'story',
          title: 'Story learning',
          category: 'code',
          context: 'Context',
          insight: 'Insight',
          suggestedAction: 'Action',
          appliesTo: 'file.md',
          applied: 'file.md (2026-01-20)',
        },
        {
          id: 'L002',
          source: '/test/story2.md',
          sourceType: 'story',
          title: 'Another story learning',
          category: 'process',
          context: 'Context',
          insight: 'Insight',
          suggestedAction: 'Action',
          appliesTo: 'other.md',
          applied: '',
        },
        {
          id: 'ML001',
          source: '/test/milestone.md',
          sourceType: 'milestone',
          title: 'Milestone learning',
          category: 'architecture',
          context: 'Context',
          insight: 'Insight',
          suggestedAction: 'Action',
          appliesTo: 'DESIGN.md',
          applied: '',
        },
        {
          id: 'L001',
          source: '/test/adhoc.md',
          sourceType: 'adhoc',
          title: 'Ad-hoc learning',
          category: 'verification',
          context: 'Context',
          insight: 'Insight',
          suggestedAction: 'Action',
          appliesTo: 'tests/',
          applied: 'tests/ (2026-01-19)',
        },
      ];

      const summary = getLearningsSummary(learnings);

      expect(summary.total).toBe(4);
      expect(summary.byType.story).toBe(2);
      expect(summary.byType.milestone).toBe(1);
      expect(summary.byType.adhoc).toBe(1);
      expect(summary.applied).toBe(2);
      expect(summary.pending).toBe(2);
    });

    test('handles empty learnings array', () => {
      const summary = getLearningsSummary([]);

      expect(summary.total).toBe(0);
      expect(summary.byType.story).toBe(0);
      expect(summary.byType.milestone).toBe(0);
      expect(summary.byType.adhoc).toBe(0);
      expect(summary.applied).toBe(0);
      expect(summary.pending).toBe(0);
    });
  });
});
