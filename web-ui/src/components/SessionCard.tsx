import { Link } from '@tanstack/react-router';
import type { SessionState } from '../lib/types';

interface SessionCardProps {
  id: string;
  name?: string;
  state: SessionState | string;
  createdAt: string | number;
  isOwner?: boolean;
  subscriberCount?: number;
  onKill?: () => void;
}

const stateLabels: Record<string, string> = {
  idle: 'Idle',
  Idle: 'Idle',
  processing: 'Processing',
  Processing: 'Processing',
  waiting_permission: 'Waiting',
  WaitingPermission: 'Waiting',
  finished: 'Finished',
  Finished: 'Finished',
  failed: 'Failed',
  Failed: 'Failed',
};

function formatDate(createdAt: string | number): string {
  if (typeof createdAt === 'number') {
    return new Date(createdAt * 1000).toLocaleString();
  }
  return new Date(createdAt).toLocaleString();
}

/** Check if session state indicates active processing */
function isActiveState(state: SessionState | string): boolean {
  const normalized = typeof state === 'string' ? state.toLowerCase().replace(/_/g, '') : state;
  return normalized === 'processing' || normalized === 'waiting' || normalized === 'waitingpermission';
}

export function SessionCard({
  id,
  name,
  state,
  createdAt,
  isOwner,
  subscriberCount,
  onKill,
}: SessionCardProps) {
  const displayName = name || id.slice(0, 8);
  const shortId = id.slice(0, 8);
  const formattedDate = formatDate(createdAt);
  const stateKey = typeof state === 'string' ? state.toLowerCase().replace(/_/g, '') : state;
  const stateLabel = stateLabels[state] || state;
  const isActive = isActiveState(state);

  const handleKill = (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (onKill && confirm(`Kill session "${displayName}"?`)) {
      onKill();
    }
  };

  const cardClassName = `session-card ${isActive ? 'session-active' : 'session-inactive'}`;

  return (
    <Link to="/sessions/$sessionId" params={{ sessionId: id }} className={cardClassName}>
      <div className="session-card-header">
        <h3>
          <span className={`status-dot status-dot-${stateKey}`} />
          {displayName}
          {isOwner && <span className="owner-badge" title="You own this session">&#x2605;</span>}
        </h3>
        <span className={`status status-${stateKey}`}>{stateLabel}</span>
      </div>
      <div className="session-card-meta">
        <span className="session-id">{shortId}</span>
        {subscriberCount !== undefined && (
          <span className="subscriber-count" title="Connected clients">
            &#x1F465; {subscriberCount}
          </span>
        )}
        <span className="session-date">{formattedDate}</span>
      </div>
      {onKill && (
        <button
          className="session-kill-btn"
          onClick={handleKill}
          title="Kill session"
        >
          &#x2715;
        </button>
      )}
    </Link>
  );
}
