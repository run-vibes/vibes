// web-ui/src/pages/assessment/AssessmentHistory.tsx
import { useEffect, useState } from 'react';
import { PageHeader, EmptyState, Card } from '@vibes/design-system';
import './AssessmentHistory.css';

interface SessionHistoryItem {
  session_id: string;
  event_count: number;
  result_types: string[];
}

interface AssessmentHistoryResponse {
  sessions: SessionHistoryItem[];
  has_more: boolean;
  page: number;
  per_page: number;
  total: number;
  total_pages: number;
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
  const [currentPage, setCurrentPage] = useState<number>(1);

  useEffect(() => {
    async function fetchHistory() {
      try {
        const params = new URLSearchParams();
        if (selectedSession) {
          params.set('session', selectedSession);
        }
        params.set('page', currentPage.toString());

        const url = `/api/groove/assess/history?${params.toString()}`;

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
  }, [selectedSession, currentPage]);

  const handleSessionChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
    setSelectedSession(event.target.value);
    setCurrentPage(1); // Reset to page 1 when session filter changes
  };

  const handlePrevPage = () => {
    if (currentPage > 1) {
      setCurrentPage(currentPage - 1);
    }
  };

  const handleNextPage = () => {
    if (state.status === 'success' && state.data.page < state.data.total_pages) {
      setCurrentPage(currentPage + 1);
    }
  };

  if (state.status === 'loading') {
    return (
      <div className="history-page">
        <PageHeader title="HISTORY" />
        <div className="history-loading">Loading...</div>
      </div>
    );
  }

  if (state.status === 'error') {
    return (
      <div className="history-page">
        <PageHeader title="HISTORY" />
        <div className="history-error">
          <h2>Error</h2>
          <p>{state.error}</p>
        </div>
      </div>
    );
  }

  const { sessions, page, total_pages } = state.data;

  if (sessions.length === 0) {
    return (
      <div className="history-page">
        <PageHeader title="HISTORY" />
        <Card variant="crt">
          <EmptyState
            icon="📊"
            message="No assessment data available yet"
            hint="Assessment data will appear here after sessions are evaluated."
          />
        </Card>
      </div>
    );
  }

  return (
    <div className="history-page">
      <PageHeader title="HISTORY" />
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

      {total_pages > 1 && (
        <div className="pagination-controls">
          <button
            className="pagination-button"
            onClick={handlePrevPage}
            disabled={page <= 1}
          >
            Prev
          </button>
          <span className="pagination-info">Page {page} of {total_pages}</span>
          <button
            className="pagination-button"
            onClick={handleNextPage}
            disabled={page >= total_pages}
          >
            Next
          </button>
        </div>
      )}
    </div>
  );
}
