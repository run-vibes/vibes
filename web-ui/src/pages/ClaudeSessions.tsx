import { useNavigate } from '@tanstack/react-router';
import { SessionList } from '../components/SessionList';
import { useSessionList, useWebSocket } from '../hooks';

export function ClaudeSessions() {
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
      navigate({ to: '/claude/$sessionId', params: { sessionId } });
    } catch (err) {
      console.error('Failed to create session:', err);
    }
  };

  return (
    <div className="page">
      <div className="page-header">
        <h1>Claude Sessions</h1>
        <span className={`connection-status connection-${connectionState}`}>
          {connectionState}
        </span>
      </div>
      <SessionList
        sessions={sessions}
        isLoading={isLoading}
        isCreating={isCreating}
        error={error}
        onKill={killSession}
        onRefresh={refresh}
        onCreateSession={handleCreateSession}
      />
    </div>
  );
}
