// web-ui/src/pages/groove/TrendsPage.tsx
// Detailed session trends analytics page
import { Card, PageHeader } from '@vibes/design-system';
import { useDashboardOverview } from '../../hooks';
import { Sparkline } from '../../components/charts/Sparkline';
import type { TrendDirection } from '../../hooks/useDashboard';
import '../dashboard/DashboardPages.css';
import './TrendsPage.css';

const TREND_LABELS: Record<TrendDirection, string> = {
  rising: 'Improving',
  falling: 'Declining',
  stable: 'Stable',
};

const TREND_ICONS: Record<TrendDirection, string> = {
  rising: '↑',
  falling: '↓',
  stable: '→',
};

export function TrendsPage() {
  const { data, isLoading, isError } = useDashboardOverview();

  if (isLoading) {
    return (
      <div className="dashboard-page trends-page">
        <PageHeader title="TRENDS" />
        <p className="dashboard-loading">Loading trends...</p>
      </div>
    );
  }

  if (isError || !data) {
    return (
      <div className="dashboard-page trends-page">
        <PageHeader title="TRENDS" />
        <Card variant="crt" title="Connection Error">
          <div className="error-state">
            <span className="error-state__icon">⚠</span>
            <p className="error-state__message">Failed to load trends data.</p>
            <p className="error-state__hint">Check that the vibes daemon is running and try again.</p>
          </div>
        </Card>
      </div>
    );
  }

  const { trends } = data;
  const sparklineData = trends.sparkline_data || [];

  // Calculate period metrics
  const currentPeriodSessions = trends.session_count;
  const periodDays = trends.period_days || 7;
  const avgSessionsPerDay = currentPeriodSessions / periodDays;

  // Calculate sparkline stats for the expanded view
  const maxValue = sparklineData.length > 0 ? Math.max(...sparklineData) : 0;
  const minValue = sparklineData.length > 0 ? Math.min(...sparklineData) : 0;
  const avgValue = sparklineData.length > 0
    ? sparklineData.reduce((a, b) => a + b, 0) / sparklineData.length
    : 0;

  return (
    <div className="dashboard-page trends-page">
      <PageHeader title="TRENDS" />

      {/* Main Trend Overview */}
      <Card variant="crt" title="Session Trend Overview" className="trends-page__overview">
        <div className="trends-page__hero">
          <div className="trends-page__sparkline-large">
            <Sparkline
              data={sparklineData}
              width={400}
              height={120}
              showArea
            />
          </div>
          <div className="trends-page__hero-stats">
            <div className="trends-page__primary-stat">
              <span className={`trends-page__trend-indicator trends-page__trend-indicator--${trends.trend_direction}`}>
                {TREND_ICONS[trends.trend_direction]}
              </span>
              <span className="trends-page__primary-value">{trends.improvement_percent}%</span>
              <span className="trends-page__primary-label">improvement</span>
            </div>
            <div className="trends-page__trend-status">
              <span className={`trends-page__status-badge trends-page__status-badge--${trends.trend_direction}`}>
                {TREND_LABELS[trends.trend_direction]}
              </span>
            </div>
          </div>
        </div>
      </Card>

      {/* Metrics Grid */}
      <div className="trends-page__grid">
        {/* Period Summary */}
        <Card variant="crt" title="Period Summary">
          <div className="trends-page__metrics">
            <div className="trends-page__metric">
              <span className="trends-page__metric-value">{currentPeriodSessions}</span>
              <span className="trends-page__metric-label">Total Sessions</span>
            </div>
            <div className="trends-page__metric">
              <span className="trends-page__metric-value">{periodDays}</span>
              <span className="trends-page__metric-label">Day Period</span>
            </div>
            <div className="trends-page__metric">
              <span className="trends-page__metric-value">{avgSessionsPerDay.toFixed(1)}</span>
              <span className="trends-page__metric-label">Avg/Day</span>
            </div>
          </div>
        </Card>

        {/* Sparkline Stats */}
        <Card variant="crt" title="Daily Activity Range">
          <div className="trends-page__metrics">
            <div className="trends-page__metric">
              <span className="trends-page__metric-value trends-page__metric-value--high">{maxValue}</span>
              <span className="trends-page__metric-label">Peak</span>
            </div>
            <div className="trends-page__metric">
              <span className="trends-page__metric-value">{avgValue.toFixed(1)}</span>
              <span className="trends-page__metric-label">Average</span>
            </div>
            <div className="trends-page__metric">
              <span className="trends-page__metric-value trends-page__metric-value--low">{minValue}</span>
              <span className="trends-page__metric-label">Low</span>
            </div>
          </div>
        </Card>

        {/* Improvement Breakdown */}
        <Card variant="crt" title="Improvement Metrics">
          <div className="trends-page__breakdown">
            <div className="trends-page__breakdown-row">
              <span className="trends-page__breakdown-label">Trend Direction</span>
              <span className={`trends-page__breakdown-value trends-page__breakdown-value--${trends.trend_direction}`}>
                {TREND_ICONS[trends.trend_direction]} {TREND_LABELS[trends.trend_direction]}
              </span>
            </div>
            <div className="trends-page__breakdown-row">
              <span className="trends-page__breakdown-label">Improvement Rate</span>
              <span className="trends-page__breakdown-value">{trends.improvement_percent}%</span>
            </div>
            <div className="trends-page__breakdown-row">
              <span className="trends-page__breakdown-label">Data Points</span>
              <span className="trends-page__breakdown-value">{sparklineData.length}</span>
            </div>
          </div>
        </Card>
      </div>

      {/* Daily Breakdown */}
      {sparklineData.length > 0 && (
        <Card variant="crt" title="Daily Activity" className="trends-page__daily">
          <div className="trends-page__daily-grid">
            {sparklineData.map((value, index) => {
              const dayLabel = `Day ${index + 1}`;
              const heightPercent = maxValue > 0 ? (value / maxValue) * 100 : 0;
              return (
                <div key={index} className="trends-page__daily-bar">
                  <div className="trends-page__bar-container">
                    <div
                      className="trends-page__bar-fill"
                      style={{ height: `${heightPercent}%` }}
                    />
                  </div>
                  <span className="trends-page__bar-value">{value}</span>
                  <span className="trends-page__bar-label">{dayLabel}</span>
                </div>
              );
            })}
          </div>
        </Card>
      )}
    </div>
  );
}
