import { SessionList } from '../components/SessionList';
import { useSessionList, useWebSocket } from '../hooks';

export function ClaudeSessions() {
  const { send, addMessageHandler, isConnected, connectionState } = useWebSocket();
  const { sessions, isLoading, error, refresh, killSession } = useSessionList({
    send,
    addMessageHandler,
    isConnected,
    autoRefresh: true,
  });

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
        error={error}
        onKill={killSession}
        onRefresh={refresh}
      />
    </div>
  );
}
