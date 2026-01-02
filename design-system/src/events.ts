// design-system/src/types/events.ts

/**
 * Unified event type for all display purposes.
 * Used by StreamView for list display and EventInspector for detail view.
 */
export interface DisplayEvent {
  /** Unique event identifier (UUIDv7 from server) */
  id: string;
  /** When the event occurred */
  timestamp: Date;
  /** Event category: SESSION, CLAUDE, TOOL, HOOK, ERROR, ASSESS */
  type: string;
  /** Human-readable one-line summary */
  summary: string;
  /** Session ID if event is session-scoped */
  session?: string;
  /** Session display name */
  sessionName?: string;
  /** EventLog offset (for inspector metadata display) */
  offset?: number;
  /** Raw event payload (for inspector JSON view) */
  payload?: unknown;
}

/**
 * Event in the context window around a selected event.
 * Used by EventInspector to show surrounding events.
 */
export interface ContextEvent {
  /** Position relative to selected event: -2, -1, 0, +1, +2 */
  relativePosition: number;
  /** When the event occurred */
  timestamp: Date;
  /** Event category */
  type: string;
  /** Human-readable one-line summary */
  summary: string;
}
