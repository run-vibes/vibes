import { StrategyBar } from './StrategyBar';
import type { CategoryDistribution } from '../../../hooks/useDashboard';
import './DistributionCard.css';

export interface DistributionCardProps {
  distribution: CategoryDistribution;
}

export function DistributionCard({ distribution }: DistributionCardProps) {
  return (
    <article className="distribution-card">
      <header className="distribution-card__header">
        <h4 className="distribution-card__title">{distribution.label}</h4>
        <span className="distribution-card__sessions">{distribution.session_count} sessions</span>
      </header>

      <div className="distribution-card__bars">
        {distribution.weights.map((w) => (
          <StrategyBar key={w.strategy} strategy={w.strategy} weight={w.weight} />
        ))}
      </div>
    </article>
  );
}
