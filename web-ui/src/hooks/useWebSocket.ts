import { useCallback, useEffect, useRef, useState } from 'react';
import type { ClientMessage, ServerMessage } from '../lib/types';

type ConnectionState = 'connecting' | 'connected' | 'disconnected';
type MessageHandler = (message: ServerMessage) => void;

interface UseWebSocketOptions {
  /** Auto-connect on mount (default: true) */
  autoConnect?: boolean;
  /** Reconnection delay in ms (default: 3000) */
  reconnectDelay?: number;
  /** Maximum reconnection attempts (default: 5) */
  maxReconnectAttempts?: number;
}

interface UseWebSocketReturn {
  /** Current connection state */
  connectionState: ConnectionState;
  /** Whether the socket is connected */
  isConnected: boolean;
  /** Send a message to the server */
  send: (message: ClientMessage) => void;
  /** Subscribe to messages for specific sessions */
  subscribe: (sessionIds: string[]) => void;
  /** Unsubscribe from session messages */
  unsubscribe: (sessionIds: string[]) => void;
  /** Add a message handler (returns cleanup function) */
  addMessageHandler: (handler: MessageHandler) => () => void;
  /** Manually connect */
  connect: () => void;
  /** Manually disconnect */
  disconnect: () => void;
}

/**
 * WebSocket hook for connecting to vibes daemon
 */
export function useWebSocket(options: UseWebSocketOptions = {}): UseWebSocketReturn {
  const {
    autoConnect = true,
    reconnectDelay = 3000,
    maxReconnectAttempts = 5,
  } = options;

  const [connectionState, setConnectionState] = useState<ConnectionState>('disconnected');
  const wsRef = useRef<WebSocket | null>(null);
  const handlersRef = useRef<Set<MessageHandler>>(new Set());
  const reconnectAttemptsRef = useRef(0);
  const reconnectTimeoutRef = useRef<number | null>(null);

  // Clear any pending reconnect
  const clearReconnect = useCallback(() => {
    if (reconnectTimeoutRef.current !== null) {
      window.clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }
  }, []);

  // Disconnect handler
  const disconnect = useCallback(() => {
    clearReconnect();
    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }
    setConnectionState('disconnected');
  }, [clearReconnect]);

  // Connect handler
  const connect = useCallback(() => {
    // Don't create duplicate connections
    if (wsRef.current?.readyState === WebSocket.OPEN ||
        wsRef.current?.readyState === WebSocket.CONNECTING) {
      return;
    }

    setConnectionState('connecting');

    // Use relative URL - Vite proxy handles /ws
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const url = `${protocol}//${window.location.host}/ws`;

    const ws = new WebSocket(url);
    wsRef.current = ws;

    ws.onopen = () => {
      console.log('[vibes] WebSocket connected');
      setConnectionState('connected');
      reconnectAttemptsRef.current = 0;
    };

    ws.onclose = (event) => {
      console.log('[vibes] WebSocket closed:', event.code, event.reason);
      wsRef.current = null;
      setConnectionState('disconnected');

      // Attempt reconnection if not a clean close
      if (event.code !== 1000 && reconnectAttemptsRef.current < maxReconnectAttempts) {
        reconnectAttemptsRef.current++;
        console.log(`[vibes] Reconnecting (attempt ${reconnectAttemptsRef.current}/${maxReconnectAttempts})...`);
        reconnectTimeoutRef.current = window.setTimeout(connect, reconnectDelay);
      }
    };

    ws.onerror = (error) => {
      console.error('[vibes] WebSocket error:', error);
    };

    ws.onmessage = (event) => {
      try {
        const message: ServerMessage = JSON.parse(event.data);
        // Notify all handlers
        handlersRef.current.forEach((handler) => {
          try {
            handler(message);
          } catch (e) {
            console.error('[vibes] Message handler error:', e);
          }
        });
      } catch (e) {
        console.error('[vibes] Failed to parse message:', e);
      }
    };
  }, [maxReconnectAttempts, reconnectDelay]);

  // Send message
  const send = useCallback((message: ClientMessage) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(message));
    } else {
      console.warn('[vibes] Cannot send - WebSocket not connected');
    }
  }, []);

  // Subscribe to sessions
  const subscribe = useCallback((sessionIds: string[]) => {
    send({ type: 'subscribe', session_ids: sessionIds });
  }, [send]);

  // Unsubscribe from sessions
  const unsubscribe = useCallback((sessionIds: string[]) => {
    send({ type: 'unsubscribe', session_ids: sessionIds });
  }, [send]);

  // Add message handler
  const addMessageHandler = useCallback((handler: MessageHandler): (() => void) => {
    handlersRef.current.add(handler);
    return () => {
      handlersRef.current.delete(handler);
    };
  }, []);

  // Auto-connect on mount
  useEffect(() => {
    if (autoConnect) {
      connect();
    }
    return () => {
      disconnect();
    };
  }, [autoConnect, connect, disconnect]);

  return {
    connectionState,
    isConnected: connectionState === 'connected',
    send,
    subscribe,
    unsubscribe,
    addMessageHandler,
    connect,
    disconnect,
  };
}
