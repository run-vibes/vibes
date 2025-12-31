// web-ui/src/pages/Debug.tsx
import { useState, useEffect, useCallback, useRef } from 'react';
import { Panel, Badge, Button, Text } from '@vibes/design-system';
import { useWebSocket } from '../hooks/useWebSocket';
import { useTunnelStatus } from '../hooks/useTunnelStatus';
import './Debug.css';

interface LogEntry {
  id: number;
  timestamp: Date;
  level: 'info' | 'warn' | 'error';
  message: string;
}

export function DebugPage() {
  const { isConnected, connectionState } = useWebSocket();
  const { data: tunnel, isLoading: tunnelLoading } = useTunnelStatus();
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const nextIdRef = useRef(0);

  // Add log entry helper - uses ref to avoid stale closure
  const addLog = useCallback((level: LogEntry['level'], message: string) => {
    const id = nextIdRef.current++;
    setLogs((prev) => [
      { id, timestamp: new Date(), level, message },
      ...prev.slice(0, 99), // Keep last 100 logs
    ]);
  }, []);

  // Log WebSocket state changes
  useEffect(() => {
    addLog('info', `WebSocket: ${connectionState}`);
  }, [connectionState, addLog]);

  // Log tunnel state changes
  useEffect(() => {
    if (tunnel) {
      addLog('info', `Tunnel: ${tunnel.state}${tunnel.url ? ` (${tunnel.url})` : ''}`);
    }
  }, [tunnel?.state, tunnel?.url, addLog]);

  const clearLogs = () => setLogs([]);

  return (
    <div className="debug-page">
      <div className="debug-header">
        <h1>Debug Console</h1>
        <Text intensity="dim">System diagnostics and connection status</Text>
      </div>

      <div className="debug-content">
        <div className="debug-status-grid">
          <Panel title="WebSocket" className="debug-panel">
            <div className="status-row">
              <span>Connection</span>
              <Badge status={isConnected ? 'success' : 'error'}>
                {connectionState}
              </Badge>
            </div>
          </Panel>

          <Panel title="Tunnel" className="debug-panel">
            {tunnelLoading ? (
              <Text intensity="dim">Loading...</Text>
            ) : tunnel ? (
              <>
                <div className="status-row">
                  <span>State</span>
                  <Badge status={tunnel.state === 'connected' ? 'success' : tunnel.state === 'failed' ? 'error' : 'warning'}>
                    {tunnel.state}
                  </Badge>
                </div>
                {tunnel.mode && (
                  <div className="status-row">
                    <span>Mode</span>
                    <code>{tunnel.mode}</code>
                  </div>
                )}
                {tunnel.url && (
                  <div className="status-row">
                    <span>URL</span>
                    <a href={tunnel.url} target="_blank" rel="noopener noreferrer" className="tunnel-url">
                      {tunnel.url}
                    </a>
                  </div>
                )}
              </>
            ) : (
              <Text intensity="dim">Not configured</Text>
            )}
          </Panel>

          <Panel title="Environment" className="debug-panel">
            <div className="status-row">
              <span>Protocol</span>
              <code>{window.location.protocol}</code>
            </div>
            <div className="status-row">
              <span>Host</span>
              <code>{window.location.host}</code>
            </div>
            <div className="status-row">
              <span>User Agent</span>
              <code className="truncate">{navigator.userAgent.split(' ')[0]}</code>
            </div>
          </Panel>
        </div>

        <Panel title="Activity Log" className="debug-logs-panel">
          <div className="logs-header">
            <Text intensity="dim">{logs.length} entries</Text>
            <Button variant="ghost" onClick={clearLogs}>Clear</Button>
          </div>
          <div className="logs-container">
            {logs.length === 0 ? (
              <div className="logs-empty">
                <Text intensity="dim">No log entries</Text>
              </div>
            ) : (
              logs.map((log) => (
                <div key={log.id} className={`log-entry log-${log.level}`}>
                  <span className="log-time">
                    {log.timestamp.toLocaleTimeString()}
                  </span>
                  <span className={`log-level log-level-${log.level}`}>
                    {log.level.toUpperCase()}
                  </span>
                  <span className="log-message">{log.message}</span>
                </div>
              ))
            )}
          </div>
        </Panel>
      </div>
    </div>
  );
}
