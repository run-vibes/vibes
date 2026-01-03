import { describe, test, expect, vi, beforeAll, afterAll } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { FirehosePage } from './Firehose';

// Mock ResizeObserver for virtualization - deferred callback to avoid sync issues
class MockResizeObserver {
  callback: ResizeObserverCallback;
  constructor(callback: ResizeObserverCallback) {
    this.callback = callback;
  }
  observe(target: Element) {
    // Defer callback to next tick to avoid sync issues with virtualizer
    setTimeout(() => {
      this.callback([{
        target,
        contentRect: { width: 400, height: 500, x: 0, y: 0, top: 0, left: 0, bottom: 500, right: 400 } as DOMRectReadOnly,
        borderBoxSize: [{ blockSize: 500, inlineSize: 400 }],
        contentBoxSize: [{ blockSize: 500, inlineSize: 400 }],
        devicePixelContentBoxSize: [{ blockSize: 500, inlineSize: 400 }],
      }], this);
    }, 0);
  }
  unobserve() {}
  disconnect() {}
}

// Mock clipboard API
Object.assign(navigator, {
  clipboard: {
    writeText: vi.fn(),
  },
});

// Mock useFirehose hook
const mockEvents = [
  {
    event_id: '01936f8a-0001-7000-8000-000000000001',
    offset: 0,
    event: { type: 'session_created', session_id: 'sess-1', name: 'auth-refactor' },
  },
  {
    event_id: '01936f8a-0002-7000-8000-000000000002',
    offset: 1,
    // Note: summarizeClaudeEvent checks e.delta, not e.text
    event: { type: 'claude', session_id: 'sess-1', event: { type: 'text_delta', delta: 'Let me analyze the code...' } },
  },
  {
    event_id: '01936f8a-0003-7000-8000-000000000003',
    offset: 2,
    event: { type: 'hook', session_id: 'sess-1', event: { type: 'pre_tool_use', tool_name: 'Read' } },
  },
  {
    event_id: '01936f8a-0004-7000-8000-000000000004',
    offset: 3,
    event: { type: 'client_connected', client_id: 'client-abc' },
  },
];

vi.mock('../hooks/useFirehose', () => ({
  useFirehose: () => ({
    events: mockEvents,
    isConnected: true,
    isFollowing: true,
    isLoadingOlder: false,
    hasMore: false,
    error: null,
    fetchOlder: vi.fn(),
    setFilters: vi.fn(),
    setIsFollowing: vi.fn(),
  }),
}));

describe('FirehosePage', () => {
  beforeAll(() => {
    global.ResizeObserver = MockResizeObserver as unknown as typeof ResizeObserver;

    Object.defineProperty(HTMLElement.prototype, 'clientHeight', {
      configurable: true,
      get() { return 500; }
    });
    Object.defineProperty(HTMLElement.prototype, 'scrollHeight', {
      configurable: true,
      get() { return 1000; }
    });
    Object.defineProperty(HTMLElement.prototype, 'scrollTop', {
      configurable: true,
      get() { return 0; },
      set() {}
    });
    Object.defineProperty(HTMLElement.prototype, 'offsetHeight', {
      configurable: true,
      get() { return 36; }
    });

    Element.prototype.getBoundingClientRect = () => ({
      width: 400,
      height: 500,
      top: 0,
      left: 0,
      bottom: 500,
      right: 400,
      x: 0,
      y: 0,
      toJSON: () => ({})
    });
  });

  afterAll(() => {
    delete (global as Record<string, unknown>).ResizeObserver;
  });

  test('search input filters events by content', async () => {
    render(<FirehosePage />);

    // Wait for events to render
    await waitFor(() => {
      expect(screen.getByText(/auth-refactor/)).toBeInTheDocument();
    });

    // Find the search input (should have placeholder like "Search...")
    const searchInput = screen.getByPlaceholderText(/search/i);
    expect(searchInput).toBeInTheDocument();

    // Type a search query that should match only the claude event
    fireEvent.change(searchInput, { target: { value: 'analyze' } });

    // The claude event mentioning "analyze" should still be visible
    await waitFor(() => {
      expect(screen.getByText(/Let me analyze/)).toBeInTheDocument();
    });

    // The session_created event should be filtered out (doesn't contain "analyze")
    expect(screen.queryByText(/auth-refactor/)).not.toBeInTheDocument();
  });

  test('search is case-insensitive', async () => {
    render(<FirehosePage />);

    await waitFor(() => {
      expect(screen.getByText(/auth-refactor/)).toBeInTheDocument();
    });

    const searchInput = screen.getByPlaceholderText(/search/i);
    fireEvent.change(searchInput, { target: { value: 'AUTH' } });

    // Should still find auth-refactor even with uppercase search
    await waitFor(() => {
      expect(screen.getByText(/auth-refactor/)).toBeInTheDocument();
    });
  });

  test('empty search shows all events', async () => {
    render(<FirehosePage />);

    await waitFor(() => {
      expect(screen.getByText(/auth-refactor/)).toBeInTheDocument();
    });

    const searchInput = screen.getByPlaceholderText(/search/i);

    // Type something then clear
    fireEvent.change(searchInput, { target: { value: 'xyz' } });
    fireEvent.change(searchInput, { target: { value: '' } });

    // All events should be visible again
    await waitFor(() => {
      expect(screen.getByText(/auth-refactor/)).toBeInTheDocument();
      expect(screen.getByText(/Let me analyze/)).toBeInTheDocument();
    });
  });
});
