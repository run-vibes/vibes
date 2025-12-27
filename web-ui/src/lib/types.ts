/**
 * WebSocket protocol types - matches vibes-server/src/ws/protocol.rs
 */

// ============================================================
// Client -> Server Messages
// ============================================================

export type ClientMessage =
  | { type: 'subscribe'; session_ids: string[] }
  | { type: 'unsubscribe'; session_ids: string[] }
  | { type: 'create_session'; name?: string; request_id: string }
  | { type: 'input'; session_id: string; content: string }
  | { type: 'permission_response'; session_id: string; request_id: string; approved: boolean };

// ============================================================
// Server -> Client Messages
// ============================================================

export type ServerMessage =
  | { type: 'session_created'; request_id: string; session_id: string; name?: string }
  | { type: 'session_notification'; session_id: string; name?: string }
  | { type: 'claude'; session_id: string; event: ClaudeEvent }
  | { type: 'session_state'; session_id: string; state: SessionState }
  | { type: 'error'; session_id?: string; message: string; code: string };

// ============================================================
// Claude Events - matches vibes-core/src/events/types.rs
// ============================================================

export type ClaudeEvent =
  | { type: 'text_delta'; text: string }
  | { type: 'thinking_delta'; text: string }
  | { type: 'tool_use_start'; id: string; name: string }
  | { type: 'tool_input_delta'; id: string; delta: string }
  | { type: 'tool_result'; id: string; output: string; is_error: boolean }
  | { type: 'turn_start' }
  | { type: 'turn_complete'; usage: Usage }
  | { type: 'error'; message: string; recoverable: boolean }
  | { type: 'permission_request'; id: string; tool: string; description: string };

// ============================================================
// Supporting Types
// ============================================================

export type SessionState =
  | 'idle'
  | 'processing'
  | 'waiting_permission'
  | 'finished'
  | 'failed';

export interface Usage {
  input_tokens: number;
  output_tokens: number;
}

export interface Session {
  id: string;
  name?: string;
  state: SessionState;
  created_at: string;
  usage?: Usage;
  pending_permission?: {
    id: string;
    tool: string;
    description: string;
  };
}

// ============================================================
// Type Guards
// ============================================================

export function isClaudeMessage(msg: ServerMessage): msg is Extract<ServerMessage, { type: 'claude' }> {
  return msg.type === 'claude';
}

export function isSessionStateMessage(msg: ServerMessage): msg is Extract<ServerMessage, { type: 'session_state' }> {
  return msg.type === 'session_state';
}

export function isErrorMessage(msg: ServerMessage): msg is Extract<ServerMessage, { type: 'error' }> {
  return msg.type === 'error';
}
