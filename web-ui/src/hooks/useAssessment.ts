/**
 * Hook for connecting to the assessment WebSocket endpoint
 *
 * Features:
 * - Event ID tracking for pagination (UUIDv7 for time-ordering)
 * - Server-side session filtering
 * - Auto-follow with manual scroll detection
 * - Fetch older events on demand
 */
import { useCallback, useEffect, useRef, useState } from 'react';

/**
 * Assessment context from the server
 */
interface AssessmentContext {
  session_id: string;
  event_id: string;
  timestamp: string;
  active_learnings: string[];
  injection_method: string;
  injection_scope: string | null;
  harness_type: string;
  harness_version: string | null;
  project_id: string | null;
  user_id: string | null;
}

/**
 * Assessment event from the server.
 * Uses `tier` as the discriminator (lightweight, medium, heavy).
 */
export interface AssessmentEvent {
  /** Globally unique, time-ordered event ID (UUIDv7) */
  event_id: string;
  /** Event tier: lightweight, medium, or heavy */
  tier: 'lightweight' | 'medium' | 'heavy';
  /** Assessment context with session info */
  context: AssessmentContext;
  /** Event payload (varies by tier) */
  [key: string]: unknown;
}

/**
 * Filter state for server-side event filtering
 */
export interface AssessmentFilters {
  sessionId: string | null;
}

/**
 * State model for the assessment hook
 */
export interface AssessmentState {
  events: AssessmentEvent[];
  /** Oldest event ID in current view (for pagination) */
  oldestEventId: string | null;
  isLoadingOlder: boolean;
  isFollowing: boolean;
  hasMore: boolean;
  filters: AssessmentFilters;
}

export interface AssessmentOptions {
  /** Auto-connect on mount (default: true) */
  autoConnect?: boolean;
}

export interface UseAssessmentReturn extends AssessmentState {
  isConnected: boolean;
  error: Error | null;
  connect: () => void;
  disconnect: () => void;
  fetchOlder: () => void;
  setFilters: (filters: Partial<AssessmentFilters>) => void;
  setIsFollowing: (value: boolean) => void;
}

/**
 * Wire format from server for PluginAssessmentResult.
 * Note: context is embedded in the parsed payload, not at the top level.
 */
interface WireAssessmentResult {
  event_id: string;
  result_type: 'lightweight' | 'checkpoint' | 'session_end';
  session_id: string;
  payload: string; // JSON-serialized LightweightEvent, MediumEvent, etc.
}

/**
 * Server to client message types
 */
interface AssessmentEventsBatchMessage {
  type: 'assessment_events_batch';
  events: WireAssessmentResult[];
  oldest_event_id: string | null;
  has_more: boolean;
}

interface LiveAssessmentEventMessage {
  type: 'assessment_event';
  event_id: string;
  result_type: 'lightweight' | 'checkpoint' | 'session_end';
  session_id: string;
  payload: string;
}

type AssessmentServerMessage = AssessmentEventsBatchMessage | LiveAssessmentEventMessage;

/**
 * Map server result_type to frontend tier names.
 */
function mapResultTypeToTier(
  resultType: string
): 'lightweight' | 'medium' | 'heavy' {
  switch (resultType) {
    case 'lightweight':
      return 'lightweight';
    case 'checkpoint':
      return 'medium';
    case 'session_end':
      return 'heavy';
    default:
      return 'lightweight';
  }
}

/**
 * Transform wire format to internal AssessmentEvent.
 */
function wireToAssessmentEvent(wire: WireAssessmentResult): AssessmentEvent {
  // Parse the payload to extract context and other fields
  let parsedPayload: Record<string, unknown> = {};
  try {
    parsedPayload = JSON.parse(wire.payload);
  } catch {
    // If payload parsing fails, use empty object
  }

  // Extract context from parsed payload, or create minimal context from wire data
  const context: AssessmentContext =
    (parsedPayload.context as AssessmentContext) ?? {
      session_id: wire.session_id,
      event_id: wire.event_id,
      timestamp: new Date().toISOString(),
      active_learnings: [],
      injection_method: 'unknown',
      injection_scope: null,
      harness_type: 'unknown',
      harness_version: null,
      project_id: null,
      user_id: null,
    };

  return {
    event_id: wire.event_id,
    tier: mapResultTypeToTier(wire.result_type),
    context,
    // Include parsed payload fields for inspection
    ...parsedPayload,
  };
}

/**
 * Client to server message types
 */
interface FetchOlderMessage {
  type: 'fetch_older';
  before_event_id: string;
  limit?: number;
}

interface SetFiltersMessage {
  type: 'set_filters';
  session?: string;
}

type AssessmentClientMessage = FetchOlderMessage | SetFiltersMessage;

export function useAssessment(options: AssessmentOptions = {}): UseAssessmentReturn {
  const { autoConnect = true } = options;

  // Core state
  const [events, setEvents] = useState<AssessmentEvent[]>([]);
  const [oldestEventId, setOldestEventId] = useState<string | null>(null);
  const [isLoadingOlder, setIsLoadingOlder] = useState(false);
  const [isFollowing, setIsFollowing] = useState(true);
  const [hasMore, setHasMore] = useState(false);
  const [filters, setFiltersState] = useState<AssessmentFilters>({
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

  const sendMessage = useCallback((msg: AssessmentClientMessage) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(msg));
    }
  }, []);

  const handleMessage = useCallback((data: string) => {
    try {
      const msg = JSON.parse(data) as AssessmentServerMessage;

      if (msg.type === 'assessment_events_batch') {
        // Transform wire events to internal format
        const batchEvents = msg.events.map(wireToAssessmentEvent);

        if (isLoadingOlderRef.current) {
          // Prepend older events
          setEvents((prev) => [...batchEvents, ...prev]);
          setIsLoadingOlder(false);
        } else {
          // Initial batch or filter reset - replace all events
          setEvents(batchEvents);
        }

        // Update cursors
        setOldestEventId(msg.oldest_event_id);
        setHasMore(msg.has_more);
      } else if (msg.type === 'assessment_event') {
        // Live event - transform to internal format
        const wireEvent: WireAssessmentResult = {
          event_id: msg.event_id,
          result_type: msg.result_type,
          session_id: msg.session_id,
          payload: msg.payload,
        };
        const event = wireToAssessmentEvent(wireEvent);
        setEvents((prev) => [...prev, event]);
      }
    } catch (e) {
      console.error('Failed to parse assessment message:', e);
    }
  }, []);

  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) return;

    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const url = `${protocol}//${window.location.host}/ws/assessment`;

    const ws = new WebSocket(url);
    wsRef.current = ws;

    ws.onopen = () => {
      setIsConnected(true);
      setError(null);

      // Request initial events from server
      ws.send(JSON.stringify({ type: 'set_filters' }));
    };

    ws.onclose = () => {
      setIsConnected(false);
    };

    ws.onerror = () => {
      setError(new Error('Assessment WebSocket connection failed'));
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
    // Guard: don't fetch if already loading, no more history, or no oldest event ID
    if (isLoadingOlderRef.current || !hasMore || oldestEventId === null) {
      return;
    }

    // Set ref immediately to prevent duplicate calls
    isLoadingOlderRef.current = true;
    setIsLoadingOlder(true);
    sendMessage({
      type: 'fetch_older',
      before_event_id: oldestEventId,
    });
  }, [hasMore, oldestEventId, sendMessage]);

  const setFilters = useCallback(
    (newFilters: Partial<AssessmentFilters>) => {
      setFiltersState((prev) => {
        const updated: AssessmentFilters = {
          sessionId: newFilters.sessionId !== undefined ? newFilters.sessionId : prev.sessionId,
        };

        // Send to server
        sendMessage({
          type: 'set_filters',
          session: updated.sessionId ?? undefined,
        });

        return updated;
      });

      // Clear current events - server will send fresh batch
      setEvents([]);
      setOldestEventId(null);
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
    oldestEventId,
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
