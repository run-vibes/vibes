import { HTMLAttributes, forwardRef } from 'react';
import styles from './Badge.module.css';

export interface BadgeProps extends HTMLAttributes<HTMLSpanElement> {
  status?: 'idle' | 'success' | 'warning' | 'error' | 'info' | 'accent';
}

export const Badge = forwardRef<HTMLSpanElement, BadgeProps>(
  ({ status = 'idle', className = '', children, ...props }, ref) => {
    const classes = [styles.badge, styles[status], className].filter(Boolean).join(' ');

    return (
      <span ref={ref} className={classes} {...props}>
        {children}
      </span>
    );
  }
);

Badge.displayName = 'Badge';
