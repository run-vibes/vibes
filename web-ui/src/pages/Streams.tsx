// web-ui/src/pages/Streams.tsx
import { Link } from '@tanstack/react-router';
import { Card, Badge, Text } from '@vibes/design-system';
import { useWebSocket } from '../hooks/useWebSocket';
import { useFirehose } from '../hooks/useFirehose';
import { useTunnelStatus } from '../hooks/useTunnelStatus';
import './Streams.css';

export function StreamsPage() {
  const { isConnected } = useWebSocket();
  const { data: tunnel } = useTunnelStatus();
  const { events, isConnected: firehoseConnected } = useFirehose();

  // Count events by type
  const eventCounts = events.reduce((acc, e) => {
    const type = e.event.type.replace(/_/g, ' ');
    acc[type] = (acc[type] || 0) + 1;
    return acc;
  }, {} as Record<string, number>);

  return (
    <div className="streams-page">
      <div className="streams-header">
        <h1>Streams</h1>
        <Text intensity="dim">Real-time event monitoring dashboard</Text>
      </div>

      <div className="streams-content">
        <div className="streams-status-bar">
          <div className="status-item">
            <span>Server</span>
            <Badge status={isConnected ? 'success' : 'error'}>
              {isConnected ? 'Connected' : 'Disconnected'}
            </Badge>
          </div>
          <div className="status-item">
            <span>Firehose</span>
            <Badge status={firehoseConnected ? 'success' : 'error'}>
              {firehoseConnected ? 'Live' : 'Offline'}
            </Badge>
          </div>
          <div className="status-item">
            <span>Tunnel</span>
            <Badge status={tunnel?.state === 'connected' ? 'success' : tunnel?.state === 'failed' ? 'error' : 'idle'}>
              {tunnel?.state || 'disabled'}
            </Badge>
          </div>
        </div>

        <div className="streams-grid">
          <Link to="/firehose" className="stream-card-link">
            <Card title="Firehose" className="stream-card">
              <div className="stream-card-content">
                <div className="stream-icon">🔥</div>
                <Text intensity="dim">
                  Real-time event stream with filtering and inspection
                </Text>
                <div className="stream-stats">
                  <span className="stat-value">{events.length}</span>
                  <span className="stat-label">recent events</span>
                </div>
              </div>
            </Card>
          </Link>

          <Link to="/debug" className="stream-card-link">
            <Card title="Debug Console" className="stream-card">
              <div className="stream-card-content">
                <div className="stream-icon">🔧</div>
                <Text intensity="dim">
                  System diagnostics and connection monitoring
                </Text>
                <div className="stream-stats">
                  <span className="stat-value">{isConnected ? '●' : '○'}</span>
                  <span className="stat-label">{isConnected ? 'online' : 'offline'}</span>
                </div>
              </div>
            </Card>
          </Link>

          <Link to="/sessions" className="stream-card-link">
            <Card title="Sessions" className="stream-card">
              <div className="stream-card-content">
                <div className="stream-icon">💬</div>
                <Text intensity="dim">
                  Manage and monitor active sessions
                </Text>
                <div className="stream-stats">
                  <span className="stat-value">→</span>
                  <span className="stat-label">view sessions</span>
                </div>
              </div>
            </Card>
          </Link>
        </div>

        {Object.keys(eventCounts).length > 0 && (
          <Card title="Recent Activity" className="activity-panel">
            <div className="activity-summary">
              {Object.entries(eventCounts).map(([type, count]) => (
                <div key={type} className="activity-item">
                  <span className="activity-type">{type}</span>
                  <span className="activity-count">{count}</span>
                </div>
              ))}
            </div>
          </Card>
        )}
      </div>
    </div>
  );
}
