import { useCallback, useEffect, useState } from 'react';
import type { ClientMessage, ServerMessage, SessionInfo } from '../lib/types';

function generateRequestId(): string {
  return `${Date.now()}-${Math.random().toString(36).slice(2, 11)}`;
}

interface UseSessionListOptions {
  /** Function to send WebSocket messages */
  send: (message: ClientMessage) => void;
  /** Function to add a message handler */
  addMessageHandler: (handler: (message: ServerMessage) => void) => () => void;
  /** Whether the WebSocket is connected */
  isConnected: boolean;
  /** Auto-refresh on connect (default: true) */
  autoRefresh?: boolean;
  /** Refresh interval in ms (default: 0 = no auto-refresh) */
  refreshInterval?: number;
}

interface UseSessionListReturn {
  /** List of active sessions */
  sessions: SessionInfo[];
  /** Whether the list is currently loading */
  isLoading: boolean;
  /** Error message if fetch failed */
  error: string | null;
  /** Refresh the session list */
  refresh: () => void;
  /** Kill a session by ID */
  killSession: (sessionId: string) => void;
}

/**
 * Hook for managing the list of active sessions
 */
export function useSessionList(options: UseSessionListOptions): UseSessionListReturn {
  const {
    send,
    addMessageHandler,
    isConnected,
    autoRefresh = true,
    refreshInterval = 0,
  } = options;

  const [sessions, setSessions] = useState<SessionInfo[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [pendingRequestId, setPendingRequestId] = useState<string | null>(null);

  // Refresh the session list
  const refresh = useCallback(() => {
    if (!isConnected) {
      setError('Not connected');
      return;
    }

    const requestId = generateRequestId();
    setPendingRequestId(requestId);
    setIsLoading(true);
    setError(null);

    send({ type: 'list_sessions', request_id: requestId });
  }, [isConnected, send]);

  // Kill a session
  const killSession = useCallback(
    (sessionId: string) => {
      if (!isConnected) {
        return;
      }
      send({ type: 'kill_session', session_id: sessionId });
    },
    [isConnected, send]
  );

  // Handle incoming messages
  useEffect(() => {
    const cleanup = addMessageHandler((message: ServerMessage) => {
      switch (message.type) {
        case 'session_list':
          if (message.request_id === pendingRequestId) {
            setSessions(message.sessions);
            setIsLoading(false);
            setPendingRequestId(null);
          }
          break;

        case 'session_removed':
          setSessions((prev) =>
            prev.filter((s) => s.id !== message.session_id)
          );
          break;

        case 'ownership_transferred':
          setSessions((prev) =>
            prev.map((s) =>
              s.id === message.session_id
                ? {
                    ...s,
                    owner_id: message.new_owner_id,
                    is_owner: message.you_are_owner,
                  }
                : s
            )
          );
          break;

        case 'session_created':
          // A new session was created - refresh the list to get full details
          refresh();
          break;

        case 'session_state':
          // Update the state of an existing session
          setSessions((prev) =>
            prev.map((s) =>
              s.id === message.session_id ? { ...s, state: message.state } : s
            )
          );
          break;

        case 'error':
          if (pendingRequestId) {
            setError(message.message);
            setIsLoading(false);
            setPendingRequestId(null);
          }
          break;
      }
    });

    return cleanup;
  }, [addMessageHandler, pendingRequestId, refresh]);

  // Auto-refresh on connect
  useEffect(() => {
    if (isConnected && autoRefresh) {
      refresh();
    }
  }, [isConnected, autoRefresh, refresh]);

  // Periodic refresh
  useEffect(() => {
    if (!isConnected || refreshInterval <= 0) {
      return;
    }

    const intervalId = setInterval(refresh, refreshInterval);
    return () => clearInterval(intervalId);
  }, [isConnected, refreshInterval, refresh]);

  return {
    sessions,
    isLoading,
    error,
    refresh,
    killSession,
  };
}
