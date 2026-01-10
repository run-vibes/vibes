import { HTMLAttributes, forwardRef } from 'react';
import styles from './StatusIndicator.module.css';

export interface StatusIndicatorProps extends HTMLAttributes<HTMLDivElement> {
  state: 'live' | 'paused' | 'offline' | 'error' | 'ok' | 'degraded';
  label?: string;
}

export const StatusIndicator = forwardRef<HTMLDivElement, StatusIndicatorProps>(
  ({ state, label, className = '', ...props }, ref) => {
    const classes = [styles.indicator, styles[state], className].filter(Boolean).join(' ');

    return (
      <div ref={ref} className={classes} {...props}>
        <span className={styles.dot} />
        {label && <span className={styles.label}>{label}</span>}
      </div>
    );
  }
);

StatusIndicator.displayName = 'StatusIndicator';
