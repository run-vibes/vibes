import { Link } from '@tanstack/react-router';
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
      <div className="dashboard-card">
        <h3 className="dashboard-card__title">Health</h3>
        <p className="empty-text">No health data</p>
      </div>
    );
  }

  const { overall_status, assessment_coverage, ablation_coverage, last_activity } = data;

  return (
    <div className="dashboard-card">
      <h3 className="dashboard-card__title">Health</h3>

      <div className="dashboard-card__status">
        <span
          className={`status-indicator status-indicator--${overall_status}`}
          data-testid="status-indicator"
        />
        <span className="status-label">{STATUS_LABELS[overall_status]}</span>
      </div>

      <div className="dashboard-card__metrics">
        <div className="metric">
          <span className="metric__label">Assessment</span>
          <span className="metric__value">{assessment_coverage}%</span>
        </div>
        <div className="metric">
          <span className="metric__label">Ablation</span>
          <span className="metric__value">{ablation_coverage}%</span>
        </div>
      </div>

      {last_activity && (
        <p className="dashboard-card__activity">
          Last activity: {formatTimeAgo(last_activity)}
        </p>
      )}

      <Link to="/groove/dashboard/health" className="dashboard-card__link">
        View →
      </Link>
    </div>
  );
}
