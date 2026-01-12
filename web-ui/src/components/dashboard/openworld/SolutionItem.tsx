/**
 * Individual solution item row
 *
 * Shows solution details: action type, gap context, confidence, and actions.
 */
import type { SolutionEntry } from '../../../hooks/useDashboard';
import { SolutionConfidenceBadge } from './SolutionConfidenceBadge';
import { SolutionActions } from './SolutionActions';
import './SolutionItem.css';

export interface SolutionItemProps {
  solution: SolutionEntry;
  onApply?: (id: string) => void;
  onDismiss?: (id: string) => void;
  isLoading?: boolean;
}

const ACTION_TYPE_LABELS: Record<string, string> = {
  AddKnowledge: 'Add Knowledge',
  UpdatePattern: 'Update Pattern',
  AddTool: 'Add Tool',
  ContextAdjustment: 'Context Adjustment',
};

export function SolutionItem({ solution, onApply, onDismiss, isLoading }: SolutionItemProps) {
  const { id, action_type, description, confidence, gap_context, status } = solution;

  return (
    <div className="solution-item" data-testid={`solution-${id}`}>
      <div className="solution-item__header">
        <span className="solution-item__type">
          {ACTION_TYPE_LABELS[action_type] || action_type}
        </span>
        <SolutionConfidenceBadge confidence={confidence} />
      </div>

      <p className="solution-item__description">{description}</p>

      <div className="solution-item__context">
        <span className="solution-item__context-label">Gap:</span>
        <span className="solution-item__context-value">{gap_context}</span>
      </div>

      <div className="solution-item__footer">
        <SolutionActions
          solutionId={id}
          status={status}
          onApply={onApply}
          onDismiss={onDismiss}
          isLoading={isLoading}
        />
      </div>
    </div>
  );
}
