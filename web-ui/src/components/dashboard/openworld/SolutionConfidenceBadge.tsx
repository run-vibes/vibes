/**
 * Confidence badge for solutions
 *
 * Shows confidence level with color-coded indicator:
 * - High (>=80%): phosphor green
 * - Medium (50-79%): amber
 * - Low (<50%): dim
 */
import './SolutionConfidenceBadge.css';

export interface SolutionConfidenceBadgeProps {
  confidence: number;
}

function getConfidenceLevel(confidence: number): 'high' | 'medium' | 'low' {
  if (confidence >= 0.8) return 'high';
  if (confidence >= 0.5) return 'medium';
  return 'low';
}

export function SolutionConfidenceBadge({ confidence }: SolutionConfidenceBadgeProps) {
  const level = getConfidenceLevel(confidence);
  const percentage = Math.round(confidence * 100);

  return (
    <span
      className={`solution-confidence-badge solution-confidence-badge--${level}`}
      data-testid={`confidence-${level}`}
      title={`${percentage}% confidence`}
    >
      {percentage}%
    </span>
  );
}
