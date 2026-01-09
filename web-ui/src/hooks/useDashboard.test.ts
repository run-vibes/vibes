/**
 * Tests for the useDashboard hooks
 *
 * Tests cover:
 * - Query key generation
 * - Data fetching
 * - Error handling
 * - Filter parameter encoding
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { createElement, ReactNode } from 'react';
import {
  useDashboardOverview,
  useDashboardLearnings,
  useDashboardLearningDetail,
  useDashboardAttribution,
  useDashboardHealth,
  useDashboardStrategyDistributions,
  useDashboardStrategyOverrides,
} from './useDashboard';

// Mock fetch
const mockFetch = vi.fn();
globalThis.fetch = mockFetch;

// Create a wrapper component for React Query
function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });
  return function Wrapper({ children }: { children: ReactNode }) {
    return createElement(QueryClientProvider, { client: queryClient }, children);
  };
}

beforeEach(() => {
  mockFetch.mockReset();
});

afterEach(() => {
  vi.clearAllMocks();
});

// ============================================================================
// Overview Hook Tests
// ============================================================================

describe('useDashboardOverview', () => {
  it('fetches overview data from correct endpoint', async () => {
    const mockData = {
      data_type: 'overview',
      trends: {
        sparkline_data: [0.8, 0.85, 0.9],
        improvement_percent: 12.5,
        trend_direction: 'rising',
        session_count: 42,
        period_days: 7,
      },
      learnings: { total: 10, active: 8, recent: [], by_category: {} },
      attribution: { top_contributors: [], under_review_count: 0, negative_count: 0 },
      health: {
        overall_status: 'ok',
        assessment_coverage: 0.85,
        ablation_coverage: 0.45,
      },
    };

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: () => Promise.resolve(mockData),
    });

    const { result } = renderHook(() => useDashboardOverview(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(mockFetch).toHaveBeenCalledWith('/api/groove/dashboard/overview');
    expect(result.current.data?.data_type).toBe('overview');
    expect(result.current.data?.trends.trend_direction).toBe('rising');
  });

  it('handles fetch errors', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: false,
      status: 500,
    });

    const { result } = renderHook(() => useDashboardOverview(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isError).toBe(true));
    expect(result.current.error?.message).toBe('Failed to fetch dashboard overview');
  });
});

// ============================================================================
// Learnings Hook Tests
// ============================================================================

describe('useDashboardLearnings', () => {
  it('fetches learnings without filters', async () => {
    const mockData = {
      data_type: 'learnings',
      learnings: [],
      total: 0,
    };

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: () => Promise.resolve(mockData),
    });

    const { result } = renderHook(() => useDashboardLearnings(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(mockFetch).toHaveBeenCalledWith('/api/groove/dashboard/learnings');
    expect(result.current.data?.data_type).toBe('learnings');
  });

  it('includes filter parameters in URL', async () => {
    const mockData = {
      data_type: 'learnings',
      learnings: [],
      total: 0,
    };

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: () => Promise.resolve(mockData),
    });

    const { result } = renderHook(
      () =>
        useDashboardLearnings({
          category: 'Correction',
          status: 'active',
        }),
      { wrapper: createWrapper() }
    );

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    const calledUrl = mockFetch.mock.calls[0][0];
    expect(calledUrl).toContain('category=Correction');
    expect(calledUrl).toContain('status=active');
  });
});

// ============================================================================
// Learning Detail Hook Tests
// ============================================================================

describe('useDashboardLearningDetail', () => {
  it('fetches learning detail by id', async () => {
    const mockData = {
      data_type: 'learning_detail',
      id: '123e4567-e89b-12d3-a456-426614174000',
      content: 'Test learning',
      category: 'Correction',
      scope: { Project: 'test' },
      status: 'active',
      estimated_value: 0.5,
      confidence: 0.8,
      times_injected: 10,
      activation_rate: 0.7,
      session_count: 15,
      created_at: '2024-01-01T00:00:00Z',
      extraction_method: 'correction',
    };

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: () => Promise.resolve(mockData),
    });

    const { result } = renderHook(
      () => useDashboardLearningDetail('123e4567-e89b-12d3-a456-426614174000'),
      { wrapper: createWrapper() }
    );

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(mockFetch).toHaveBeenCalledWith(
      '/api/groove/dashboard/learnings/123e4567-e89b-12d3-a456-426614174000'
    );
    expect(result.current.data?.content).toBe('Test learning');
  });

  it('does not fetch when id is empty', async () => {
    renderHook(() => useDashboardLearningDetail(''), {
      wrapper: createWrapper(),
    });

    // Wait a bit to ensure no fetch was triggered
    await new Promise((r) => setTimeout(r, 100));
    expect(mockFetch).not.toHaveBeenCalled();
  });
});

// ============================================================================
// Health Hook Tests
// ============================================================================

describe('useDashboardHealth', () => {
  it('fetches health data', async () => {
    const mockData = {
      data_type: 'health',
      overall_status: 'ok',
      assessment: { status: 'ok', coverage: 0.9 },
      extraction: { status: 'ok', coverage: 0.8 },
      attribution: { status: 'ok', coverage: 0.7 },
      adaptive_params: [],
      recent_activity: [],
    };

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: () => Promise.resolve(mockData),
    });

    const { result } = renderHook(() => useDashboardHealth(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(mockFetch).toHaveBeenCalledWith('/api/groove/dashboard/health');
    expect(result.current.data?.overall_status).toBe('ok');
  });
});

// ============================================================================
// Strategy Distribution Hook Tests
// ============================================================================

describe('useDashboardStrategyDistributions', () => {
  it('fetches strategy distributions', async () => {
    const mockData = {
      data_type: 'strategy_distributions',
      distributions: [],
      specialized_count: 0,
      total_learnings: 0,
    };

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: () => Promise.resolve(mockData),
    });

    const { result } = renderHook(() => useDashboardStrategyDistributions(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(mockFetch).toHaveBeenCalledWith('/api/groove/dashboard/strategy/distributions');
  });
});

// ============================================================================
// Strategy Overrides Hook Tests
// ============================================================================

describe('useDashboardStrategyOverrides', () => {
  it('fetches strategy overrides', async () => {
    const mockData = {
      data_type: 'strategy_overrides',
      overrides: [],
    };

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: () => Promise.resolve(mockData),
    });

    const { result } = renderHook(() => useDashboardStrategyOverrides(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(mockFetch).toHaveBeenCalledWith('/api/groove/dashboard/strategy/overrides');
  });
});

// ============================================================================
// Attribution Hook Tests
// ============================================================================

describe('useDashboardAttribution', () => {
  it('fetches attribution data without period', async () => {
    const mockData = {
      data_type: 'attribution',
      top_contributors: [],
      negative_impact: [],
      ablation_coverage: { coverage_percent: 0, completed: 0, in_progress: 0, pending: 0 },
    };

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: () => Promise.resolve(mockData),
    });

    const { result } = renderHook(() => useDashboardAttribution(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(mockFetch).toHaveBeenCalledWith('/api/groove/dashboard/attribution');
  });

  it('includes days parameter when specified', async () => {
    const mockData = {
      data_type: 'attribution',
      top_contributors: [],
      negative_impact: [],
      ablation_coverage: { coverage_percent: 0, completed: 0, in_progress: 0, pending: 0 },
    };

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: () => Promise.resolve(mockData),
    });

    const { result } = renderHook(() => useDashboardAttribution(7), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));

    expect(mockFetch).toHaveBeenCalledWith('/api/groove/dashboard/attribution?days=7');
  });
});
