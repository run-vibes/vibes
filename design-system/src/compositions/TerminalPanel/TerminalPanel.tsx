// design-system/src/compositions/TerminalPanel/TerminalPanel.tsx
import { forwardRef, type HTMLAttributes, type ReactNode } from 'react';
import styles from './TerminalPanel.module.css';

export interface TerminalPanelProps extends HTMLAttributes<HTMLDivElement> {
  /** Whether this terminal is focused */
  focused?: boolean;
  /** Content for the header slot */
  header?: ReactNode;
  /** Content for the input slot */
  input?: ReactNode;
  /** Main body content (children) */
  children?: ReactNode;
}

export const TerminalPanel = forwardRef<HTMLDivElement, TerminalPanelProps>(
  function TerminalPanel({ focused, header, input, children, className, ...props }, ref) {
    const classNames = [
      styles.terminalPanel,
      focused && styles.focused,
      className,
    ]
      .filter(Boolean)
      .join(' ');

    return (
      <div ref={ref} className={classNames} {...props}>
        {header && <div className={styles.headerSlot}>{header}</div>}
        <div className={styles.bodySlot}>{children}</div>
        {input && <div className={styles.inputSlot}>{input}</div>}
      </div>
    );
  }
);
