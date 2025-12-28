/**
 * WebSocket protocol types - matches vibes-server/src/ws/protocol.rs
 */

// ============================================================
// Client -> Server Messages
// ============================================================

export type ClientMessage =
  | { type: 'subscribe'; session_ids: string[]; catch_up?: boolean }
  | { type: 'unsubscribe'; session_ids: string[] }
  /** @deprecated With PTY mode, sessions are created via 'attach' */
  | { type: 'create_session'; name?: string; request_id: string }
  /** @deprecated Use 'pty_input' for PTY sessions */
  | { type: 'input'; session_id: string; content: string }
  /** @deprecated With PTY mode, permissions are handled through the terminal UI */
  | { type: 'permission_response'; session_id: string; request_id: string; approved: boolean }
  | { type: 'list_sessions'; request_id: string }
  | { type: 'kill_session'; session_id: string }
  | { type: 'request_history'; session_id: string; before_seq: number; limit: number }
  // PTY messages (preferred)
  | { type: 'attach'; session_id: string }
  | { type: 'detach'; session_id: string }
  | { type: 'pty_input'; session_id: string; data: string }  // base64 encoded
  | { type: 'pty_resize'; session_id: string; cols: number; rows: number };

// ============================================================
// Server -> Client Messages
// ============================================================

export type ServerMessage =
  | { type: 'session_created'; request_id: string; session_id: string; name?: string }
  | { type: 'session_notification'; session_id: string; name?: string }
  /** @deprecated With PTY mode, output is sent via 'pty_output' instead */
  | { type: 'claude'; session_id: string; event: ClaudeEvent }
  | { type: 'session_state'; session_id: string; state: SessionState }
  | { type: 'error'; session_id?: string; message: string; code: string }
  | { type: 'tunnel_state'; state: string; url?: string }
  | { type: 'session_list'; request_id: string; sessions: SessionInfo[] }
  | { type: 'session_removed'; session_id: string; reason: RemovalReason }
  | { type: 'ownership_transferred'; session_id: string; new_owner_id: string; you_are_owner: boolean }
  | { type: 'subscribe_ack'; session_id: string; current_seq: number; history: HistoryEvent[]; has_more: boolean }
  | { type: 'history_page'; session_id: string; events: HistoryEvent[]; has_more: boolean; oldest_seq: number }
  /** @deprecated With PTY mode, user input is sent via 'pty_input' */
  | { type: 'user_input'; session_id: string; content: string; source: InputSource }
  | AuthContextMessage
  // PTY messages
  | { type: 'pty_output'; session_id: string; data: string }  // base64 encoded
  | { type: 'pty_exit'; session_id: string; exit_code?: number }
  | { type: 'attach_ack'; session_id: string; cols: number; rows: number }
  | { type: 'pty_replay'; session_id: string; data: string };  // base64 encoded scrollback

// ============================================================
// Auth Context - matches vibes-core/src/auth/context.rs
// ============================================================

export type AuthContextMessage =
  | { type: 'auth_context'; source: 'local' }
  | { type: 'auth_context'; source: 'anonymous' }
  | { type: 'auth_context'; source: 'authenticated'; identity: AccessIdentity };

export interface AccessIdentity {
  email: string;
  name?: string;
  identity_provider?: string;
  expires_at: string;
}

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

export type RemovalReason =
  | 'killed'
  | 'owner_disconnected'
  | 'session_finished';

export type InputSource = 'cli' | 'web_ui' | 'unknown';

export interface Usage {
  input_tokens: number;
  output_tokens: number;
}

/** Event from history catch-up */
export interface HistoryEvent {
  seq: number;
  event: VibesEvent;
  timestamp: number;
}

/** Vibes event wrapper (from history) */
export type VibesEvent =
  | { type: 'user_input'; session_id: string; content: string; source?: InputSource }
  | { type: 'claude'; session_id: string; event: ClaudeEvent };

/** Message for display in chat UI */
export interface Message {
  id: string;
  role: 'user' | 'assistant' | 'tool_use' | 'tool_result';
  content: string;
  timestamp: number;
  source?: InputSource;
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

/** Session info returned by list_sessions - matches vibes-server/src/ws/protocol.rs */
export interface SessionInfo {
  id: string;
  name?: string;
  state: string;
  owner_id: string;
  is_owner: boolean;
  subscriber_count: number;
  created_at: number;
  last_activity_at: number;
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

export function isSessionListMessage(msg: ServerMessage): msg is Extract<ServerMessage, { type: 'session_list' }> {
  return msg.type === 'session_list';
}

export function isSessionRemovedMessage(msg: ServerMessage): msg is Extract<ServerMessage, { type: 'session_removed' }> {
  return msg.type === 'session_removed';
}

export function isOwnershipTransferredMessage(msg: ServerMessage): msg is Extract<ServerMessage, { type: 'ownership_transferred' }> {
  return msg.type === 'ownership_transferred';
}

export function isSubscribeAckMessage(msg: ServerMessage): msg is Extract<ServerMessage, { type: 'subscribe_ack' }> {
  return msg.type === 'subscribe_ack';
}

export function isHistoryPageMessage(msg: ServerMessage): msg is Extract<ServerMessage, { type: 'history_page' }> {
  return msg.type === 'history_page';
}

export function isUserInputMessage(msg: ServerMessage): msg is Extract<ServerMessage, { type: 'user_input' }> {
  return msg.type === 'user_input';
}

// PTY message type guards
export function isPtyOutputMessage(msg: ServerMessage): msg is Extract<ServerMessage, { type: 'pty_output' }> {
  return msg.type === 'pty_output';
}

export function isPtyExitMessage(msg: ServerMessage): msg is Extract<ServerMessage, { type: 'pty_exit' }> {
  return msg.type === 'pty_exit';
}

export function isAttachAckMessage(msg: ServerMessage): msg is Extract<ServerMessage, { type: 'attach_ack' }> {
  return msg.type === 'attach_ack';
}

export function isPtyReplayMessage(msg: ServerMessage): msg is Extract<ServerMessage, { type: 'pty_replay' }> {
  return msg.type === 'pty_replay';
}
