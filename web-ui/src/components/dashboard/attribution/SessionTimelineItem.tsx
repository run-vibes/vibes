import { useState } from 'react';
import type { SessionTimelineEntry } from '../../../hooks/useDashboard';
import './SessionTimelineItem.css';

export interface SessionTimelineItemProps {
  session: SessionTimelineEntry;
  onClick?: (sessionId: string) => void;
}

function formatContribution(value: number): string {
  const sign = value >= 0 ? '+' : '';
  return `${sign}${value.toFixed(2)}`;
}

export function SessionTimelineItem({ session, onClick }: SessionTimelineItemProps) {
  const [expanded, setExpanded] = useState(false);

  const handleClick = () => {
    setExpanded(!expanded);
    onClick?.(session.session_id);
  };

  const learningCount = session.activated_learnings.length;
  const isNegative = session.outcome === 'negative';

  return (
    <div className={`timeline-item ${isNegative ? 'timeline-item--negative' : ''}`}>
      <button
        type="button"
        className="timeline-item__header"
        onClick={handleClick}
        aria-expanded={expanded}
      >
        <span className="timeline-item__session-id">{session.session_id}</span>

        <div className="timeline-item__score">
          <span className="timeline-item__score-value">{session.score.toFixed(2)}</span>
          <div
            className="timeline-item__score-bar"
            role="progressbar"
            aria-valuenow={session.score * 100}
            aria-valuemin={0}
            aria-valuemax={100}
          >
            <div
              className="timeline-item__score-bar-fill"
              style={{ width: `${session.score * 100}%` }}
            />
          </div>
        </div>

        <span className="timeline-item__count">{learningCount} learnings</span>

        {isNegative && <span className="timeline-item__outcome">negative</span>}

        <span className="timeline-item__expand">{expanded ? '▼' : '▶'}</span>
      </button>

      {expanded && (
        <div className="timeline-item__details">
          <ul className="timeline-item__learnings">
            {session.activated_learnings.map((learning) => (
              <li key={learning.learning_id} className="timeline-item__learning">
                <span className="timeline-item__learning-content">{learning.content}</span>
                <span
                  className={`timeline-item__learning-contribution ${
                    learning.contribution < 0 ? 'timeline-item__learning-contribution--negative' : ''
                  }`}
                >
                  {formatContribution(learning.contribution)}
                </span>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
