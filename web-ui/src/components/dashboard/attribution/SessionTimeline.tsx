import { EmptyState } from '@vibes/design-system';
import { SessionTimelineItem } from './SessionTimelineItem';
import type { SessionTimelineEntry, SessionOutcome } from '../../../hooks/useDashboard';
import './SessionTimeline.css';

export interface SessionTimelineProps {
  sessions: SessionTimelineEntry[];
  period?: number;
  onPeriodChange?: (days: number) => void;
  onOutcomeFilter?: (outcome: SessionOutcome | 'all') => void;
  onSessionClick?: (sessionId: string) => void;
}

interface GroupedSessions {
  label: string;
  sessions: SessionTimelineEntry[];
}

function getDayLabel(timestamp: string): string {
  const date = new Date(timestamp);
  const today = new Date();
  const yesterday = new Date(today);
  yesterday.setDate(yesterday.getDate() - 1);

  // Compare dates (ignoring time)
  const dateStr = date.toDateString();
  if (dateStr === today.toDateString()) {
    return 'Today';
  }
  if (dateStr === yesterday.toDateString()) {
    return 'Yesterday';
  }

  return date.toLocaleDateString('en-US', {
    weekday: 'long',
    month: 'short',
    day: 'numeric',
  });
}

function groupSessionsByDay(sessions: SessionTimelineEntry[]): GroupedSessions[] {
  const groups = new Map<string, SessionTimelineEntry[]>();

  sessions.forEach((session) => {
    const label = getDayLabel(session.timestamp);
    const existing = groups.get(label) || [];
    groups.set(label, [...existing, session]);
  });

  // Convert to array and sort by most recent first
  return Array.from(groups.entries()).map(([label, sessions]) => ({
    label,
    sessions,
  }));
}

export function SessionTimeline({
  sessions,
  period = 7,
  onPeriodChange,
  onOutcomeFilter,
  onSessionClick,
}: SessionTimelineProps) {
  const handlePeriodChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    onPeriodChange?.(parseInt(e.target.value, 10));
  };

  const handleOutcomeChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const value = e.target.value as SessionOutcome | 'all';
    onOutcomeFilter?.(value);
  };

  const groupedSessions = groupSessionsByDay(sessions);

  return (
    <div className="session-timeline">
      <header className="session-timeline__header">
        <h3 className="session-timeline__title">Session Timeline</h3>
        <div className="session-timeline__filters">
          <label className="session-timeline__filter">
            <span className="visually-hidden">Outcome</span>
            <select
              className="session-timeline__select"
              onChange={handleOutcomeChange}
              aria-label="Outcome"
              defaultValue="all"
            >
              <option value="all">All Outcomes</option>
              <option value="positive">Positive</option>
              <option value="negative">Negative</option>
              <option value="neutral">Neutral</option>
            </select>
          </label>
          <label className="session-timeline__filter">
            <span className="visually-hidden">Period</span>
            <select
              className="session-timeline__select"
              value={period}
              onChange={handlePeriodChange}
              aria-label="Period"
            >
              <option value="7">Last 7 days</option>
              <option value="30">Last 30 days</option>
              <option value="90">Last 90 days</option>
            </select>
          </label>
        </div>
      </header>

      {sessions.length === 0 ? (
        <EmptyState message="No sessions found for this period" size="sm" />
      ) : (
        <ul className="session-timeline__groups" role="list">
          {groupedSessions.map((group) => (
            <li key={group.label} className="session-timeline__group">
              <h4 className="session-timeline__day">{group.label}</h4>
              <div className="session-timeline__sessions">
                {group.sessions.map((session) => (
                  <SessionTimelineItem
                    key={session.session_id}
                    session={session}
                    onClick={onSessionClick}
                  />
                ))}
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
