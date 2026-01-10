import type { AttributionEntry } from '../../../hooks/useDashboard';
import { LearningStatusBadge } from '../learnings/LearningStatusBadge';
import './NegativeImpact.css';

export interface NegativeImpactProps {
  entries: AttributionEntry[];
  onDisable?: (learningId: string) => void;
}

function formatValue(value: number): string {
  const sign = value >= 0 ? '+' : '';
  return `${sign}${value.toFixed(2)}`;
}

export function NegativeImpact({ entries, onDisable }: NegativeImpactProps) {
  if (entries.length === 0) {
    return null;
  }

  return (
    <section className="negative-impact">
      <h4 className="negative-impact__title">Negative Impact</h4>
      <ul className="negative-impact__list">
        {entries.map((entry) => (
          <li key={entry.learning_id} className="negative-impact__item">
            <div className="negative-impact__content">
              <span className="negative-impact__name">{entry.content}</span>
              <div className="negative-impact__meta">
                <span className="negative-impact__value">{formatValue(entry.estimated_value)}</span>
                <LearningStatusBadge status={entry.status} />
              </div>
            </div>
            {entry.status === 'active' && (
              <button
                type="button"
                className="negative-impact__action"
                onClick={() => onDisable?.(entry.learning_id)}
              >
                Disable
              </button>
            )}
          </li>
        ))}
      </ul>
    </section>
  );
}
