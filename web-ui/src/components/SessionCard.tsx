import { Link } from '@tanstack/react-router';
import type { SessionState } from '../lib/types';

interface SessionCardProps {
  id: string;
  name?: string;
  state: SessionState;
  createdAt: string;
}

const stateLabels: Record<SessionState, string> = {
  idle: 'Idle',
  processing: 'Processing',
  waiting_permission: 'Waiting',
  finished: 'Finished',
  failed: 'Failed',
};

export function SessionCard({ id, name, state, createdAt }: SessionCardProps) {
  const displayName = name || id.slice(0, 8);
  const shortId = id.slice(0, 8);
  const formattedDate = new Date(createdAt).toLocaleString();

  return (
    <Link to="/claude/$sessionId" params={{ sessionId: id }} className="session-card">
      <div className="session-card-header">
        <h3>{displayName}</h3>
        <span className={`status status-${state}`}>{stateLabels[state]}</span>
      </div>
      <div className="session-card-meta">
        <span className="session-id">{shortId}</span>
        <span className="session-date">{formattedDate}</span>
      </div>
    </Link>
  );
}
