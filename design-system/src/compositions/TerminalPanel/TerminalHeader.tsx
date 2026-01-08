// design-system/src/compositions/TerminalPanel/TerminalHeader.tsx
import { forwardRef, type HTMLAttributes, type ReactNode } from 'react';
import styles from './TerminalHeader.module.css';

export type TerminalStatus = 'active' | 'idle' | 'error';

export interface TerminalHeaderProps extends HTMLAttributes<HTMLDivElement> {
  /** Status indicator color */
  status?: TerminalStatus;
  /** Terminal/session name */
  name?: string;
  /** Terminal/session ID (truncated) */
  id?: string;
  /** Metadata items (e.g., "47 tools", "1h 23m") */
  metadata?: string[];
  /** Action buttons/elements */
  actions?: ReactNode;
}

export const TerminalHeader = forwardRef<HTMLDivElement, TerminalHeaderProps>(
  function TerminalHeader(
    { status = 'idle', name, id, metadata, actions, className, ...props },
    ref
  ) {
    const classNames = [styles.terminalHeader, className].filter(Boolean).join(' ');

    return (
      <div ref={ref} className={classNames} {...props}>
        <div className={styles.info}>
          <span className={`${styles.status} ${styles[status]}`} />
          {name && <span className={styles.name}>{name}</span>}
          {id && <span className={styles.id}>{id}</span>}
        </div>

        {metadata && metadata.length > 0 && (
          <div className={styles.meta}>
            {metadata.map((item, i) => (
              <span key={i}>{item}</span>
            ))}
          </div>
        )}

        {actions && <div className={styles.actions}>{actions}</div>}
      </div>
    );
  }
);
