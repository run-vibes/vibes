/**
 * Tests for DashboardStrategy page
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { DashboardStrategy } from './DashboardStrategy';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch;

const mockDistributionsData = {
  data_type: 'strategy_distributions',
  distributions: [
    {
      category_key: 'correction_interactive',
      label: 'Correction + Interactive',
      session_count: 245,
      weights: [
        { strategy: 'Prefix', weight: 0.72 },
        { strategy: 'EarlyContext', weight: 0.31 },
      ],
    },
  ],
  specialized_count: 12,
  total_learnings: 47,
};

const mockOverridesData = {
  data_type: 'strategy_overrides',
  overrides: [
    {
      learning_id: 'learn-123',
      content: 'Use snake_case for variables',
      session_count: 47,
      is_specialized: true,
      base_category: 'Correction + Interactive',
      override_weights: [{ strategy: 'Prefix', weight: 0.81 }],
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

describe('DashboardStrategy', () => {
  beforeEach(() => {
    mockFetch.mockReset();
    mockFetch.mockImplementation((url: string) => {
      if (url.includes('distributions')) {
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve(mockDistributionsData),
        });
      }
      if (url.includes('overrides')) {
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve(mockOverridesData),
        });
      }
      return Promise.reject(new Error('Unknown URL'));
    });
  });

  it('renders tab navigation', () => {
    render(<DashboardStrategy />, { wrapper: createWrapper() });

    expect(screen.getByRole('tab', { name: /distributions/i })).toBeInTheDocument();
    expect(screen.getByRole('tab', { name: /overrides/i })).toBeInTheDocument();
  });

  it('shows distributions view by default', async () => {
    render(<DashboardStrategy />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('Correction + Interactive')).toBeInTheDocument();
    });
  });

  it('fetches distributions data on mount', async () => {
    render(<DashboardStrategy />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/groove/dashboard/strategy/distributions')
      );
    });
  });

  it('switches to overrides view when tab clicked', async () => {
    render(<DashboardStrategy />, { wrapper: createWrapper() });

    fireEvent.click(screen.getByRole('tab', { name: /overrides/i }));

    await waitFor(() => {
      expect(screen.getByText('Learning Overrides')).toBeInTheDocument();
    });
  });

  it('fetches overrides data when overrides tab active', async () => {
    render(<DashboardStrategy />, { wrapper: createWrapper() });

    fireEvent.click(screen.getByRole('tab', { name: /overrides/i }));

    await waitFor(() => {
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/groove/dashboard/strategy/overrides')
      );
    });
  });

  it('shows loading state while fetching', () => {
    mockFetch.mockImplementation(() => new Promise(() => {}));
    render(<DashboardStrategy />, { wrapper: createWrapper() });

    expect(screen.getByText(/loading/i)).toBeInTheDocument();
  });

  it('shows error state on fetch failure', async () => {
    mockFetch.mockRejectedValue(new Error('Network error'));
    render(<DashboardStrategy />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText(/error/i)).toBeInTheDocument();
    });
  });

  it('shows specialized count in distributions view', async () => {
    render(<DashboardStrategy />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText(/12 of 47/i)).toBeInTheDocument();
    });
  });
});
