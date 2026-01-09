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

  test('displays pagination controls when response has multiple pages', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: () =>
        Promise.resolve({
          sessions: [
            { session_id: 'page-session-1', event_count: 5, result_types: ['lightweight'] },
          ],
          has_more: true,
          page: 1,
          per_page: 20,
          total: 45,
          total_pages: 3,
        }),
    });

    render(<AssessmentHistory />);

    await waitFor(() => {
      expect(screen.getByText('5 events')).toBeInTheDocument();
    });

    // Should display page info
    expect(screen.getByText(/page 1 of 3/i)).toBeInTheDocument();

    // Should have next button enabled
    const nextButton = screen.getByRole('button', { name: /next/i });
    expect(nextButton).not.toBeDisabled();

    // Should have previous button disabled on first page
    const prevButton = screen.getByRole('button', { name: /prev/i });
    expect(prevButton).toBeDisabled();
  });

  test('clicking next page fetches next page of results', async () => {
    // First page response
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: () =>
        Promise.resolve({
          sessions: [
            { session_id: 'page1-session', event_count: 10, result_types: ['lightweight'] },
          ],
          has_more: true,
          page: 1,
          per_page: 20,
          total: 40,
          total_pages: 2,
        }),
    });

    render(<AssessmentHistory />);

    await waitFor(() => {
      expect(screen.getByText('10 events')).toBeInTheDocument();
    });

    // Mock second page response
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: () =>
        Promise.resolve({
          sessions: [
            { session_id: 'page2-session', event_count: 20, result_types: ['medium'] },
          ],
          has_more: false,
          page: 2,
          per_page: 20,
          total: 40,
          total_pages: 2,
        }),
    });

    // Click next button
    const nextButton = screen.getByRole('button', { name: /next/i });
    fireEvent.click(nextButton);

    // Verify fetch was called with page=2
    await waitFor(() => {
      expect(mockFetch).toHaveBeenCalledTimes(2);
    });

    const secondCall = mockFetch.mock.calls[1][0];
    expect(secondCall).toContain('page=2');
  });

  test('hides pagination controls when only one page exists', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: () =>
        Promise.resolve({
          sessions: [
            { session_id: 'single-page-session', event_count: 3, result_types: ['lightweight'] },
          ],
          has_more: false,
          page: 1,
          per_page: 20,
          total: 1,
          total_pages: 1,
        }),
    });

    render(<AssessmentHistory />);

    await waitFor(() => {
      expect(screen.getByText('3 events')).toBeInTheDocument();
    });

    // Pagination controls should not be visible for single page
    expect(screen.queryByRole('button', { name: /next/i })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /prev/i })).not.toBeInTheDocument();
  });
});
