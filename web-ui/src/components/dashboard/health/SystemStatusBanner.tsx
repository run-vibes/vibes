import type { SystemStatus } from '../../../hooks/useDashboard';
import './SystemStatusBanner.css';

export interface SystemStatusBannerProps {
  status: SystemStatus;
  lastCheck?: string;
}

function getStatusLabel(status: SystemStatus): string {
  switch (status) {
    case 'ok':
      return 'All Systems Operational';
    case 'degraded':
      return 'System Degraded';
    case 'error':
      return 'System Error';
  }
}

function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp);
  return date.toLocaleTimeString(undefined, {
    hour: '2-digit',
    minute: '2-digit',
  });
}

export function SystemStatusBanner({ status, lastCheck }: SystemStatusBannerProps) {
  return (
    <div className={`status-banner status-banner--${status}`}>
      <span className="status-banner__label">{getStatusLabel(status)}</span>
      {lastCheck && (
        <span className="status-banner__timestamp">
          Last check: {formatTimestamp(lastCheck)}
        </span>
      )}
    </div>
  );
}
