import { describe, test, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import { AssessmentHistory } from './AssessmentHistory';

// Mock fetch globally
const mockFetch = vi.fn();
global.fetch = mockFetch;

describe('AssessmentHistory', () => {
  beforeEach(() => {
    mockFetch.mockReset();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  test('displays loading state initially', () => {
    mockFetch.mockImplementation(() => new Promise(() => {})); // Never resolves

    render(<AssessmentHistory />);

    expect(screen.getByText(/loading/i)).toBeInTheDocument();
  });

  test('displays session list when data loads', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: () =>
        Promise.resolve({
          sessions: [
            { session_id: 'session-alpha', event_count: 15, result_types: ['lightweight', 'medium'] },
            { session_id: 'session-beta', event_count: 8, result_types: ['heavy'] },
          ],
          has_more: false,
        }),
    });

    render(<AssessmentHistory />);

    // Wait for history cards to appear (using event counts which are unique to cards)
    await waitFor(() => {
      expect(screen.getByText('15 events')).toBeInTheDocument();
    });

    expect(screen.getByText('8 events')).toBeInTheDocument();
    // Session IDs appear in both dropdown and cards - use getAllByText
    expect(screen.getAllByText('session-alpha').length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText('session-beta').length).toBeGreaterThanOrEqual(1);
  });

  test('displays result types for each session', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: () =>
        Promise.resolve({
          sessions: [
            { session_id: 'typed-session', event_count: 5, result_types: ['lightweight', 'medium', 'heavy'] },
          ],
          has_more: false,
        }),
    });

    render(<AssessmentHistory />);

    // Wait for the session's event count to appear (unique to card)
    await waitFor(() => {
      expect(screen.getByText('5 events')).toBeInTheDocument();
    });

    // Check result type badges are displayed
    expect(screen.getByText('lightweight')).toBeInTheDocument();
    expect(screen.getByText('medium')).toBeInTheDocument();
    expect(screen.getByText('heavy')).toBeInTheDocument();
  });

  test('displays empty state when no sessions exist', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: () =>
        Promise.resolve({
          sessions: [],
          has_more: false,
        }),
    });

    render(<AssessmentHistory />);

    await waitFor(() => {
      expect(screen.getByText(/no assessment data/i)).toBeInTheDocument();
    });
  });

  test('displays error state when fetch fails', async () => {
    mockFetch.mockRejectedValueOnce(new Error('Network error'));

    render(<AssessmentHistory />);

    await waitFor(() => {
      expect(screen.getByText(/error/i)).toBeInTheDocument();
    });
  });

  test('displays error when server returns 503', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: false,
      status: 503,
      json: () =>
        Promise.resolve({
          error: 'Assessment processor not initialized',
          code: 'NOT_INITIALIZED',
        }),
    });

    render(<AssessmentHistory />);

    await waitFor(() => {
      expect(screen.getByText(/not initialized/i)).toBeInTheDocument();
    });
  });

  test('can filter by session using selector', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: () =>
        Promise.resolve({
          sessions: [
            { session_id: 'filter-session-1', event_count: 10, result_types: ['lightweight'] },
            { session_id: 'filter-session-2', event_count: 20, result_types: ['medium'] },
          ],
          has_more: false,
        }),
    });

    render(<AssessmentHistory />);

    // Wait for event counts to appear (unique to cards)
    await waitFor(() => {
      expect(screen.getByText('10 events')).toBeInTheDocument();
    });

    // Find and use the session selector
    const selector = screen.getByRole('combobox');
    fireEvent.change(selector, { target: { value: 'filter-session-1' } });

    // After selecting, a new fetch should be made with session filter
    await waitFor(() => {
      expect(mockFetch).toHaveBeenCalledTimes(2);
    });

    // Verify the second call included the session filter
    const secondCall = mockFetch.mock.calls[1][0];
    expect(secondCall).toContain('session=filter-session-1');
  });
});
