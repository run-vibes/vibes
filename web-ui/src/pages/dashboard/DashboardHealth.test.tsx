/**
 * Tests for DashboardHealth page
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { DashboardHealth } from './DashboardHealth';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch;

const mockHealthData = {
  data_type: 'health',
  overall_status: 'ok',
  assessment: {
    status: 'ok',
    coverage: 0.82,
    last_activity: '2026-01-09T14:30:00Z',
    item_count: 47,
  },
  extraction: {
    status: 'degraded',
    coverage: 0.45,
    last_activity: '2026-01-09T14:00:00Z',
  },
  attribution: {
    status: 'ok',
    coverage: 0.91,
    last_activity: '2026-01-09T14:25:00Z',
    item_count: 128,
  },
  adaptive_params: [
    { name: 'threshold', current_value: 0.72, confidence: 0.85, trend: 'rising' },
    { name: 'confidence', current_value: 0.45, confidence: 0.72, trend: 'falling' },
  ],
  recent_activity: [
    {
      timestamp: '2026-01-09T14:30:00Z',
      message: 'Extracted 3 patterns',
      activity_type: 'extraction',
    },
    {
      timestamp: '2026-01-09T14:25:00Z',
      message: 'Updated scores',
      activity_type: 'attribution',
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

describe('DashboardHealth', () => {
  beforeEach(() => {
    mockFetch.mockReset();
    mockFetch.mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(mockHealthData),
    });
  });

  it('renders system status banner', async () => {
    render(<DashboardHealth />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText(/operational/i)).toBeInTheDocument();
    });
  });

  it('fetches health data on mount', async () => {
    render(<DashboardHealth />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(mockFetch).toHaveBeenCalledWith('/api/groove/dashboard/health');
    });
  });

  it('renders subsystem cards', async () => {
    render(<DashboardHealth />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Assessment')).toBeInTheDocument();
      expect(screen.getByText('Extraction')).toBeInTheDocument();
      expect(screen.getByText('Attribution')).toBeInTheDocument();
    });
  });

  it('shows adaptive parameters table', async () => {
    render(<DashboardHealth />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('threshold')).toBeInTheDocument();
      expect(screen.getByText('confidence')).toBeInTheDocument();
    });
  });

  it('shows recent activity feed', async () => {
    render(<DashboardHealth />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText(/Extracted 3 patterns/)).toBeInTheDocument();
      expect(screen.getByText(/Updated scores/)).toBeInTheDocument();
    });
  });

  it('shows loading state while fetching', () => {
    mockFetch.mockImplementation(() => new Promise(() => {}));
    render(<DashboardHealth />, { wrapper: createWrapper() });

    expect(screen.getByText(/loading/i)).toBeInTheDocument();
  });

  it('shows error state on fetch failure', async () => {
    mockFetch.mockRejectedValue(new Error('Network error'));
    render(<DashboardHealth />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText(/error/i)).toBeInTheDocument();
    });
  });

  it('shows degraded status for extraction subsystem', async () => {
    render(<DashboardHealth />, { wrapper: createWrapper() });

    await waitFor(() => {
      // Degraded indicator
      expect(screen.getByText('◐')).toBeInTheDocument();
    });
  });
});
