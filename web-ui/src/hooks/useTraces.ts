import { useCallback, useEffect, useRef, useState } from 'react';
import type {
  ClientMessage,
  ServerMessage,
  SpanNode,
  TraceEvent,
  TraceTree,
} from '../lib/types';
import { isTraceEventMessage, isTraceSubscribedMessage, isTraceUnsubscribedMessage } from '../lib/types';

/** Maximum number of traces to keep in buffer */
const MAX_TRACES = 100;

interface UseTracesOptions {
  send: (message: ClientMessage) => void;
  addMessageHandler: (handler: (message: ServerMessage) => void) => () => void;
  isConnected: boolean;
}

export interface UseTracesFilters {
  sessionId?: string;
  agentId?: string;
  level?: string;
}

interface UseTracesReturn {
  traces: TraceTree[];
  isSubscribed: boolean;
  isFollowing: boolean;
  subscribe: (filters?: UseTracesFilters) => void;
  unsubscribe: () => void;
  setFollowing: (following: boolean) => void;
  clear: () => void;
}

/**
 * Build a trace tree from a flat list of spans
 */
function buildTraceTree(spans: TraceEvent[]): TraceTree | null {
  if (spans.length === 0) return null;

  // Create a map for quick lookup
  const spanMap = new Map<string, SpanNode>();
  for (const span of spans) {
    spanMap.set(span.span_id, { event: span, children: [] });
  }

  // Find root and build tree
  let rootNode: SpanNode | null = null;
  for (const span of spans) {
    const node = spanMap.get(span.span_id)!;
    if (span.parent_span_id) {
      const parent = spanMap.get(span.parent_span_id);
      if (parent) {
        parent.children.push(node);
      } else {
        // Orphaned span - treat as root if no root exists
        if (!rootNode) rootNode = node;
      }
    } else {
      // Root span
      rootNode = node;
    }
  }

  if (!rootNode) {
    // No clear root - use first span
    rootNode = spanMap.get(spans[0].span_id)!;
  }

  // Sort children by timestamp
  const sortChildren = (node: SpanNode) => {
    node.children.sort((a, b) =>
      new Date(a.event.timestamp).getTime() - new Date(b.event.timestamp).getTime()
    );
    node.children.forEach(sortChildren);
  };
  sortChildren(rootNode);

  // Calculate total duration and check for errors
  const hasErrors = spans.some(s => s.status === 'error');
  const totalDuration = rootNode.event.duration_ms;

  return {
    trace_id: rootNode.event.trace_id,
    root_span: rootNode,
    session_id: rootNode.event.session_id,
    agent_id: rootNode.event.agent_id,
    timestamp: rootNode.event.timestamp,
    total_duration_ms: totalDuration,
    has_errors: hasErrors,
  };
}

export function useTraces(options: UseTracesOptions): UseTracesReturn {
  const { send, addMessageHandler, isConnected } = options;

  const [isSubscribed, setIsSubscribed] = useState(false);
  const [isFollowing, setIsFollowing] = useState(true);

  // Store raw spans grouped by trace_id
  const spansRef = useRef<Map<string, TraceEvent[]>>(new Map());
  // Store built traces for rendering
  const [traces, setTraces] = useState<TraceTree[]>([]);
  // Track trace order (newest first)
  const traceOrderRef = useRef<string[]>([]);

  const rebuildTraces = useCallback(() => {
    const newTraces: TraceTree[] = [];
    for (const traceId of traceOrderRef.current) {
      const spans = spansRef.current.get(traceId);
      if (spans) {
        const tree = buildTraceTree(spans);
        if (tree) newTraces.push(tree);
      }
    }
    setTraces(newTraces);
  }, []);

  const addSpan = useCallback((event: TraceEvent) => {
    const { trace_id } = event;

    // Add span to the trace
    if (!spansRef.current.has(trace_id)) {
      spansRef.current.set(trace_id, []);
      // Add to front of order (newest first)
      traceOrderRef.current.unshift(trace_id);

      // Evict oldest if over limit
      if (traceOrderRef.current.length > MAX_TRACES) {
        const oldestId = traceOrderRef.current.pop()!;
        spansRef.current.delete(oldestId);
      }
    }
    spansRef.current.get(trace_id)!.push(event);

    rebuildTraces();
  }, [rebuildTraces]);

  const subscribe = useCallback((filters?: UseTracesFilters) => {
    if (!isConnected) return;

    send({
      type: 'subscribe_traces',
      session_id: filters?.sessionId,
      agent_id: filters?.agentId,
      level: filters?.level,
    });
  }, [isConnected, send]);

  const unsubscribe = useCallback(() => {
    if (!isConnected) return;
    send({ type: 'unsubscribe_traces' });
  }, [isConnected, send]);

  const clear = useCallback(() => {
    spansRef.current.clear();
    traceOrderRef.current = [];
    setTraces([]);
  }, []);

  // Handle incoming messages
  useEffect(() => {
    const cleanup = addMessageHandler((message: ServerMessage) => {
      if (isTraceEventMessage(message)) {
        const event: TraceEvent = {
          trace_id: message.trace_id,
          span_id: message.span_id,
          parent_span_id: message.parent_span_id,
          name: message.name,
          level: message.level,
          timestamp: message.timestamp,
          duration_ms: message.duration_ms,
          session_id: message.session_id,
          agent_id: message.agent_id,
          attributes: message.attributes,
          status: message.status,
        };
        addSpan(event);
      } else if (isTraceSubscribedMessage(message)) {
        setIsSubscribed(true);
      } else if (isTraceUnsubscribedMessage(message)) {
        setIsSubscribed(false);
      }
    });

    return cleanup;
  }, [addMessageHandler, addSpan]);

  // Unsubscribe on disconnect
  useEffect(() => {
    if (!isConnected) {
      setIsSubscribed(false);
    }
  }, [isConnected]);

  return {
    traces,
    isSubscribed,
    isFollowing,
    subscribe,
    unsubscribe,
    setFollowing: setIsFollowing,
    clear,
  };
}
