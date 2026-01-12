/**
 * Action buttons for solutions
 *
 * Shows Apply/Dismiss buttons for pending solutions,
 * or status indicator for already processed solutions.
 */
import type { SolutionStatus } from '../../../hooks/useDashboard';
import './SolutionActions.css';

export interface SolutionActionsProps {
  solutionId: string;
  status: SolutionStatus;
  onApply?: (id: string) => void;
  onDismiss?: (id: string) => void;
  isLoading?: boolean;
}

export function SolutionActions({
  solutionId,
  status,
  onApply,
  onDismiss,
  isLoading,
}: SolutionActionsProps) {
  if (status === 'Applied') {
    return (
      <div className="solution-actions">
        <span className="solution-actions__status solution-actions__status--applied">
          Applied
        </span>
      </div>
    );
  }

  if (status === 'Dismissed') {
    return (
      <div className="solution-actions">
        <span className="solution-actions__status solution-actions__status--dismissed">
          Dismissed
        </span>
      </div>
    );
  }

  return (
    <div className="solution-actions">
      <button
        type="button"
        className="solution-actions__btn solution-actions__btn--apply"
        onClick={() => onApply?.(solutionId)}
        disabled={isLoading}
        data-testid={`apply-${solutionId}`}
      >
        {isLoading ? '...' : 'Apply'}
      </button>
      <button
        type="button"
        className="solution-actions__btn solution-actions__btn--dismiss"
        onClick={() => onDismiss?.(solutionId)}
        disabled={isLoading}
        data-testid={`dismiss-${solutionId}`}
      >
        Dismiss
      </button>
    </div>
  );
}
