import { forwardRef, HTMLAttributes } from 'react';
import { Badge } from '../../primitives/Badge';
import styles from './SessionCard.module.css';

export interface SessionCardProps extends HTMLAttributes<HTMLElement> {
  id: string;
  name?: string;
  status: 'idle' | 'processing' | 'waiting' | 'finished' | 'failed';
  subscribers?: number;
  updatedAt: Date;
}

const statusMap = {
  idle: 'idle',
  processing: 'accent',
  waiting: 'warning',
  finished: 'success',
  failed: 'error',
} as const;

export const SessionCard = forwardRef<HTMLElement, SessionCardProps>(
  ({ id, name, status, subscribers = 0, updatedAt, className = '', onClick, ...props }, ref) => {
    const classes = [styles.card, className].filter(Boolean).join(' ');

    const timeAgo = formatTimeAgo(updatedAt);

    return (
      <article ref={ref} className={classes} onClick={onClick} {...props}>
        <div className={styles.header}>
          <div>
            {name && <h3 className={styles.title}>{name}</h3>}
            <div className={styles.id}>{id}</div>
          </div>
          <Badge status={statusMap[status]}>{status}</Badge>
        </div>
        <div className={styles.meta}>
          <div className={styles.subscribers}>
            <span className={styles.subscriberIcon}>👤</span>
            <span>{subscribers}</span>
          </div>
          <span className={styles.time}>{timeAgo}</span>
        </div>
      </article>
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
