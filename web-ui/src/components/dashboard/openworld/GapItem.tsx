import type { GapBrief } from '../../../hooks/useDashboard';
import { GapSeverityBadge } from './GapSeverityBadge';
import './GapItem.css';

export interface GapItemProps {
  gap: GapBrief;
  isSelected?: boolean;
  onClick?: () => void;
}

function formatTimeAgo(dateStr: string): string {
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMinutes = Math.floor(diffMs / (1000 * 60));
  const diffHours = Math.floor(diffMinutes / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffMinutes < 1) return 'now';
  if (diffMinutes < 60) return `${diffMinutes}m`;
  if (diffHours < 24) return `${diffHours}h`;
  if (diffDays === 1) return '1d';
  return `${diffDays}d`;
}

const CATEGORY_LABELS: Record<string, string> = {
  MissingKnowledge: 'Knowledge',
  IncorrectPattern: 'Pattern',
  ContextMismatch: 'Context',
  ToolGap: 'Tool',
};

export function GapItem({ gap, isSelected, onClick }: GapItemProps) {
  const { id, category, severity, status, context_pattern, failure_count, last_seen, solution_count } = gap;

  return (
    <button
      type="button"
      className={`gap-item ${isSelected ? 'gap-item--selected' : ''}`}
      onClick={onClick}
      data-testid={`gap-${id}`}
    >
      <div className="gap-item__header">
        <GapSeverityBadge severity={severity} />
        <span className="gap-item__category">{CATEGORY_LABELS[category] || category}</span>
        <span className="gap-item__status">{status}</span>
      </div>
      <div className="gap-item__context">{context_pattern}</div>
      <div className="gap-item__meta">
        <span className="gap-item__failures">
          <span className="gap-item__failures-count">{failure_count}</span> failures
        </span>
        <span className="gap-item__solutions">
          {solution_count > 0 ? `${solution_count} solutions` : 'No solutions'}
        </span>
        <span className="gap-item__time">{formatTimeAgo(last_seen)}</span>
      </div>
    </button>
  );
}
