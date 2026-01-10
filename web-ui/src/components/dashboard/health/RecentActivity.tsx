import type { ActivityEvent, EventType } from '../../../hooks/useDashboard';
import './RecentActivity.css';

export interface RecentActivityProps {
  events: ActivityEvent[];
  maxItems?: number;
}

function getEventIcon(type: EventType): string {
  switch (type) {
    case 'extraction':
      return '⚡';
    case 'attribution':
      return '📊';
    case 'strategy':
      return '🎯';
    case 'error':
      return '⚠️';
  }
}

function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp);
  return date.toLocaleTimeString(undefined, {
    hour: '2-digit',
    minute: '2-digit',
  });
}

export function RecentActivity({ events, maxItems }: RecentActivityProps) {
  const displayEvents = maxItems ? events.slice(0, maxItems) : events;

  if (displayEvents.length === 0) {
    return (
      <div className="recent-activity recent-activity--empty">
        No recent activity
      </div>
    );
  }

  return (
    <div className="recent-activity">
      <ul className="recent-activity__list">
        {displayEvents.map((event) => (
          <li key={event.id} className={`recent-activity__item recent-activity__item--${event.type}`}>
            <span className="recent-activity__icon">{getEventIcon(event.type)}</span>
            <span className="recent-activity__description">{event.description}</span>
            <time className="recent-activity__time" dateTime={event.timestamp}>
              {formatTimestamp(event.timestamp)}
            </time>
          </li>
        ))}
      </ul>
    </div>
  );
}
