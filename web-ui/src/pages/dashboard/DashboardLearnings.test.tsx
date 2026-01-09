/**
 * Tests for DashboardLearnings page
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { DashboardLearnings } from './DashboardLearnings';

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

const mockLearningsData = {
  data_type: 'learnings',
  learnings: [
    {
      id: '1',
      content: 'First learning',
      category: 'Correction',
      scope: { User: 'test' },
      status: 'active',
      estimated_value: 0.8,
      created_at: '2024-01-15T10:00:00Z',
    },
    {
      id: '2',
      content: 'Second learning',
      category: 'Pattern',
      scope: { Project: 'vibes' },
      status: 'under_review',
      estimated_value: -0.3,
      created_at: '2024-01-14T10:00:00Z',
    },
  ],
  total: 2,
};

const mockDetailData = {
  data_type: 'learning_detail',
  id: '1',
  content: 'First learning',
  category: 'Correction',
  scope: { User: 'test' },
  status: 'active',
  estimated_value: 0.8,
  confidence: 0.9,
  times_injected: 10,
  activation_rate: 0.7,
  session_count: 5,
  created_at: '2024-01-15T10:00:00Z',
  extraction_method: 'explicit_instruction',
};

beforeEach(() => {
  mockFetch.mockReset();
});

describe('DashboardLearnings', () => {
  it('renders loading state initially', () => {
    mockFetch.mockImplementation(() => new Promise(() => {}));

    render(<DashboardLearnings />, { wrapper: createWrapper() });

    expect(screen.getByText('Loading learnings...')).toBeInTheDocument();
  });

  it('renders split panel layout with filters, list, and detail', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(mockLearningsData),
    });

    render(<DashboardLearnings />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('First learning')).toBeInTheDocument();
    });

    // Check filters are present
    expect(screen.getByLabelText('Scope')).toBeInTheDocument();
    expect(screen.getByLabelText('Category')).toBeInTheDocument();
    expect(screen.getByLabelText('Status')).toBeInTheDocument();

    // Check list items
    expect(screen.getByText('First learning')).toBeInTheDocument();
    expect(screen.getByText('Second learning')).toBeInTheDocument();

    // Check empty detail state
    expect(screen.getByText('Select a learning to view details')).toBeInTheDocument();
  });

  it('shows detail when learning is selected', async () => {
    mockFetch
      .mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(mockLearningsData),
      })
      .mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(mockDetailData),
      });

    render(<DashboardLearnings />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('First learning')).toBeInTheDocument();
    });

    // Click on a learning
    fireEvent.click(screen.getByText('First learning'));

    await waitFor(() => {
      // Detail should show metrics
      expect(screen.getByText('Value')).toBeInTheDocument();
      expect(screen.getByText('Confidence')).toBeInTheDocument();
    });
  });

  it('filters learnings when filter changes', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(mockLearningsData),
    });

    render(<DashboardLearnings />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText('First learning')).toBeInTheDocument();
    });

    // Change category filter
    const categorySelect = screen.getByLabelText('Category');
    fireEvent.change(categorySelect, { target: { value: 'Correction' } });

    // Should have called fetch with filter params
    await waitFor(() => {
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('category=Correction')
      );
    });
  });

  it('displays error state when fetch fails', async () => {
    mockFetch.mockResolvedValue({
      ok: false,
      status: 500,
    });

    render(<DashboardLearnings />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(screen.getByText(/Failed to load learnings/)).toBeInTheDocument();
    });
  });

  it('fetches learnings on mount', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(mockLearningsData),
    });

    render(<DashboardLearnings />, { wrapper: createWrapper() });

    await waitFor(() => {
      expect(mockFetch).toHaveBeenCalledWith('/api/groove/dashboard/learnings');
    });
  });
});
