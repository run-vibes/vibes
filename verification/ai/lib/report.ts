/**
 * Report Generator for AI Verification
 *
 * Generates markdown reports from AI verification verdicts.
 * Reports are saved to verification/reports/<scope>/<id>-ai.md
 */

import * as fs from 'node:fs/promises';
import * as path from 'node:path';
import { loadConfig, type ModelConfig, type Verdict } from './router.js';
import type { Criterion, ParsedStory } from './parser.js';
import type { CollectedArtifact } from './collector.js';

// Re-export types for convenience
export type { ParsedStory, Criterion, Verdict, ModelConfig, CollectedArtifact };

/**
 * Confidence thresholds loaded from config
 */
export interface ConfidenceThresholds {
  high: number; // >=high = trusted (Pass/Fail)
  medium: number; // >=medium and <high = Needs Review
  // <medium = Low Confidence (Needs Review)
}

/**
 * Result of a single criterion verification
 */
export interface CriterionResult {
  criterion: Criterion;
  artifact: CollectedArtifact | null;
  verdict: Verdict | null;
}

/**
 * Classified verdict for display
 */
export type VerdictClassification =
  | 'pass'
  | 'fail'
  | 'needs-review'
  | 'artifact-missing'
  | 'error';

/**
 * A processed criterion result ready for the report
 */
export interface ProcessedCriterion {
  index: number;
  text: string;
  artifactPath: string | null;
  classification: VerdictClassification;
  confidenceLevel: 'High' | 'Medium' | 'Low' | null;
  confidence: number | null;
  evidence: string | null;
  suggestion: string | null;
}

/**
 * Summary counts for the report
 */
export interface ReportSummary {
  pass: number;
  fail: number;
  needsReview: number;
}

/**
 * Complete report data
 */
export interface ReportData {
  story: ParsedStory;
  model: ModelConfig;
  timestamp: Date;
  summary: ReportSummary;
  criteria: ProcessedCriterion[];
}

/**
 * Default confidence thresholds if not configured
 */
const DEFAULT_THRESHOLDS: ConfidenceThresholds = {
  high: 80,
  medium: 50,
};

/**
 * Load confidence thresholds from config file.
 *
 * @param configPath - Path to config.toml
 * @returns Confidence thresholds
 */
export function getConfidenceThresholds(configPath: string): ConfidenceThresholds {
  try {
    const config = loadConfig(configPath);
    return {
      high: config.ai.confidence?.high ?? DEFAULT_THRESHOLDS.high,
      medium: config.ai.confidence?.medium ?? DEFAULT_THRESHOLDS.medium,
    };
  } catch {
    return DEFAULT_THRESHOLDS;
  }
}

/**
 * Classify a verdict based on confidence thresholds.
 *
 * Rules:
 * - If verdict is 'pass' and confidence >= high: 'pass'
 * - If verdict is 'fail' and confidence >= high: 'fail'
 * - If verdict is 'unclear' or confidence < high: 'needs-review'
 *
 * @param verdict - The AI verdict
 * @param thresholds - Confidence thresholds
 * @returns Classification for display
 */
export function classifyVerdict(
  verdict: Verdict,
  thresholds: ConfidenceThresholds
): VerdictClassification {
  // If verdict is unclear, always needs review
  if (verdict.verdict === 'unclear') {
    return 'needs-review';
  }

  // If confidence is below high threshold, needs review
  if (verdict.confidence < thresholds.high) {
    return 'needs-review';
  }

  // High confidence pass or fail
  return verdict.verdict;
}

/**
 * Get the confidence level label for a confidence score.
 *
 * @param confidence - The confidence score (0-100)
 * @param thresholds - Confidence thresholds
 * @returns 'High', 'Medium', or 'Low'
 */
export function getConfidenceLevel(
  confidence: number,
  thresholds: ConfidenceThresholds
): 'High' | 'Medium' | 'Low' {
  if (confidence >= thresholds.high) {
    return 'High';
  }
  if (confidence >= thresholds.medium) {
    return 'Medium';
  }
  return 'Low';
}

/**
 * Get emoji/symbol for verdict classification.
 *
 * @param classification - The verdict classification
 * @returns Emoji/symbol string
 */
export function getVerdictSymbol(classification: VerdictClassification): string {
  switch (classification) {
    case 'pass':
      return 'Pass';
    case 'fail':
      return 'Fail';
    case 'needs-review':
      return 'Needs Review';
    case 'artifact-missing':
      return 'Artifact not found';
    case 'error':
      return 'Error';
  }
}

/**
 * Get the relative artifact path from the verification directory.
 *
 * @param artifactPath - Full artifact path
 * @param verificationDir - Base verification directory
 * @returns Relative path for display
 */
export function getRelativeArtifactPath(
  artifactPath: string,
  verificationDir: string
): string {
  const relative = path.relative(verificationDir, artifactPath);
  // Ensure forward slashes for consistency in markdown
  return relative.replace(/\\/g, '/');
}

/**
 * Process criterion results into report-ready format.
 *
 * @param results - Array of criterion verification results
 * @param thresholds - Confidence thresholds
 * @param verificationDir - Base verification directory
 * @returns Processed criteria for the report
 */
export function processCriteriaResults(
  results: CriterionResult[],
  thresholds: ConfidenceThresholds,
  verificationDir: string
): ProcessedCriterion[] {
  return results.map((result, index) => {
    const { criterion, artifact, verdict } = result;

    // Handle missing artifact
    if (!artifact) {
      return {
        index: index + 1,
        text: criterion.text,
        artifactPath: criterion.annotation
          ? `${criterion.annotation.type}:${criterion.annotation.name}`
          : null,
        classification: 'artifact-missing' as VerdictClassification,
        confidenceLevel: null,
        confidence: null,
        evidence: null,
        suggestion: null,
      };
    }

    // Handle missing verdict (error case)
    if (!verdict) {
      return {
        index: index + 1,
        text: criterion.text,
        artifactPath: getRelativeArtifactPath(artifact.path, verificationDir),
        classification: 'error' as VerdictClassification,
        confidenceLevel: null,
        confidence: null,
        evidence: 'Analysis failed or timed out',
        suggestion: null,
      };
    }

    // Process normal verdict
    const classification = classifyVerdict(verdict, thresholds);
    const confidenceLevel = getConfidenceLevel(verdict.confidence, thresholds);

    return {
      index: index + 1,
      text: criterion.text,
      artifactPath: getRelativeArtifactPath(artifact.path, verificationDir),
      classification,
      confidenceLevel,
      confidence: verdict.confidence,
      evidence: verdict.evidence,
      suggestion: verdict.suggestion,
    };
  });
}

/**
 * Calculate summary counts from processed criteria.
 *
 * @param criteria - Processed criteria
 * @returns Summary counts
 */
export function calculateSummary(criteria: ProcessedCriterion[]): ReportSummary {
  let pass = 0;
  let fail = 0;
  let needsReview = 0;

  for (const criterion of criteria) {
    switch (criterion.classification) {
      case 'pass':
        pass++;
        break;
      case 'fail':
        fail++;
        break;
      case 'needs-review':
      case 'artifact-missing':
      case 'error':
        needsReview++;
        break;
    }
  }

  return { pass, fail, needsReview };
}

/**
 * Format a timestamp for the report.
 *
 * @param date - The date to format
 * @returns Formatted timestamp string (YYYY-MM-DD HH:mm:ss)
 */
export function formatTimestamp(date: Date): string {
  const pad = (n: number): string => n.toString().padStart(2, '0');
  return (
    `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ` +
    `${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`
  );
}

/**
 * Generate the markdown report content.
 *
 * @param data - The report data
 * @returns Markdown content string
 */
export function generateReportContent(data: ReportData): string {
  const { story, model, timestamp, summary, criteria } = data;

  const lines: string[] = [];

  // Header
  lines.push(`# AI Verification Report: ${story.id}`);
  lines.push('');
  lines.push(`**Story:** ${story.title}`);
  lines.push(`**Scope:** ${story.scope}`);
  lines.push(`**Model:** ${model.provider}:${model.model}`);
  lines.push(`**Generated:** ${formatTimestamp(timestamp)}`);
  lines.push('');

  // Summary table
  lines.push('## Summary');
  lines.push('');
  lines.push('| Result | Count |');
  lines.push('|--------|-------|');
  lines.push(`| Pass | ${summary.pass} |`);
  lines.push(`| Fail | ${summary.fail} |`);
  lines.push(`| Needs Review | ${summary.needsReview} |`);
  lines.push('');

  // Criteria section
  lines.push('## Criteria');
  lines.push('');

  for (const criterion of criteria) {
    lines.push(`### ${criterion.index}. ${criterion.text}`);

    if (criterion.artifactPath) {
      lines.push(`**Artifact:** \`${criterion.artifactPath}\``);
    } else {
      lines.push('**Artifact:** (no annotation)');
    }

    // Verdict line based on classification
    switch (criterion.classification) {
      case 'pass':
        lines.push(
          `**Verdict:** Pass (${criterion.confidenceLevel} confidence: ${criterion.confidence}%)`
        );
        break;
      case 'fail':
        lines.push(
          `**Verdict:** Fail (${criterion.confidenceLevel} confidence: ${criterion.confidence}%)`
        );
        break;
      case 'needs-review':
        lines.push(
          `**Verdict:** Needs Review (${criterion.confidenceLevel} confidence: ${criterion.confidence}%)`
        );
        break;
      case 'artifact-missing':
        lines.push('**Verdict:** Artifact not found');
        break;
      case 'error':
        lines.push('**Verdict:** Analysis error');
        break;
    }

    // Evidence as blockquote
    if (criterion.evidence) {
      lines.push('');
      // Split evidence into lines and prefix each with >
      const evidenceLines = criterion.evidence.split('\n');
      for (const line of evidenceLines) {
        lines.push(`> ${line}`);
      }
    }

    // Suggestion for failed criteria
    if (criterion.suggestion && criterion.classification === 'fail') {
      lines.push('');
      lines.push(`**Suggested fix:** ${criterion.suggestion}`);
    }

    lines.push('');
  }

  // Failed criteria summary section
  const failedCriteria = criteria.filter((c) => c.classification === 'fail');
  if (failedCriteria.length > 0) {
    lines.push('## Failed Criteria Summary');
    lines.push('');
    lines.push('The following criteria did not pass and may require attention:');
    lines.push('');
    for (const criterion of failedCriteria) {
      const suggestion = criterion.suggestion || 'No specific suggestion';
      lines.push(`- **${criterion.text}** (${criterion.confidence}%): ${suggestion}`);
    }
    lines.push('');
  }

  // Needs review summary section
  const needsReviewCriteria = criteria.filter(
    (c) =>
      c.classification === 'needs-review' ||
      c.classification === 'artifact-missing' ||
      c.classification === 'error'
  );
  if (needsReviewCriteria.length > 0) {
    lines.push('## Criteria Needing Review');
    lines.push('');
    lines.push('The following criteria require human review:');
    lines.push('');
    for (const criterion of needsReviewCriteria) {
      let reason: string;
      switch (criterion.classification) {
        case 'needs-review':
          reason =
            criterion.confidenceLevel === 'Low'
              ? 'Low confidence'
              : 'Confidence below threshold';
          break;
        case 'artifact-missing':
          reason = 'Artifact not found';
          break;
        case 'error':
          reason = 'Analysis failed';
          break;
        default:
          reason = 'Unknown';
      }
      lines.push(`- **${criterion.text}**: ${reason}`);
    }
    lines.push('');
  }

  return lines.join('\n');
}

/**
 * Generate a complete report from verification results.
 *
 * @param story - The parsed story
 * @param results - Array of criterion verification results
 * @param model - The model configuration used
 * @param configPath - Path to config.toml (for thresholds)
 * @param verificationDir - Base verification directory
 * @returns Report data object
 */
export function generateReport(
  story: ParsedStory,
  results: CriterionResult[],
  model: ModelConfig,
  configPath: string,
  verificationDir: string = path.join(process.cwd(), 'verification')
): ReportData {
  const thresholds = getConfidenceThresholds(configPath);
  const criteria = processCriteriaResults(results, thresholds, verificationDir);
  const summary = calculateSummary(criteria);

  return {
    story,
    model,
    timestamp: new Date(),
    summary,
    criteria,
  };
}

/**
 * Get the report file path for a story.
 *
 * @param story - The parsed story
 * @param verificationDir - Base verification directory
 * @returns Full path to the report file
 */
export function getReportPath(
  story: ParsedStory,
  verificationDir: string = path.join(process.cwd(), 'verification')
): string {
  return path.join(verificationDir, 'reports', story.scope, `${story.id}-ai.md`);
}

/**
 * Save a report to the filesystem.
 *
 * Creates the directory structure if it doesn't exist.
 *
 * @param data - The report data
 * @param verificationDir - Base verification directory
 * @returns Path to the saved report
 */
export async function saveReport(
  data: ReportData,
  verificationDir: string = path.join(process.cwd(), 'verification')
): Promise<string> {
  const reportPath = getReportPath(data.story, verificationDir);
  const reportDir = path.dirname(reportPath);

  // Create directory structure if needed
  await fs.mkdir(reportDir, { recursive: true });

  // Generate and write content
  const content = generateReportContent(data);
  await fs.writeFile(reportPath, content, 'utf-8');

  return reportPath;
}

/**
 * Generate and save a report in one call.
 *
 * This is the main entry point for the report generator.
 *
 * @param story - The parsed story
 * @param results - Array of criterion verification results
 * @param model - The model configuration used
 * @param configPath - Path to config.toml
 * @param verificationDir - Base verification directory
 * @returns Path to the saved report
 */
export async function generateAndSaveReport(
  story: ParsedStory,
  results: CriterionResult[],
  model: ModelConfig,
  configPath: string,
  verificationDir: string = path.join(process.cwd(), 'verification')
): Promise<string> {
  const reportData = generateReport(story, results, model, configPath, verificationDir);
  return saveReport(reportData, verificationDir);
}
