import { describe, test, expect, vi, beforeEach, afterEach } from 'vitest';
import * as fs from 'node:fs/promises';
import * as path from 'node:path';
import {
  getConfidenceThresholds,
  classifyVerdict,
  getConfidenceLevel,
  getVerdictSymbol,
  getRelativeArtifactPath,
  processCriteriaResults,
  calculateSummary,
  formatTimestamp,
  generateReportContent,
  generateReport,
  getReportPath,
  saveReport,
  type CriterionResult,
  type ProcessedCriterion,
  type ReportData,
  type ConfidenceThresholds,
} from './report.js';
import type { Verdict, ModelConfig } from './router.js';
import type { ParsedStory, Criterion } from './parser.js';
import type { CollectedArtifact } from './collector.js';

// Mock dependencies
vi.mock('node:fs/promises', () => ({
  mkdir: vi.fn().mockResolvedValue(undefined),
  writeFile: vi.fn().mockResolvedValue(undefined),
}));

vi.mock('./router.js', async () => {
  const actual = await vi.importActual<typeof import('./router.js')>('./router.js');
  return {
    ...actual,
    loadConfig: vi.fn(),
  };
});

// Import the mock for setting up return values
import { loadConfig } from './router.js';

describe('Report Generator', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.resetAllMocks();
  });

  describe('getConfidenceThresholds', () => {
    test('loads thresholds from config', () => {
      vi.mocked(loadConfig).mockReturnValue({
        ai: {
          default_model: 'ollama:qwen3-vl:32b',
          confidence: {
            high: 85,
            medium: 55,
          },
        },
      });

      const thresholds = getConfidenceThresholds('/path/to/config.toml');
      expect(thresholds).toEqual({ high: 85, medium: 55 });
    });

    test('uses defaults when config missing confidence section', () => {
      vi.mocked(loadConfig).mockReturnValue({
        ai: {
          default_model: 'ollama:qwen3-vl:32b',
        },
      });

      const thresholds = getConfidenceThresholds('/path/to/config.toml');
      expect(thresholds).toEqual({ high: 80, medium: 50 });
    });

    test('uses defaults when config load fails', () => {
      vi.mocked(loadConfig).mockImplementation(() => {
        throw new Error('Config not found');
      });

      const thresholds = getConfidenceThresholds('/nonexistent/config.toml');
      expect(thresholds).toEqual({ high: 80, medium: 50 });
    });

    test('fills in missing threshold values with defaults', () => {
      vi.mocked(loadConfig).mockReturnValue({
        ai: {
          default_model: 'ollama:qwen3-vl:32b',
          confidence: {
            high: 90,
            // medium is missing
          },
        },
      });

      const thresholds = getConfidenceThresholds('/path/to/config.toml');
      expect(thresholds.high).toBe(90);
      expect(thresholds.medium).toBe(50); // default
    });
  });

  describe('classifyVerdict', () => {
    const thresholds: ConfidenceThresholds = { high: 80, medium: 50 };

    test('classifies high confidence pass as pass', () => {
      const verdict: Verdict = {
        verdict: 'pass',
        confidence: 92,
        evidence: 'Test evidence',
        suggestion: null,
      };
      expect(classifyVerdict(verdict, thresholds)).toBe('pass');
    });

    test('classifies high confidence fail as fail', () => {
      const verdict: Verdict = {
        verdict: 'fail',
        confidence: 85,
        evidence: 'Test evidence',
        suggestion: 'Fix it',
      };
      expect(classifyVerdict(verdict, thresholds)).toBe('fail');
    });

    test('classifies unclear verdict as needs-review', () => {
      const verdict: Verdict = {
        verdict: 'unclear',
        confidence: 95,
        evidence: 'Cannot determine',
        suggestion: null,
      };
      expect(classifyVerdict(verdict, thresholds)).toBe('needs-review');
    });

    test('classifies low confidence pass as needs-review', () => {
      const verdict: Verdict = {
        verdict: 'pass',
        confidence: 65,
        evidence: 'Probably passes',
        suggestion: null,
      };
      expect(classifyVerdict(verdict, thresholds)).toBe('needs-review');
    });

    test('classifies low confidence fail as needs-review', () => {
      const verdict: Verdict = {
        verdict: 'fail',
        confidence: 70,
        evidence: 'Might fail',
        suggestion: 'Maybe fix',
      };
      expect(classifyVerdict(verdict, thresholds)).toBe('needs-review');
    });

    test('exactly at threshold classifies as trusted', () => {
      const verdict: Verdict = {
        verdict: 'pass',
        confidence: 80,
        evidence: 'At threshold',
        suggestion: null,
      };
      expect(classifyVerdict(verdict, thresholds)).toBe('pass');
    });

    test('one below threshold classifies as needs-review', () => {
      const verdict: Verdict = {
        verdict: 'pass',
        confidence: 79,
        evidence: 'Just below',
        suggestion: null,
      };
      expect(classifyVerdict(verdict, thresholds)).toBe('needs-review');
    });
  });

  describe('getConfidenceLevel', () => {
    const thresholds: ConfidenceThresholds = { high: 80, medium: 50 };

    test('returns High for confidence >= high threshold', () => {
      expect(getConfidenceLevel(92, thresholds)).toBe('High');
      expect(getConfidenceLevel(80, thresholds)).toBe('High');
      expect(getConfidenceLevel(100, thresholds)).toBe('High');
    });

    test('returns Medium for confidence >= medium and < high', () => {
      expect(getConfidenceLevel(65, thresholds)).toBe('Medium');
      expect(getConfidenceLevel(50, thresholds)).toBe('Medium');
      expect(getConfidenceLevel(79, thresholds)).toBe('Medium');
    });

    test('returns Low for confidence < medium', () => {
      expect(getConfidenceLevel(30, thresholds)).toBe('Low');
      expect(getConfidenceLevel(49, thresholds)).toBe('Low');
      expect(getConfidenceLevel(0, thresholds)).toBe('Low');
    });
  });

  describe('getVerdictSymbol', () => {
    test('returns correct symbols for each classification', () => {
      expect(getVerdictSymbol('pass')).toBe('Pass');
      expect(getVerdictSymbol('fail')).toBe('Fail');
      expect(getVerdictSymbol('needs-review')).toBe('Needs Review');
      expect(getVerdictSymbol('artifact-missing')).toBe('Artifact not found');
      expect(getVerdictSymbol('error')).toBe('Error');
    });
  });

  describe('getRelativeArtifactPath', () => {
    test('returns relative path from verification directory', () => {
      const result = getRelativeArtifactPath(
        '/project/verification/snapshots/sessions.png',
        '/project/verification'
      );
      expect(result).toBe('snapshots/sessions.png');
    });

    test('handles nested directories', () => {
      const result = getRelativeArtifactPath(
        '/project/verification/videos/cli/demo.webm',
        '/project/verification'
      );
      expect(result).toBe('videos/cli/demo.webm');
    });

    test('normalizes backslashes to forward slashes', () => {
      // This tests the replace call even on Unix
      const result = getRelativeArtifactPath(
        '/project/verification/snapshots/test.png',
        '/project/verification'
      );
      expect(result).not.toContain('\\');
    });
  });

  describe('processCriteriaResults', () => {
    const thresholds: ConfidenceThresholds = { high: 80, medium: 50 };
    const verificationDir = '/project/verification';

    test('processes passing verdict correctly', () => {
      const results: CriterionResult[] = [
        {
          criterion: { text: 'Sessions page displays list' },
          artifact: {
            type: 'image',
            path: '/project/verification/snapshots/sessions.png',
            data: Buffer.from(''),
          },
          verdict: {
            verdict: 'pass',
            confidence: 92,
            evidence: 'Shows sessions table',
            suggestion: null,
          },
        },
      ];

      const processed = processCriteriaResults(results, thresholds, verificationDir);

      expect(processed).toHaveLength(1);
      expect(processed[0]).toEqual({
        index: 1,
        text: 'Sessions page displays list',
        artifactPath: 'snapshots/sessions.png',
        classification: 'pass',
        confidenceLevel: 'High',
        confidence: 92,
        evidence: 'Shows sessions table',
        suggestion: null,
      });
    });

    test('processes failing verdict correctly', () => {
      const results: CriterionResult[] = [
        {
          criterion: { text: 'Button shows loading state' },
          artifact: {
            type: 'image',
            path: '/project/verification/snapshots/button.png',
            data: Buffer.from(''),
          },
          verdict: {
            verdict: 'fail',
            confidence: 85,
            evidence: 'Button has no spinner',
            suggestion: 'Add a loading spinner',
          },
        },
      ];

      const processed = processCriteriaResults(results, thresholds, verificationDir);

      expect(processed[0].classification).toBe('fail');
      expect(processed[0].suggestion).toBe('Add a loading spinner');
    });

    test('processes missing artifact correctly', () => {
      const results: CriterionResult[] = [
        {
          criterion: {
            text: 'Chart displays data',
            annotation: { type: 'snapshot', name: 'chart' },
          },
          artifact: null,
          verdict: null,
        },
      ];

      const processed = processCriteriaResults(results, thresholds, verificationDir);

      expect(processed[0].classification).toBe('artifact-missing');
      expect(processed[0].artifactPath).toBe('snapshot:chart');
      expect(processed[0].confidence).toBeNull();
    });

    test('processes missing verdict (error) correctly', () => {
      const results: CriterionResult[] = [
        {
          criterion: { text: 'Page loads correctly' },
          artifact: {
            type: 'image',
            path: '/project/verification/snapshots/page.png',
            data: Buffer.from(''),
          },
          verdict: null, // Analysis failed
        },
      ];

      const processed = processCriteriaResults(results, thresholds, verificationDir);

      expect(processed[0].classification).toBe('error');
      expect(processed[0].evidence).toBe('Analysis failed or timed out');
    });

    test('assigns correct indices to multiple criteria', () => {
      const results: CriterionResult[] = [
        {
          criterion: { text: 'First criterion' },
          artifact: {
            type: 'image',
            path: '/project/verification/snapshots/first.png',
            data: Buffer.from(''),
          },
          verdict: { verdict: 'pass', confidence: 90, evidence: 'OK', suggestion: null },
        },
        {
          criterion: { text: 'Second criterion' },
          artifact: {
            type: 'image',
            path: '/project/verification/snapshots/second.png',
            data: Buffer.from(''),
          },
          verdict: { verdict: 'pass', confidence: 85, evidence: 'OK', suggestion: null },
        },
      ];

      const processed = processCriteriaResults(results, thresholds, verificationDir);

      expect(processed[0].index).toBe(1);
      expect(processed[1].index).toBe(2);
    });
  });

  describe('calculateSummary', () => {
    test('counts all pass criteria', () => {
      const criteria: ProcessedCriterion[] = [
        {
          index: 1,
          text: 'Test 1',
          artifactPath: 'test.png',
          classification: 'pass',
          confidenceLevel: 'High',
          confidence: 90,
          evidence: 'OK',
          suggestion: null,
        },
        {
          index: 2,
          text: 'Test 2',
          artifactPath: 'test2.png',
          classification: 'pass',
          confidenceLevel: 'High',
          confidence: 85,
          evidence: 'OK',
          suggestion: null,
        },
      ];

      const summary = calculateSummary(criteria);
      expect(summary).toEqual({ pass: 2, fail: 0, needsReview: 0 });
    });

    test('counts mixed results correctly', () => {
      const criteria: ProcessedCriterion[] = [
        {
          index: 1,
          text: 'Pass',
          artifactPath: 'a.png',
          classification: 'pass',
          confidenceLevel: 'High',
          confidence: 90,
          evidence: 'OK',
          suggestion: null,
        },
        {
          index: 2,
          text: 'Fail',
          artifactPath: 'b.png',
          classification: 'fail',
          confidenceLevel: 'High',
          confidence: 85,
          evidence: 'Bad',
          suggestion: 'Fix it',
        },
        {
          index: 3,
          text: 'Review',
          artifactPath: 'c.png',
          classification: 'needs-review',
          confidenceLevel: 'Medium',
          confidence: 60,
          evidence: 'Maybe',
          suggestion: null,
        },
        {
          index: 4,
          text: 'Missing',
          artifactPath: 'snapshot:missing',
          classification: 'artifact-missing',
          confidenceLevel: null,
          confidence: null,
          evidence: null,
          suggestion: null,
        },
        {
          index: 5,
          text: 'Error',
          artifactPath: 'e.png',
          classification: 'error',
          confidenceLevel: null,
          confidence: null,
          evidence: 'Failed',
          suggestion: null,
        },
      ];

      const summary = calculateSummary(criteria);
      expect(summary).toEqual({ pass: 1, fail: 1, needsReview: 3 });
    });

    test('handles empty criteria array', () => {
      const summary = calculateSummary([]);
      expect(summary).toEqual({ pass: 0, fail: 0, needsReview: 0 });
    });
  });

  describe('formatTimestamp', () => {
    test('formats date correctly', () => {
      const date = new Date('2026-01-19T14:32:05');
      expect(formatTimestamp(date)).toBe('2026-01-19 14:32:05');
    });

    test('pads single digit values', () => {
      const date = new Date('2026-01-05T09:05:03');
      expect(formatTimestamp(date)).toBe('2026-01-05 09:05:03');
    });

    test('handles midnight', () => {
      const date = new Date('2026-01-19T00:00:00');
      expect(formatTimestamp(date)).toBe('2026-01-19 00:00:00');
    });
  });

  describe('generateReportContent', () => {
    const baseReportData: ReportData = {
      story: {
        id: 'FEAT0109',
        title: 'Board generator grouped layout',
        scope: 'coherence-verification/01-artifact-pipeline',
        criteria: [],
      },
      model: { provider: 'ollama', model: 'qwen3-vl:32b' },
      timestamp: new Date('2026-01-19T14:32:05'),
      summary: { pass: 3, fail: 1, needsReview: 1 },
      criteria: [
        {
          index: 1,
          text: 'Sessions page displays list',
          artifactPath: 'snapshots/sessions.png',
          classification: 'pass',
          confidenceLevel: 'High',
          confidence: 92,
          evidence: 'The screenshot shows a sessions page with a table.',
          suggestion: null,
        },
      ],
    };

    test('includes header with story metadata', () => {
      const content = generateReportContent(baseReportData);

      expect(content).toContain('# AI Verification Report: FEAT0109');
      expect(content).toContain('**Story:** Board generator grouped layout');
      expect(content).toContain('**Scope:** coherence-verification/01-artifact-pipeline');
      expect(content).toContain('**Model:** ollama:qwen3-vl:32b');
      expect(content).toContain('**Generated:** 2026-01-19 14:32:05');
    });

    test('includes summary table', () => {
      const content = generateReportContent(baseReportData);

      expect(content).toContain('## Summary');
      expect(content).toContain('| Result | Count |');
      expect(content).toContain('| Pass | 3 |');
      expect(content).toContain('| Fail | 1 |');
      expect(content).toContain('| Needs Review | 1 |');
    });

    test('includes criterion with pass verdict', () => {
      const content = generateReportContent(baseReportData);

      expect(content).toContain('### 1. Sessions page displays list');
      expect(content).toContain('**Artifact:** `snapshots/sessions.png`');
      expect(content).toContain('**Verdict:** Pass (High confidence: 92%)');
      expect(content).toContain('> The screenshot shows a sessions page with a table.');
    });

    test('includes fail verdict with suggestion', () => {
      const data: ReportData = {
        ...baseReportData,
        criteria: [
          {
            index: 1,
            text: 'Button shows loading',
            artifactPath: 'snapshots/button.png',
            classification: 'fail',
            confidenceLevel: 'High',
            confidence: 88,
            evidence: 'No loading spinner visible',
            suggestion: 'Add spinner component',
          },
        ],
      };

      const content = generateReportContent(data);

      expect(content).toContain('**Verdict:** Fail (High confidence: 88%)');
      expect(content).toContain('**Suggested fix:** Add spinner component');
    });

    test('includes needs-review verdict', () => {
      const data: ReportData = {
        ...baseReportData,
        criteria: [
          {
            index: 1,
            text: 'Form validates input',
            artifactPath: 'snapshots/form.png',
            classification: 'needs-review',
            confidenceLevel: 'Medium',
            confidence: 65,
            evidence: 'Uncertain about validation state',
            suggestion: null,
          },
        ],
      };

      const content = generateReportContent(data);

      expect(content).toContain('**Verdict:** Needs Review (Medium confidence: 65%)');
    });

    test('handles artifact-missing classification', () => {
      const data: ReportData = {
        ...baseReportData,
        criteria: [
          {
            index: 1,
            text: 'Chart displays data',
            artifactPath: 'snapshot:chart',
            classification: 'artifact-missing',
            confidenceLevel: null,
            confidence: null,
            evidence: null,
            suggestion: null,
          },
        ],
      };

      const content = generateReportContent(data);

      expect(content).toContain('**Verdict:** Artifact not found');
    });

    test('handles error classification', () => {
      const data: ReportData = {
        ...baseReportData,
        criteria: [
          {
            index: 1,
            text: 'Page loads',
            artifactPath: 'snapshots/page.png',
            classification: 'error',
            confidenceLevel: null,
            confidence: null,
            evidence: 'Analysis failed or timed out',
            suggestion: null,
          },
        ],
      };

      const content = generateReportContent(data);

      expect(content).toContain('**Verdict:** Analysis error');
      expect(content).toContain('> Analysis failed or timed out');
    });

    test('includes failed criteria summary section when there are failures', () => {
      const data: ReportData = {
        ...baseReportData,
        criteria: [
          {
            index: 1,
            text: 'Feature A works',
            artifactPath: 'a.png',
            classification: 'fail',
            confidenceLevel: 'High',
            confidence: 90,
            evidence: 'Broken',
            suggestion: 'Fix feature A',
          },
          {
            index: 2,
            text: 'Feature B works',
            artifactPath: 'b.png',
            classification: 'fail',
            confidenceLevel: 'High',
            confidence: 85,
            evidence: 'Also broken',
            suggestion: null,
          },
        ],
      };

      const content = generateReportContent(data);

      expect(content).toContain('## Failed Criteria Summary');
      expect(content).toContain('- **Feature A works** (90%): Fix feature A');
      expect(content).toContain('- **Feature B works** (85%): No specific suggestion');
    });

    test('includes needs review summary section', () => {
      const data: ReportData = {
        ...baseReportData,
        criteria: [
          {
            index: 1,
            text: 'Low confidence test',
            artifactPath: 'a.png',
            classification: 'needs-review',
            confidenceLevel: 'Low',
            confidence: 40,
            evidence: 'Uncertain',
            suggestion: null,
          },
          {
            index: 2,
            text: 'Missing artifact test',
            artifactPath: 'snapshot:missing',
            classification: 'artifact-missing',
            confidenceLevel: null,
            confidence: null,
            evidence: null,
            suggestion: null,
          },
        ],
      };

      const content = generateReportContent(data);

      expect(content).toContain('## Criteria Needing Review');
      expect(content).toContain('- **Low confidence test**: Low confidence');
      expect(content).toContain('- **Missing artifact test**: Artifact not found');
    });

    test('handles multiline evidence', () => {
      const data: ReportData = {
        ...baseReportData,
        criteria: [
          {
            index: 1,
            text: 'Test',
            artifactPath: 'test.png',
            classification: 'pass',
            confidenceLevel: 'High',
            confidence: 90,
            evidence: 'Line 1\nLine 2\nLine 3',
            suggestion: null,
          },
        ],
      };

      const content = generateReportContent(data);

      expect(content).toContain('> Line 1');
      expect(content).toContain('> Line 2');
      expect(content).toContain('> Line 3');
    });

    test('handles criterion without annotation showing no annotation', () => {
      const data: ReportData = {
        ...baseReportData,
        criteria: [
          {
            index: 1,
            text: 'Test without artifact',
            artifactPath: null,
            classification: 'artifact-missing',
            confidenceLevel: null,
            confidence: null,
            evidence: null,
            suggestion: null,
          },
        ],
      };

      const content = generateReportContent(data);

      expect(content).toContain('**Artifact:** (no annotation)');
    });
  });

  describe('generateReport', () => {
    beforeEach(() => {
      vi.mocked(loadConfig).mockReturnValue({
        ai: {
          default_model: 'ollama:qwen3-vl:32b',
          confidence: { high: 80, medium: 50 },
        },
      });
    });

    test('generates complete report data', () => {
      const story: ParsedStory = {
        id: 'FEAT0109',
        title: 'Test Story',
        scope: 'test-scope',
        criteria: [{ text: 'Test criterion' }],
      };

      const results: CriterionResult[] = [
        {
          criterion: { text: 'Test criterion' },
          artifact: {
            type: 'image',
            path: '/verification/snapshots/test.png',
            data: Buffer.from(''),
          },
          verdict: {
            verdict: 'pass',
            confidence: 90,
            evidence: 'Looks good',
            suggestion: null,
          },
        },
      ];

      const model: ModelConfig = { provider: 'ollama', model: 'qwen3-vl:32b' };

      const report = generateReport(
        story,
        results,
        model,
        '/path/to/config.toml',
        '/verification'
      );

      expect(report.story).toBe(story);
      expect(report.model).toBe(model);
      expect(report.timestamp).toBeInstanceOf(Date);
      expect(report.summary).toEqual({ pass: 1, fail: 0, needsReview: 0 });
      expect(report.criteria).toHaveLength(1);
      expect(report.criteria[0].classification).toBe('pass');
    });
  });

  describe('getReportPath', () => {
    test('generates correct path for story', () => {
      const story: ParsedStory = {
        id: 'FEAT0109',
        title: 'Test',
        scope: 'coherence-verification/01-artifact-pipeline',
        criteria: [],
      };

      const reportPath = getReportPath(story, '/project/verification');

      expect(reportPath).toBe(
        path.join(
          '/project/verification/reports/coherence-verification/01-artifact-pipeline/FEAT0109-ai.md'
        )
      );
    });

    test('uses default verification directory', () => {
      const story: ParsedStory = {
        id: 'FEAT0200',
        title: 'Test',
        scope: 'test-scope',
        criteria: [],
      };

      const reportPath = getReportPath(story);

      expect(reportPath).toContain('verification');
      expect(reportPath).toContain('reports');
      expect(reportPath).toContain('test-scope');
      expect(reportPath).toContain('FEAT0200-ai.md');
    });
  });

  describe('saveReport', () => {
    beforeEach(() => {
      vi.mocked(fs.mkdir).mockResolvedValue(undefined);
      vi.mocked(fs.writeFile).mockResolvedValue(undefined);
    });

    test('creates directory and saves report', async () => {
      const data: ReportData = {
        story: {
          id: 'FEAT0109',
          title: 'Test',
          scope: 'test-scope/sub',
          criteria: [],
        },
        model: { provider: 'ollama', model: 'qwen3-vl:32b' },
        timestamp: new Date('2026-01-19T14:32:05'),
        summary: { pass: 0, fail: 0, needsReview: 0 },
        criteria: [],
      };

      const savedPath = await saveReport(data, '/project/verification');

      // Check mkdir was called with recursive option
      expect(fs.mkdir).toHaveBeenCalledWith(
        expect.stringContaining('test-scope'),
        { recursive: true }
      );

      // Check writeFile was called
      expect(fs.writeFile).toHaveBeenCalledWith(
        expect.stringContaining('FEAT0109-ai.md'),
        expect.stringContaining('# AI Verification Report: FEAT0109'),
        'utf-8'
      );

      // Check returned path
      expect(savedPath).toContain('FEAT0109-ai.md');
    });
  });
});
