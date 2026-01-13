import { Link } from '@tanstack/react-router';
import { Card, Metric, StatusIndicator } from '@vibes/design-system';
import type { HealthSummary, SystemStatus } from '../../hooks/useDashboard';
import './DashboardCards.css';

export interface HealthCardProps {
  data?: HealthSummary;
}

const STATUS_LABELS: Record<SystemStatus, string> = {
  ok: 'OK',
  degraded: 'Degraded',
  error: 'Error',
};

// Map SystemStatus to StatusIndicator state
type StatusIndicatorState = 'ok' | 'degraded' | 'error';
const toIndicatorState = (status: SystemStatus): StatusIndicatorState => status;

function formatTimeAgo(dateStr: string): string {
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffHours = Math.floor(diffMs / (1000 * 60 * 60));

  if (diffHours < 1) return 'just now';
  if (diffHours < 24) return `${diffHours}h ago`;

  const diffDays = Math.floor(diffHours / 24);
  if (diffDays === 1) return 'yesterday';
  return `${diffDays}d ago`;
}

export function HealthCard({ data }: HealthCardProps) {
  if (!data) {
    return (
      <Card variant="crt" title="Health" className="dashboard-card">
        <p className="empty-text">No health data</p>
      </Card>
    );
  }

  const { overall_status, assessment_coverage, ablation_coverage, last_activity } = data;

  return (
    <Card
      variant="crt"
      title="Health"
      className="dashboard-card"
      actions={
        <StatusIndicator
          state={toIndicatorState(overall_status)}
          label={STATUS_LABELS[overall_status]}
          data-testid="status-indicator"
        />
      }
    >
      <div className="dashboard-card__metrics">
        <Metric label="Assessment" value={`${assessment_coverage}%`} />
        <Metric label="Ablation" value={`${ablation_coverage}%`} />
      </div>

      {last_activity && (
        <p className="dashboard-card__activity">
          Last activity: {formatTimeAgo(last_activity)}
        </p>
      )}

      <Link to="/groove/dashboard/health" className="dashboard-card__link">
        View →
      </Link>
    </Card>
  );
}
