import { Link } from '@tanstack/react-router';
import { Card } from '@vibes/design-system';
import { Sparkline } from '../charts/Sparkline';
import './TrendCard.css';
import './DashboardCards.css';

export type TrendDirection = 'rising' | 'falling' | 'stable';

export interface SecondaryMetric {
  label: string;
  value: string;
}

export interface TrendCardProps {
  title: string;
  primaryValue: string;
  primaryLabel: string;
  trendDirection: TrendDirection;
  secondaryMetrics?: SecondaryMetric[];
  sparklineData?: number[];
  href?: string;
}

const TREND_INDICATORS: Record<TrendDirection, string> = {
  rising: '↑',
  falling: '↓',
  stable: '→',
};

export function TrendCard({
  title,
  primaryValue,
  primaryLabel,
  trendDirection,
  secondaryMetrics,
  sparklineData,
  href,
}: TrendCardProps) {
  return (
    <Card
      variant="crt"
      title={title}
      className="trend-card"
      footer={
        href ? (
          <Link to={href} className="card-footer-link">
            View →
          </Link>
        ) : undefined
      }
    >
      {sparklineData && sparklineData.length > 0 && (
        <div className="trend-card__sparkline" data-testid="sparkline-placeholder">
          <Sparkline data={sparklineData} width={120} height={32} showArea />
        </div>
      )}

      <div className="trend-card__primary">
        <span className={`trend-indicator trend-indicator--${trendDirection}`}>
          {TREND_INDICATORS[trendDirection]}
        </span>
        <span className="trend-card__value">{primaryValue}</span>
        <span className="trend-card__label">{primaryLabel}</span>
      </div>

      {secondaryMetrics && secondaryMetrics.length > 0 && (
        <div className="trend-card__secondary">
          {secondaryMetrics.map((metric) => (
            <div key={metric.label} className="secondary-metric">
              <span className="secondary-metric__label">{metric.label}</span>
              <span className="secondary-metric__value">{metric.value}</span>
            </div>
          ))}
        </div>
      )}
    </Card>
  );
}
