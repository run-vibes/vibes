import { useState } from 'react';

export interface SearchFilters {
  q: string;
  state: string;
  sort: string;
  order: string;
}

interface HistorySearchProps {
  onSearch: (filters: SearchFilters) => void;
  isLoading?: boolean;
}

export function HistorySearch({ onSearch, isLoading }: HistorySearchProps) {
  const [q, setQ] = useState('');
  const [state, setState] = useState('');
  const [sort, setSort] = useState('created_at');
  const [order, setOrder] = useState('desc');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSearch({ q, state, sort, order });
  };

  return (
    <form onSubmit={handleSubmit} className="history-search">
      <input
        type="text"
        placeholder="Search messages..."
        value={q}
        onChange={(e) => setQ(e.target.value)}
        className="search-input"
      />

      <select value={state} onChange={(e) => setState(e.target.value)}>
        <option value="">All states</option>
        <option value="Finished">Finished</option>
        <option value="Failed">Failed</option>
        <option value="Idle">Idle</option>
      </select>

      <select value={sort} onChange={(e) => setSort(e.target.value)}>
        <option value="created_at">Created</option>
        <option value="last_accessed_at">Last Active</option>
        <option value="message_count">Messages</option>
        <option value="total_tokens">Tokens</option>
      </select>

      <select value={order} onChange={(e) => setOrder(e.target.value)}>
        <option value="desc">Newest</option>
        <option value="asc">Oldest</option>
      </select>

      <button type="submit" disabled={isLoading}>
        {isLoading ? 'Searching...' : 'Search'}
      </button>
    </form>
  );
}
