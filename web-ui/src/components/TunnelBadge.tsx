import { Link } from '@tanstack/react-router';
import { useTunnelStatus } from '../hooks/useTunnelStatus';

const BADGE_CONFIG = {
  disabled: { color: '#9CA3AF', icon: '○', tooltip: 'No tunnel configured' },
  starting: { color: '#F59E0B', icon: '◐', tooltip: 'Connecting...' },
  connected: { color: '#10B981', icon: '●', tooltip: '' }, // URL set dynamically
  reconnecting: { color: '#F59E0B', icon: '◐', tooltip: 'Reconnecting...' },
  failed: { color: '#EF4444', icon: '●', tooltip: 'Connection failed' },
  stopped: { color: '#9CA3AF', icon: '○', tooltip: 'Tunnel stopped' },
} as const;

export function TunnelBadge() {
  const { data: status, isLoading } = useTunnelStatus();

  if (isLoading || !status) {
    return (
      <span style={{ color: '#9CA3AF' }} title="Loading...">
        ○
      </span>
    );
  }

  const config = BADGE_CONFIG[status.state];
  const tooltip = status.state === 'connected' && status.url
    ? status.url
    : status.error || config.tooltip;

  return (
    <Link to="/status" style={{ textDecoration: 'none' }}>
      <span
        style={{
          color: config.color,
          fontSize: '1rem',
          cursor: 'pointer',
        }}
        title={tooltip}
      >
        {config.icon}
      </span>
    </Link>
  );
}
