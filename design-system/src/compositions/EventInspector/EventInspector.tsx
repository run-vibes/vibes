// design-system/src/compositions/EventInspector/EventInspector.tsx
import { forwardRef, HTMLAttributes } from 'react';
import { Button } from '../../primitives/Button';
import type { DisplayEvent, ContextEvent } from '../../events';
import styles from './EventInspector.module.css';

export interface EventInspectorProps extends HTMLAttributes<HTMLDivElement> {
  event: DisplayEvent | null;
  contextEvents?: ContextEvent[];
  onCopyJson?: () => void;
  onClose?: () => void;
}

const typeToClass: Record<string, string> = {
  SESSION: 'session',
  CLAUDE: 'claude',
  TOOL: 'tool',
  HOOK: 'hook',
  ERROR: 'error',
  ASSESS: 'assess',
};

function formatTimestamp(date: Date): string {
  return date.toISOString().replace('T', ' ').replace('Z', '');
}

function formatTime(date: Date): string {
  return date.toLocaleTimeString('en-US', {
    hour12: false,
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });
}

export const EventInspector = forwardRef<HTMLDivElement, EventInspectorProps>(
  ({ event, contextEvents = [], onCopyJson, onClose, className = '', ...props }, ref) => {
    const classes = [styles.inspector, className].filter(Boolean).join(' ');

    if (!event) {
      return (
        <div ref={ref} className={classes} {...props}>
          <div className={styles.header}>
            <h3 className={styles.title}>Event Inspector</h3>
          </div>
          <div className={styles.empty}>No event selected</div>
        </div>
      );
    }

    const typeClass = typeToClass[event.type.toUpperCase()] || '';
    const payloadJson = event.payload
      ? JSON.stringify(event.payload, null, 2)
      : null;

    return (
      <div ref={ref} className={classes} {...props}>
        <div className={styles.header}>
          <h3 className={styles.title}>Event Inspector</h3>
          <div className={styles.actions}>
            {onCopyJson && (
              <Button size="sm" variant="ghost" onClick={onCopyJson}>
                Copy JSON
              </Button>
            )}
            {onClose && (
              <Button size="sm" variant="ghost" onClick={onClose}>
                ×
              </Button>
            )}
          </div>
        </div>

        <div className={styles.content}>
          <div className={styles.section}>
            <div className={styles.sectionTitle}>Metadata</div>
            <div className={styles.metadata}>
              <span className={styles.label}>Event ID:</span>
              <span className={styles.value}>{event.id}</span>

              <span className={styles.label}>Timestamp:</span>
              <span className={styles.value}>{formatTimestamp(event.timestamp)}</span>

              <span className={styles.label}>Type:</span>
              <span className={`${styles.value} ${styles.typeValue} ${typeClass ? styles[typeClass] : ''}`}>
                {event.type}
              </span>

              {event.session && (
                <>
                  <span className={styles.label}>Session:</span>
                  <span className={styles.value}>
                    {event.session}
                    {event.sessionName && ` (${event.sessionName})`}
                  </span>
                </>
              )}

              {event.offset !== undefined && (
                <>
                  <span className={styles.label}>Offset:</span>
                  <span className={styles.value}>{event.offset.toLocaleString()}</span>
                </>
              )}
            </div>
          </div>

          {payloadJson && (
            <div className={styles.section}>
              <div className={styles.sectionTitle}>Payload</div>
              <div className={styles.payload}>
                <pre className={styles.json}>{payloadJson}</pre>
              </div>
            </div>
          )}

          {contextEvents.length > 0 && (
            <div className={styles.section}>
              <div className={styles.sectionTitle}>Context (events before/after)</div>
              <div className={styles.contextEvents}>
                {contextEvents.map((ctx, i) => {
                  const ctxTypeClass = typeToClass[ctx.type.toUpperCase()] || '';
                  const isCurrent = ctx.relativePosition === 0;

                  return (
                    <div
                      key={i}
                      className={`${styles.contextEvent} ${isCurrent ? styles.currentEvent : ''}`}
                    >
                      <span className={styles.contextOffset}>
                        {ctx.relativePosition === 0 ? '►' : ctx.relativePosition > 0 ? `+${ctx.relativePosition}` : ctx.relativePosition}
                      </span>
                      <span className={styles.contextTime}>{formatTime(ctx.timestamp)}</span>
                      <span className={`${styles.contextType} ${ctxTypeClass ? styles[ctxTypeClass] : ''}`}>
                        {ctx.type}
                      </span>
                      <span className={styles.contextSummary}>
                        {ctx.summary}
                        {isCurrent && <span className={styles.currentMarker}> ◄</span>}
                      </span>
                    </div>
                  );
                })}
              </div>
            </div>
          )}
        </div>
      </div>
    );
  }
);

EventInspector.displayName = 'EventInspector';
