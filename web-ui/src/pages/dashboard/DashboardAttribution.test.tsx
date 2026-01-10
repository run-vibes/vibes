/**
 * Tests for DashboardAttribution page
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { DashboardAttribution } from './DashboardAttribution';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch;

const mockAttributionData = {
  data_type: 'attribution',
  top_contributors: [
    {
      learning_id: 'learn-1',
      content: 'Use semantic HTML',
      estimated_value: 0.34,
      confidence: 0.87,
      session_count: 23,
      status: 'active',
    },
  ],
  negative_impact: [],
  ablation_coverage: {
    coverage_percent: 42,
    completed: 12,
    in_progress: 5,
    pending: 30,
  },
};

const mockTimelineData = {
  data_type: 'session_timeline',
  sessions: [
    {
      session_id: 'session-abc',
      timestamp: new Date().toISOString(),
      score: 0.82,
      activated_learnings: [
        { learning_id: 'learn-1', content: 'Use semantic HTML', contribution: 0.15 },
      ],
      outcome: 'positive',
    },
  ],
};

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
    },
  });
  return function Wrapper({ children }: { children: React.ReactNode }) {
    return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
  };
}

describe('DashboardAttribution', () => {
  beforeEach(() => {
    mockFetch.mockReset();
    mockFetch.mockImplementation((url: string) => {
      if (url.includes('attribution')) {
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve(mockAttributionData),
        });
      }
      if (url.includes('session-timeline')) {
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve(mockTimelineData),
        });
      }
      return Promise.reject(new Error('Unknown URL'));
    });
  });

  it('renders tab navigation', () => {
    render(<DashboardAttribution />, { wrapper: createWrapper() });

    expect(screen.getByRole('tab', { name: /leaderboard/i })).toBeInTheDocument();
    expect(screen.getByRole('tab', { name: /timeline/i })).toBeInTheDocument();
  });

  it('shows leaderboard view by default', async () => {
    render(<DashboardAttribution />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Top Contributors')).toBeInTheDocument();
    });
  });

  it('fetches attribution data on mount', async () => {
    render(<DashboardAttribution />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/groove/dashboard/attribution')
      );
    });
  });

  it('switches to timeline view when tab clicked', async () => {
    render(<DashboardAttribution />, { wrapper: createWrapper() });

    fireEvent.click(screen.getByRole('tab', { name: /timeline/i }));

    await waitFor(() => {
      expect(screen.getByText('Session Timeline')).toBeInTheDocument();
    });
  });

  it('fetches session timeline data when timeline tab active', async () => {
    render(<DashboardAttribution />, { wrapper: createWrapper() });

    fireEvent.click(screen.getByRole('tab', { name: /timeline/i }));

    await waitFor(() => {
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/groove/dashboard/session-timeline')
      );
    });
  });

  it('shows loading state while fetching', () => {
    mockFetch.mockImplementation(() => new Promise(() => {})); // Never resolves
    render(<DashboardAttribution />, { wrapper: createWrapper() });

    expect(screen.getByText(/loading/i)).toBeInTheDocument();
  });

  it('shows error state on fetch failure', async () => {
    mockFetch.mockRejectedValue(new Error('Network error'));
    render(<DashboardAttribution />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText(/error/i)).toBeInTheDocument();
    });
  });
});
