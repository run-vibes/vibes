import { forwardRef, HTMLAttributes, ReactNode } from 'react';
import styles from './Header.module.css';

export interface NavItem {
  label: string;
  href: string;
  isActive?: boolean;
  isGroove?: boolean;
  hasSubnav?: boolean;
}

export interface LinkProps {
  href: string;
  className: string;
  children: ReactNode;
}

export interface HeaderProps extends HTMLAttributes<HTMLElement> {
  navItems?: NavItem[];
  identity?: { email: string; provider?: string };
  theme?: 'dark' | 'light';
  onThemeToggle?: () => void;
  settingsHref?: string;
  renderLink?: (props: LinkProps) => ReactNode;
  toolbarItems?: ReactNode;
}

const DefaultLink = ({ href, className, children }: LinkProps) => (
  <a href={href} className={className}>{children}</a>
);

export const Header = forwardRef<HTMLElement, HeaderProps>(
  ({ navItems = [], identity, theme = 'dark', onThemeToggle, settingsHref, renderLink, toolbarItems, className = '', ...props }, ref) => {
    const classes = [styles.header, className].filter(Boolean).join(' ');

    const Link = renderLink ?? DefaultLink;

    return (
      <header ref={ref} className={classes} {...props}>
        <Link href="/" className={styles.logo}>VIBES</Link>

        <nav className={styles.nav}>
          {navItems.map((item) => (
            <Link
              key={item.href}
              href={item.href}
              className={[
                styles.navLink,
                item.isActive && styles.navLinkActive,
                item.isGroove && styles.grooveLink,
                item.hasSubnav && styles.hasSubnav,
              ].filter(Boolean).join(' ')}
            >
              {item.label}
              {item.hasSubnav && <span className={styles.subnavIndicator}>▾</span>}
            </Link>
          ))}
        </nav>

        <div className={styles.actions}>
          {toolbarItems}
          {identity && <span className={styles.identity}>{identity.email}</span>}
          {onThemeToggle && (
            <button
              className={styles.themeToggle}
              onClick={onThemeToggle}
              aria-label="Toggle theme"
            >
              <span className={styles.actionIcon}>◐</span>
              <span className={styles.actionLabel}>THEME</span>
            </button>
          )}
          {settingsHref && (
            <Link href={settingsHref} className={styles.settingsLink} aria-label="Settings">
              <span className={styles.actionIcon}>⚙</span>
              <span className={styles.actionLabel}>SETTINGS</span>
            </Link>
          )}
        </div>
      </header>
    );
  }
);

Header.displayName = 'Header';
