import { Link } from '@tanstack/react-router';
import { Panel, Metric } from '@vibes/design-system';
import type { StrategyDistributionsData } from '../../hooks/useDashboard';
import './DashboardCards.css';

export interface StrategyCardProps {
  data?: StrategyDistributionsData;
}

export function StrategyCard({ data }: StrategyCardProps) {
  const distributions = data?.distributions ?? [];
  const specializedCount = data?.specialized_count ?? 0;
  const totalLearnings = data?.total_learnings ?? 0;

  return (
    <Panel variant="crt" title="Strategy" className="dashboard-card">
      <div className="dashboard-card__metrics">
        <Metric label="Distributions" value={distributions.length} />
        <Metric label="Specialized" value={`${specializedCount}/${totalLearnings}`} />
      </div>

      {distributions.length > 0 && (
        <div className="dashboard-card__list">
          <span className="list-label">Active Categories:</span>
          <ul className="category-list">
            {distributions.slice(0, 3).map((dist) => (
              <li key={dist.category_key} className="category-item">
                <span className="category-label">{dist.label}</span>
                <span className="category-sessions">{dist.session_count} sessions</span>
              </li>
            ))}
          </ul>
        </div>
      )}

      <Link to="/groove/dashboard/strategy" className="dashboard-card__link">
        View →
      </Link>
    </Panel>
  );
}
