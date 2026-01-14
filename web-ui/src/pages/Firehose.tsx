// web-ui/src/pages/Firehose.tsx
import { useState, useMemo, useCallback, useEffect, useRef } from 'react';
import { StreamView, EventInspector, Badge, Card } from '@vibes/design-system';
import type { DisplayEvent, ContextEvent } from '@vibes/design-system';
import { useFirehose } from '../hooks/useFirehose';
import type { VibesEvent } from '../lib/types';
import { extractTimestampFromUuidv7 } from '../lib/uuidv7';
import './Firehose.css';

const EVENT_TYPES = ['SESSION', 'HOOK', 'ERROR'] as const;

interface SessionInfo {
  id: string;
  name?: string;
  status: 'active' | 'stale' | 'error';
  eventCount: number;
  lastActivity: Date;
}

function toDisplayEvent(event: VibesEvent, eventId: string): DisplayEvent {
  const baseEvent = {
    id: eventId,
    // Extract timestamp from UUIDv7 event_id for accurate, consistent ordering
    timestamp: extractTimestampFromUuidv7(eventId),
  };

  switch (event.type) {
    case 'session_created':
      return { ...baseEvent, type: 'SESSION', session: event.session_id, summary: `Created "${event.name || 'unnamed'}"` };
    case 'session_state_changed':
      return { ...baseEvent, type: 'SESSION', session: event.session_id, summary: `State: ${event.state}` };
    case 'claude':
      return { ...baseEvent, type: 'CLAUDE', session: event.session_id, summary: summarizeClaudeEvent(event.event) };
    case 'user_input':
      return { ...baseEvent, type: 'CLAUDE', session: event.session_id, summary: `Input: ${event.content.slice(0, 50)}...` };
    case 'permission_response':
      return { ...baseEvent, type: 'CLAUDE', session: event.session_id, summary: `Permission: ${event.approved ? 'approved' : 'denied'}` };
    case 'hook':
      return { ...baseEvent, type: 'HOOK', session: event.session_id, summary: summarizeHookEvent(event.event) };
    case 'client_connected':
      return { ...baseEvent, type: 'SESSION', summary: `Client connected: ${event.client_id}` };
    case 'client_disconnected':
      return { ...baseEvent, type: 'SESSION', summary: `Client disconnected: ${event.client_id}` };
    case 'tunnel_state_changed':
      return { ...baseEvent, type: 'SESSION', summary: `Tunnel: ${event.state}${event.url ? ` (${event.url})` : ''}` };
    case 'ownership_transferred':
      return { ...baseEvent, type: 'SESSION', session: event.session_id, summary: `Ownership → ${event.new_owner_id}` };
    case 'session_removed':
      return { ...baseEvent, type: 'SESSION', session: event.session_id, summary: `Removed: ${event.reason}` };
    default:
      return { ...baseEvent, type: 'HOOK', summary: 'Unknown event' };
  }
}

function summarizeClaudeEvent(event: unknown): string {
  if (!event || typeof event !== 'object') return 'Unknown Claude event';
  const e = event as Record<string, unknown>;
  if (e.type === 'text_delta') return `TextDelta: "${String(e.delta || '').slice(0, 40)}..."`;
  if (e.type === 'tool_use') return `Tool: ${e.name || 'unknown'}`;
  if (e.type === 'tool_result') return `ToolResult: ${e.success ? 'success' : 'failed'}`;
  return `Claude: ${String(e.type || 'event')}`;
}

function summarizeHookEvent(event: unknown): string {
  if (!event || typeof event !== 'object') return 'Unknown hook event';
  const e = event as Record<string, unknown>;
  if (e.type === 'pre_tool_use') return `PreToolUse: ${e.tool_name || 'unknown'}`;
  if (e.type === 'post_tool_use') return `PostToolUse: ${e.tool_name || 'unknown'}`;
  if (e.type === 'notification') return `Notification: ${e.title || 'untitled'}`;
  if (e.type === 'stop') return `Stop: ${e.reason || 'unknown'}`;
  return `Hook: ${String(e.type || 'event')}`;
}

// Generate sparkline characters for event rate visualization
function generateSparkline(rates: number[]): { chars: string[]; peaks: boolean[] } {
  const bars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
  const max = Math.max(...rates, 1);
  return {
    chars: rates.map(r => bars[Math.min(Math.floor((r / max) * 7), 7)]),
    peaks: rates.map(r => r === max && r > 0),
  };
}

export function FirehosePage() {
  const [selectedTypes, setSelectedTypes] = useState<string[]>([]);
  const [selectedSessions, setSelectedSessions] = useState<string[]>([]);
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [selectedEventId, setSelectedEventId] = useState<string | null>(null);

  const {
    events: rawEvents,
    isConnected,
    isFollowing,
    isLoadingOlder,
    hasMore,
    error,
    fetchOlder,
    setFilters,
    setIsFollowing,
  } = useFirehose();

  // Skip the first useEffect run - the initial request is sent by
  // useFirehose in onopen. Only send filter updates when the user
  // actually changes filters (not on initial mount).
  const isInitialMount = useRef(true);

  useEffect(() => {
    if (isInitialMount.current) {
      isInitialMount.current = false;
      return;
    }
    // Only send type filters to server; search is client-side
    setFilters({
      types: selectedTypes.length > 0 ? selectedTypes : null,
      sessionId: selectedSessions.length === 1 ? selectedSessions[0] : null,
    });
  }, [selectedTypes, selectedSessions, setFilters]);

  // Extract unique sessions from events
  const sessions = useMemo((): SessionInfo[] => {
    const sessionMap = new Map<string, SessionInfo>();

    for (const raw of rawEvents) {
      const sessionId = 'session_id' in raw.event ? (raw.event as { session_id?: string }).session_id : undefined;
      if (!sessionId) continue;

      const existing = sessionMap.get(sessionId);
      const timestamp = extractTimestampFromUuidv7(raw.event_id);

      if (!existing) {
        const name = raw.event.type === 'session_created' && 'name' in raw.event
          ? (raw.event as { name?: string }).name
          : undefined;
        sessionMap.set(sessionId, {
          id: sessionId,
          name,
          status: 'active',
          eventCount: 1,
          lastActivity: timestamp,
        });
      } else {
        existing.eventCount++;
        if (timestamp > existing.lastActivity) {
          existing.lastActivity = timestamp;
        }
        // Update name if this is the session_created event
        if (raw.event.type === 'session_created' && 'name' in raw.event) {
          existing.name = (raw.event as { name?: string }).name;
        }
      }
    }

    // Determine session status based on last activity
    const now = Date.now();
    for (const session of sessionMap.values()) {
      const ageMs = now - session.lastActivity.getTime();
      if (ageMs > 5 * 60 * 1000) { // 5 minutes
        session.status = 'stale';
      }
    }

    return Array.from(sessionMap.values()).sort((a, b) =>
      b.lastActivity.getTime() - a.lastActivity.getTime()
    );
  }, [rawEvents]);

  // Calculate metrics
  const metrics = useMemo(() => {
    const typeCounts: Record<string, number> = {};
    const recentRates: number[] = [];
    const now = Date.now();

    // Count events per type
    for (const raw of rawEvents) {
      const display = toDisplayEvent(raw.event, raw.event_id);
      typeCounts[display.type] = (typeCounts[display.type] || 0) + 1;
    }

    // Calculate events per minute for last 10 minutes (for sparkline)
    for (let i = 9; i >= 0; i--) {
      const windowStart = now - (i + 1) * 60 * 1000;
      const windowEnd = now - i * 60 * 1000;
      const count = rawEvents.filter(e => {
        const ts = extractTimestampFromUuidv7(e.event_id).getTime();
        return ts >= windowStart && ts < windowEnd;
      }).length;
      recentRates.push(count);
    }

    // Events per hour (last hour)
    const oneHourAgo = now - 60 * 60 * 1000;
    const eventsPerHour = rawEvents.filter(e =>
      extractTimestampFromUuidv7(e.event_id).getTime() >= oneHourAgo
    ).length;

    return {
      typeCounts,
      eventsPerHour,
      activeSessions: sessions.filter(s => s.status === 'active').length,
      totalErrors: typeCounts['ERROR'] || 0,
      sparklineRates: recentRates,
    };
  }, [rawEvents, sessions]);

  // Convert raw events to display format, then apply client-side search filter
  const displayEvents = useMemo(() => {
    let events = rawEvents.map((e) => toDisplayEvent(e.event, e.event_id));

    // Apply session filter (client-side for multi-select)
    if (selectedSessions.length > 1) {
      events = events.filter(e => e.session && selectedSessions.includes(e.session));
    }

    // Apply client-side search filter
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      events = events.filter((e) => e.summary.toLowerCase().includes(query));
    }

    return events;
  }, [rawEvents, selectedSessions, searchQuery]);

  const selectedEvent = useMemo((): DisplayEvent | null => {
    if (!selectedEventId) return null;
    // Find event by event_id (UUIDv7)
    const rawIndex = rawEvents.findIndex((e) => e.event_id === selectedEventId);
    if (rawIndex === -1) return null;

    const raw = rawEvents[rawIndex];
    const display = displayEvents[rawIndex];
    if (!raw || !display) return null;

    return {
      id: display.id,
      timestamp: display.timestamp,
      type: display.type,
      summary: display.summary,
      session: display.session,
      payload: raw.event,
    };
  }, [selectedEventId, rawEvents, displayEvents]);

  const contextEvents = useMemo((): ContextEvent[] => {
    if (!selectedEventId) return [];
    // Find index by event_id (UUIDv7)
    const eventIndex = rawEvents.findIndex((e) => e.event_id === selectedEventId);
    if (eventIndex === -1) return [];

    const context: ContextEvent[] = [];
    for (let i = Math.max(0, eventIndex - 2); i <= Math.min(displayEvents.length - 1, eventIndex + 2); i++) {
      const e = displayEvents[i];
      context.push({
        relativePosition: i - eventIndex,
        timestamp: e.timestamp,
        type: e.type,
        summary: e.summary,
      });
    }

    return context;
  }, [selectedEventId, rawEvents, displayEvents]);

  const toggleType = useCallback((type: string) => {
    setSelectedTypes((prev) =>
      prev.includes(type) ? prev.filter((t) => t !== type) : [...prev, type]
    );
  }, []);

  const toggleSession = useCallback((sessionId: string) => {
    setSelectedSessions((prev) =>
      prev.includes(sessionId) ? prev.filter((s) => s !== sessionId) : [...prev, sessionId]
    );
  }, []);

  const handleCopyJson = useCallback(() => {
    if (selectedEvent?.payload) {
      navigator.clipboard.writeText(JSON.stringify(selectedEvent.payload, null, 2));
    }
  }, [selectedEvent]);

  const handleJumpToLatest = useCallback(() => {
    setIsFollowing(true);
  }, [setIsFollowing]);

  const sparkline = generateSparkline(metrics.sparklineRates);

  return (
    <div className="firehose-page">
      {/* Header */}
      <div className="firehose-header">
        <div className="firehose-header-left">
          <h1 className="firehose-title">FIREHOSE</h1>
          <div className="firehose-status">
            {isConnected ? (
              <Badge status="success">Connected</Badge>
            ) : (
              <Badge status="error">Disconnected</Badge>
            )}
            {!isFollowing && <Badge status="warning">Paused</Badge>}
            {isLoadingOlder && <Badge status="idle">Loading...</Badge>}
            {error && <Badge status="error">{error.message}</Badge>}
          </div>
        </div>

        <div className="firehose-header-right">
          <input
            type="text"
            placeholder="Search events..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="search-input"
          />
        </div>
      </div>

      {/* 4-Column Layout */}
      <div className="firehose-content">
        {/* Column 1: Session Tabs */}
        <div className="session-tabs">
          <div className="session-tabs-header">
            <span>SESSIONS</span>
            <span className="session-count">{sessions.length}</span>
          </div>
          <div className="session-list">
            {sessions.map((session) => (
              <div
                key={session.id}
                className={`session-item ${selectedSessions.includes(session.id) ? 'selected' : ''}`}
                onClick={() => toggleSession(session.id)}
              >
                <div className="session-id">
                  <span className="session-checkbox">
                    {selectedSessions.includes(session.id) && '✓'}
                  </span>
                  {session.id.slice(0, 8)}
                </div>
                {session.name && <div className="session-name">{session.name}</div>}
                <div className="session-meta">
                  <span className={`session-status-dot ${session.status}`} />
                  <span>{session.eventCount} events</span>
                </div>
              </div>
            ))}
            {sessions.length === 0 && (
              <div className="session-item">
                <div className="session-name">No sessions</div>
              </div>
            )}
          </div>
        </div>

        {/* Column 2: Sidebar (Metrics + Filters) */}
        <div className="firehose-sidebar">
          {/* Metrics Panel */}
          <Card variant="crt" title="METRICS">
            <div className="stat-grid">
              <div className="stat">
                <div className="stat-value">{metrics.eventsPerHour}</div>
                <div className="stat-label">EVT/HR</div>
              </div>
              <div className="stat">
                <div className="stat-value">{metrics.activeSessions}</div>
                <div className="stat-label">ACTIVE</div>
              </div>
              <div className="stat">
                <div className={`stat-value ${metrics.totalErrors > 0 ? 'error' : ''}`}>
                  {metrics.totalErrors}
                </div>
                <div className="stat-label">ERRORS</div>
              </div>
            </div>
            <div className="sparkline">
              {sparkline.chars.map((char, i) => (
                <span key={i} className={sparkline.peaks[i] ? 'peak' : ''}>
                  {char}
                </span>
              ))}
            </div>
          </Card>

          {/* Filters Panel */}
          <Card variant="crt" title="FILTERS">
            <div className="filter-list">
              {EVENT_TYPES.map((type) => (
                <div
                  key={type}
                  className={`filter-item ${selectedTypes.includes(type) ? 'active' : ''}`}
                  onClick={() => toggleType(type)}
                >
                  <span>{type}</span>
                  <span className="filter-count">{metrics.typeCounts[type] || 0}</span>
                </div>
              ))}
            </div>
          </Card>
        </div>

        {/* Column 3: Stream */}
        <div className="firehose-stream">
          <StreamView
            events={displayEvents}
            title="Event Stream"
            isLive={isConnected}
            isPaused={!isFollowing}
            selectedId={selectedEventId ?? undefined}
            onEventClick={(e) => setSelectedEventId(e.id)}
            onLoadMore={fetchOlder}
            isLoadingMore={isLoadingOlder}
            hasMore={hasMore}
            onFollowingChange={setIsFollowing}
          />
          {!isFollowing && (
            <button className="jump-to-latest" onClick={handleJumpToLatest}>
              ↓ Jump to latest
            </button>
          )}
        </div>

        {/* Column 4: Event Details */}
        <div className="firehose-inspector">
          <EventInspector
            event={selectedEvent}
            contextEvents={contextEvents}
            onCopyJson={handleCopyJson}
            onClose={() => setSelectedEventId(null)}
          />
        </div>
      </div>
    </div>
  );
}
