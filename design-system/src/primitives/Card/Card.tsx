import { HTMLAttributes, ReactNode, forwardRef } from 'react';
import styles from './Card.module.css';

export interface CardProps extends HTMLAttributes<HTMLDivElement> {
  title?: string;
  variant?: 'default' | 'elevated' | 'inset' | 'crt';
  actions?: ReactNode;
  footer?: ReactNode;
  noPadding?: boolean;
}

export const Card = forwardRef<HTMLDivElement, CardProps>(
  ({ title, variant = 'default', actions, footer, noPadding, className = '', children, ...props }, ref) => {
    const classes = [
      styles.card,
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
        {footer && <div className={styles.footer}>{footer}</div>}
      </div>
    );
  }
);

Card.displayName = 'Card';
