import { Card } from '@vibes/design-system';
import { useDashboardOverview, useDashboardStrategyDistributions } from '../../hooks';
import { TrendCard } from '../../components/dashboard/TrendCard';
import { LearningsCard } from '../../components/dashboard/LearningsCard';
import { AttributionCard } from '../../components/dashboard/AttributionCard';
import { HealthCard } from '../../components/dashboard/HealthCard';
import { StrategyCard } from '../../components/dashboard/StrategyCard';
import './DashboardPages.css';

export function DashboardOverview() {
  const { data: overview, isLoading: overviewLoading, isError: overviewError } = useDashboardOverview();
  const { data: strategy, isLoading: strategyLoading } = useDashboardStrategyDistributions();

  const isLoading = overviewLoading || strategyLoading;

  if (isLoading) {
    return (
      <div className="dashboard-page">
        <p className="loading-text">Loading dashboard...</p>
      </div>
    );
  }

  if (overviewError) {
    return (
      <div className="dashboard-page">
        <Card variant="crt" title="Connection Error">
          <div className="error-state">
            <span className="error-state__icon">⚠</span>
            <p className="error-state__message">Failed to load dashboard.</p>
            <p className="error-state__hint">Check that the vibes daemon is running and try again.</p>
          </div>
        </Card>
      </div>
    );
  }

  const trends = overview?.trends;

  return (
    <div className="dashboard-page">
      <div className="dashboard-overview-grid">
        {/* Trend Card - spans full width on mobile, half on larger screens */}
        <TrendCard
          title="Session Trends"
          primaryValue={`${trends?.improvement_percent ?? 0}%`}
          primaryLabel="improvement"
          trendDirection={trends?.trend_direction ?? 'stable'}
          sparklineData={trends?.sparkline_data}
          secondaryMetrics={[
            { label: 'Sessions', value: String(trends?.session_count ?? 0) },
            { label: 'Period', value: `${trends?.period_days ?? 7}d` },
          ]}
          href="/groove/dashboard/trends"
        />

        {/* Domain-specific cards */}
        <LearningsCard data={overview?.learnings} />
        <AttributionCard data={overview?.attribution} />
        <HealthCard data={overview?.health} />
        <StrategyCard data={strategy} />
      </div>
    </div>
  );
}
