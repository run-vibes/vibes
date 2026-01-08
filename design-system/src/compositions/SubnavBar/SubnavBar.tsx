import { forwardRef, HTMLAttributes, ReactNode } from 'react';
import styles from './SubnavBar.module.css';

export interface SubnavItem {
  label: string;
  href: string;
  icon?: string;
  isActive?: boolean;
}

export interface SubnavLinkProps {
  href: string;
  className: string;
  children: ReactNode;
}

export interface SubnavBarProps extends HTMLAttributes<HTMLDivElement> {
  isOpen?: boolean;
  label?: string;
  items?: SubnavItem[];
  plugin?: 'groove' | 'default';
  renderLink?: (props: SubnavLinkProps) => ReactNode;
}

const DefaultLink = ({ href, className, children }: SubnavLinkProps) => (
  <a href={href} className={className}>{children}</a>
);

export const SubnavBar = forwardRef<HTMLDivElement, SubnavBarProps>(
  ({ isOpen = false, label, items = [], plugin = 'default', renderLink, className = '', ...props }, ref) => {
    const classes = [
      styles.subnavBar,
      isOpen && styles.open,
      plugin && styles[`plugin${plugin.charAt(0).toUpperCase()}${plugin.slice(1)}`],
      className,
    ].filter(Boolean).join(' ');

    const Link = renderLink ?? DefaultLink;

    return (
      <div ref={ref} className={classes} {...props}>
        {label && <span className={styles.label}>{label}</span>}
        <nav className={styles.subnav}>
          {items.map((item) => (
            <Link
              key={item.href}
              href={item.href}
              className={[
                styles.subnavItem,
                item.isActive && styles.subnavItemActive,
              ].filter(Boolean).join(' ')}
            >
              {item.icon && <span className={styles.icon}>{item.icon}</span>}
              {item.label}
            </Link>
          ))}
        </nav>
      </div>
    );
  }
);

SubnavBar.displayName = 'SubnavBar';
