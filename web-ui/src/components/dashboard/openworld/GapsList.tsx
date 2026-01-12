import type { GapBrief, OpenWorldGapsFilter } from '../../../hooks/useDashboard';
import { GapsFilters } from './GapsFilters';
import { GapItem } from './GapItem';
import './GapsList.css';

export interface GapsListProps {
  gaps?: GapBrief[];
  total?: number;
  filters?: OpenWorldGapsFilter;
  selectedId?: string;
  isLoading?: boolean;
  onFiltersChange: (filters: OpenWorldGapsFilter) => void;
  onSelectGap: (id: string) => void;
}

export function GapsList({
  gaps,
  total,
  filters,
  selectedId,
  isLoading,
  onFiltersChange,
  onSelectGap,
}: GapsListProps) {
  return (
    <div className="gaps-list">
      <GapsFilters value={filters} onChange={onFiltersChange} />

      <div className="gaps-list__header">
        <span className="gaps-list__count">
          {isLoading ? 'Loading...' : `${total ?? 0} gaps`}
        </span>
      </div>

      <div className="gaps-list__items">
        {isLoading && (
          <div className="gaps-list__loading">
            <span>Loading gaps...</span>
          </div>
        )}

        {!isLoading && (!gaps || gaps.length === 0) && (
          <div className="gaps-list__empty" data-testid="gaps-list-empty">
            <span className="gaps-list__empty-icon">○</span>
            <span>No capability gaps detected</span>
          </div>
        )}

        {!isLoading &&
          gaps?.map((gap) => (
            <GapItem
              key={gap.id}
              gap={gap}
              isSelected={gap.id === selectedId}
              onClick={() => onSelectGap(gap.id)}
            />
          ))}
      </div>
    </div>
  );
}
