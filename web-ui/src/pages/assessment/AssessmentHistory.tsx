// web-ui/src/pages/assessment/AssessmentHistory.tsx
import { useEffect, useState } from 'react';
import './AssessmentHistory.css';

interface SessionHistoryItem {
  session_id: string;
  event_count: number;
  result_types: string[];
}

interface AssessmentHistoryResponse {
  sessions: SessionHistoryItem[];
  has_more: boolean;
}

interface ErrorResponse {
  error: string;
  code: string;
}

type FetchState =
  | { status: 'loading' }
  | { status: 'success'; data: AssessmentHistoryResponse }
  | { status: 'error'; error: string };

export function AssessmentHistory() {
  const [state, setState] = useState<FetchState>({ status: 'loading' });
  const [selectedSession, setSelectedSession] = useState<string>('');

  useEffect(() => {
    async function fetchHistory() {
      try {
        const url = selectedSession
          ? `/api/groove/assess/history?session=${encodeURIComponent(selectedSession)}`
          : '/api/groove/assess/history';

        const response = await fetch(url);

        if (!response.ok) {
          const errorData: ErrorResponse = await response.json();
          if (errorData.code === 'NOT_INITIALIZED') {
            setState({ status: 'error', error: 'Assessment processor not initialized' });
          } else {
            setState({ status: 'error', error: errorData.error || 'Failed to load history' });
          }
          return;
        }

        const data: AssessmentHistoryResponse = await response.json();
        setState({ status: 'success', data });
      } catch {
        setState({ status: 'error', error: 'Failed to connect to server' });
      }
    }

    fetchHistory();
  }, [selectedSession]);

  const handleSessionChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
    setSelectedSession(event.target.value);
  };

  if (state.status === 'loading') {
    return (
      <div className="history-page">
        <div className="history-loading">Loading...</div>
      </div>
    );
  }

  if (state.status === 'error') {
    return (
      <div className="history-page">
        <div className="history-error">
          <h2>Error</h2>
          <p>{state.error}</p>
        </div>
      </div>
    );
  }

  const { sessions } = state.data;

  if (sessions.length === 0) {
    return (
      <div className="history-page">
        <div className="history-empty">
          <p>No assessment data available yet.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="history-page">
      <div className="history-controls">
        <label htmlFor="session-selector" className="selector-label">
          Filter by session:
        </label>
        <select
          id="session-selector"
          className="session-selector"
          value={selectedSession}
          onChange={handleSessionChange}
        >
          <option value="">All sessions</option>
          {sessions.map((session) => (
            <option key={session.session_id} value={session.session_id}>
              {session.session_id}
            </option>
          ))}
        </select>
      </div>

      <div className="history-list">
        {sessions.map((session) => (
          <div key={session.session_id} className="history-card">
            <div className="history-card-header">
              <span className="session-id">{session.session_id}</span>
              <span className="event-count">{session.event_count} events</span>
            </div>
            <div className="history-card-types">
              {session.result_types.map((type) => (
                <span key={type} className={`type-badge type-${type}`}>
                  {type}
                </span>
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
