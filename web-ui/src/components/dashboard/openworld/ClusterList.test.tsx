/**
 * Tests for the ClusterList component
 */
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';

import { ClusterList } from './ClusterList';
import type { ClusterBrief } from '../../../hooks/useDashboard';

const mockClusters: ClusterBrief[] = [
  {
    id: 'aaaaaaaa-1234-7000-8000-000000000001',
    member_count: 5,
    category_hint: 'Missing Knowledge',
    created_at: new Date(Date.now() - 1000 * 60 * 30).toISOString(), // 30 min ago
    last_seen: new Date().toISOString(),
  },
  {
    id: 'bbbbbbbb-5678-7000-8000-000000000002',
    member_count: 3,
    category_hint: 'Context Mismatch',
    created_at: new Date(Date.now() - 1000 * 60 * 60 * 2).toISOString(), // 2 hours ago
    last_seen: new Date().toISOString(),
  },
  {
    id: 'cccccccc-9abc-7000-8000-000000000003',
    member_count: 8,
    created_at: new Date(Date.now() - 1000 * 60 * 60 * 24).toISOString(), // 1 day ago
    last_seen: new Date().toISOString(),
  },
];

describe('ClusterList', () => {
  it('renders title', () => {
    render(<ClusterList clusters={mockClusters} />);
    expect(screen.getByText('Recent Clusters')).toBeInTheDocument();
  });

  it('renders cluster table with headers', () => {
    render(<ClusterList clusters={mockClusters} />);
    expect(screen.getByTestId('cluster-list-table')).toBeInTheDocument();
    expect(screen.getByText('ID')).toBeInTheDocument();
    expect(screen.getByText('Category')).toBeInTheDocument();
    expect(screen.getByText('Members')).toBeInTheDocument();
    expect(screen.getByText('Age')).toBeInTheDocument();
  });

  it('renders cluster items', () => {
    render(<ClusterList clusters={mockClusters} />);
    // Check first cluster (truncated ID)
    expect(screen.getByText('aaaaaaaa')).toBeInTheDocument();
    expect(screen.getByText('Missing Knowledge')).toBeInTheDocument();
    expect(screen.getByText('5')).toBeInTheDocument();
  });

  it('renders dash for missing category hint', () => {
    render(<ClusterList clusters={mockClusters} />);
    // Third cluster has no category_hint
    expect(screen.getByText('—')).toBeInTheDocument();
  });

  it('shows loading state', () => {
    render(<ClusterList isLoading />);
    expect(screen.getByText('Loading...')).toBeInTheDocument();
  });

  it('shows empty state when no clusters', () => {
    render(<ClusterList clusters={[]} />);
    expect(screen.getByTestId('cluster-list-empty')).toBeInTheDocument();
    expect(screen.getByText('No clusters formed yet')).toBeInTheDocument();
  });

  it('shows empty state when clusters undefined', () => {
    render(<ClusterList />);
    expect(screen.getByTestId('cluster-list-empty')).toBeInTheDocument();
  });

  it('renders correct number of cluster rows', () => {
    render(<ClusterList clusters={mockClusters} />);
    const rows = screen.getAllByRole('row');
    // 1 header row + 3 data rows
    expect(rows).toHaveLength(4);
  });

  it('formats time as minutes ago', () => {
    const recentCluster: ClusterBrief = {
      id: 'test-id',
      member_count: 1,
      created_at: new Date(Date.now() - 1000 * 60 * 5).toISOString(), // 5 min ago
      last_seen: new Date().toISOString(),
    };
    render(<ClusterList clusters={[recentCluster]} />);
    expect(screen.getByText('5m ago')).toBeInTheDocument();
  });

  it('formats time as hours ago', () => {
    const olderCluster: ClusterBrief = {
      id: 'test-id',
      member_count: 1,
      created_at: new Date(Date.now() - 1000 * 60 * 60 * 3).toISOString(), // 3 hours ago
      last_seen: new Date().toISOString(),
    };
    render(<ClusterList clusters={[olderCluster]} />);
    expect(screen.getByText('3h ago')).toBeInTheDocument();
  });
});
