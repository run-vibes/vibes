/**
 * Colored badge for response action/event types
 */
import type { OpenWorldEventType } from '../../../hooks/useDashboard';
import './ResponseActionBadge.css';

export interface ResponseActionBadgeProps {
  eventType: OpenWorldEventType;
}

const EVENT_TYPE_LABELS: Record<OpenWorldEventType, string> = {
  novelty_detected: 'Novelty',
  cluster_updated: 'Cluster',
  gap_created: 'Gap',
  gap_status_changed: 'Status',
  solution_generated: 'Solution',
  strategy_feedback: 'Feedback',
};

const EVENT_TYPE_VARIANTS: Record<OpenWorldEventType, 'info' | 'warning' | 'success' | 'error'> = {
  novelty_detected: 'info',
  cluster_updated: 'info',
  gap_created: 'warning',
  gap_status_changed: 'info',
  solution_generated: 'success',
  strategy_feedback: 'success',
};

export function ResponseActionBadge({ eventType }: ResponseActionBadgeProps) {
  const label = EVENT_TYPE_LABELS[eventType] || eventType;
  const variant = EVENT_TYPE_VARIANTS[eventType] || 'info';

  return (
    <span
      className={`response-action-badge response-action-badge--${variant}`}
      data-testid={`event-type-${eventType}`}
    >
      {label}
    </span>
  );
}
