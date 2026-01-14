// web-ui/src/pages/assessment/AssessmentStream.tsx
import { useState, useMemo, useCallback, useEffect, useRef } from 'react';
import { StreamView, EventInspector, Badge, PageHeader } from '@vibes/design-system';
import type { DisplayEvent, ContextEvent } from '@vibes/design-system';
import { useAssessment } from '../../hooks/useAssessment';
import type { AssessmentEvent } from '../../hooks/useAssessment';
import { extractTimestampFromUuidv7 } from '../../lib/uuidv7';
import './AssessmentStream.css';

const ASSESSMENT_TIERS = ['lightweight', 'medium', 'heavy'] as const;

function toDisplayEvent(event: AssessmentEvent): DisplayEvent {
  const baseEvent = {
    id: event.event_id,
    timestamp: extractTimestampFromUuidv7(event.event_id),
    session: event.context.session_id,
  };

  switch (event.tier) {
    case 'lightweight':
      return {
        ...baseEvent,
        type: 'ASSESS',
        summary: summarizeLightweightEvent(event),
      };
    case 'medium':
      return {
        ...baseEvent,
        type: 'ASSESS',
        summary: summarizeMediumEvent(event),
      };
    case 'heavy':
      return {
        ...baseEvent,
        type: 'ASSESS',
        summary: summarizeHeavyEvent(event),
      };
    default:
      return {
        ...baseEvent,
        type: 'ASSESS',
        summary: `Assessment: ${event.tier}`,
      };
  }
}

function summarizeLightweightEvent(event: AssessmentEvent): string {
  // Lightweight events contain pattern match signals
  const signals = event.signals as { positive?: number; negative?: number } | undefined;
  if (signals) {
    const parts: string[] = [];
    if (signals.positive && signals.positive > 0) parts.push(`+${signals.positive}`);
    if (signals.negative && signals.negative > 0) parts.push(`-${signals.negative}`);
    if (parts.length > 0) return `Lightweight: ${parts.join(' ')}`;
  }
  return 'Lightweight: pattern check';
}

function summarizeMediumEvent(event: AssessmentEvent): string {
  // Medium events are LLM-generated assessments
  const assessment = event.assessment as string | undefined;
  if (assessment) {
    return `Medium: ${assessment.slice(0, 40)}${assessment.length > 40 ? '...' : ''}`;
  }
  return 'Medium: LLM assessment';
}

function summarizeHeavyEvent(event: AssessmentEvent): string {
  // Heavy events are comprehensive behavioral analyses
  const verdict = event.verdict as string | undefined;
  if (verdict) {
    return `Heavy: ${verdict}`;
  }
  return 'Heavy: behavioral analysis';
}

export function AssessmentStream() {
  const [selectedTypes, setSelectedTypes] = useState<string[]>([]);
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
  } = useAssessment();

  // Skip the first useEffect run - the initial request is sent by
  // useAssessment in onopen. Only send filter updates when the user
  // actually changes filters (not on initial mount).
  const isInitialMount = useRef(true);

  useEffect(() => {
    if (isInitialMount.current) {
      isInitialMount.current = false;
      return;
    }
    // Session filter is handled by server; type filtering is client-side
    setFilters({ sessionId: null });
  }, [setFilters]);

  // Convert raw events to display format, then apply client-side filters
  const displayEvents = useMemo(() => {
    let events = rawEvents.map((e) => toDisplayEvent(e));

    // Apply tier filter (client-side)
    if (selectedTypes.length > 0) {
      events = events.filter((e) => {
        const rawEvent = rawEvents.find((r) => r.event_id === e.id);
        return rawEvent && selectedTypes.includes(rawEvent.tier);
      });
    }

    // Apply search filter (client-side)
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      events = events.filter((e) => e.summary.toLowerCase().includes(query));
    }

    return events;
  }, [rawEvents, selectedTypes, searchQuery]);

  const selectedEvent = useMemo((): DisplayEvent | null => {
    if (!selectedEventId) return null;
    const raw = rawEvents.find((e) => e.event_id === selectedEventId);
    if (!raw) return null;

    const display = toDisplayEvent(raw);
    return {
      id: display.id,
      timestamp: display.timestamp,
      type: display.type,
      summary: display.summary,
      session: display.session,
      payload: raw,
    };
  }, [selectedEventId, rawEvents]);

  const contextEvents = useMemo((): ContextEvent[] => {
    if (!selectedEventId) return [];
    const eventIndex = displayEvents.findIndex((e) => e.id === selectedEventId);
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
  }, [selectedEventId, displayEvents]);

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

  const statusBadges = (
    <div className="assessment-status">
      {isConnected ? (
        <Badge status="success">Connected</Badge>
      ) : (
        <Badge status="error">Disconnected</Badge>
      )}
      {!isFollowing && <Badge status="warning">Paused</Badge>}
      {isLoadingOlder && <Badge status="idle">Loading...</Badge>}
      {error && <Badge status="error">{error.message}</Badge>}
    </div>
  );

  const headerControls = (
    <div className="assessment-header-controls">
      <div className="assessment-filters">
        {ASSESSMENT_TIERS.map((tier) => (
          <button
            key={tier}
            className={`filter-chip ${selectedTypes.includes(tier) ? 'active' : ''}`}
            onClick={() => toggleType(tier)}
          >
            {tier}
          </button>
        ))}
      </div>

      <input
        type="text"
        placeholder="Search events..."
        value={searchQuery}
        onChange={(e) => setSearchQuery(e.target.value)}
        className="search-input"
      />
    </div>
  );

  return (
    <div className="assessment-page">
      <PageHeader
        title="ASSESSMENT"
        leftContent={statusBadges}
        rightContent={headerControls}
      />

      <div className="assessment-content">
        <div className="assessment-stream">
          <StreamView
            events={displayEvents}
            title="Assessment Events"
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

        <div className="assessment-inspector">
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
