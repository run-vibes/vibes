// design-system/src/compositions/StreamView/StreamView.tsx
import { forwardRef, HTMLAttributes, useEffect, useRef } from 'react';
import type { DisplayEvent } from '../../events';
import styles from './StreamView.module.css';

export interface StreamViewProps extends HTMLAttributes<HTMLDivElement> {
  events: DisplayEvent[];
  title?: string;
  isLive?: boolean;
  isPaused?: boolean;
  autoScroll?: boolean;
  selectedId?: string;
  onEventClick?: (event: DisplayEvent) => void;
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
  return date.toLocaleTimeString('en-US', {
    hour12: false,
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });
}

export const StreamView = forwardRef<HTMLDivElement, StreamViewProps>(
  (
    {
      events,
      title = 'Stream',
      isLive = false,
      isPaused = false,
      autoScroll = true,
      selectedId,
      onEventClick,
      className = '',
      ...props
    },
    ref
  ) => {
    const eventsRef = useRef<HTMLDivElement>(null);
    const classes = [styles.streamView, className].filter(Boolean).join(' ');

    // Auto-scroll to bottom when new events arrive
    useEffect(() => {
      if (autoScroll && !isPaused && eventsRef.current) {
        eventsRef.current.scrollTop = eventsRef.current.scrollHeight;
      }
    }, [events.length, autoScroll, isPaused]);

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

        <div ref={eventsRef} className={styles.events}>
          {events.length === 0 ? (
            <div className={styles.empty}>No events yet</div>
          ) : (
            <>
              {events.map((event) => {
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
                    className={eventClasses}
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
                <div className={styles.streaming}>streaming...</div>
              )}
            </>
          )}
        </div>
      </div>
    );
  }
);

StreamView.displayName = 'StreamView';
