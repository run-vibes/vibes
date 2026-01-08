// design-system/src/compositions/TerminalPanel/TerminalInput.tsx
import {
  forwardRef,
  type InputHTMLAttributes,
  type KeyboardEvent,
  useRef,
  useImperativeHandle,
} from 'react';
import styles from './TerminalInput.module.css';

export interface TerminalInputProps
  extends Omit<InputHTMLAttributes<HTMLInputElement>, 'onSubmit'> {
  /** Prompt character (default: ⟩) */
  prompt?: string;
  /** Show blinking cursor indicator */
  showCursor?: boolean;
  /** Called when user submits (Enter key) */
  onSubmit?: (value: string) => void;
}

export interface TerminalInputHandle {
  focus: () => void;
  clear: () => void;
}

export const TerminalInput = forwardRef<TerminalInputHandle, TerminalInputProps>(
  function TerminalInput(
    { prompt = '⟩', showCursor = false, onSubmit, className, onKeyDown, ...props },
    ref
  ) {
    const inputRef = useRef<HTMLInputElement>(null);

    useImperativeHandle(ref, () => ({
      focus: () => inputRef.current?.focus(),
      clear: () => {
        if (inputRef.current) inputRef.current.value = '';
      },
    }));

    const handleKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
      if (e.key === 'Enter' && onSubmit && inputRef.current) {
        e.preventDefault();
        onSubmit(inputRef.current.value);
      }
      onKeyDown?.(e);
    };

    const classNames = [styles.terminalInput, className].filter(Boolean).join(' ');

    return (
      <div className={classNames}>
        <span className={styles.prompt}>{prompt}</span>
        <input
          ref={inputRef}
          type="text"
          className={styles.field}
          onKeyDown={handleKeyDown}
          {...props}
        />
        {showCursor && <span className={styles.cursor} />}
      </div>
    );
  }
);
