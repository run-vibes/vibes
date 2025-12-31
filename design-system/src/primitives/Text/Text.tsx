import { ElementType, HTMLAttributes, forwardRef } from 'react';
import styles from './Text.module.css';

export interface TextProps extends HTMLAttributes<HTMLElement> {
  as?: ElementType;
  intensity?: 'high' | 'normal' | 'dim';
  size?: 'xs' | 'sm' | 'base' | 'lg' | 'xl' | '2xl';
  mono?: boolean;
}

export const Text = forwardRef<HTMLElement, TextProps>(
  ({ as: Component = 'span', intensity = 'normal', size, mono, className = '', children, ...props }, ref) => {
    const classes = [
      styles.text,
      styles[intensity],
      size && styles[size === '2xl' ? 'xxl' : size],
      mono && styles.mono,
      className,
    ].filter(Boolean).join(' ');

    return (
      <Component ref={ref} className={classes} {...props}>
        {children}
      </Component>
    );
  }
);

Text.displayName = 'Text';
