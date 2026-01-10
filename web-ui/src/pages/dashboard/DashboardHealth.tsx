import { useDashboardHealth } from '../../hooks/useDashboard';
import { SystemStatusBanner } from '../../components/dashboard/health/SystemStatusBanner';
import { SubsystemCard } from '../../components/dashboard/health/SubsystemCard';
import { AdaptiveParamsTable } from '../../components/dashboard/health/AdaptiveParamsTable';
import { RecentActivity } from '../../components/dashboard/health/RecentActivity';
import type { AdaptiveParam, ActivityEvent } from '../../hooks/useDashboard';
import './DashboardHealth.css';

export function DashboardHealth() {
  const { data, isLoading, error } = useDashboardHealth();

  if (isLoading) {
    return (
      <div className="dashboard-health">
        <div className="dashboard-health__loading">Loading health data...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="dashboard-health">
        <div className="dashboard-health__error">Error loading health data</div>
      </div>
    );
  }

  if (!data) {
    return null;
  }

  // Transform API data to component format
  const adaptiveParams: AdaptiveParam[] = (data.adaptive_params || []).map((p) => ({
    name: p.name,
    current: p.current_value,
    mean: p.current_value * 0.95, // Approximate mean from current
    trend: p.trend === 'rising' ? 'up' : p.trend === 'falling' ? 'down' : 'stable',
  }));

  const activityEvents: ActivityEvent[] = (data.recent_activity || []).map((a, i) => ({
    id: `evt-${i}`,
    type: a.activity_type === 'assessment' ? 'extraction' : a.activity_type,
    description: a.message,
    timestamp: a.timestamp,
  }));

  return (
    <div className="dashboard-health">
      <SystemStatusBanner status={data.overall_status} />

      <section className="dashboard-health__subsystems">
        <h3>Subsystems</h3>
        <div className="dashboard-health__grid">
          <SubsystemCard name="Assessment" health={data.assessment} />
          <SubsystemCard name="Extraction" health={data.extraction} />
          <SubsystemCard name="Attribution" health={data.attribution} />
        </div>
      </section>

      <div className="dashboard-health__split">
        <section className="dashboard-health__params">
          <h3>Adaptive Parameters</h3>
          <AdaptiveParamsTable params={adaptiveParams} />
        </section>

        <section className="dashboard-health__activity">
          <h3>Recent Activity</h3>
          <RecentActivity events={activityEvents} maxItems={10} />
        </section>
      </div>
    </div>
  );
}
