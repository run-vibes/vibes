import { HTMLAttributes, ReactNode, forwardRef } from 'react';
import styles from './PageHeader.module.css';

export interface PageHeaderProps extends HTMLAttributes<HTMLDivElement> {
  /** The page title displayed in the header */
  title: string;
  /** Optional content rendered on the left side after the title */
  leftContent?: ReactNode;
  /** Optional content rendered on the right side */
  rightContent?: ReactNode;
}

/**
 * PageHeader component for consistent page-level headers across the application.
 * Provides a standardized header with title, optional left/right content areas.
 */
export const PageHeader = forwardRef<HTMLDivElement, PageHeaderProps>(
  ({ title, leftContent, rightContent, className = '', ...props }, ref) => {
    const classes = [styles.pageHeader, className].filter(Boolean).join(' ');

    return (
      <header ref={ref} className={classes} {...props}>
        <div className={styles.left}>
          <h1 className={styles.title}>{title}</h1>
          {leftContent}
        </div>
        {rightContent && <div className={styles.right}>{rightContent}</div>}
      </header>
    );
  }
);

PageHeader.displayName = 'PageHeader';
