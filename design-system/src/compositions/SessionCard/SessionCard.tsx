import { forwardRef, HTMLAttributes, MouseEvent, ReactNode } from 'react';
import { Badge } from '../../primitives/Badge';
import styles from './SessionCard.module.css';

export type SessionStatus = 'idle' | 'processing' | 'waiting' | 'finished' | 'failed';

export interface SessionAction {
  icon: ReactNode;
  label: string;
  onClick: (e: MouseEvent<HTMLButtonElement>) => void;
}

export interface SessionCardProps extends HTMLAttributes<HTMLElement> {
  id: string;
  name?: string;
  status: SessionStatus;
  subscribers?: number;
  updatedAt: Date;
  /** Duration of the session in seconds */
  duration?: number;
  /** Number of events in the session */
  eventCount?: number;
  /** Quick action buttons shown on hover */
  actions?: SessionAction[];
  /** URL to navigate to - renders as anchor when provided */
  href?: string;
}

const statusMap = {
  idle: 'idle',
  processing: 'accent',
  waiting: 'warning',
  finished: 'success',
  failed: 'error',
} as const;

/** Check if status indicates an active session */
function isActiveStatus(status: SessionStatus): boolean {
  return status === 'processing' || status === 'waiting';
}

export const SessionCard = forwardRef<HTMLElement, SessionCardProps>(
  (
    {
      id,
      name,
      status,
      subscribers = 0,
      updatedAt,
      duration,
      eventCount,
      actions,
      href,
      className = '',
      onClick,
      ...props
    },
    ref
  ) => {
    const isActive = isActiveStatus(status);
    const classes = [
      styles.card,
      isActive ? styles.active : styles.inactive,
      className,
    ]
      .filter(Boolean)
      .join(' ');

    const timeAgo = formatTimeAgo(updatedAt);
    const durationStr = duration !== undefined ? formatDuration(duration) : undefined;

    const Tag = href ? 'a' : 'article';

    return (
      <Tag ref={ref as React.Ref<HTMLAnchorElement & HTMLElement>} className={classes} onClick={onClick} href={href} {...props}>
        <div className={styles.header}>
          <div>
            <div className={styles.titleSection}>
              <span className={`${styles.statusDot} ${styles[status]}`} />
              {name && <h3 className={styles.title}>{name}</h3>}
            </div>
            <div className={styles.id}>{id}</div>
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--space-2)' }}>
            {actions && actions.length > 0 && (
              <div className={styles.actions}>
                {actions.map((action, i) => (
                  <button
                    key={i}
                    type="button"
                    className={styles.actionButton}
                    onClick={(e) => {
                      e.stopPropagation();
                      action.onClick(e);
                    }}
                    aria-label={action.label}
                    title={action.label}
                  >
                    {action.icon}
                  </button>
                ))}
              </div>
            )}
            <Badge status={statusMap[status]}>{status}</Badge>
          </div>
        </div>
        <div className={styles.meta}>
          <div className={styles.subscribers}>
            <span className={styles.subscriberIcon}>&#x1F464;</span>
            <span>{subscribers}</span>
          </div>
          <span className={styles.time}>{timeAgo}</span>
          {(durationStr || eventCount !== undefined) && (
            <div className={styles.badgeGroup}>
              {durationStr && <span className={styles.metaBadge}>{durationStr}</span>}
              {eventCount !== undefined && (
                <span className={styles.metaBadge}>{eventCount} events</span>
              )}
            </div>
          )}
        </div>
      </Tag>
    );
  }
);

SessionCard.displayName = 'SessionCard';

function formatTimeAgo(date: Date): string {
  const seconds = Math.floor((Date.now() - date.getTime()) / 1000);
  if (seconds < 60) return 'just now';
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}

function formatDuration(seconds: number): string {
  if (seconds < 60) return `${seconds}s`;
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.floor(minutes / 60);
  const remainingMinutes = minutes % 60;
  if (remainingMinutes === 0) return `${hours}h`;
  return `${hours}h ${remainingMinutes}m`;
}
