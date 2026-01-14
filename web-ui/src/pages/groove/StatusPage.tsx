// web-ui/src/pages/groove/StatusPage.tsx
// Merged Status page combining DashboardOverview with AssessmentStatus
import { useEffect, useState } from 'react';
import { Link } from '@tanstack/react-router';
import { Card, PageHeader } from '@vibes/design-system';
import { Group } from '@visx/group';
import { Pie } from '@visx/shape';
import { scaleOrdinal } from '@visx/scale';
import { useDashboardOverview, useDashboardStrategyDistributions } from '../../hooks';
import { TrendCard } from '../../components/dashboard/TrendCard';
import { LearningsCard } from '../../components/dashboard/LearningsCard';
import { AttributionCard } from '../../components/dashboard/AttributionCard';
import { HealthCard } from '../../components/dashboard/HealthCard';
import { StrategyCard } from '../../components/dashboard/StrategyCard';
import '../dashboard/DashboardPages.css';
import '../assessment/AssessmentStatus.css';

// Assessment status types
interface CircuitBreakerStatus {
  enabled: boolean;
  cooldown_seconds: number;
  max_interventions_per_session: number;
}

interface SamplingStatus {
  base_rate: number;
  burnin_sessions: number;
}

interface ActivityStatus {
  active_sessions: number;
  events_stored: number;
  sessions: string[];
  intervention_count?: number;
}

interface AssessmentStatusResponse {
  circuit_breaker: CircuitBreakerStatus;
  sampling: SamplingStatus;
  activity: ActivityStatus;
}

interface TierDistribution {
  lightweight: number;
  medium: number;
  heavy: number;
  checkpoint: number;
}

interface AssessmentStatsResponse {
  tier_distribution: TierDistribution;
  total_assessments: number;
}

type AssessmentFetchState =
  | { status: 'loading' }
  | { status: 'success'; data: AssessmentStatusResponse }
  | { status: 'error'; error: string };

type StatsFetchState =
  | { status: 'loading' }
  | { status: 'success'; data: AssessmentStatsResponse }
  | { status: 'error'; error: string };

// Pie chart data format
interface TierDatum {
  tier: string;
  count: number;
}

// Color scale for tiers
const tierColors: Record<string, string> = {
  lightweight: 'var(--phosphor)',
  medium: 'var(--amber)',
  heavy: 'var(--red)',
  checkpoint: 'var(--cyan)',
};

const getColor = scaleOrdinal<string, string>({
  domain: ['lightweight', 'medium', 'heavy', 'checkpoint'],
  range: [tierColors.lightweight, tierColors.medium, tierColors.heavy, tierColors.checkpoint],
});

// Pie chart component
function TierPieChart({ data }: { data: TierDatum[] }) {
  const width = 160;
  const height = 160;
  const radius = Math.min(width, height) / 2;

  return (
    <svg width={width} height={height}>
      <Group top={height / 2} left={width / 2}>
        <Pie
          data={data}
          pieValue={(d) => d.count}
          outerRadius={radius - 8}
          innerRadius={radius - 32}
          padAngle={0.02}
        >
          {(pie) =>
            pie.arcs.map((arc, index) => {
              const [centroidX, centroidY] = pie.path.centroid(arc);
              const hasSpaceForLabel = arc.endAngle - arc.startAngle >= 0.4;
              return (
                <g key={`arc-${index}`}>
                  <path d={pie.path(arc) || ''} fill={getColor(arc.data.tier)} />
                  {hasSpaceForLabel && (
                    <text
                      x={centroidX}
                      y={centroidY}
                      dy=".33em"
                      fill="var(--text)"
                      fontSize={10}
                      fontFamily="var(--font-mono)"
                      textAnchor="middle"
                    >
                      {arc.data.count}
                    </text>
                  )}
                </g>
              );
            })
          }
        </Pie>
      </Group>
    </svg>
  );
}

export function StatusPage() {
  // Dashboard data
  const { data: overview, isLoading: overviewLoading, isError: overviewError } = useDashboardOverview();
  const { data: strategy, isLoading: strategyLoading } = useDashboardStrategyDistributions();

  // Assessment status data
  const [assessmentState, setAssessmentState] = useState<AssessmentFetchState>({ status: 'loading' });
  const [statsState, setStatsState] = useState<StatsFetchState>({ status: 'loading' });

  useEffect(() => {
    async function fetchAssessmentStatus() {
      try {
        const response = await fetch('/api/groove/assess/status');
        if (!response.ok) {
          const errorData = await response.json();
          setAssessmentState({ status: 'error', error: errorData.error || 'Failed to load status' });
          return;
        }
        const data: AssessmentStatusResponse = await response.json();
        setAssessmentState({ status: 'success', data });
      } catch {
        setAssessmentState({ status: 'error', error: 'Failed to connect to server' });
      }
    }

    async function fetchStats() {
      try {
        const response = await fetch('/api/groove/assess/stats');
        if (!response.ok) {
          const errorData = await response.json();
          setStatsState({ status: 'error', error: errorData.error || 'Failed to load stats' });
          return;
        }
        const data: AssessmentStatsResponse = await response.json();
        setStatsState({ status: 'success', data });
      } catch {
        setStatsState({ status: 'error', error: 'Failed to connect to server' });
      }
    }

    fetchAssessmentStatus();
    fetchStats();
  }, []);

  const isLoading = overviewLoading || strategyLoading;

  if (isLoading) {
    return (
      <div className="status-page">
        <PageHeader title="STATUS" />
        <p className="status-loading">Loading dashboard...</p>
      </div>
    );
  }

  if (overviewError) {
    return (
      <div className="status-page">
        <PageHeader title="STATUS" />
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

  // Prepare pie chart data if stats loaded successfully
  const pieData: TierDatum[] =
    statsState.status === 'success'
      ? [
          { tier: 'lightweight', count: statsState.data.tier_distribution.lightweight },
          { tier: 'medium', count: statsState.data.tier_distribution.medium },
          { tier: 'heavy', count: statsState.data.tier_distribution.heavy },
          { tier: 'checkpoint', count: statsState.data.tier_distribution.checkpoint },
        ].filter((d) => d.count > 0)
      : [];

  return (
    <div className="status-page">
      <PageHeader title="STATUS" />

      {/* Dashboard Overview Section */}
      <div className="dashboard-overview-grid">
        {/* Health card first */}
        <HealthCard data={overview?.health} />

        {/* Trend Card */}
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
          href="/groove/trends"
        />

        {/* Other domain cards */}
        <LearningsCard data={overview?.learnings} />
        <AttributionCard data={overview?.attribution} />
        <StrategyCard data={strategy} />
      </div>

      {/* Assessment Status Section */}
      {assessmentState.status === 'success' && (
        <div className="status-grid">
          {/* Circuit Breaker Card */}
          <Card variant="crt" title="Circuit Breaker" className="status-card">
            <div className="status-card-content">
              <div className="status-row">
                <span className="status-label">Status</span>
                <span className={`status-value ${assessmentState.data.circuit_breaker.enabled ? 'enabled' : 'disabled'}`}>
                  {assessmentState.data.circuit_breaker.enabled ? 'Enabled' : 'Disabled'}
                </span>
              </div>
              <div className="status-row">
                <span className="status-label">Cooldown</span>
                <span className="status-value">{assessmentState.data.circuit_breaker.cooldown_seconds}s</span>
              </div>
              <div className="status-row">
                <span className="status-label">Max Interventions</span>
                <span className="status-value">{assessmentState.data.circuit_breaker.max_interventions_per_session}</span>
              </div>
            </div>
          </Card>

          {/* Sampling Card */}
          <Card variant="crt" title="Sampling" className="status-card">
            <div className="status-card-content">
              <div className="status-row">
                <span className="status-label">Base Rate</span>
                <span className="status-value">{Math.round(assessmentState.data.sampling.base_rate * 100)}%</span>
              </div>
              <div className="status-row">
                <span className="status-label">Burn-in Sessions</span>
                <span className="status-value">{assessmentState.data.sampling.burnin_sessions}</span>
              </div>
            </div>
          </Card>

          {/* Activity Card */}
          <Card variant="crt" title="Activity" className="status-card">
            <div className="status-card-content">
              <div className="status-row">
                <span className="status-label">Active Sessions</span>
                <span className="status-value highlight">{assessmentState.data.activity.active_sessions}</span>
              </div>
              <div className="status-row">
                <span className="status-label">Events Stored</span>
                <span className="status-value highlight">{assessmentState.data.activity.events_stored}</span>
              </div>
              {assessmentState.data.activity.intervention_count !== undefined && assessmentState.data.activity.intervention_count > 0 && (
                <div className="status-row">
                  <span className="status-label">Interventions</span>
                  <span className="status-value warning">{assessmentState.data.activity.intervention_count}</span>
                </div>
              )}
            </div>
            {assessmentState.data.activity.sessions.length > 0 && (
              <div className="status-sessions">
                <span className="status-label">Sessions</span>
                <ul className="session-list">
                  {assessmentState.data.activity.sessions.slice(0, 5).map((session) => (
                    <li key={session} className="session-item">
                      {session}
                    </li>
                  ))}
                  {assessmentState.data.activity.sessions.length > 5 && (
                    <li className="session-item more">
                      <Link to="/groove/history">+{assessmentState.data.activity.sessions.length - 5} more</Link>
                    </li>
                  )}
                </ul>
              </div>
            )}
          </Card>

          {/* Tier Distribution Card (from stats) */}
          {statsState.status === 'success' && statsState.data.total_assessments > 0 && (
            <Card variant="crt" title="Tier Distribution" className="status-card tier-card">
              <div className="tier-content">
                <div className="tier-total">
                  <span className="total-value">{statsState.data.total_assessments}</span>
                  <span className="total-label">Total Assessments</span>
                </div>
                <div className="tier-chart">
                  <TierPieChart data={pieData} />
                  <div className="tier-legend">
                    {pieData.map((item) => (
                      <div key={item.tier} className="legend-item">
                        <span
                          className="legend-color"
                          style={{ backgroundColor: tierColors[item.tier] }}
                        />
                        <span className="legend-label">{item.tier}</span>
                        <span className="legend-value">{item.count}</span>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </Card>
          )}
        </div>
      )}
    </div>
  );
}
