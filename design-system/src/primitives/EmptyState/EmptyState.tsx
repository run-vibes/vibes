import { HTMLAttributes, ReactNode, forwardRef } from 'react';
import styles from './EmptyState.module.css';

export interface EmptyStateProps extends HTMLAttributes<HTMLDivElement> {
  /** Icon or visual element displayed above the message */
  icon?: ReactNode;
  /** Primary message text */
  message: string;
  /** Secondary hint or description text */
  hint?: string;
  /** Optional action button or link */
  action?: ReactNode;
  /** Size variant */
  size?: 'sm' | 'md' | 'lg';
}

/**
 * EmptyState component for displaying empty or zero-data states
 * with consistent styling across the application.
 */
export const EmptyState = forwardRef<HTMLDivElement, EmptyStateProps>(
  ({ icon, message, hint, action, size = 'md', className = '', ...props }, ref) => {
    const classes = [styles.emptyState, styles[size], className].filter(Boolean).join(' ');

    return (
      <div ref={ref} className={classes} {...props}>
        {icon && <div className={styles.icon}>{icon}</div>}
        <p className={styles.message}>{message}</p>
        {hint && <p className={styles.hint}>{hint}</p>}
        {action && <div className={styles.action}>{action}</div>}
      </div>
    );
  }
);

EmptyState.displayName = 'EmptyState';
