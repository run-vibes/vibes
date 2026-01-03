// design-system/src/compositions/StreamView/StreamView.tsx
import {
  forwardRef,
  HTMLAttributes,
  useEffect,
  useRef,
  useCallback,
} from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';
import type { DisplayEvent } from '../../events';
import styles from './StreamView.module.css';

export interface StreamViewProps extends HTMLAttributes<HTMLDivElement> {
  events: DisplayEvent[];
  title?: string;
  isLive?: boolean;
  isPaused?: boolean;
  selectedId?: string;
  onEventClick?: (event: DisplayEvent) => void;
  /** Called when user scrolls near the top to load older events */
  onLoadMore?: () => void;
  /** Shows loading indicator at top when fetching older events */
  isLoadingMore?: boolean;
  /** Whether there are more events to load */
  hasMore?: boolean;
  /** Called when following state changes (user scrolls away from bottom) */
  onFollowingChange?: (isFollowing: boolean) => void;
}

const typeToClass: Record<string, string> = {
  SESSION: 'session',
  CLAUDE: 'claude',
  TOOL: 'tool',
  HOOK: 'hook',
  ERROR: 'error',
  ASSESS: 'assess',
};

function formatTime(date: Date): string {
  const time = date.toLocaleTimeString('en-US', {
    hour12: false,
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });
  // Add milliseconds for precise event ordering visibility
  const ms = date.getMilliseconds().toString().padStart(3, '0');
  return `${time}.${ms}`;
}

// Threshold in pixels for triggering load more
const LOAD_MORE_THRESHOLD = 100;
// Threshold in pixels for considering "at bottom" for auto-follow
const AT_BOTTOM_THRESHOLD = 50;
// Estimated row height for virtualization
const ESTIMATED_ROW_HEIGHT = 36;

export const StreamView = forwardRef<HTMLDivElement, StreamViewProps>(
  (
    {
      events,
      title = 'Stream',
      isLive = false,
      isPaused = false,
      selectedId,
      onEventClick,
      onLoadMore,
      isLoadingMore = false,
      hasMore = false,
      onFollowingChange,
      className = '',
      ...props
    },
    ref
  ) => {
    const parentRef = useRef<HTMLDivElement>(null);
    const isFollowingRef = useRef(true);
    const prevEventCountRef = useRef(events.length);
    const classes = [styles.streamView, className].filter(Boolean).join(' ');

    const virtualizer = useVirtualizer({
      count: events.length,
      getScrollElement: () => parentRef.current,
      estimateSize: () => ESTIMATED_ROW_HEIGHT,
      overscan: 10,
      // Initial dimensions allow rendering before measurement (useful for SSR/tests)
      initialRect: { width: 400, height: 500 },
    });

    // Check if at bottom for auto-follow
    const checkIfAtBottom = useCallback(() => {
      const scrollElement = parentRef.current;
      if (!scrollElement) return true;

      const { scrollTop, scrollHeight, clientHeight } = scrollElement;
      const distanceFromBottom = scrollHeight - scrollTop - clientHeight;
      return distanceFromBottom <= AT_BOTTOM_THRESHOLD;
    }, []);

    // Handle scroll events
    const handleScroll = useCallback(() => {
      const scrollElement = parentRef.current;
      if (!scrollElement) return;

      // Check if near top for pagination
      if (
        scrollElement.scrollTop < LOAD_MORE_THRESHOLD &&
        hasMore &&
        !isLoadingMore &&
        onLoadMore
      ) {
        onLoadMore();
      }

      // Update following state
      const isAtBottom = checkIfAtBottom();
      if (isFollowingRef.current !== isAtBottom) {
        isFollowingRef.current = isAtBottom;
        onFollowingChange?.(isAtBottom);
      }
    }, [hasMore, isLoadingMore, onLoadMore, onFollowingChange, checkIfAtBottom]);

    // Auto-scroll to bottom when new events arrive (if following)
    useEffect(() => {
      if (events.length > prevEventCountRef.current && isFollowingRef.current && !isPaused) {
        virtualizer.scrollToIndex(events.length - 1, { align: 'end' });
      }
      prevEventCountRef.current = events.length;
    }, [events.length, isPaused, virtualizer]);

    // Preserve scroll position when older events are prepended
    useEffect(() => {
      const scrollElement = parentRef.current;
      if (!scrollElement) return;

      // If we were loading more and now have more events, we need to
      // adjust scroll position to keep the same events in view
      // This is handled by the virtualizer automatically when items change
    }, [events]);

    return (
      <div ref={ref} className={classes} {...props}>
        <div className={styles.header}>
          <h3 className={styles.title}>{title}</h3>
          <div className={styles.status}>
            {isLive && !isPaused && (
              <span className={styles.liveIndicator}>LIVE</span>
            )}
            {isPaused && <span className={styles.pausedIndicator}>PAUSED</span>}
            <span className={styles.eventCount}>{events.length} events</span>
          </div>
        </div>

        <div
          ref={parentRef}
          className={styles.events}
          onScroll={handleScroll}
        >
          {events.length === 0 ? (
            <div className={styles.empty}>No events yet</div>
          ) : (
            <div
              style={{
                height: `${virtualizer.getTotalSize()}px`,
                width: '100%',
                position: 'relative',
              }}
            >
              {isLoadingMore && (
                <div className={styles.loadingMore}>Loading older events...</div>
              )}
              {virtualizer.getVirtualItems().map((virtualRow) => {
                const event = events[virtualRow.index];
                const typeClass = typeToClass[event.type.toUpperCase()] || '';
                const isSelected = selectedId === event.id;
                const eventClasses = [
                  styles.event,
                  typeClass && styles[typeClass],
                  isSelected && styles.selected,
                ]
                  .filter(Boolean)
                  .join(' ');

                return (
                  <div
                    key={event.id}
                    data-index={virtualRow.index}
                    ref={virtualizer.measureElement}
                    className={eventClasses}
                    style={{
                      position: 'absolute',
                      top: 0,
                      left: 0,
                      width: '100%',
                      transform: `translateY(${virtualRow.start}px)`,
                    }}
                    onClick={() => onEventClick?.(event)}
                  >
                    <span className={styles.timestamp}>
                      {formatTime(event.timestamp)}
                    </span>
                    <span className={styles.type}>{event.type}</span>
                    <span className={styles.summary}>{event.summary}</span>
                  </div>
                );
              })}
              {isLive && !isPaused && (
                <div
                  className={styles.streaming}
                  style={{
                    position: 'absolute',
                    top: 0,
                    left: 0,
                    width: '100%',
                    transform: `translateY(${virtualizer.getTotalSize()}px)`,
                  }}
                >
                  streaming...
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    );
  }
);

StreamView.displayName = 'StreamView';
