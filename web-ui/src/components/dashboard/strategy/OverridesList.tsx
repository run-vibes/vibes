import { OverrideItem } from './OverrideItem';
import type { LearningOverrideEntry } from '../../../hooks/useDashboard';
import './OverridesList.css';

export type OverrideFilter = 'all' | 'specialized' | 'inheriting';

export interface OverridesListProps {
  overrides: LearningOverrideEntry[];
  specializedCount?: number;
  totalCount?: number;
  filter?: OverrideFilter;
  onFilterChange?: (filter: OverrideFilter) => void;
}

export function OverridesList({
  overrides,
  specializedCount,
  totalCount,
  filter = 'all',
  onFilterChange,
}: OverridesListProps) {
  const handleFilterChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    onFilterChange?.(e.target.value as OverrideFilter);
  };

  return (
    <div className="overrides-list">
      <header className="overrides-list__header">
        <div className="overrides-list__title">
          <h3>Learning Overrides</h3>
          {specializedCount !== undefined && totalCount !== undefined && (
            <span className="overrides-list__count">
              {specializedCount} of {totalCount} specialized
            </span>
          )}
        </div>
        <label className="overrides-list__filter">
          <span className="visually-hidden">Filter</span>
          <select
            className="overrides-list__select"
            value={filter}
            onChange={handleFilterChange}
            aria-label="Filter"
          >
            <option value="all">All Overrides</option>
            <option value="specialized">Specialized Only</option>
            <option value="inheriting">Inheriting Only</option>
          </select>
        </label>
      </header>

      {overrides.length === 0 ? (
        <p className="overrides-list__empty">No overrides found.</p>
      ) : (
        <div className="overrides-list__items">
          {overrides.map((override) => (
            <OverrideItem key={override.learning_id} override={override} />
          ))}
        </div>
      )}
    </div>
  );
}
