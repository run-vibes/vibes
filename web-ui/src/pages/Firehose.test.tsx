import { describe, test, expect, vi, beforeAll, afterAll } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { FirehosePage } from './Firehose';

// Mock ResizeObserver for virtualization - use large dimensions to render all items
class MockResizeObserver {
  callback: ResizeObserverCallback;
  constructor(callback: ResizeObserverCallback) {
    this.callback = callback;
  }
  observe(target: Element) {
    // Defer callback to next tick to avoid sync issues with virtualizer
    // Use large dimensions to ensure all test events render
    setTimeout(() => {
      this.callback([{
        target,
        contentRect: { width: 800, height: 2000, x: 0, y: 0, top: 0, left: 0, bottom: 2000, right: 800 } as DOMRectReadOnly,
        borderBoxSize: [{ blockSize: 2000, inlineSize: 800 }],
        contentBoxSize: [{ blockSize: 2000, inlineSize: 800 }],
        devicePixelContentBoxSize: [{ blockSize: 2000, inlineSize: 800 }],
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
      get() { return 2000; }
    });
    Object.defineProperty(HTMLElement.prototype, 'scrollHeight', {
      configurable: true,
      get() { return 2000; }
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
      width: 800,
      height: 2000,
      top: 0,
      left: 0,
      bottom: 2000,
      right: 800,
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

    // Wait for events to render - use getAllByText since name appears in both
    // sessions sidebar and event stream
    await waitFor(() => {
      expect(screen.getAllByText(/auth-refactor/).length).toBeGreaterThan(0);
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

    // The session_created event summary should be filtered out
    // Note: session sidebar still shows session name, but stream filters out
    expect(screen.queryByText(/Created "auth-refactor"/)).not.toBeInTheDocument();
  });

  test('search is case-insensitive', async () => {
    render(<FirehosePage />);

    // Wait for events to render
    await waitFor(() => {
      expect(screen.getAllByText(/auth-refactor/).length).toBeGreaterThan(0);
    });

    const searchInput = screen.getByPlaceholderText(/search/i);
    fireEvent.change(searchInput, { target: { value: 'AUTH' } });

    // Should still find auth-refactor in stream even with uppercase search
    await waitFor(() => {
      expect(screen.getByText(/Created "auth-refactor"/)).toBeInTheDocument();
    });
  });

  test('empty search shows all events', async () => {
    render(<FirehosePage />);

    // Wait for events to render
    await waitFor(() => {
      expect(screen.getAllByText(/auth-refactor/).length).toBeGreaterThan(0);
    });

    const searchInput = screen.getByPlaceholderText(/search/i);

    // Type something then clear
    fireEvent.change(searchInput, { target: { value: 'xyz' } });
    fireEvent.change(searchInput, { target: { value: '' } });

    // All events in stream should be visible again
    await waitFor(() => {
      expect(screen.getByText(/Created "auth-refactor"/)).toBeInTheDocument();
      expect(screen.getByText(/Let me analyze/)).toBeInTheDocument();
    });
  });
});
