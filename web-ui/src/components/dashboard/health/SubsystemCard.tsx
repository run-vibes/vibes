import { StatusIndicator } from '@vibes/design-system';
import type { ComponentHealth, SystemStatus } from '../../../hooks/useDashboard';
import './SubsystemCard.css';

export interface SubsystemCardProps {
  name: string;
  health: ComponentHealth;
  warning?: string;
}

// Map SystemStatus to StatusIndicator state
type StatusIndicatorState = 'ok' | 'degraded' | 'error';
const toIndicatorState = (status: SystemStatus): StatusIndicatorState => status;

function formatPercent(value: number): string {
  return `${Math.round(value * 100)}%`;
}

function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp);
  return date.toLocaleString(undefined, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

export function SubsystemCard({ name, health, warning }: SubsystemCardProps) {
  return (
    <div className={`subsystem-card subsystem-card--${health.status}`}>
      <div className="subsystem-card__header">
        <StatusIndicator state={toIndicatorState(health.status)} />
        <h4 className="subsystem-card__name">{name}</h4>
      </div>

      <div className="subsystem-card__stats">
        <div className="subsystem-card__stat">
          <span className="subsystem-card__stat-label">Coverage</span>
          <span className="subsystem-card__stat-value">{formatPercent(health.coverage)}</span>
        </div>

        {health.item_count !== undefined && (
          <div className="subsystem-card__stat">
            <span className="subsystem-card__stat-label">Items</span>
            <span className="subsystem-card__stat-value">{health.item_count}</span>
          </div>
        )}
      </div>

      {health.last_activity && (
        <div className="subsystem-card__activity">
          Last activity: {formatTimestamp(health.last_activity)}
        </div>
      )}

      {warning && (
        <div className="subsystem-card__warning">{warning}</div>
      )}
    </div>
  );
}
