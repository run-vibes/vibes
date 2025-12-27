import { useQuery } from '@tanstack/react-query';
import { SessionCard } from '../components/SessionCard';
import type { Session } from '../lib/types';

interface SessionsResponse {
  sessions: Session[];
}

async function fetchSessions(): Promise<SessionsResponse> {
  const response = await fetch('/api/claude/sessions');
  if (!response.ok) {
    throw new Error('Failed to fetch sessions');
  }
  return response.json();
}

export function ClaudeSessions() {
  const { data, isLoading, error } = useQuery({
    queryKey: ['sessions'],
    queryFn: fetchSessions,
    refetchInterval: 5000, // Refresh every 5 seconds
  });

  if (isLoading) {
    return (
      <div className="page">
        <h1>Claude Sessions</h1>
        <p>Loading sessions...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="page">
        <h1>Claude Sessions</h1>
        <p className="error">Failed to load sessions. Is the daemon running?</p>
      </div>
    );
  }

  const sessions = data?.sessions ?? [];

  return (
    <div className="page">
      <h1>Claude Sessions</h1>
      {sessions.length === 0 ? (
        <p>No active sessions. Start one with <code>vibes claude "your prompt"</code></p>
      ) : (
        <div className="session-grid">
          {sessions.map((session) => (
            <SessionCard
              key={session.id}
              id={session.id}
              name={session.name}
              state={session.state}
              createdAt={session.created_at}
            />
          ))}
        </div>
      )}
    </div>
  );
}
