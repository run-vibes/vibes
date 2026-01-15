import { useCallback, useEffect, useRef, useState } from 'react';
import { Badge, Button } from '@vibes/design-system';
import { useWebSocket } from '../hooks';
import { useTraces, type UseTracesFilters } from '../hooks/useTraces';
import type { SpanNode, TraceTree } from '../lib/types';
import './Traces.css';

const LEVELS = ['trace', 'debug', 'info', 'warn', 'error'];

function formatDuration(ms: number | undefined): string {
  if (ms === undefined) return '—';
  if (ms < 1) return `${(ms * 1000).toFixed(0)}µs`;
  if (ms < 1000) return `${ms.toFixed(1)}ms`;
  return `${(ms / 1000).toFixed(2)}s`;
}

function formatTimestamp(iso: string): string {
  const date = new Date(iso);
  return date.toLocaleTimeString('en-US', { hour12: false });
}

function truncateId(id: string, length = 8): string {
  return id.slice(0, length) + '...';
}

interface SpanRowProps {
  node: SpanNode;
  depth: number;
  isLast: boolean;
  expanded: Set<string>;
  toggleExpand: (spanId: string) => void;
}

function SpanRow({ node, depth, isLast, expanded, toggleExpand }: SpanRowProps) {
  const { event, children } = node;
  const hasChildren = children.length > 0;
  const isExpanded = expanded.has(event.span_id);
  const isError = event.status === 'error';

  // Build tree line prefix
  const prefix = depth === 0 ? '' : (isLast ? '└─ ' : '├─ ');

  return (
    <>
      <div
        className={`span-row ${isError ? 'span-error' : ''}`}
        style={{ paddingLeft: `${depth * 20 + 8}px` }}
        onClick={() => toggleExpand(event.span_id)}
      >
        <span className="span-prefix">{prefix}</span>
        <span className="span-name">{event.name}</span>
        <span className="span-duration">({formatDuration(event.duration_ms)})</span>
        {Object.entries(event.attributes).slice(0, 2).map(([key, value]) => (
          <span key={key} className="span-attr">{key}={value}</span>
        ))}
        <span className={`span-status ${isError ? 'status-error' : 'status-ok'}`}>
          {isError ? '✗' : '✓'}
        </span>
      </div>

      {/* Expanded details */}
      {isExpanded && (
        <div className="span-details" style={{ paddingLeft: `${depth * 20 + 28}px` }}>
          <div className="span-detail-row">
            <span className="span-detail-label">span_id:</span>
            <span className="span-detail-value">{event.span_id}</span>
          </div>
          <div className="span-detail-row">
            <span className="span-detail-label">level:</span>
            <span className="span-detail-value">{event.level}</span>
          </div>
          {event.session_id && (
            <div className="span-detail-row">
              <span className="span-detail-label">session:</span>
              <span className="span-detail-value">{event.session_id}</span>
            </div>
          )}
          {event.agent_id && (
            <div className="span-detail-row">
              <span className="span-detail-label">agent:</span>
              <span className="span-detail-value">{event.agent_id}</span>
            </div>
          )}
          {Object.entries(event.attributes).map(([key, value]) => (
            <div key={key} className="span-detail-row">
              <span className="span-detail-label">{key}:</span>
              <span className="span-detail-value">{value}</span>
            </div>
          ))}
        </div>
      )}

      {/* Render children */}
      {hasChildren && children.map((child, i) => (
        <SpanRow
          key={child.event.span_id}
          node={child}
          depth={depth + 1}
          isLast={i === children.length - 1}
          expanded={expanded}
          toggleExpand={toggleExpand}
        />
      ))}
    </>
  );
}

interface TraceRowProps {
  trace: TraceTree;
  isOpen: boolean;
  onToggle: () => void;
  expanded: Set<string>;
  toggleExpand: (spanId: string) => void;
}

function TraceRow({ trace, isOpen, onToggle, expanded, toggleExpand }: TraceRowProps) {
  return (
    <div className={`trace-row ${trace.has_errors ? 'trace-has-errors' : ''}`}>
      <div className="trace-header" onClick={onToggle}>
        <span className="trace-toggle">{isOpen ? '▼' : '▶'}</span>
        <span className="trace-id">Trace {truncateId(trace.trace_id)}</span>
        <span className="trace-time">{formatTimestamp(trace.timestamp)}</span>
        <span className="trace-duration">{formatDuration(trace.total_duration_ms)}</span>
        {trace.session_id && (
          <span className="trace-session">{truncateId(trace.session_id)}</span>
        )}
        {trace.has_errors && (
          <span className="trace-error-badge">✗</span>
        )}
      </div>

      {isOpen && (
        <div className="trace-spans">
          <SpanRow
            node={trace.root_span}
            depth={0}
            isLast={true}
            expanded={expanded}
            toggleExpand={toggleExpand}
          />
        </div>
      )}
    </div>
  );
}

export function Traces() {
  const { send, addMessageHandler, isConnected, connectionState } = useWebSocket();
  const {
    traces,
    isSubscribed,
    isFollowing,
    subscribe,
    unsubscribe,
    setFollowing,
    clear,
  } = useTraces({ send, addMessageHandler, isConnected });

  // Filter state
  const [sessionFilter, setSessionFilter] = useState('');
  const [agentFilter, setAgentFilter] = useState('');
  const [levelFilter, setLevelFilter] = useState('info');

  // UI state
  const [openTraces, setOpenTraces] = useState<Set<string>>(new Set());
  const [expandedSpans, setExpandedSpans] = useState<Set<string>>(new Set());
  const listRef = useRef<HTMLDivElement>(null);

  // Subscribe with current filters
  const applyFilters = useCallback(() => {
    const filters: UseTracesFilters = {
      level: levelFilter,
    };
    if (sessionFilter) filters.sessionId = sessionFilter;
    if (agentFilter) filters.agentId = agentFilter;

    if (isSubscribed) {
      unsubscribe();
    }
    clear();
    subscribe(filters);
  }, [sessionFilter, agentFilter, levelFilter, isSubscribed, subscribe, unsubscribe, clear]);

  // Auto-subscribe on connect
  useEffect(() => {
    if (isConnected && !isSubscribed) {
      applyFilters();
    }
  }, [isConnected, isSubscribed, applyFilters]);

  // Auto-scroll when following
  useEffect(() => {
    if (isFollowing && listRef.current) {
      listRef.current.scrollTop = 0; // Newest at top
    }
  }, [traces, isFollowing]);

  // Handle scroll to pause following
  const handleScroll = useCallback(() => {
    if (!listRef.current) return;
    const { scrollTop } = listRef.current;
    // If user scrolls down (away from top), pause following
    if (scrollTop > 10 && isFollowing) {
      setFollowing(false);
    }
  }, [isFollowing, setFollowing]);

  const toggleTrace = useCallback((traceId: string) => {
    setOpenTraces(prev => {
      const next = new Set(prev);
      if (next.has(traceId)) {
        next.delete(traceId);
      } else {
        next.add(traceId);
      }
      return next;
    });
  }, []);

  const toggleSpan = useCallback((spanId: string) => {
    setExpandedSpans(prev => {
      const next = new Set(prev);
      if (next.has(spanId)) {
        next.delete(spanId);
      } else {
        next.add(spanId);
      }
      return next;
    });
  }, []);

  const handleClear = useCallback(() => {
    clear();
    setOpenTraces(new Set());
    setExpandedSpans(new Set());
  }, [clear]);

  return (
    <div className="traces-page">
      {/* Header */}
      <div className="traces-header">
        <div className="traces-header-left">
          <h1 className="traces-title">TRACES</h1>
          <div className="traces-status">
            {isConnected ? (
              <Badge status="success">Connected</Badge>
            ) : (
              <Badge status="error">{connectionState}</Badge>
            )}
            {isSubscribed && (
              <Badge status="accent">Streaming</Badge>
            )}
          </div>
        </div>

        <div className="traces-header-right">
          <span className="traces-count">
            {traces.length} trace{traces.length !== 1 ? 's' : ''}
          </span>
          <Button
            variant="secondary"
            size="sm"
            onClick={handleClear}
          >
            Clear
          </Button>
        </div>
      </div>

      {/* Filters */}
      <div className="traces-filters">
        <div className="filter-group">
          <label className="filter-label">Session</label>
          <input
            type="text"
            className="filter-input"
            placeholder="Filter by session..."
            value={sessionFilter}
            onChange={(e) => setSessionFilter(e.target.value)}
          />
        </div>

        <div className="filter-group">
          <label className="filter-label">Agent</label>
          <input
            type="text"
            className="filter-input"
            placeholder="Filter by agent..."
            value={agentFilter}
            onChange={(e) => setAgentFilter(e.target.value)}
          />
        </div>

        <div className="filter-group">
          <label className="filter-label">Level</label>
          <select
            className="filter-select"
            value={levelFilter}
            onChange={(e) => setLevelFilter(e.target.value)}
          >
            {LEVELS.map(level => (
              <option key={level} value={level}>{level}</option>
            ))}
          </select>
        </div>

        <Button
          variant="primary"
          size="sm"
          onClick={applyFilters}
        >
          Apply
        </Button>
      </div>

      {/* Trace List */}
      <div
        className="traces-content"
        ref={listRef}
        onScroll={handleScroll}
      >
        {traces.length === 0 ? (
          <div className="traces-empty">
            <div className="traces-empty-text">No traces</div>
            <div className="traces-empty-hint">
              Traces will appear here as they are recorded
            </div>
          </div>
        ) : (
          <div className="traces-list">
            {traces.map(trace => (
              <TraceRow
                key={trace.trace_id}
                trace={trace}
                isOpen={openTraces.has(trace.trace_id)}
                onToggle={() => toggleTrace(trace.trace_id)}
                expanded={expandedSpans}
                toggleExpand={toggleSpan}
              />
            ))}
          </div>
        )}
      </div>

      {/* Footer */}
      <div className="traces-footer">
        {isFollowing ? (
          <span className="traces-following">↓ Following</span>
        ) : (
          <Button
            variant="secondary"
            size="sm"
            onClick={() => setFollowing(true)}
          >
            Resume ↓
          </Button>
        )}
      </div>
    </div>
  );
}
