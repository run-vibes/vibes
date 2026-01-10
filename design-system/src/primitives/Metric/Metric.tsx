import { HTMLAttributes, forwardRef, ReactNode } from 'react';
import styles from './Metric.module.css';

export interface MetricProps extends HTMLAttributes<HTMLDivElement> {
  label: string;
  value: ReactNode;
  size?: 'sm' | 'md' | 'lg' | 'xl';
}

export const Metric = forwardRef<HTMLDivElement, MetricProps>(
  ({ label, value, size = 'md', className = '', ...props }, ref) => {
    const classes = [
      styles.metric,
      size !== 'md' && styles[size],
      className,
    ].filter(Boolean).join(' ');

    return (
      <div ref={ref} className={classes} {...props}>
        <span className={styles.label}>{label}</span>
        <span className={styles.value}>{value}</span>
      </div>
    );
  }
);

Metric.displayName = 'Metric';
