// History API types and client functions

export interface SessionSummary {
  id: string;
  name: string | null;
  state: 'Idle' | 'Processing' | 'WaitingPermission' | 'Failed' | 'Finished';
  created_at: number;
  last_accessed_at: number;
  message_count: number;
  total_tokens: number;
  preview: string;
}

export interface SessionListResult {
  sessions: SessionSummary[];
  total: number;
  limit: number;
  offset: number;
}

export interface HistoricalSession {
  id: string;
  name: string | null;
  claude_session_id: string | null;
  state: string;
  created_at: number;
  last_accessed_at: number;
  total_input_tokens: number;
  total_output_tokens: number;
  message_count: number;
  error_message: string | null;
}

export interface HistoricalMessage {
  id: number;
  session_id: string;
  role: 'user' | 'assistant' | 'tool_use' | 'tool_result';
  content: string;
  tool_name: string | null;
  tool_id: string | null;
  created_at: number;
  input_tokens: number | null;
  output_tokens: number | null;
}

export interface MessageListResult {
  messages: HistoricalMessage[];
  total: number;
}

export interface SessionQueryParams {
  q?: string;
  name?: string;
  state?: string;
  tool?: string;
  min_tokens?: number;
  after?: number;
  before?: number;
  limit?: number;
  offset?: number;
  sort?: 'created_at' | 'last_accessed_at' | 'message_count' | 'total_tokens';
  order?: 'asc' | 'desc';
}

export interface MessageQueryParams {
  limit?: number;
  offset?: number;
  role?: 'user' | 'assistant' | 'tool_use' | 'tool_result';
}

export interface ResumeResponse {
  session_id: string;
  claude_session_id: string | null;
}

function buildQueryString(params: object): string {
  const searchParams = new URLSearchParams();
  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined && value !== null) {
      searchParams.append(key, String(value));
    }
  }
  const qs = searchParams.toString();
  return qs ? `?${qs}` : '';
}

export async function listSessions(params: SessionQueryParams = {}): Promise<SessionListResult> {
  const response = await fetch(`/api/history/sessions${buildQueryString(params)}`);
  if (!response.ok) {
    throw new Error(`Failed to list sessions: ${response.statusText}`);
  }
  return response.json();
}

export async function getSession(id: string): Promise<HistoricalSession> {
  const response = await fetch(`/api/history/sessions/${encodeURIComponent(id)}`);
  if (!response.ok) {
    throw new Error(`Failed to get session: ${response.statusText}`);
  }
  return response.json();
}

export async function getSessionMessages(
  sessionId: string,
  params: MessageQueryParams = {}
): Promise<MessageListResult> {
  const response = await fetch(
    `/api/history/sessions/${encodeURIComponent(sessionId)}/messages${buildQueryString(params)}`
  );
  if (!response.ok) {
    throw new Error(`Failed to get messages: ${response.statusText}`);
  }
  return response.json();
}

export async function resumeSession(id: string): Promise<ResumeResponse> {
  const response = await fetch(`/api/history/sessions/${encodeURIComponent(id)}/resume`, {
    method: 'POST',
  });
  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: response.statusText }));
    throw new Error(error.error || 'Failed to resume session');
  }
  return response.json();
}

export async function deleteSession(id: string): Promise<void> {
  const response = await fetch(`/api/history/sessions/${encodeURIComponent(id)}`, {
    method: 'DELETE',
  });
  if (!response.ok) {
    throw new Error(`Failed to delete session: ${response.statusText}`);
  }
}
