import { Panel, Metric } from '@vibes/design-system';
import type { OpenWorldOverviewData } from '../../../hooks/useDashboard';
import '../DashboardCards.css';

export interface NoveltyStatsProps {
  data?: OpenWorldOverviewData;
  isLoading?: boolean;
}

export function NoveltyStats({ data, isLoading }: NoveltyStatsProps) {
  if (isLoading) {
    return (
      <Panel variant="crt" title="Novelty Detection" className="dashboard-card">
        <p className="empty-text">Loading...</p>
      </Panel>
    );
  }

  if (!data) {
    return (
      <Panel variant="crt" title="Novelty Detection" className="dashboard-card">
        <p className="empty-text">No novelty data available</p>
      </Panel>
    );
  }

  const { novelty_threshold, pending_outliers, cluster_count } = data;

  return (
    <Panel variant="crt" title="Novelty Detection" className="dashboard-card">
      <div className="dashboard-card__metrics">
        <Metric
          label="Threshold"
          value={novelty_threshold.toFixed(2)}
          data-testid="threshold-metric"
        />
        <Metric
          label="Pending"
          value={String(pending_outliers)}
          data-testid="pending-metric"
        />
        <Metric
          label="Clusters"
          value={String(cluster_count)}
          data-testid="clusters-metric"
        />
      </div>
    </Panel>
  );
}
