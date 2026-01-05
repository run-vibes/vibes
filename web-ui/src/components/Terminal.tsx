/**
 * Terminal component using xterm.js for PTY session display
 */
import { useEffect, useRef, useCallback, useImperativeHandle, forwardRef } from 'react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebLinksAddon } from '@xterm/addon-web-links';
import '@xterm/xterm/css/xterm.css';
import './Terminal.css';
import { encodeForTransport, decodeFromTransport } from '../lib/encoding';

interface SessionTerminalProps {
  sessionId: string;
  onInput: (data: string) => void;
  onResize: (cols: number, rows: number) => void;
}

export interface SessionTerminalHandle {
  write: (data: string) => void;
  focus: () => void;
}

/**
 * Terminal component that wraps xterm.js for PTY session I/O.
 *
 * Uses forwardRef to expose write() and focus() methods to parent components.
 */
export const SessionTerminal = forwardRef<SessionTerminalHandle, SessionTerminalProps>(
  function SessionTerminal({ sessionId, onInput, onResize }, ref) {
    const containerRef = useRef<HTMLDivElement>(null);
    const terminalRef = useRef<Terminal | null>(null);
    const fitAddonRef = useRef<FitAddon | null>(null);

    // Buffer for data that arrives before terminal is initialized.
    // useImperativeHandle runs before useEffect, so write() may be called
    // before the xterm Terminal instance exists.
    const pendingWritesRef = useRef<string[]>([]);

    // Expose write and focus methods via ref
    useImperativeHandle(ref, () => ({
      write: (data: string) => {
        if (terminalRef.current) {
          try {
            const decoded = decodeFromTransport(data);
            terminalRef.current.write(decoded);
          } catch (e) {
            // If not valid base64, write directly (this is expected for some messages)
            console.warn('Failed to decode terminal data, writing directly:', e);
            terminalRef.current.write(data);
          }
        } else {
          // Terminal not ready yet, buffer the data
          pendingWritesRef.current.push(data);
        }
      },
      focus: () => {
        terminalRef.current?.focus();
      },
    }), []);

    // Stable callbacks to avoid re-initialization
    const handleInput = useCallback((data: string) => {
      onInput(encodeForTransport(data));
    }, [onInput]);

    const handleResize = useCallback((event: { cols: number; rows: number }) => {
      onResize(event.cols, event.rows);
    }, [onResize]);

    useEffect(() => {
      if (!containerRef.current) return;

      // Get CSS variable values for terminal theme
      const styles = getComputedStyle(document.documentElement);
      const getVar = (name: string, fallback: string) =>
        styles.getPropertyValue(name).trim() || fallback;

      // Create terminal with CRT design system theme
      const term = new Terminal({
        cursorBlink: true,
        fontSize: 14,
        fontFamily: '"IBM Plex Mono", "JetBrains Mono", Menlo, Monaco, monospace',
        theme: {
          background: getVar('--terminal-bg', '#0a0908'),
          foreground: getVar('--terminal-fg', '#e8e0d0'),
          cursor: getVar('--terminal-cursor', '#ffb000'),
          cursorAccent: getVar('--terminal-bg', '#0a0908'),
          selectionBackground: getVar('--terminal-selection', '#3d3830'),
          // ANSI colors matching the CRT aesthetic
          black: '#0a0908',
          red: getVar('--red', '#ff4444'),
          green: getVar('--green', '#44ff44'),
          yellow: getVar('--phosphor', '#ffb000'),
          blue: '#6b8cff',
          magenta: '#c678dd',
          cyan: getVar('--cyan', '#44ffff'),
          white: getVar('--text', '#e8e0d0'),
          brightBlack: getVar('--text-faint', '#5a5248'),
          brightRed: '#ff6b6b',
          brightGreen: '#69ff69',
          brightYellow: getVar('--phosphor-bright', '#ffc633'),
          brightBlue: '#8fa8ff',
          brightMagenta: '#da98e8',
          brightCyan: '#69ffff',
          brightWhite: '#ffffff',
        },
        allowTransparency: true,
        scrollback: 10000,
      });

      // Load addons
      const fitAddon = new FitAddon();
      term.loadAddon(fitAddon);
      term.loadAddon(new WebLinksAddon());

      // Open terminal in container
      term.open(containerRef.current);
      fitAddon.fit();

      // Store refs for later use
      terminalRef.current = term;
      fitAddonRef.current = fitAddon;

      // Flush any data that was buffered before terminal was ready
      if (pendingWritesRef.current.length > 0) {
        for (const data of pendingWritesRef.current) {
          try {
            const decoded = decodeFromTransport(data);
            term.write(decoded);
          } catch (e) {
            console.warn('Failed to decode buffered terminal data:', e);
            term.write(data);
          }
        }
        pendingWritesRef.current = [];
      }

      // Handle user input - send to server
      term.onData(handleInput);

      // Handle terminal resize - notify server
      term.onResize(handleResize);

      // Handle window resize
      const onWindowResize = () => {
        fitAddon.fit();
      };
      window.addEventListener('resize', onWindowResize);

      // Focus terminal
      term.focus();

      // Cleanup
      return () => {
        window.removeEventListener('resize', onWindowResize);
        term.dispose();
        terminalRef.current = null;
        fitAddonRef.current = null;
      };
    }, [sessionId, handleInput, handleResize]);

    return <div ref={containerRef} className="terminal-container" />;
  }
);

export default SessionTerminal;
