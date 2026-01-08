import { useNavigate } from '@tanstack/react-router';
import { Badge, Button, SessionCard, type SessionStatus } from '@vibes/design-system';
import { useSessionList, useWebSocket } from '../hooks';
import './Sessions.css';

// Map backend session state to design-system SessionStatus
function mapStateToStatus(state: string): SessionStatus {
  switch (state) {
    case 'processing':
      return 'processing';
    case 'waiting':
      return 'waiting';
    case 'finished':
    case 'completed':
      return 'finished';
    case 'failed':
    case 'error':
      return 'failed';
    default:
      return 'idle';
  }
}

export function Sessions() {
  const navigate = useNavigate();
  const { send, addMessageHandler, isConnected, connectionState } = useWebSocket();
  const { sessions, isLoading, isCreating, error, refresh, killSession, createSession } = useSessionList({
    send,
    addMessageHandler,
    isConnected,
    autoRefresh: true,
  });

  const handleCreateSession = async () => {
    try {
      const sessionId = await createSession();
      navigate({ to: '/sessions/$sessionId', params: { sessionId } });
    } catch (err) {
      console.error('Failed to create session:', err);
    }
  };

  return (
    <div className="sessions-page">
      {/* Header */}
      <div className="sessions-header">
        <div className="sessions-header-left">
          <h1 className="sessions-title">SESSIONS</h1>
          <div className="sessions-status">
            {isConnected ? (
              <Badge status="success">Connected</Badge>
            ) : (
              <Badge status="error">{connectionState}</Badge>
            )}
          </div>
        </div>

        <div className="sessions-header-right">
          <span className="sessions-count">
            {sessions.length} session{sessions.length !== 1 ? 's' : ''}
          </span>
          <Button
            variant="primary"
            size="sm"
            onClick={handleCreateSession}
            disabled={isCreating}
          >
            {isCreating ? 'Creating...' : '+ New'}
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={refresh}
            disabled={isLoading}
          >
            ↻
          </Button>
        </div>
      </div>

      {/* Content */}
      <div className="sessions-content">
        {isLoading && sessions.length === 0 ? (
          <div className="sessions-loading">Loading sessions...</div>
        ) : error ? (
          <div className="sessions-error">
            <div className="sessions-error-text">Error: {error}</div>
            <Button variant="secondary" size="sm" onClick={refresh}>
              Retry
            </Button>
          </div>
        ) : sessions.length === 0 ? (
          <div className="sessions-empty">
            <div className="sessions-empty-text">No active sessions</div>
            <div className="sessions-empty-hint">
              Start one with <code>vibes claude "your prompt"</code>
            </div>
            <Button variant="primary" onClick={handleCreateSession} disabled={isCreating}>
              {isCreating ? 'Creating...' : 'New Session'}
            </Button>
          </div>
        ) : (
          <div className="sessions-grid">
            {sessions.map((session) => (
              <SessionCard
                key={session.id}
                id={session.id}
                name={session.name}
                status={mapStateToStatus(session.state)}
                updatedAt={new Date(session.last_activity_at)}
                subscribers={session.subscriber_count}
                actions={[
                  {
                    icon: '×',
                    label: 'Kill session',
                    onClick: () => killSession(session.id),
                  },
                ]}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
