import type { LearningBrief } from '../../../hooks/useDashboard';
import { LearningStatusBadge } from './LearningStatusBadge';
import { ValueBar } from './ValueBar';
import './LearningsList.css';

export interface LearningsListProps {
  learnings: LearningBrief[];
  selectedId?: string;
  isLoading?: boolean;
  onSelect: (id: string) => void;
}

export function LearningsList({
  learnings,
  selectedId,
  isLoading,
  onSelect,
}: LearningsListProps) {
  if (isLoading) {
    return (
      <div className="learnings-list">
        <p className="learnings-list__loading">Loading learnings...</p>
      </div>
    );
  }

  if (learnings.length === 0) {
    return (
      <div className="learnings-list">
        <p className="learnings-list__empty">No learnings found</p>
      </div>
    );
  }

  return (
    <ul className="learnings-list" role="list">
      {learnings.map((learning) => (
        <li
          key={learning.id}
          role="listitem"
          className={`learning-item ${selectedId === learning.id ? 'learning-item--selected' : ''}`}
          onClick={() => onSelect(learning.id)}
        >
          <div className="learning-item__header">
            <span className="learning-item__category">{learning.category}</span>
            <LearningStatusBadge status={learning.status} size="small" />
          </div>

          <p className="learning-item__content">{learning.content}</p>

          <div className="learning-item__footer">
            <ValueBar value={learning.estimated_value} />
          </div>
        </li>
      ))}
    </ul>
  );
}
