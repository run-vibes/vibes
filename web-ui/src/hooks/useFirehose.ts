/**
 * Hook for connecting to the firehose WebSocket endpoint with infinite scroll support
 *
 * Features:
 * - Offset tracking for pagination
 * - Server-side filtering
 * - Auto-follow with manual scroll detection
 * - Fetch older events on demand
 */
import { useCallback, useEffect, useRef, useState } from 'react';
import { VibesEvent } from '../lib/types';

/**
 * Raw firehose event from the server with its offset and unique ID
 */
export interface FirehoseEvent {
  /** Globally unique, time-ordered event ID (UUIDv7) */
  event_id: string;
  /** Partition-scoped offset (not unique across partitions, use event_id for keying) */
  offset: number;
  event: VibesEvent;
}

/**
 * Filter state for server-side event filtering
 */
export interface FirehoseFilters {
  types: string[] | null;
  sessionId: string | null;
}

/**
 * State model for the firehose hook
 */
export interface FirehoseState {
  events: FirehoseEvent[];
  oldestOffset: number | null;
  newestOffset: number | null;
  isLoadingOlder: boolean;
  isFollowing: boolean;
  hasMore: boolean;
  filters: FirehoseFilters;
}

export interface FirehoseOptions {
  /** Auto-connect on mount (default: true) */
  autoConnect?: boolean;
}

export interface UseFirehoseReturn extends FirehoseState {
  isConnected: boolean;
  error: Error | null;
  connect: () => void;
  disconnect: () => void;
  fetchOlder: () => void;
  setFilters: (filters: Partial<FirehoseFilters>) => void;
  setIsFollowing: (value: boolean) => void;
}

/**
 * Server to client message types
 */
interface EventsBatchMessage {
  type: 'events_batch';
  events: FirehoseEvent[];
  oldest_offset: number | null;
  has_more: boolean;
}

interface LiveEventMessage {
  type: 'event';
  /** Globally unique, time-ordered event ID (UUIDv7) */
  event_id: string;
  offset: number;
  event: VibesEvent;
}

type FirehoseServerMessage = EventsBatchMessage | LiveEventMessage;

/**
 * Client to server message types
 */
interface FetchOlderMessage {
  type: 'fetch_older';
  before_offset: number;
  limit?: number;
}

interface SetFiltersMessage {
  type: 'set_filters';
  types?: string[];
  session?: string;
}

type FirehoseClientMessage = FetchOlderMessage | SetFiltersMessage;

export function useFirehose(options: FirehoseOptions = {}): UseFirehoseReturn {
  const { autoConnect = true } = options;

  // Core state
  const [events, setEvents] = useState<FirehoseEvent[]>([]);
  const [oldestOffset, setOldestOffset] = useState<number | null>(null);
  const [newestOffset, setNewestOffset] = useState<number | null>(null);
  const [isLoadingOlder, setIsLoadingOlder] = useState(false);
  const [isFollowing, setIsFollowing] = useState(true);
  const [hasMore, setHasMore] = useState(false);
  const [filters, setFiltersState] = useState<FirehoseFilters>({
    types: null,
    sessionId: null,
  });

  // Connection state
  const [isConnected, setIsConnected] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  // Refs
  const wsRef = useRef<WebSocket | null>(null);
  const isLoadingOlderRef = useRef(false);

  // Keep ref in sync
  useEffect(() => {
    isLoadingOlderRef.current = isLoadingOlder;
  }, [isLoadingOlder]);

  const sendMessage = useCallback((msg: FirehoseClientMessage) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(msg));
    }
  }, []);

  const handleMessage = useCallback((data: string) => {
    try {
      const msg = JSON.parse(data) as FirehoseServerMessage;

      if (msg.type === 'events_batch') {
        const batchEvents = msg.events;

        if (isLoadingOlderRef.current) {
          // Prepend older events
          setEvents((prev) => [...batchEvents, ...prev]);
          setIsLoadingOlder(false);
        } else {
          // Initial batch or filter reset - replace all events
          setEvents(batchEvents);
        }

        // Update offsets
        setOldestOffset(msg.oldest_offset);
        if (batchEvents.length > 0) {
          const maxOffset = Math.max(...batchEvents.map((e) => e.offset));
          setNewestOffset((prev) => (prev === null ? maxOffset : Math.max(prev, maxOffset)));
        }
        setHasMore(msg.has_more);
      } else if (msg.type === 'event') {
        // Live event - append
        setEvents((prev) => [...prev, { event_id: msg.event_id, offset: msg.offset, event: msg.event }]);
        setNewestOffset(msg.offset);
      }
    } catch (e) {
      console.error('Failed to parse firehose message:', e);
    }
  }, []);

  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) return;

    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const url = `${protocol}//${window.location.host}/ws/firehose`;

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
      handleMessage(event.data);
    };
  }, [handleMessage]);

  const disconnect = useCallback(() => {
    wsRef.current?.close();
    wsRef.current = null;
    setIsConnected(false);
  }, []);

  const fetchOlder = useCallback(() => {
    // Guard: don't fetch if already loading, no more history, or no oldest offset
    if (isLoadingOlderRef.current || !hasMore || oldestOffset === null) {
      return;
    }

    // Set ref immediately to prevent duplicate calls
    isLoadingOlderRef.current = true;
    setIsLoadingOlder(true);
    sendMessage({
      type: 'fetch_older',
      before_offset: oldestOffset,
    });
  }, [hasMore, oldestOffset, sendMessage]);

  const setFilters = useCallback(
    (newFilters: Partial<FirehoseFilters>) => {
      // Use functional update to avoid dependency on filters state
      // This prevents infinite loops when setFilters is used in useEffect
      setFiltersState((prev) => {
        const updated: FirehoseFilters = {
          types: newFilters.types ?? prev.types,
          sessionId: newFilters.sessionId ?? prev.sessionId,
        };

        // Send to server (inside functional update to access computed value)
        sendMessage({
          type: 'set_filters',
          types: updated.types ?? undefined,
          session: updated.sessionId ?? undefined,
        });

        return updated;
      });

      // Clear current events - server will send fresh batch
      setEvents([]);
      setOldestOffset(null);
      setNewestOffset(null);
      setHasMore(false);

      // Reset following on filter change
      setIsFollowing(true);
    },
    [sendMessage]
  );

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
    oldestOffset,
    newestOffset,
    isLoadingOlder,
    isFollowing,
    hasMore,
    filters,
    isConnected,
    error,
    connect,
    disconnect,
    fetchOlder,
    setFilters,
    setIsFollowing,
  };
}
