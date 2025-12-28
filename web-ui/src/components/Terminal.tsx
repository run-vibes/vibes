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

    // Expose write and focus methods via ref
    useImperativeHandle(ref, () => ({
      write: (data: string) => {
        if (terminalRef.current) {
          try {
            const decoded = decodeFromTransport(data);
            terminalRef.current.write(decoded);
          } catch {
            // If not valid base64, write directly
            terminalRef.current.write(data);
          }
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

      // Create terminal with Claude-like theme
      const term = new Terminal({
        cursorBlink: true,
        fontSize: 14,
        fontFamily: '"JetBrains Mono", "Fira Code", Menlo, Monaco, monospace',
        theme: {
          background: '#1a1a2e',
          foreground: '#eaeaea',
          cursor: '#eaeaea',
          cursorAccent: '#1a1a2e',
          selectionBackground: '#3d3d5c',
          black: '#000000',
          red: '#ff5555',
          green: '#50fa7b',
          yellow: '#f1fa8c',
          blue: '#bd93f9',
          magenta: '#ff79c6',
          cyan: '#8be9fd',
          white: '#f8f8f2',
          brightBlack: '#6272a4',
          brightRed: '#ff6e6e',
          brightGreen: '#69ff94',
          brightYellow: '#ffffa5',
          brightBlue: '#d6acff',
          brightMagenta: '#ff92df',
          brightCyan: '#a4ffff',
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
