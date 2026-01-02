// web-ui/src/pages/Firehose.tsx
import { useState, useMemo, useCallback, useEffect } from 'react';
import { StreamView, EventInspector, Badge } from '@vibes/design-system';
import type { StreamEvent, InspectorEvent, ContextEvent } from '@vibes/design-system';
import { useFirehose } from '../hooks/useFirehose';
import type { VibesEvent } from '../lib/types';
import './Firehose.css';

const EVENT_TYPES = ['SESSION', 'CLAUDE', 'TOOL', 'HOOK', 'ERROR', 'ASSESS'] as const;

function vibesEventToStreamEvent(event: VibesEvent, _index: number, offset: number): StreamEvent {
  const baseEvent = {
    id: `${offset}`,
    timestamp: new Date(),
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

export function FirehosePage() {
  const [selectedTypes, setSelectedTypes] = useState<string[]>([]);
  const [sessionFilter, setSessionFilter] = useState<string>('');
  const [selectedEventId, setSelectedEventId] = useState<string | null>(null);

  const {
    events: rawEvents,
    isConnected,
    isFollowing,
    isLoadingOlder,
    error,
    setFilters,
    setIsFollowing,
  } = useFirehose();

  // Send filter updates to server when local filters change
  useEffect(() => {
    setFilters({
      types: selectedTypes.length > 0 ? selectedTypes : null,
      sessionId: sessionFilter || null,
    });
  }, [selectedTypes, sessionFilter, setFilters]);

  const streamEvents = useMemo(
    () => rawEvents.map((e, i) => vibesEventToStreamEvent(e.event, i, e.offset)),
    [rawEvents]
  );

  const selectedEvent = useMemo((): InspectorEvent | null => {
    if (!selectedEventId) return null;
    const eventIndex = rawEvents.findIndex((e) => String(e.offset) === selectedEventId);
    if (eventIndex === -1) return null;

    const raw = rawEvents[eventIndex];
    const stream = streamEvents[eventIndex];
    if (!raw || !stream) return null;

    return {
      id: stream.id,
      timestamp: stream.timestamp,
      type: stream.type,
      session: stream.session,
      payload: raw.event,
    };
  }, [selectedEventId, rawEvents, streamEvents]);

  const contextEvents = useMemo((): ContextEvent[] => {
    if (!selectedEventId) return [];
    const eventIndex = rawEvents.findIndex((e) => String(e.offset) === selectedEventId);
    if (eventIndex === -1) return [];

    const context: ContextEvent[] = [];
    for (let i = Math.max(0, eventIndex - 2); i <= Math.min(streamEvents.length - 1, eventIndex + 2); i++) {
      const e = streamEvents[i];
      context.push({
        offset: i - eventIndex,
        timestamp: e.timestamp,
        type: e.type,
        summary: e.summary,
      });
    }

    return context;
  }, [selectedEventId, rawEvents, streamEvents]);

  const toggleType = useCallback((type: string) => {
    setSelectedTypes((prev) =>
      prev.includes(type) ? prev.filter((t) => t !== type) : [...prev, type]
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

  return (
    <div className="firehose-page">
      <div className="firehose-header">
        <div className="firehose-title">
          <h1>Firehose</h1>
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

        <div className="firehose-controls">
          <div className="firehose-filters">
            {EVENT_TYPES.map((type) => (
              <button
                key={type}
                className={`filter-chip ${selectedTypes.includes(type) ? 'active' : ''}`}
                onClick={() => toggleType(type)}
              >
                {type}
              </button>
            ))}
          </div>

          <input
            type="text"
            placeholder="Filter by session..."
            value={sessionFilter}
            onChange={(e) => setSessionFilter(e.target.value)}
            className="session-filter"
          />
        </div>
      </div>

      <div className="firehose-content">
        <div className="firehose-stream">
          <StreamView
            events={streamEvents}
            title="Event Stream"
            isLive={isConnected}
            isPaused={!isFollowing}
            selectedId={selectedEventId ?? undefined}
            onEventClick={(e) => setSelectedEventId(e.id)}
          />
          {!isFollowing && (
            <button className="jump-to-latest" onClick={handleJumpToLatest}>
              ↓ Jump to latest
            </button>
          )}
        </div>

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
