import { useEffect, useState } from 'react';
import type { AccessIdentity, ServerMessage } from '../lib/types';

export interface AuthState {
  source: 'local' | 'authenticated' | 'anonymous' | 'unknown';
  identity: AccessIdentity | null;
  isAuthenticated: boolean;
  isLocal: boolean;
  isLoading: boolean;
}

const initialState: AuthState = {
  source: 'unknown',
  identity: null,
  isAuthenticated: false,
  isLocal: false,
  isLoading: true,
};

interface UseAuthOptions {
  /** Message handler registration function from useWebSocket */
  addMessageHandler: (handler: (message: ServerMessage) => void) => () => void;
}

/**
 * Hook to track authentication state from WebSocket connection
 *
 * The server sends an auth_context message immediately after WebSocket connection,
 * which this hook captures to determine the client's authentication status.
 */
export function useAuth({ addMessageHandler }: UseAuthOptions): AuthState {
  const [authState, setAuthState] = useState<AuthState>(initialState);

  useEffect(() => {
    const cleanup = addMessageHandler((message: ServerMessage) => {
      if (message.type === 'auth_context') {
        const source = message.source;
        setAuthState({
          source,
          identity: source === 'authenticated' ? message.identity : null,
          isAuthenticated: source === 'authenticated',
          isLocal: source === 'local',
          isLoading: false,
        });
      }
    });

    return cleanup;
  }, [addMessageHandler]);

  return authState;
}

/**
 * Type guard for auth_context messages
 */
export function isAuthContextMessage(
  msg: ServerMessage
): msg is Extract<ServerMessage, { type: 'auth_context' }> {
  return msg.type === 'auth_context';
}
