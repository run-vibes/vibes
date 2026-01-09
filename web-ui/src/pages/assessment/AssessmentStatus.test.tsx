import { describe, test, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { AssessmentStatus } from './AssessmentStatus';

// Mock TanStack Router Link
vi.mock('@tanstack/react-router', () => ({
  Link: ({ to, children }: { to: string; children: React.ReactNode }) => (
    <a href={to}>{children}</a>
  ),
}));

// Mock fetch globally
const mockFetch = vi.fn();
global.fetch = mockFetch;

// Helper to create mock responses
const createStatusResponse = (overrides = {}) => ({
  circuit_breaker: {
    enabled: true,
    cooldown_seconds: 300,
    max_interventions_per_session: 3,
  },
  sampling: {
    base_rate: 0.1,
    burnin_sessions: 5,
  },
  activity: {
    active_sessions: 2,
    events_stored: 42,
    sessions: ['session-1', 'session-2'],
  },
  ...overrides,
});

const createStatsResponse = (overrides = {}) => ({
  tier_distribution: {
    lightweight: 50,
    medium: 30,
    heavy: 15,
    checkpoint: 5,
  },
  total_assessments: 100,
  top_sessions: [],
  ...overrides,
});

describe('AssessmentStatus', () => {
  beforeEach(() => {
    mockFetch.mockReset();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  // Helper to set up both mocks
  const mockBothEndpoints = (statusData = createStatusResponse(), statsData = createStatsResponse()) => {
    mockFetch.mockImplementation((url: string) => {
      if (url.includes('/assess/status')) {
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve(statusData),
        });
      }
      if (url.includes('/assess/stats')) {
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve(statsData),
        });
      }
      return Promise.reject(new Error('Unknown URL'));
    });
  };

  test('displays loading state initially', () => {
    mockFetch.mockImplementation(() => new Promise(() => {})); // Never resolves

    render(<AssessmentStatus />);

    expect(screen.getByText(/loading/i)).toBeInTheDocument();
  });

  test('displays circuit breaker status when data loads', async () => {
    mockBothEndpoints(
      createStatusResponse({
        circuit_breaker: {
          enabled: true,
          cooldown_seconds: 300,
          max_interventions_per_session: 7,
        },
      })
    );

    render(<AssessmentStatus />);

    await waitFor(() => {
      expect(screen.getByText(/circuit breaker/i)).toBeInTheDocument();
    });

    // Circuit breaker section
    expect(screen.getByText(/enabled/i)).toBeInTheDocument();
    expect(screen.getByText('300s')).toBeInTheDocument(); // cooldown
    expect(screen.getByText('7')).toBeInTheDocument(); // max interventions (unique value)
  });

  test('displays sampling configuration', async () => {
    mockBothEndpoints(
      createStatusResponse({
        circuit_breaker: {
          enabled: false,
          cooldown_seconds: 60,
          max_interventions_per_session: 5,
        },
        sampling: {
          base_rate: 0.25,
          burnin_sessions: 10,
        },
        activity: {
          active_sessions: 0,
          events_stored: 0,
          sessions: [],
        },
      }),
      createStatsResponse({ total_assessments: 0 })
    );

    render(<AssessmentStatus />);

    await waitFor(() => {
      expect(screen.getByText(/sampling/i)).toBeInTheDocument();
    });

    expect(screen.getByText(/25%/)).toBeInTheDocument(); // base_rate as percentage
    expect(screen.getByText(/10/)).toBeInTheDocument(); // burnin_sessions
  });

  test('displays activity metrics', async () => {
    mockBothEndpoints(
      createStatusResponse({
        sampling: {
          base_rate: 0.1,
          burnin_sessions: 8,
        },
        activity: {
          active_sessions: 11,
          events_stored: 128,
          sessions: ['sess-a', 'sess-b'],
        },
      }),
      createStatsResponse({ total_assessments: 0 })
    );

    render(<AssessmentStatus />);

    await waitFor(() => {
      expect(screen.getByText(/activity/i)).toBeInTheDocument();
    });

    expect(screen.getByText('11')).toBeInTheDocument(); // active_sessions (unique)
    expect(screen.getByText('128')).toBeInTheDocument(); // events_stored (unique)
  });

  test('displays tier distribution when stats loads', async () => {
    mockBothEndpoints(
      createStatusResponse(),
      createStatsResponse({
        tier_distribution: {
          lightweight: 50,
          medium: 30,
          heavy: 15,
          checkpoint: 5,
        },
        total_assessments: 100,
      })
    );

    render(<AssessmentStatus />);

    await waitFor(() => {
      expect(screen.getByText(/tier distribution/i)).toBeInTheDocument();
    });

    // Check tier labels
    expect(screen.getByText('lightweight')).toBeInTheDocument();
    expect(screen.getByText('medium')).toBeInTheDocument();
    expect(screen.getByText('heavy')).toBeInTheDocument();
    expect(screen.getByText('checkpoint')).toBeInTheDocument();
    expect(screen.getByText('100')).toBeInTheDocument(); // total
  });

  test('displays error state when fetch fails', async () => {
    mockFetch.mockRejectedValue(new Error('Network error'));

    render(<AssessmentStatus />);

    await waitFor(() => {
      expect(screen.getByText(/error/i)).toBeInTheDocument();
    });
  });

  test('displays error state when server returns 503', async () => {
    mockFetch.mockImplementation((url: string) => {
      if (url.includes('/assess/status')) {
        return Promise.resolve({
          ok: false,
          status: 503,
          json: () =>
            Promise.resolve({
              error: 'Assessment processor not initialized',
              code: 'NOT_INITIALIZED',
            }),
        });
      }
      // Stats can also fail
      return Promise.resolve({
        ok: false,
        status: 503,
        json: () =>
          Promise.resolve({
            error: 'Assessment processor not initialized',
            code: 'NOT_INITIALIZED',
          }),
      });
    });

    render(<AssessmentStatus />);

    await waitFor(() => {
      expect(screen.getByText(/not initialized/i)).toBeInTheDocument();
    });
  });

  test('displays +more link when more than 5 sessions exist', async () => {
    mockBothEndpoints(
      createStatusResponse({
        activity: {
          active_sessions: 8,
          events_stored: 100,
          sessions: ['sess-1', 'sess-2', 'sess-3', 'sess-4', 'sess-5', 'sess-6', 'sess-7', 'sess-8'],
        },
      }),
      createStatsResponse({ total_assessments: 0 })
    );

    render(<AssessmentStatus />);

    await waitFor(() => {
      expect(screen.getByText(/activity/i)).toBeInTheDocument();
    });

    // Should show "+3 more" link
    const moreLink = screen.getByRole('link', { name: /\+3 more/i });
    expect(moreLink).toBeInTheDocument();
    expect(moreLink).toHaveAttribute('href', '/groove/assessment/history');
  });
});
