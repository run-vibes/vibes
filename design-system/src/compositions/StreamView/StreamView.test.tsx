// design-system/src/compositions/StreamView/StreamView.test.tsx
import { describe, it, expect, vi, beforeAll, afterAll } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { StreamView } from './StreamView';

// Mock ResizeObserver for virtualization
class MockResizeObserver {
  callback: ResizeObserverCallback;
  constructor(callback: ResizeObserverCallback) {
    this.callback = callback;
  }
  observe(target: Element) {
    // Immediately call back with mock dimensions
    this.callback([{
      target,
      contentRect: { width: 400, height: 500, x: 0, y: 0, top: 0, left: 0, bottom: 500, right: 400 } as DOMRectReadOnly,
      borderBoxSize: [{ blockSize: 500, inlineSize: 400 }],
      contentBoxSize: [{ blockSize: 500, inlineSize: 400 }],
      devicePixelContentBoxSize: [{ blockSize: 500, inlineSize: 400 }],
    }], this);
  }
  unobserve() {}
  disconnect() {}
}

describe('StreamView', () => {
  const mockEvents = [
    { id: '1', timestamp: new Date('2024-12-31T14:32:01Z'), type: 'SESSION', session: 'sess-abc', summary: 'Created "auth-refactor"' },
    { id: '2', timestamp: new Date('2024-12-31T14:32:02Z'), type: 'CLAUDE', session: 'sess-abc', summary: 'Let me analyze...' },
    { id: '3', timestamp: new Date('2024-12-31T14:32:03Z'), type: 'TOOL', session: 'sess-abc', summary: 'Read src/lib.rs' },
    { id: '4', timestamp: new Date('2024-12-31T14:32:04Z'), type: 'ERROR', session: 'sess-abc', summary: 'Permission denied' },
  ];

  beforeAll(() => {
    // Mock ResizeObserver for @tanstack/react-virtual
    global.ResizeObserver = MockResizeObserver as unknown as typeof ResizeObserver;

    // Mock element dimensions for virtualization
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
      get() { return 36; } // Match ESTIMATED_ROW_HEIGHT
    });

    // Mock getBoundingClientRect for element measurements
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
    // Clean up mocks
    delete (global as Record<string, unknown>).ResizeObserver;
  });

  it('renders events', async () => {
    render(<StreamView events={mockEvents} />);
    await waitFor(() => {
      expect(screen.getByText(/Created "auth-refactor"/)).toBeInTheDocument();
    });
    expect(screen.getByText(/Let me analyze/)).toBeInTheDocument();
  });

  it('renders empty state when no events', () => {
    render(<StreamView events={[]} />);
    expect(screen.getByText(/No events/i)).toBeInTheDocument();
  });

  it('applies event type classes', async () => {
    render(<StreamView events={mockEvents} />);
    await waitFor(() => {
      expect(screen.getByText(/Permission denied/)).toBeInTheDocument();
    });
    const errorEvent = screen.getByText(/Permission denied/).closest('[class*="event"]');
    expect(errorEvent?.className).toMatch(/error/);
  });

  it('calls onEventClick when event is clicked', async () => {
    const onClick = vi.fn();
    render(<StreamView events={mockEvents} onEventClick={onClick} />);
    await waitFor(() => {
      expect(screen.getByText(/Created "auth-refactor"/)).toBeInTheDocument();
    });
    fireEvent.click(screen.getByText(/Created "auth-refactor"/));
    expect(onClick).toHaveBeenCalledWith(mockEvents[0]);
  });

  it('shows live indicator when isLive', () => {
    render(<StreamView events={mockEvents} isLive />);
    expect(screen.getByText(/LIVE/i)).toBeInTheDocument();
  });

  it('shows paused indicator when isPaused', () => {
    render(<StreamView events={mockEvents} isPaused />);
    expect(screen.getByText(/PAUSED/i)).toBeInTheDocument();
  });

  it('scrolls to bottom when isPaused transitions from true to false (Jump to latest)', async () => {
    // This tests the "Jump to latest" behavior:
    // When isPaused goes from true to false, the component should:
    // 1. Scroll to the last event
    // 2. Reset internal isFollowing state to true
    // 3. Notify parent via onFollowingChange(true)
    const onFollowingChange = vi.fn();

    const { rerender } = render(
      <StreamView
        events={mockEvents}
        isLive
        isPaused={true}
        onFollowingChange={onFollowingChange}
      />
    );

    // Clear any initial calls
    onFollowingChange.mockClear();

    // Transition to not paused (simulates clicking "Jump to latest")
    rerender(
      <StreamView
        events={mockEvents}
        isLive
        isPaused={false}
        onFollowingChange={onFollowingChange}
      />
    );

    // The component should notify that we're now following (scrolled to bottom)
    await waitFor(() => {
      expect(onFollowingChange).toHaveBeenCalledWith(true);
    });
  });
});
