import type { AttributionEntry } from '../../../hooks/useDashboard';
import './ContributorCard.css';

export interface ContributorCardProps {
  entry: AttributionEntry;
  rank: number;
  onClick?: (learningId: string) => void;
}

function formatValue(value: number): string {
  const sign = value >= 0 ? '+' : '';
  return `${sign}${value.toFixed(2)}`;
}

function formatPercent(value: number): string {
  return `${Math.round(value * 100)}%`;
}

export function ContributorCard({ entry, rank, onClick }: ContributorCardProps) {
  const handleClick = () => {
    onClick?.(entry.learning_id);
  };

  // Normalize value for progress bar (assuming max value around 0.5)
  const normalizedValue = Math.min(Math.abs(entry.estimated_value) / 0.5, 1);

  return (
    <div
      className="contributor-card"
      onClick={handleClick}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => e.key === 'Enter' && handleClick()}
    >
      <span className="contributor-card__rank">{rank}</span>

      <div className="contributor-card__content">
        <p className="contributor-card__title">{entry.content}</p>
        <div className="contributor-card__meta">
          <span className="contributor-card__confidence">{formatPercent(entry.confidence)}</span>
          <span className="contributor-card__sessions">{entry.session_count} sessions</span>
        </div>
      </div>

      <div className="contributor-card__value">
        <span className="contributor-card__value-text">{formatValue(entry.estimated_value)}</span>
        <div
          className="contributor-card__value-bar"
          role="progressbar"
          aria-valuenow={normalizedValue * 100}
          aria-valuemin={0}
          aria-valuemax={100}
        >
          <div
            className="contributor-card__value-bar-fill"
            style={{ width: `${normalizedValue * 100}%` }}
          />
        </div>
      </div>
    </div>
  );
}
