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
  | { type: 'permission_response'; session_id: string; request_id: string; approved: boolean }
  | { type: 'list_sessions'; request_id: string }
  | { type: 'kill_session'; session_id: string };

// ============================================================
// Server -> Client Messages
// ============================================================

export type ServerMessage =
  | { type: 'session_created'; request_id: string; session_id: string; name?: string }
  | { type: 'session_notification'; session_id: string; name?: string }
  | { type: 'claude'; session_id: string; event: ClaudeEvent }
  | { type: 'session_state'; session_id: string; state: SessionState }
  | { type: 'error'; session_id?: string; message: string; code: string }
  | { type: 'tunnel_state'; state: string; url?: string }
  | { type: 'session_list'; request_id: string; sessions: SessionInfo[] }
  | { type: 'session_removed'; session_id: string; reason: RemovalReason }
  | { type: 'ownership_transferred'; session_id: string; new_owner_id: string; you_are_owner: boolean }
  | AuthContextMessage;

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
