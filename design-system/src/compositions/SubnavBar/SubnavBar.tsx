import { forwardRef, HTMLAttributes, ReactNode, useState, useRef, useEffect } from 'react';
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
  moreItems?: SubnavItem[];
  plugin?: 'groove' | 'default';
  renderLink?: (props: SubnavLinkProps) => ReactNode;
}

const DefaultLink = ({ href, className, children }: SubnavLinkProps) => (
  <a href={href} className={className}>{children}</a>
);

export const SubnavBar = forwardRef<HTMLDivElement, SubnavBarProps>(
  ({ isOpen = false, label, items = [], moreItems = [], plugin = 'default', renderLink, className = '', ...props }, ref) => {
    const [moreOpen, setMoreOpen] = useState(false);
    const moreRef = useRef<HTMLDivElement>(null);

    // Close dropdown when clicking outside
    useEffect(() => {
      function handleClickOutside(event: MouseEvent) {
        if (moreRef.current && !moreRef.current.contains(event.target as Node)) {
          setMoreOpen(false);
        }
      }
      if (moreOpen) {
        document.addEventListener('mousedown', handleClickOutside);
        return () => document.removeEventListener('mousedown', handleClickOutside);
      }
    }, [moreOpen]);

    const classes = [
      styles.subnavBar,
      isOpen && styles.open,
      plugin && styles[`plugin${plugin.charAt(0).toUpperCase()}${plugin.slice(1)}`],
      className,
    ].filter(Boolean).join(' ');

    const Link = renderLink ?? DefaultLink;

    const hasActiveMoreItem = moreItems.some(item => item.isActive);

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
          {moreItems.length > 0 && (
            <div ref={moreRef} className={styles.moreContainer}>
              <button
                type="button"
                className={[
                  styles.subnavItem,
                  styles.moreButton,
                  hasActiveMoreItem && styles.subnavItemActive,
                ].filter(Boolean).join(' ')}
                onClick={() => setMoreOpen(!moreOpen)}
                aria-expanded={moreOpen}
                aria-haspopup="true"
              >
                <span className={styles.icon}>⋯</span>
                More
              </button>
              {moreOpen && (
                <div className={styles.moreDropdown}>
                  {moreItems.map((item) => (
                    <Link
                      key={item.href}
                      href={item.href}
                      className={[
                        styles.moreItem,
                        item.isActive && styles.moreItemActive,
                      ].filter(Boolean).join(' ')}
                    >
                      {item.icon && <span className={styles.icon}>{item.icon}</span>}
                      {item.label}
                    </Link>
                  ))}
                </div>
              )}
            </div>
          )}
        </nav>
      </div>
    );
  }
);

SubnavBar.displayName = 'SubnavBar';
