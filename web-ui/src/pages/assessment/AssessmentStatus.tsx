// web-ui/src/pages/assessment/AssessmentStatus.tsx
import { useEffect, useState } from 'react';
import { Group } from '@visx/group';
import { Pie } from '@visx/shape';
import { scaleOrdinal } from '@visx/scale';
import './AssessmentStatus.css';

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

interface SessionStats {
  session_id: string;
  assessment_count: number;
}

interface AssessmentStatsResponse {
  tier_distribution: TierDistribution;
  total_assessments: number;
  top_sessions: SessionStats[];
}

interface ErrorResponse {
  error: string;
  code: string;
}

type StatusFetchState =
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

export function AssessmentStatus() {
  const [statusState, setStatusState] = useState<StatusFetchState>({ status: 'loading' });
  const [statsState, setStatsState] = useState<StatsFetchState>({ status: 'loading' });

  useEffect(() => {
    async function fetchStatus() {
      try {
        const response = await fetch('/api/groove/assess/status');

        if (!response.ok) {
          const errorData: ErrorResponse = await response.json();
          if (errorData.code === 'NOT_INITIALIZED') {
            setStatusState({ status: 'error', error: 'Assessment processor not initialized' });
          } else {
            setStatusState({ status: 'error', error: errorData.error || 'Failed to load status' });
          }
          return;
        }

        const data: AssessmentStatusResponse = await response.json();
        setStatusState({ status: 'success', data });
      } catch {
        setStatusState({ status: 'error', error: 'Failed to connect to server' });
      }
    }

    async function fetchStats() {
      try {
        const response = await fetch('/api/groove/assess/stats');

        if (!response.ok) {
          const errorData: ErrorResponse = await response.json();
          if (errorData.code === 'NOT_INITIALIZED') {
            setStatsState({ status: 'error', error: 'Assessment processor not initialized' });
          } else {
            setStatsState({ status: 'error', error: errorData.error || 'Failed to load stats' });
          }
          return;
        }

        const data: AssessmentStatsResponse = await response.json();
        setStatsState({ status: 'success', data });
      } catch {
        setStatsState({ status: 'error', error: 'Failed to connect to server' });
      }
    }

    fetchStatus();
    fetchStats();
  }, []);

  // Show loading if either is loading
  if (statusState.status === 'loading' || statsState.status === 'loading') {
    return (
      <div className="status-page">
        <div className="status-loading">Loading...</div>
      </div>
    );
  }

  // Show error if status failed (primary data)
  if (statusState.status === 'error') {
    return (
      <div className="status-page">
        <div className="status-error">
          <h2>Error</h2>
          <p>{statusState.error}</p>
        </div>
      </div>
    );
  }

  const { circuit_breaker, sampling, activity } = statusState.data;

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
      {/* Header */}
      <div className="status-header">
        <h1 className="status-title">STATUS</h1>
      </div>

      <div className="status-grid">
        {/* Circuit Breaker Card */}
        <div className="status-card">
          <h3 className="status-card-title">Circuit Breaker</h3>
          <div className="status-card-content">
            <div className="status-row">
              <span className="status-label">Status</span>
              <span className={`status-value ${circuit_breaker.enabled ? 'enabled' : 'disabled'}`}>
                {circuit_breaker.enabled ? 'Enabled' : 'Disabled'}
              </span>
            </div>
            <div className="status-row">
              <span className="status-label">Cooldown</span>
              <span className="status-value">{circuit_breaker.cooldown_seconds}s</span>
            </div>
            <div className="status-row">
              <span className="status-label">Max Interventions</span>
              <span className="status-value">{circuit_breaker.max_interventions_per_session}</span>
            </div>
          </div>
        </div>

        {/* Sampling Card */}
        <div className="status-card">
          <h3 className="status-card-title">Sampling</h3>
          <div className="status-card-content">
            <div className="status-row">
              <span className="status-label">Base Rate</span>
              <span className="status-value">{Math.round(sampling.base_rate * 100)}%</span>
            </div>
            <div className="status-row">
              <span className="status-label">Burn-in Sessions</span>
              <span className="status-value">{sampling.burnin_sessions}</span>
            </div>
          </div>
        </div>

        {/* Activity Card */}
        <div className="status-card">
          <h3 className="status-card-title">Activity</h3>
          <div className="status-card-content">
            <div className="status-row">
              <span className="status-label">Active Sessions</span>
              <span className="status-value highlight">{activity.active_sessions}</span>
            </div>
            <div className="status-row">
              <span className="status-label">Events Stored</span>
              <span className="status-value highlight">{activity.events_stored}</span>
            </div>
          </div>
          {activity.sessions.length > 0 && (
            <div className="status-sessions">
              <span className="status-label">Sessions</span>
              <ul className="session-list">
                {activity.sessions.slice(0, 5).map((session) => (
                  <li key={session} className="session-item">
                    {session}
                  </li>
                ))}
                {activity.sessions.length > 5 && (
                  <li className="session-item more">+{activity.sessions.length - 5} more</li>
                )}
              </ul>
            </div>
          )}
        </div>

        {/* Tier Distribution Card (from stats) */}
        {statsState.status === 'success' && statsState.data.total_assessments > 0 && (
          <div className="status-card tier-card">
            <h3 className="status-card-title">Tier Distribution</h3>
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
          </div>
        )}
      </div>
    </div>
  );
}
