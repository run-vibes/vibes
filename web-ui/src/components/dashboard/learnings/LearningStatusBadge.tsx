import type { LearningStatus } from '../../../hooks/useDashboard';
import './LearningStatusBadge.css';

export interface LearningStatusBadgeProps {
  status: LearningStatus;
  size?: 'small' | 'normal';
}

const STATUS_LABELS: Record<LearningStatus, string> = {
  active: 'Active',
  disabled: 'Disabled',
  under_review: 'Under Review',
  deprecated: 'Deprecated',
};

export function LearningStatusBadge({ status, size = 'normal' }: LearningStatusBadgeProps) {
  const sizeClass = size === 'small' ? 'status-badge--small' : '';

  return (
    <span className={`status-badge status-badge--${status} ${sizeClass}`}>
      {STATUS_LABELS[status]}
    </span>
  );
}
