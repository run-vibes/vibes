/**
 * Activity stats panel
 *
 * Displays summary metrics: outcomes, negative rate, exploration bonus.
 */
import { Card, Metric } from '@vibes/design-system';
import type { ActivitySummary } from '../../../hooks/useDashboard';
import './ActivityStats.css';
import '../DashboardCards.css';

export interface ActivityStatsProps {
  summary?: ActivitySummary;
  isLoading?: boolean;
  isLive?: boolean;
}

export function ActivityStats({ summary, isLoading, isLive }: ActivityStatsProps) {
  if (isLoading) {
    return (
      <Card variant="crt" title="Response Activity" className="dashboard-card">
        <p className="empty-text">Loading...</p>
      </Card>
    );
  }

  return (
    <Card variant="crt" title="Response Activity" className="dashboard-card activity-stats">
      {isLive && <LiveIndicator />}
      <div className="dashboard-card__metrics">
        <Metric
          label="Outcomes"
          value={String(summary?.outcomes_total ?? 0)}
          data-testid="outcomes-metric"
        />
        <Metric
          label="Negative"
          value={`${((summary?.negative_rate ?? 0) * 100).toFixed(0)}%`}
          data-testid="negative-metric"
        />
        <Metric
          label="Exploration"
          value={`+${(summary?.avg_exploration_bonus ?? 0).toFixed(2)}`}
          data-testid="exploration-metric"
        />
      </div>
    </Card>
  );
}

function LiveIndicator() {
  return (
    <span className="live-indicator" data-testid="live-indicator">
      <span className="live-indicator__dot" />
      <span className="live-indicator__text">LIVE</span>
    </span>
  );
}
