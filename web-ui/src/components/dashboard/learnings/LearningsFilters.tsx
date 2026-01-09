import type { LearningsFilter, LearningCategory, LearningStatus } from '../../../hooks/useDashboard';
import './LearningsFilters.css';

export type SortOption = 'value' | 'confidence' | 'usage' | 'recency';

export interface LearningsFiltersProps {
  value?: LearningsFilter;
  sortBy?: SortOption;
  onChange: (filters: LearningsFilter) => void;
  onSortChange?: (sort: SortOption) => void;
}

const SCOPE_OPTIONS = [
  { value: '', label: 'All Scopes' },
  { value: 'project', label: 'Project' },
  { value: 'user', label: 'User' },
  { value: 'global', label: 'Global' },
] as const;

const CATEGORY_OPTIONS: { value: LearningCategory | ''; label: string }[] = [
  { value: '', label: 'All Categories' },
  { value: 'Correction', label: 'Correction' },
  { value: 'Workflow', label: 'Workflow' },
  { value: 'Preference', label: 'Preference' },
  { value: 'Pattern', label: 'Pattern' },
  { value: 'Optimization', label: 'Optimization' },
  { value: 'Automation', label: 'Automation' },
];

const STATUS_OPTIONS: { value: LearningStatus | ''; label: string }[] = [
  { value: '', label: 'All Statuses' },
  { value: 'active', label: 'Active' },
  { value: 'disabled', label: 'Disabled' },
  { value: 'under_review', label: 'Under Review' },
  { value: 'deprecated', label: 'Deprecated' },
];

const SORT_OPTIONS: { value: SortOption; label: string }[] = [
  { value: 'value', label: 'Value' },
  { value: 'confidence', label: 'Confidence' },
  { value: 'usage', label: 'Usage' },
  { value: 'recency', label: 'Recency' },
];

function getScopeValue(scope?: LearningsFilter['scope']): string {
  if (!scope) return '';
  if (scope.Project) return 'project';
  if (scope.User) return 'user';
  if (scope.Enterprise) return 'global';
  return '';
}

export function LearningsFilters({
  value,
  sortBy = 'recency',
  onChange,
  onSortChange,
}: LearningsFiltersProps) {
  const handleScopeChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const scopeValue = e.target.value;
    const newFilters: LearningsFilter = { ...value };

    if (scopeValue === '') {
      delete newFilters.scope;
    } else if (scopeValue === 'project') {
      newFilters.scope = { Project: '*' };
    } else if (scopeValue === 'user') {
      newFilters.scope = { User: '*' };
    } else if (scopeValue === 'global') {
      newFilters.scope = { Enterprise: '*' };
    }

    onChange(newFilters);
  };

  const handleCategoryChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const category = e.target.value as LearningCategory | '';
    const newFilters: LearningsFilter = { ...value };

    if (category === '') {
      delete newFilters.category;
    } else {
      newFilters.category = category;
    }

    onChange(newFilters);
  };

  const handleStatusChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const status = e.target.value as LearningStatus | '';
    const newFilters: LearningsFilter = { ...value };

    if (status === '') {
      delete newFilters.status;
    } else {
      newFilters.status = status;
    }

    onChange(newFilters);
  };

  const handleSortChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    onSortChange?.(e.target.value as SortOption);
  };

  return (
    <div className="learnings-filters">
      <div className="filter-group">
        <label htmlFor="scope-filter">Scope</label>
        <select
          id="scope-filter"
          value={getScopeValue(value?.scope)}
          onChange={handleScopeChange}
        >
          {SCOPE_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
      </div>

      <div className="filter-group">
        <label htmlFor="category-filter">Category</label>
        <select
          id="category-filter"
          value={value?.category ?? ''}
          onChange={handleCategoryChange}
        >
          {CATEGORY_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
      </div>

      <div className="filter-group">
        <label htmlFor="status-filter">Status</label>
        <select
          id="status-filter"
          value={value?.status ?? ''}
          onChange={handleStatusChange}
        >
          {STATUS_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
      </div>

      <div className="filter-group">
        <label htmlFor="sort-filter">Sort</label>
        <select
          id="sort-filter"
          value={sortBy}
          onChange={handleSortChange}
        >
          {SORT_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
      </div>
    </div>
  );
}
