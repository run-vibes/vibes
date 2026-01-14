import { EmptyState } from '@vibes/design-system';
import { SessionCard } from './SessionCard';
import type { SessionInfo } from '../lib/types';

interface SessionListProps {
  sessions: SessionInfo[];
  isLoading: boolean;
  isCreating?: boolean;
  error: string | null;
  onKill?: (sessionId: string) => void;
  onRefresh?: () => void;
  onCreateSession?: () => void;
}

export function SessionList({
  sessions,
  isLoading,
  isCreating = false,
  error,
  onKill,
  onRefresh,
  onCreateSession,
}: SessionListProps) {
  if (isLoading && sessions.length === 0) {
    return <p className="loading-message">Loading sessions...</p>;
  }

  if (error) {
    return (
      <div className="error-container">
        <p className="error">Error: {error}</p>
        {onRefresh && (
          <button onClick={onRefresh} className="btn btn-secondary">
            Retry
          </button>
        )}
      </div>
    );
  }

  if (sessions.length === 0) {
    return (
      <EmptyState
        icon="📡"
        message="No active sessions"
        hint={onCreateSession ? undefined : 'Start one with vibes claude "your prompt"'}
        action={
          onCreateSession && (
            <button
              onClick={onCreateSession}
              className="btn btn-primary"
              disabled={isCreating}
            >
              {isCreating ? 'Creating...' : 'New Session'}
            </button>
          )
        }
      />
    );
  }

  return (
    <div className="session-list">
      <div className="session-list-header">
        <span className="session-count">
          {sessions.length} active session{sessions.length !== 1 ? 's' : ''}
        </span>
        <div className="session-list-actions">
          {onCreateSession && (
            <button
              onClick={onCreateSession}
              className="btn btn-primary btn-sm"
              disabled={isCreating}
            >
              {isCreating ? 'Creating...' : 'New Session'}
            </button>
          )}
          {onRefresh && (
            <button
              onClick={onRefresh}
              className="btn btn-icon"
              title="Refresh"
              disabled={isLoading}
            >
              🔄
            </button>
          )}
        </div>
      </div>
      <div className="session-grid">
        {sessions.map((session) => (
          <SessionCard
            key={session.id}
            id={session.id}
            name={session.name}
            state={session.state}
            createdAt={session.created_at}
            isOwner={session.is_owner}
            subscriberCount={session.subscriber_count}
            onKill={onKill ? () => onKill(session.id) : undefined}
          />
        ))}
      </div>
    </div>
  );
}
