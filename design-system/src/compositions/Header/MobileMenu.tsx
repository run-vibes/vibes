import { useEffect, useRef, useCallback, ReactNode } from 'react';
import { NavItem, LinkProps } from './Header';
import styles from './MobileMenu.module.css';

export interface MobileMenuProps {
  isOpen: boolean;
  onClose: () => void;
  navItems: NavItem[];
  theme?: 'dark' | 'light';
  onThemeToggle?: () => void;
  settingsHref?: string;
  renderLink?: (props: LinkProps) => ReactNode;
}

const DefaultLink = ({ href, className, children }: LinkProps) => (
  <a href={href} className={className}>{children}</a>
);

export function MobileMenu({
  isOpen,
  onClose,
  navItems,
  onThemeToggle,
  settingsHref,
  renderLink,
}: MobileMenuProps) {
  const panelRef = useRef<HTMLElement>(null);
  const closeButtonRef = useRef<HTMLButtonElement>(null);
  const previousFocusRef = useRef<HTMLElement | null>(null);

  const Link = renderLink ?? DefaultLink;

  // Focus trap and keyboard handling
  useEffect(() => {
    if (!isOpen) return;

    // Store the element that had focus before opening
    previousFocusRef.current = document.activeElement as HTMLElement;

    // Focus the close button when menu opens
    setTimeout(() => closeButtonRef.current?.focus(), 50);

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
        return;
      }

      // Focus trap
      if (e.key === 'Tab' && panelRef.current) {
        const focusableElements = panelRef.current.querySelectorAll<HTMLElement>(
          'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
        );
        const firstElement = focusableElements[0];
        const lastElement = focusableElements[focusableElements.length - 1];

        if (e.shiftKey && document.activeElement === firstElement) {
          e.preventDefault();
          lastElement?.focus();
        } else if (!e.shiftKey && document.activeElement === lastElement) {
          e.preventDefault();
          firstElement?.focus();
        }
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, onClose]);

  // Return focus when closing
  useEffect(() => {
    if (!isOpen && previousFocusRef.current) {
      previousFocusRef.current.focus();
      previousFocusRef.current = null;
    }
  }, [isOpen]);

  // Prevent body scroll when open
  useEffect(() => {
    if (isOpen) {
      document.body.style.overflow = 'hidden';
    } else {
      document.body.style.overflow = '';
    }
    return () => {
      document.body.style.overflow = '';
    };
  }, [isOpen]);

  const handleNavClick = useCallback(() => {
    onClose();
  }, [onClose]);

  return (
    <>
      {/* Overlay */}
      <div
        className={`${styles.overlay} ${isOpen ? styles.overlayVisible : ''}`}
        onClick={onClose}
        aria-hidden="true"
      />

      {/* Panel */}
      <aside
        ref={panelRef}
        id="mobile-menu"
        role="dialog"
        aria-modal="true"
        aria-label="Navigation menu"
        className={`${styles.panel} ${isOpen ? styles.panelOpen : ''}`}
      >
        <button
          ref={closeButtonRef}
          className={styles.closeButton}
          onClick={onClose}
          aria-label="Close menu"
        >
          ✕
        </button>

        <nav className={styles.nav}>
          {navItems.map((item, index) => (
            <Link
              key={item.href}
              href={item.href}
              className={`${styles.navLink} ${item.isActive ? styles.navLinkActive : ''}`}
            >
              <span
                className={styles.navLinkInner}
                style={{ animationDelay: isOpen ? `${index * 50}ms` : '0ms' }}
                onClick={handleNavClick}
              >
                {item.label}
                {item.hasSubnav && <span className={styles.subnavIndicator}>▾</span>}
              </span>
            </Link>
          ))}
        </nav>

        <div className={styles.divider} />

        <div className={styles.actions}>
          {onThemeToggle && (
            <button
              className={styles.actionButton}
              onClick={() => {
                onThemeToggle();
              }}
              style={{ animationDelay: isOpen ? `${navItems.length * 50}ms` : '0ms' }}
            >
              <span className={styles.actionIcon}>◐</span>
              <span className={styles.actionLabel}>THEME</span>
            </button>
          )}
          {settingsHref && (
            <Link
              href={settingsHref}
              className={styles.actionLink}
            >
              <span
                className={styles.actionLinkInner}
                style={{ animationDelay: isOpen ? `${(navItems.length + 1) * 50}ms` : '0ms' }}
                onClick={handleNavClick}
              >
                <span className={styles.actionIcon}>⚙</span>
                <span className={styles.actionLabel}>SETTINGS</span>
              </span>
            </Link>
          )}
        </div>
      </aside>
    </>
  );
}
