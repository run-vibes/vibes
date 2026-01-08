// design-system/src/compositions/TerminalPanel/TerminalLine.tsx
import { forwardRef, type HTMLAttributes, type ReactNode } from 'react';
import styles from './TerminalLine.module.css';

export type TerminalLineVariant = 'prompt' | 'output' | 'thinking' | 'tool';
export type TerminalLineStatus = 'default' | 'success' | 'error' | 'info';

export interface TerminalLineProps extends HTMLAttributes<HTMLDivElement> {
  /** Line variant determines base styling */
  variant?: TerminalLineVariant;
  /** Status variant for output lines */
  status?: TerminalLineStatus;
  /** Prompt text (for prompt variant) */
  prompt?: string;
  /** Tool name (for tool variant) */
  toolName?: string;
  /** Line content */
  children?: ReactNode;
}

export const TerminalLine = forwardRef<HTMLDivElement, TerminalLineProps>(
  function TerminalLine(
    { variant = 'output', status = 'default', prompt, toolName, children, className, ...props },
    ref
  ) {
    const classNames = [
      styles.line,
      styles[variant],
      status !== 'default' && styles[status],
      className,
    ]
      .filter(Boolean)
      .join(' ');

    // Prompt variant: shows prompt + command
    if (variant === 'prompt') {
      return (
        <div ref={ref} className={classNames} {...props}>
          <span className={styles.promptText}>{prompt ?? '~/vibes $'}</span>
          <span className={styles.command}>{children}</span>
        </div>
      );
    }

    // Tool variant: shows tool name + content
    if (variant === 'tool') {
      return (
        <div ref={ref} className={classNames} {...props}>
          {toolName && <span className={styles.toolName}>{toolName}</span>}
          <span>{children}</span>
        </div>
      );
    }

    // Thinking variant: icon + content
    if (variant === 'thinking') {
      return (
        <div ref={ref} className={classNames} {...props}>
          <span className={styles.thinkingIcon}>🧠</span>
          <span>{children}</span>
        </div>
      );
    }

    // Default output variant
    return (
      <div ref={ref} className={classNames} {...props}>
        {children}
      </div>
    );
  }
);
