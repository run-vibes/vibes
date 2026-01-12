/**
 * Individual activity event row
 *
 * Shows timestamp, event type badge, and message.
 */
import type { OpenWorldActivityEntry } from '../../../hooks/useDashboard';
import { ResponseActionBadge } from './ResponseActionBadge';
import './ActivityItem.css';

export interface ActivityItemProps {
  event: OpenWorldActivityEntry;
}

function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffSecs = Math.floor(diffMs / 1000);
  const diffMins = Math.floor(diffSecs / 60);
  const diffHours = Math.floor(diffMins / 60);

  if (diffSecs < 60) {
    return 'just now';
  } else if (diffMins < 60) {
    return `${diffMins}m ago`;
  } else if (diffHours < 24) {
    return `${diffHours}h ago`;
  } else {
    return date.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
  }
}

export function ActivityItem({ event }: ActivityItemProps) {
  const { timestamp, event_type, message, gap_id, learning_id } = event;

  return (
    <div className="activity-item" data-testid={`activity-${timestamp}`}>
      <div className="activity-item__header">
        <ResponseActionBadge eventType={event_type} />
        <span className="activity-item__time">{formatTimestamp(timestamp)}</span>
      </div>
      <p className="activity-item__message">{message}</p>
      {(gap_id || learning_id) && (
        <div className="activity-item__refs">
          {gap_id && (
            <span className="activity-item__ref" data-testid="activity-gap-ref">
              Gap: {gap_id.slice(0, 8)}
            </span>
          )}
          {learning_id && (
            <span className="activity-item__ref" data-testid="activity-learning-ref">
              Learning: {learning_id.slice(0, 8)}
            </span>
          )}
        </div>
      )}
    </div>
  );
}
