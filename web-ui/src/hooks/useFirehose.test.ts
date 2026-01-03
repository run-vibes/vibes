/**
 * Tests for the useFirehose hook
 *
 * Tests cover:
 * - State initialization with proper defaults
 * - Handling events_batch messages (initial and pagination)
 * - Handling live event messages
 * - fetchOlder() pagination requests
 * - setFilters() filter management
 * - isFollowing state transitions
 */
import { describe, it, expect, vi, beforeEach, afterEach, afterAll } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useFirehose } from './useFirehose';

// Mock WebSocket storage
let mockInstances: MockWebSocket[] = [];

// Mock WebSocket
class MockWebSocket {
  static CONNECTING = 0;
  static OPEN = 1;
  static CLOSING = 2;
  static CLOSED = 3;

  url: string;
  readyState: number = MockWebSocket.CONNECTING;
  onopen: ((ev: Event) => void) | null = null;
  onclose: ((ev: CloseEvent) => void) | null = null;
  onmessage: ((ev: MessageEvent) => void) | null = null;
  onerror: ((ev: Event) => void) | null = null;

  sentMessages: string[] = [];

  constructor(url: string) {
    this.url = url;
    mockInstances.push(this);
  }

  send(data: string) {
    this.sentMessages.push(data);
  }

  close() {
    this.readyState = MockWebSocket.CLOSED;
    this.onclose?.(new CloseEvent('close'));
  }

  // Test helpers
  simulateOpen() {
    this.readyState = MockWebSocket.OPEN;
    this.onopen?.(new Event('open'));
  }

  simulateMessage(data: object) {
    this.onmessage?.(new MessageEvent('message', { data: JSON.stringify(data) }));
  }
}

// Store original and replace at module level
const OriginalWebSocket = globalThis.WebSocket;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
(globalThis as any).WebSocket = MockWebSocket;

beforeEach(() => {
  mockInstances = [];
});

afterEach(() => {
  vi.clearAllMocks();
});

// Restore after all tests
afterAll(() => {
  globalThis.WebSocket = OriginalWebSocket;
});

function getLastMockWs(): MockWebSocket {
  return mockInstances[mockInstances.length - 1];
}

describe('useFirehose', () => {
  describe('state initialization', () => {
    it('initializes with empty events array', () => {
      const { result } = renderHook(() => useFirehose({ autoConnect: false }));
      expect(result.current.events).toEqual([]);
    });

    it('initializes with null cursors', () => {
      const { result } = renderHook(() => useFirehose({ autoConnect: false }));
      expect(result.current.oldestEventId).toBeNull();
      expect(result.current.newestOffset).toBeNull();
    });

    it('initializes with isFollowing true', () => {
      const { result } = renderHook(() => useFirehose({ autoConnect: false }));
      expect(result.current.isFollowing).toBe(true);
    });

    it('initializes with hasMore false', () => {
      const { result } = renderHook(() => useFirehose({ autoConnect: false }));
      expect(result.current.hasMore).toBe(false);
    });

    it('initializes with isLoadingOlder false', () => {
      const { result } = renderHook(() => useFirehose({ autoConnect: false }));
      expect(result.current.isLoadingOlder).toBe(false);
    });

    it('initializes with empty filters', () => {
      const { result } = renderHook(() => useFirehose({ autoConnect: false }));
      expect(result.current.filters.types).toBeNull();
      expect(result.current.filters.sessionId).toBeNull();
    });
  });

  describe('connection', () => {
    it('connects automatically by default', () => {
      renderHook(() => useFirehose());
      expect(mockInstances.length).toBe(1);
    });

    it('does not auto-connect when autoConnect is false', () => {
      renderHook(() => useFirehose({ autoConnect: false }));
      expect(mockInstances.length).toBe(0);
    });

    it('sets isConnected to true on open', async () => {
      const { result } = renderHook(() => useFirehose());
      const ws = getLastMockWs();

      act(() => {
        ws.simulateOpen();
      });

      expect(result.current.isConnected).toBe(true);
    });

    it('sets isConnected to false on close', async () => {
      const { result } = renderHook(() => useFirehose());
      const ws = getLastMockWs();

      act(() => {
        ws.simulateOpen();
      });

      act(() => {
        ws.close();
      });

      expect(result.current.isConnected).toBe(false);
    });
  });

  describe('events_batch handling', () => {
    it('populates events from initial events_batch', async () => {
      const { result } = renderHook(() => useFirehose());
      const ws = getLastMockWs();

      act(() => {
        ws.simulateOpen();
        ws.simulateMessage({
          type: 'events_batch',
          events: [
            { event_id: 'evt-1', offset: 10, event: { type: 'client_connected', client_id: 'c1' } },
            { event_id: 'evt-2', offset: 11, event: { type: 'client_connected', client_id: 'c2' } },
          ],
          oldest_event_id: 'evt-1',
          has_more: true,
        });
      });

      expect(result.current.events).toHaveLength(2);
      expect(result.current.events[0].offset).toBe(10);
      expect(result.current.events[1].offset).toBe(11);
    });

    it('updates oldestEventId from events_batch', async () => {
      const { result } = renderHook(() => useFirehose());
      const ws = getLastMockWs();

      act(() => {
        ws.simulateOpen();
        ws.simulateMessage({
          type: 'events_batch',
          events: [
            { event_id: 'evt-1', offset: 10, event: { type: 'client_connected', client_id: 'c1' } },
          ],
          oldest_event_id: 'evt-1',
          has_more: true,
        });
      });

      expect(result.current.oldestEventId).toBe('evt-1');
    });

    it('updates newestOffset from events_batch', async () => {
      const { result } = renderHook(() => useFirehose());
      const ws = getLastMockWs();

      act(() => {
        ws.simulateOpen();
        ws.simulateMessage({
          type: 'events_batch',
          events: [
            { event_id: 'evt-1', offset: 10, event: { type: 'client_connected', client_id: 'c1' } },
            { event_id: 'evt-2', offset: 15, event: { type: 'client_connected', client_id: 'c2' } },
          ],
          oldest_event_id: 'evt-1',
          has_more: true,
        });
      });

      expect(result.current.newestOffset).toBe(15);
    });

    it('updates hasMore from events_batch', async () => {
      const { result } = renderHook(() => useFirehose());
      const ws = getLastMockWs();

      act(() => {
        ws.simulateOpen();
        ws.simulateMessage({
          type: 'events_batch',
          events: [],
          oldest_event_id: null,
          has_more: true,
        });
      });

      expect(result.current.hasMore).toBe(true);
    });

    it('prepends events when fetching older', async () => {
      const { result } = renderHook(() => useFirehose());
      const ws = getLastMockWs();

      // Initial batch
      act(() => {
        ws.simulateOpen();
        ws.simulateMessage({
          type: 'events_batch',
          events: [
            { event_id: 'evt-50', offset: 50, event: { type: 'client_connected', client_id: 'c1' } },
          ],
          oldest_event_id: 'evt-50',
          has_more: true,
        });
      });

      // Fetch older - request
      act(() => {
        result.current.fetchOlder();
      });

      // Older batch response
      act(() => {
        ws.simulateMessage({
          type: 'events_batch',
          events: [
            { event_id: 'evt-40', offset: 40, event: { type: 'client_connected', client_id: 'older' } },
          ],
          oldest_event_id: 'evt-40',
          has_more: true,
        });
      });

      // Older events should be prepended
      expect(result.current.events).toHaveLength(2);
      expect(result.current.events[0].offset).toBe(40);
      expect(result.current.events[1].offset).toBe(50);
    });
  });

  describe('live event handling', () => {
    it('appends live events to end of list', async () => {
      const { result } = renderHook(() => useFirehose());
      const ws = getLastMockWs();

      act(() => {
        ws.simulateOpen();
        ws.simulateMessage({
          type: 'events_batch',
          events: [
            { event_id: 'evt-10', offset: 10, event: { type: 'client_connected', client_id: 'c1' } },
          ],
          oldest_event_id: 'evt-10',
          has_more: false,
        });
      });

      act(() => {
        ws.simulateMessage({
          type: 'event',
          event_id: 'evt-11',
          offset: 11,
          event: { type: 'client_connected', client_id: 'new' },
        });
      });

      expect(result.current.events).toHaveLength(2);
      expect(result.current.events[1].offset).toBe(11);
    });

    it('updates newestOffset on live event', async () => {
      const { result } = renderHook(() => useFirehose());
      const ws = getLastMockWs();

      act(() => {
        ws.simulateOpen();
        ws.simulateMessage({
          type: 'event',
          event_id: 'evt-100',
          offset: 100,
          event: { type: 'client_connected', client_id: 'c1' },
        });
      });

      expect(result.current.newestOffset).toBe(100);
    });
  });

  describe('fetchOlder', () => {
    it('sends fetch_older message with correct before_event_id', async () => {
      const { result } = renderHook(() => useFirehose());
      const ws = getLastMockWs();

      act(() => {
        ws.simulateOpen();
        ws.simulateMessage({
          type: 'events_batch',
          events: [
            { event_id: 'evt-50', offset: 50, event: { type: 'client_connected', client_id: 'c1' } },
          ],
          oldest_event_id: 'evt-50',
          has_more: true,
        });
      });

      act(() => {
        result.current.fetchOlder();
      });

      const sent = JSON.parse(ws.sentMessages[0]);
      expect(sent.type).toBe('fetch_older');
      expect(sent.before_event_id).toBe('evt-50');
    });

    it('sets isLoadingOlder while waiting', async () => {
      const { result } = renderHook(() => useFirehose());
      const ws = getLastMockWs();

      act(() => {
        ws.simulateOpen();
        ws.simulateMessage({
          type: 'events_batch',
          events: [
            { event_id: 'evt-50', offset: 50, event: { type: 'client_connected', client_id: 'c1' } },
          ],
          oldest_event_id: 'evt-50',
          has_more: true,
        });
      });

      act(() => {
        result.current.fetchOlder();
      });

      expect(result.current.isLoadingOlder).toBe(true);

      // Response clears loading
      act(() => {
        ws.simulateMessage({
          type: 'events_batch',
          events: [],
          oldest_event_id: 'evt-40',
          has_more: false,
        });
      });

      expect(result.current.isLoadingOlder).toBe(false);
    });

    it('does not send duplicate requests while loading', async () => {
      const { result } = renderHook(() => useFirehose());
      const ws = getLastMockWs();

      act(() => {
        ws.simulateOpen();
        ws.simulateMessage({
          type: 'events_batch',
          events: [
            { event_id: 'evt-50', offset: 50, event: { type: 'client_connected', client_id: 'c1' } },
          ],
          oldest_event_id: 'evt-50',
          has_more: true,
        });
      });

      act(() => {
        result.current.fetchOlder();
        result.current.fetchOlder(); // duplicate
        result.current.fetchOlder(); // duplicate
      });

      expect(ws.sentMessages).toHaveLength(1);
    });

    it('does not send request when hasMore is false', async () => {
      const { result } = renderHook(() => useFirehose());
      const ws = getLastMockWs();

      act(() => {
        ws.simulateOpen();
        ws.simulateMessage({
          type: 'events_batch',
          events: [
            { event_id: 'evt-50', offset: 50, event: { type: 'client_connected', client_id: 'c1' } },
          ],
          oldest_event_id: 'evt-50',
          has_more: false, // No more history
        });
      });

      act(() => {
        result.current.fetchOlder();
      });

      expect(ws.sentMessages).toHaveLength(0);
    });
  });

  describe('setFilters', () => {
    it('sends set_filters message', async () => {
      const { result } = renderHook(() => useFirehose());
      const ws = getLastMockWs();

      act(() => {
        ws.simulateOpen();
      });

      act(() => {
        result.current.setFilters({ types: ['Claude', 'Hook'] });
      });

      const sent = JSON.parse(ws.sentMessages[0]);
      expect(sent.type).toBe('set_filters');
      expect(sent.types).toEqual(['Claude', 'Hook']);
    });

    it('has stable reference after being called (prevents infinite loops in useEffect)', async () => {
      // CRITICAL: setFilters must have a stable reference so it can be safely used
      // as a dependency in useEffect without causing infinite re-renders.
      // Bug regression test: if setFilters depends on `filters` state, calling it
      // would update filters → recreate setFilters → trigger useEffect → infinite loop
      const { result } = renderHook(() => useFirehose());
      const ws = getLastMockWs();

      act(() => {
        ws.simulateOpen();
      });

      // Capture reference before calling
      const setFiltersBefore = result.current.setFilters;

      act(() => {
        result.current.setFilters({ types: ['Claude'] });
      });

      // Reference should be the same after calling (stable callback)
      expect(result.current.setFilters).toBe(setFiltersBefore);

      // Call again with different filters
      act(() => {
        result.current.setFilters({ sessionId: 'sess-123' });
      });

      // Still the same reference
      expect(result.current.setFilters).toBe(setFiltersBefore);
    });

    it('updates local filter state', async () => {
      const { result } = renderHook(() => useFirehose());
      const ws = getLastMockWs();

      act(() => {
        ws.simulateOpen();
      });

      act(() => {
        result.current.setFilters({ types: ['Claude'], sessionId: 'sess-1' });
      });

      expect(result.current.filters.types).toEqual(['Claude']);
      expect(result.current.filters.sessionId).toBe('sess-1');
    });

    it('clears events on filter change', async () => {
      const { result } = renderHook(() => useFirehose());
      const ws = getLastMockWs();

      act(() => {
        ws.simulateOpen();
        ws.simulateMessage({
          type: 'events_batch',
          events: [
            { event_id: 'evt-10', offset: 10, event: { type: 'client_connected', client_id: 'c1' } },
          ],
          oldest_event_id: 'evt-10',
          has_more: false,
        });
      });

      expect(result.current.events).toHaveLength(1);

      act(() => {
        result.current.setFilters({ types: ['Claude'] });
      });

      // Events should be cleared, waiting for new batch from server
      expect(result.current.events).toEqual([]);
    });

    it('sets isFollowing to true on filter change', async () => {
      const { result } = renderHook(() => useFirehose());
      const ws = getLastMockWs();

      act(() => {
        ws.simulateOpen();
      });

      act(() => {
        result.current.setIsFollowing(false);
      });

      expect(result.current.isFollowing).toBe(false);

      act(() => {
        result.current.setFilters({ types: ['Claude'] });
      });

      expect(result.current.isFollowing).toBe(true);
    });
  });

  describe('isFollowing state', () => {
    it('can be toggled via setIsFollowing', async () => {
      const { result } = renderHook(() => useFirehose({ autoConnect: false }));

      expect(result.current.isFollowing).toBe(true);

      act(() => {
        result.current.setIsFollowing(false);
      });

      expect(result.current.isFollowing).toBe(false);

      act(() => {
        result.current.setIsFollowing(true);
      });

      expect(result.current.isFollowing).toBe(true);
    });
  });

  describe('disconnect/reconnect', () => {
    it('clears state on disconnect', async () => {
      const { result } = renderHook(() => useFirehose());
      const ws = getLastMockWs();

      act(() => {
        ws.simulateOpen();
        ws.simulateMessage({
          type: 'events_batch',
          events: [
            { event_id: 'evt-10', offset: 10, event: { type: 'client_connected', client_id: 'c1' } },
          ],
          oldest_event_id: 'evt-10',
          has_more: true,
        });
      });

      expect(result.current.events).toHaveLength(1);

      act(() => {
        result.current.disconnect();
      });

      expect(result.current.isConnected).toBe(false);
    });
  });
});
