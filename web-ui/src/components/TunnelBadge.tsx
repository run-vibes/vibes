import { Link } from '@tanstack/react-router';
import { useTunnelStatus } from '../hooks/useTunnelStatus';

const BADGE_CONFIG = {
  disabled: { color: 'var(--status-disabled)', icon: '○', tooltip: 'No tunnel configured' },
  starting: { color: 'var(--status-starting)', icon: '◐', tooltip: 'Connecting...' },
  connected: { color: 'var(--status-connected)', icon: '●', tooltip: '' }, // URL set dynamically
  reconnecting: { color: 'var(--status-starting)', icon: '◐', tooltip: 'Reconnecting...' },
  failed: { color: 'var(--status-failed)', icon: '●', tooltip: 'Connection failed' },
  stopped: { color: 'var(--status-disabled)', icon: '○', tooltip: 'Tunnel stopped' },
} as const;

export function TunnelBadge() {
  const { data: status, isLoading } = useTunnelStatus();

  if (isLoading || !status) {
    return (
      <span style={{ color: 'var(--status-disabled)' }} title="Loading...">
        ○
      </span>
    );
  }

  const config = BADGE_CONFIG[status.state];
  const tooltip = status.state === 'connected' && status.url
    ? status.url
    : status.error || config.tooltip;

  return (
    <Link to="/settings" style={{ textDecoration: 'none' }}>
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
