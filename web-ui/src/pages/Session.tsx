/**
 * Session page - displays a PTY terminal for interacting with a session
 */
import { useParams, Link } from '@tanstack/react-router';
import { useEffect, useRef, useState, useCallback } from 'react';
import { useWebSocket } from '../hooks/useWebSocket';
import { SessionTerminal, type SessionTerminalHandle } from '../components/Terminal';
import { TerminalPanel, TerminalHeader, Button } from '@vibes/design-system';
import type { TerminalStatus } from '@vibes/design-system';
import type { ServerMessage } from '../lib/types';
import './Session.css';

type ConnectionState = 'connecting' | 'attached' | 'exited' | 'error';

function connectionStateToTerminalStatus(state: ConnectionState): TerminalStatus {
  switch (state) {
    case 'attached':
      return 'active';
    case 'error':
      return 'error';
    default:
      return 'idle';
  }
}

export function Session() {
  const { sessionId } = useParams({ from: '/sessions/$sessionId' });
  const { isConnected, addMessageHandler, send } = useWebSocket();

  const [connectionState, setConnectionState] = useState<ConnectionState>('connecting');
  const [exitCode, setExitCode] = useState<number | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [terminalSize, setTerminalSize] = useState({ cols: 80, rows: 24 });

  const terminalRef = useRef<SessionTerminalHandle | null>(null);

  // Buffer for terminal data that arrives before terminal is mounted.
  // This fixes a race condition where pty_replay arrives before attach_ack,
  // but the terminal only renders after connectionState becomes 'attached'.
  const pendingDataRef = useRef<string[]>([]);

  // Callback ref that flushes pending data when terminal mounts.
  // Using a callback ref (vs useEffect) ensures flush happens synchronously
  // during React's commit phase, before any subsequent messages can arrive.
  const terminalCallbackRef = useCallback((handle: SessionTerminalHandle | null) => {
    terminalRef.current = handle;
    if (handle && pendingDataRef.current.length > 0) {
      for (const data of pendingDataRef.current) {
        handle.write(data);
      }
      pendingDataRef.current = [];
    }
  }, []);

  // Send attach request when connected
  useEffect(() => {
    if (isConnected && connectionState === 'connecting') {
      send({
        type: 'attach',
        session_id: sessionId,
      });
    }
  }, [isConnected, sessionId, send, connectionState]);

  // Write data to terminal or buffer it if terminal isn't mounted yet.
  // Buffered data is flushed by terminalCallbackRef when terminal mounts.
  const writeToTerminal = useCallback((data: string) => {
    if (terminalRef.current) {
      terminalRef.current.write(data);
    } else {
      pendingDataRef.current.push(data);
    }
  }, []);

  // Handle incoming messages
  const handleMessage = useCallback((msg: ServerMessage) => {
    switch (msg.type) {
      case 'attach_ack':
        if (msg.session_id === sessionId) {
          setConnectionState('attached');
          setTerminalSize({ cols: msg.cols, rows: msg.rows });
        }
        break;

      case 'pty_output':
        if (msg.session_id === sessionId) {
          writeToTerminal(msg.data);
        }
        break;

      case 'pty_replay':
        if (msg.session_id === sessionId) {
          writeToTerminal(msg.data);
        }
        break;

      case 'pty_exit':
        if (msg.session_id === sessionId) {
          setConnectionState('exited');
          setExitCode(msg.exit_code ?? null);
        }
        break;

      case 'error':
        if (msg.session_id === sessionId) {
          setConnectionState('error');
          setErrorMessage(msg.message);
        }
        break;
    }
  }, [sessionId, writeToTerminal]);

  useEffect(() => {
    return addMessageHandler(handleMessage);
  }, [addMessageHandler, handleMessage]);

  // Send detach when unmounting
  useEffect(() => {
    return () => {
      if (isConnected) {
        send({
          type: 'detach',
          session_id: sessionId,
        });
      }
    };
  }, [isConnected, sessionId, send]);

  // Terminal callbacks
  const handleInput = useCallback((data: string) => {
    send({
      type: 'pty_input',
      session_id: sessionId,
      data,
    });
  }, [sessionId, send]);

  const handleResize = useCallback((cols: number, rows: number) => {
    setTerminalSize({ cols, rows });
    send({
      type: 'pty_resize',
      session_id: sessionId,
      cols,
      rows,
    });
  }, [sessionId, send]);

  // Build metadata array
  const metadata: string[] = [`${terminalSize.cols}×${terminalSize.rows}`];
  if (connectionState === 'exited' && exitCode !== null) {
    metadata.push(`exit: ${exitCode}`);
  }
  if (connectionState === 'error') {
    metadata.push('ERROR');
  }

  // Header actions
  const headerActions = (
    <>
      <Link to="/sessions" className="term-action-btn">← BACK</Link>
      {connectionState === 'error' && (
        <Button variant="ghost" size="sm">RETRY</Button>
      )}
      {connectionState === 'exited' && (
        <Button variant="ghost" size="sm">LOGS</Button>
      )}
    </>
  );

  return (
    <div className="session-page">
      {!isConnected && (
        <div className="connection-banner">
          Connecting to daemon...
        </div>
      )}

      {connectionState === 'error' && errorMessage && (
        <div className="error-banner">
          <strong>Error:</strong> {errorMessage}
        </div>
      )}

      {connectionState === 'exited' && (
        <div className="exit-banner">
          Session ended {exitCode !== null && `(exit code: ${exitCode})`}
        </div>
      )}

      <TerminalPanel
        focused={connectionState === 'attached'}
        header={
          <TerminalHeader
            status={connectionStateToTerminalStatus(connectionState)}
            name={`session-${sessionId.slice(0, 6)}`}
            id={sessionId.slice(0, 8)}
            metadata={metadata}
            actions={headerActions}
          />
        }
      >
        {(connectionState === 'attached' || connectionState === 'exited') && (
          <SessionTerminal
            ref={terminalCallbackRef}
            sessionId={sessionId}
            onInput={handleInput}
            onResize={handleResize}
          />
        )}
        {connectionState === 'connecting' && (
          <div className="terminal-placeholder">
            Attaching to session...
          </div>
        )}
      </TerminalPanel>
    </div>
  );
}
