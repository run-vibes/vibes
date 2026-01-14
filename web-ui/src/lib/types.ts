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
  | { type: 'list_models'; request_id: string }
  | { type: 'kill_session'; session_id: string }
  // PTY messages (preferred)
  | { type: 'attach'; session_id: string; name?: string; cols?: number; rows?: number }
  | { type: 'detach'; session_id: string }
  | { type: 'pty_input'; session_id: string; data: string }  // base64 encoded
  | { type: 'pty_resize'; session_id: string; cols: number; rows: number }
  // Agent messages
  | { type: 'list_agents'; request_id: string }
  | { type: 'spawn_agent'; request_id: string; agent_type: AgentType; name?: string; task?: string }
  | { type: 'agent_status'; request_id: string; agent_id: string }
  | { type: 'pause_agent'; request_id: string; agent_id: string }
  | { type: 'resume_agent'; request_id: string; agent_id: string }
  | { type: 'cancel_agent'; request_id: string; agent_id: string }
  | { type: 'stop_agent'; request_id: string; agent_id: string };

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
  | { type: 'model_list'; request_id: string; models: ModelInfo[] }
  | { type: 'session_removed'; session_id: string; reason: RemovalReason }
  | { type: 'ownership_transferred'; session_id: string; new_owner_id: string; you_are_owner: boolean }
  /** @deprecated With PTY mode, user input is sent via 'pty_input' */
  | { type: 'user_input'; session_id: string; content: string; source: InputSource }
  | AuthContextMessage
  // PTY messages
  | { type: 'pty_output'; session_id: string; data: string }  // base64 encoded
  | { type: 'pty_exit'; session_id: string; exit_code?: number }
  | { type: 'attach_ack'; session_id: string; cols: number; rows: number }
  | { type: 'pty_replay'; session_id: string; data: string }  // base64 encoded scrollback
  // Agent messages
  | { type: 'agent_list'; request_id: string; agents: AgentInfo[] }
  | { type: 'agent_spawned'; request_id: string; agent: AgentInfo }
  | { type: 'agent_status_response'; request_id: string; agent: AgentInfo }
  | { type: 'agent_ack'; request_id: string; agent_id: string; operation: string };

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
// VibesEvent - matches vibes-core/src/events/types.rs
// ============================================================

export type VibesEvent =
  | { type: 'claude'; session_id: string; event: ClaudeEvent }
  | { type: 'user_input'; session_id: string; content: string; source: InputSource }
  | { type: 'permission_response'; session_id: string; request_id: string; approved: boolean }
  | { type: 'session_created'; session_id: string; name?: string }
  | { type: 'session_state_changed'; session_id: string; state: string }
  | { type: 'client_connected'; client_id: string }
  | { type: 'client_disconnected'; client_id: string }
  | { type: 'tunnel_state_changed'; state: string; url?: string }
  | { type: 'ownership_transferred'; session_id: string; new_owner_id: string }
  | { type: 'session_removed'; session_id: string; reason: string }
  | { type: 'hook'; session_id?: string; event: HookEvent };

export type HookEvent =
  | { type: 'pre_tool_use'; tool_name: string; input: string; session_id?: string }
  | { type: 'post_tool_use'; tool_name: string; input: string; output: string; session_id?: string }
  | { type: 'notification'; title: string; body: string; session_id?: string }
  | { type: 'stop'; transcript_path?: string; reason?: string; session_id?: string };

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
// Agent Types - matches vibes-core/src/agent/types.rs
// ============================================================

export type AgentType = 'AdHoc' | 'Background' | 'Subagent' | 'Interactive';

export type AgentStatus =
  | { idle: true }
  | { running: { task: string; started: string } }
  | { paused: { task: string; reason: string } }
  | { waiting_for_input: { prompt: string } }
  | { failed: { error: string } };

export interface ExecutionLocation {
  type: 'Local' | 'Remote';
  endpoint?: string;
}

export interface ResourceLimits {
  max_tokens?: number;
  max_duration?: { secs: number; nanos: number };
  max_tool_calls?: number;
}

export interface Permissions {
  filesystem: boolean;
  network: boolean;
  shell: boolean;
}

export interface AgentContext {
  location: ExecutionLocation;
  model: { id: string };
  tools: { id: string }[];
  permissions: Permissions;
  resource_limits: ResourceLimits;
}

export interface TaskMetrics {
  duration: { secs: number; nanos: number };
  tokens_used: number;
  tool_calls: number;
  iterations: number;
}

/** Agent info returned by list_agents - matches vibes-server/src/ws/protocol.rs */
export interface AgentInfo {
  id: string;
  name: string;
  agent_type: AgentType;
  status: AgentStatus;
  context: AgentContext;
  current_task_metrics?: TaskMetrics;
}

/** Model info returned by list_models - matches vibes-models/src/types.rs */
export interface ModelInfo {
  id: string;
  provider: string;
  name: string;
  context_window: number;
  max_output?: number;
  capabilities: {
    chat: boolean;
    vision: boolean;
    tools: boolean;
    embeddings: boolean;
    streaming: boolean;
  };
  pricing?: {
    input_per_million: number;
    output_per_million: number;
  };
  local: boolean;
  size_bytes?: number;
  modified_at?: string;
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

export function isModelListMessage(msg: ServerMessage): msg is Extract<ServerMessage, { type: 'model_list' }> {
  return msg.type === 'model_list';
}

export function isSessionRemovedMessage(msg: ServerMessage): msg is Extract<ServerMessage, { type: 'session_removed' }> {
  return msg.type === 'session_removed';
}

export function isOwnershipTransferredMessage(msg: ServerMessage): msg is Extract<ServerMessage, { type: 'ownership_transferred' }> {
  return msg.type === 'ownership_transferred';
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

// Agent message type guards
export function isAgentListMessage(msg: ServerMessage): msg is Extract<ServerMessage, { type: 'agent_list' }> {
  return msg.type === 'agent_list';
}

export function isAgentSpawnedMessage(msg: ServerMessage): msg is Extract<ServerMessage, { type: 'agent_spawned' }> {
  return msg.type === 'agent_spawned';
}

export function isAgentStatusResponseMessage(msg: ServerMessage): msg is Extract<ServerMessage, { type: 'agent_status_response' }> {
  return msg.type === 'agent_status_response';
}

export function isAgentAckMessage(msg: ServerMessage): msg is Extract<ServerMessage, { type: 'agent_ack' }> {
  return msg.type === 'agent_ack';
}

// Agent status helpers
export function getAgentStatusVariant(status: AgentStatus): 'idle' | 'running' | 'paused' | 'waiting_for_input' | 'failed' {
  if ('idle' in status) return 'idle';
  if ('running' in status) return 'running';
  if ('paused' in status) return 'paused';
  if ('waiting_for_input' in status) return 'waiting_for_input';
  if ('failed' in status) return 'failed';
  return 'idle'; // fallback
}
