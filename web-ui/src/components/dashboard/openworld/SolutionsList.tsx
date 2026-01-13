/**
 * Solutions list with grouping by status
 *
 * Displays solutions organized into sections:
 * - Pending Review
 * - Applied
 * - Dismissed
 */
import { Card } from '@vibes/design-system';
import type { SolutionEntry, SolutionStatus } from '../../../hooks/useDashboard';
import { SolutionItem } from './SolutionItem';
import './SolutionsList.css';

export interface SolutionsListProps {
  solutions?: SolutionEntry[];
  total?: number;
  isLoading?: boolean;
  onApply?: (id: string) => void;
  onDismiss?: (id: string) => void;
  actionLoading?: string; // ID of solution being actioned
}

interface GroupedSolutions {
  pending: SolutionEntry[];
  applied: SolutionEntry[];
  dismissed: SolutionEntry[];
}

function groupSolutions(solutions: SolutionEntry[]): GroupedSolutions {
  return solutions.reduce<GroupedSolutions>(
    (groups, solution) => {
      switch (solution.status) {
        case 'Pending':
          groups.pending.push(solution);
          break;
        case 'Applied':
          groups.applied.push(solution);
          break;
        case 'Dismissed':
          groups.dismissed.push(solution);
          break;
      }
      return groups;
    },
    { pending: [], applied: [], dismissed: [] }
  );
}

const STATUS_LABELS: Record<SolutionStatus, string> = {
  Pending: 'Pending Review',
  Applied: 'Applied',
  Dismissed: 'Dismissed',
};

export function SolutionsList({
  solutions,
  total,
  isLoading,
  onApply,
  onDismiss,
  actionLoading,
}: SolutionsListProps) {
  if (isLoading) {
    return (
      <Card variant="crt" title="Suggested Solutions" className="solutions-list">
        <div className="solutions-list__loading">
          <span>Loading solutions...</span>
        </div>
      </Card>
    );
  }

  const grouped = groupSolutions(solutions || []);
  const isEmpty = !solutions?.length;

  return (
    <Card variant="crt" title="Suggested Solutions" className="solutions-list">
      <div className="solutions-list__header">
        <span className="solutions-list__count">
          {total ?? solutions?.length ?? 0} solutions
        </span>
      </div>

      {isEmpty ? (
        <div className="solutions-list__empty" data-testid="solutions-list-empty">
          <span className="solutions-list__empty-icon">◇</span>
          <span>No solutions pending review</span>
        </div>
      ) : (
        <div className="solutions-list__groups">
          {grouped.pending.length > 0 && (
            <SolutionGroup
              status="Pending"
              solutions={grouped.pending}
              onApply={onApply}
              onDismiss={onDismiss}
              actionLoading={actionLoading}
            />
          )}

          {grouped.applied.length > 0 && (
            <SolutionGroup
              status="Applied"
              solutions={grouped.applied}
              onApply={onApply}
              onDismiss={onDismiss}
              actionLoading={actionLoading}
            />
          )}

          {grouped.dismissed.length > 0 && (
            <SolutionGroup
              status="Dismissed"
              solutions={grouped.dismissed}
              onApply={onApply}
              onDismiss={onDismiss}
              actionLoading={actionLoading}
            />
          )}
        </div>
      )}
    </Card>
  );
}

interface SolutionGroupProps {
  status: SolutionStatus;
  solutions: SolutionEntry[];
  onApply?: (id: string) => void;
  onDismiss?: (id: string) => void;
  actionLoading?: string;
}

function SolutionGroup({
  status,
  solutions,
  onApply,
  onDismiss,
  actionLoading,
}: SolutionGroupProps) {
  return (
    <div className="solutions-list__group" data-testid={`group-${status.toLowerCase()}`}>
      <h4 className="solutions-list__group-title">
        {STATUS_LABELS[status]}
        <span className="solutions-list__group-count">{solutions.length}</span>
      </h4>
      <div className="solutions-list__items">
        {solutions.map((solution) => (
          <SolutionItem
            key={solution.id}
            solution={solution}
            onApply={onApply}
            onDismiss={onDismiss}
            isLoading={actionLoading === solution.id}
          />
        ))}
      </div>
    </div>
  );
}
