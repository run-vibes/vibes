import { HTMLAttributes, ReactNode, forwardRef } from 'react';
import styles from './Panel.module.css';

export interface PanelProps extends HTMLAttributes<HTMLDivElement> {
  title?: string;
  variant?: 'default' | 'elevated' | 'inset' | 'crt';
  actions?: ReactNode;
  noPadding?: boolean;
}

export const Panel = forwardRef<HTMLDivElement, PanelProps>(
  ({ title, variant = 'default', actions, noPadding, className = '', children, ...props }, ref) => {
    const classes = [
      styles.panel,
      styles[variant],
      noPadding && styles.noPadding,
      className,
    ].filter(Boolean).join(' ');

    return (
      <div ref={ref} className={classes} {...props}>
        {title && (
          <div className={styles.header}>
            <h3 className={styles.title}>{title}</h3>
            {actions}
          </div>
        )}
        <div className={styles.content}>{children}</div>
      </div>
    );
  }
);

Panel.displayName = 'Panel';
