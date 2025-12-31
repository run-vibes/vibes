/**
 * Hook for connecting to the firehose WebSocket endpoint
 */
import { useCallback, useEffect, useRef, useState } from 'react';
import { VibesEvent } from '../lib/types';

export interface FirehoseOptions {
  /** Event types to filter (e.g., ['Claude', 'Hook']) */
  types?: string[];
  /** Session ID to filter */
  session?: string;
  /** Maximum events to keep in buffer */
  bufferSize?: number;
  /** Auto-connect on mount */
  autoConnect?: boolean;
}

export interface UseFirehoseReturn {
  events: VibesEvent[];
  isConnected: boolean;
  isPaused: boolean;
  error: Error | null;
  connect: () => void;
  disconnect: () => void;
  pause: () => void;
  resume: () => void;
  clear: () => void;
}

export function useFirehose(options: FirehoseOptions = {}): UseFirehoseReturn {
  const { types, session, bufferSize = 1000, autoConnect = true } = options;

  const [events, setEvents] = useState<VibesEvent[]>([]);
  const [isConnected, setIsConnected] = useState(false);
  const [isPaused, setIsPaused] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const wsRef = useRef<WebSocket | null>(null);
  const bufferRef = useRef<VibesEvent[]>([]);
  const isPausedRef = useRef(isPaused);

  // Keep ref in sync with state
  useEffect(() => {
    isPausedRef.current = isPaused;
  }, [isPaused]);

  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) return;

    const params = new URLSearchParams();
    if (types?.length) params.set('types', types.join(','));
    if (session) params.set('session', session);

    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const queryString = params.toString();
    const url = `${protocol}//${window.location.host}/ws/firehose${queryString ? `?${queryString}` : ''}`;

    const ws = new WebSocket(url);
    wsRef.current = ws;

    ws.onopen = () => {
      setIsConnected(true);
      setError(null);
    };

    ws.onclose = () => {
      setIsConnected(false);
    };

    ws.onerror = () => {
      setError(new Error('WebSocket connection failed'));
    };

    ws.onmessage = (event) => {
      try {
        const vibesEvent = JSON.parse(event.data) as VibesEvent;

        // Always buffer (for resume)
        bufferRef.current = [...bufferRef.current.slice(-(bufferSize - 1)), vibesEvent];

        // Only update state if not paused
        if (!isPausedRef.current) {
          setEvents((prev) => [...prev.slice(-(bufferSize - 1)), vibesEvent]);
        }
      } catch (e) {
        console.error('Failed to parse firehose event:', e);
      }
    };
  }, [types, session, bufferSize]);

  const disconnect = useCallback(() => {
    wsRef.current?.close();
    wsRef.current = null;
    setIsConnected(false);
  }, []);

  const pause = useCallback(() => {
    setIsPaused(true);
  }, []);

  const resume = useCallback(() => {
    // Sync buffer to events on resume
    setEvents(bufferRef.current);
    setIsPaused(false);
  }, []);

  const clear = useCallback(() => {
    setEvents([]);
    bufferRef.current = [];
  }, []);

  // Auto-connect
  useEffect(() => {
    if (autoConnect) {
      connect();
    }
    return () => {
      wsRef.current?.close();
    };
  }, [autoConnect, connect]);

  return {
    events,
    isConnected,
    isPaused,
    error,
    connect,
    disconnect,
    pause,
    resume,
    clear,
  };
}
