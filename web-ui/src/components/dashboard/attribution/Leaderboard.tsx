import { EmptyState } from '@vibes/design-system';
import { ContributorCard } from './ContributorCard';
import { NegativeImpact } from './NegativeImpact';
import { AblationCoverage } from './AblationCoverage';
import type {
  AttributionEntry,
  AblationCoverage as AblationCoverageType,
} from '../../../hooks/useDashboard';
import './Leaderboard.css';

export interface LeaderboardProps {
  contributors: AttributionEntry[];
  negativeImpact: AttributionEntry[];
  ablationCoverage: AblationCoverageType;
  period?: number;
  onPeriodChange?: (days: number) => void;
  onContributorClick?: (learningId: string) => void;
  onDisableLearning?: (learningId: string) => void;
}

export function Leaderboard({
  contributors,
  negativeImpact,
  ablationCoverage,
  period = 7,
  onPeriodChange,
  onContributorClick,
  onDisableLearning,
}: LeaderboardProps) {
  const handlePeriodChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    onPeriodChange?.(parseInt(e.target.value, 10));
  };

  return (
    <div className="leaderboard">
      <header className="leaderboard__header">
        <h3 className="leaderboard__title">Top Contributors</h3>
        <label className="leaderboard__period">
          <span className="visually-hidden">Period</span>
          <select
            className="leaderboard__period-select"
            value={period}
            onChange={handlePeriodChange}
            aria-label="Period"
          >
            <option value="7">Last 7 days</option>
            <option value="30">Last 30 days</option>
            <option value="90">Last 90 days</option>
          </select>
        </label>
      </header>

      {contributors.length === 0 ? (
        <EmptyState message="No attribution data available yet" size="sm" />
      ) : (
        <div className="leaderboard__contributors">
          {contributors.map((entry, index) => (
            <ContributorCard
              key={entry.learning_id}
              entry={entry}
              rank={index + 1}
              onClick={onContributorClick}
            />
          ))}
        </div>
      )}

      <NegativeImpact entries={negativeImpact} onDisable={onDisableLearning} />

      <AblationCoverage coverage={ablationCoverage} />
    </div>
  );
}
