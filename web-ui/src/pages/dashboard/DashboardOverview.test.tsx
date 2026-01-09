/**
 * Tests for the DashboardOverview page
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';

// Mock TanStack Router Link component
vi.mock('@tanstack/react-router', () => ({
  Link: ({ to, children }: { to: string; children: React.ReactNode }) => (
    <a href={to}>{children}</a>
  ),
}));

import { DashboardOverview } from './DashboardOverview';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch;

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });
  return function Wrapper({ children }: { children: React.ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );
  };
}

beforeEach(() => {
  mockFetch.mockReset();
});

describe('DashboardOverview', () => {
  it('renders loading state initially', () => {
    mockFetch.mockImplementation(() => new Promise(() => {})); // Never resolves

    render(<DashboardOverview />, { wrapper: createWrapper() });

    expect(screen.getByText('Loading dashboard...')).toBeInTheDocument();
  });

  it('renders all cards when data loads successfully', async () => {
    const mockOverviewData = {
      data_type: 'overview',
      trends: {
        sparkline_data: [0.8, 0.85, 0.9],
        improvement_percent: 12.5,
        trend_direction: 'rising',
        session_count: 42,
        period_days: 7,
      },
      learnings: {
        total: 10,
        active: 8,
        recent: [],
        by_category: {},
      },
      attribution: {
        top_contributors: [],
        under_review_count: 0,
        negative_count: 0,
      },
      health: {
        overall_status: 'ok',
        assessment_coverage: 85,
        ablation_coverage: 70,
      },
    };

    const mockStrategyData = {
      data_type: 'strategy-distributions',
      distributions: [],
      specialized_count: 5,
      total_learnings: 10,
    };

    mockFetch
      .mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(mockOverviewData),
      })
      .mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(mockStrategyData),
      });

    render(<DashboardOverview />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Session Trends')).toBeInTheDocument();
    });

    // Check all card titles are present
    expect(screen.getByText('Learnings')).toBeInTheDocument();
    expect(screen.getByText('Attribution')).toBeInTheDocument();
    expect(screen.getByText('Health')).toBeInTheDocument();
    expect(screen.getByText('Strategy')).toBeInTheDocument();
  });

  it('shows trend data in TrendCard', async () => {
    const mockOverviewData = {
      data_type: 'overview',
      trends: {
        sparkline_data: [0.8, 0.85, 0.9],
        improvement_percent: 12.5,
        trend_direction: 'rising',
        session_count: 42,
        period_days: 7,
      },
      learnings: { total: 0, active: 0, recent: [], by_category: {} },
      attribution: { top_contributors: [], under_review_count: 0, negative_count: 0 },
      health: { overall_status: 'ok', assessment_coverage: 0, ablation_coverage: 0 },
    };

    mockFetch
      .mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(mockOverviewData),
      })
      .mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve({ data_type: 'strategy-distributions', distributions: [], specialized_count: 0, total_learnings: 0 }),
      });

    render(<DashboardOverview />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('12.5%')).toBeInTheDocument();
    });

    expect(screen.getByText('improvement')).toBeInTheDocument();
    expect(screen.getByText('↑')).toBeInTheDocument(); // Rising indicator
  });

  it('displays error state when API fails', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: false,
      status: 500,
    });

    render(<DashboardOverview />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText(/Failed to load dashboard/)).toBeInTheDocument();
    });
  });

  it('fetches overview and strategy data on mount', async () => {
    mockFetch
      .mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve({
          data_type: 'overview',
          trends: { sparkline_data: [], improvement_percent: 0, trend_direction: 'stable', session_count: 0, period_days: 7 },
          learnings: { total: 0, active: 0, recent: [], by_category: {} },
          attribution: { top_contributors: [], under_review_count: 0, negative_count: 0 },
          health: { overall_status: 'ok', assessment_coverage: 0, ablation_coverage: 0 },
        }),
      })
      .mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve({ data_type: 'strategy-distributions', distributions: [], specialized_count: 0, total_learnings: 0 }),
      });

    render(<DashboardOverview />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(mockFetch).toHaveBeenCalledWith('/api/groove/dashboard/overview');
    });

    await waitFor(() => {
      expect(mockFetch).toHaveBeenCalledWith('/api/groove/dashboard/strategy/distributions');
    });
  });
});
