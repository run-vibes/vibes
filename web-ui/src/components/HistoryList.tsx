import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { listSessions, deleteSession, SessionSummary } from '../api/history';
import { HistorySearch, SearchFilters } from './HistorySearch';

interface HistoryListProps {
  onSelectSession: (sessionId: string) => void;
}

export function HistoryList({ onSelectSession }: HistoryListProps) {
  const [filters, setFilters] = useState<SearchFilters>({
    q: '',
    state: '',
    sort: 'created_at',
    order: 'desc',
  });
  const [page, setPage] = useState(0);
  const limit = 20;

  const { data, isLoading, error, refetch } = useQuery({
    queryKey: ['history-sessions', filters, page],
    queryFn: () => listSessions({
      q: filters.q || undefined,
      state: filters.state || undefined,
      sort: filters.sort as 'created_at' | 'last_accessed_at' | 'message_count' | 'total_tokens',
      order: filters.order as 'asc' | 'desc',
      limit,
      offset: page * limit,
    }),
  });

  const handleSearch = (newFilters: SearchFilters) => {
    setFilters(newFilters);
    setPage(0);
  };

  const handleDelete = async (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    if (confirm('Delete this session?')) {
      await deleteSession(id);
      refetch();
    }
  };

  const formatDate = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleString();
  };

  const formatTokens = (tokens: number) => {
    if (tokens >= 1000) {
      return `${(tokens / 1000).toFixed(1)}k`;
    }
    return tokens.toString();
  };

  if (error) {
    return <div className="error">Error loading history: {(error as Error).message}</div>;
  }

  const totalPages = data ? Math.ceil(data.total / limit) : 0;

  return (
    <div className="history-list">
      <HistorySearch onSearch={handleSearch} isLoading={isLoading} />

      {isLoading ? (
        <div className="loading">Loading...</div>
      ) : data?.sessions.length === 0 ? (
        <div className="empty">No sessions found</div>
      ) : (
        <>
          <ul className="session-list">
            {data?.sessions.map((session: SessionSummary) => (
              <li
                key={session.id}
                className={`session-item state-${session.state.toLowerCase()}`}
                onClick={() => onSelectSession(session.id)}
              >
                <div className="session-header">
                  <span className="session-name">
                    {session.name || 'Unnamed Session'}
                  </span>
                  <span className={`session-state ${session.state.toLowerCase()}`}>
                    {session.state}
                  </span>
                </div>
                <div className="session-preview">{session.preview}</div>
                <div className="session-meta">
                  <span>{formatDate(session.created_at)}</span>
                  <span>{session.message_count} messages</span>
                  <span>{formatTokens(session.total_tokens)} tokens</span>
                  <button
                    className="delete-btn"
                    onClick={(e) => handleDelete(session.id, e)}
                  >
                    Delete
                  </button>
                </div>
              </li>
            ))}
          </ul>

          {totalPages > 1 && (
            <div className="pagination">
              <button
                disabled={page === 0}
                onClick={() => setPage(p => p - 1)}
              >
                Previous
              </button>
              <span>Page {page + 1} of {totalPages}</span>
              <button
                disabled={page >= totalPages - 1}
                onClick={() => setPage(p => p + 1)}
              >
                Next
              </button>
            </div>
          )}
        </>
      )}
    </div>
  );
}
