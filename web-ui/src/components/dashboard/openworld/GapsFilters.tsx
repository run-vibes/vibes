import type {
  OpenWorldGapsFilter,
  GapSeverity,
  GapStatus,
  GapCategory,
} from '../../../hooks/useDashboard';
import './GapsFilters.css';

export interface GapsFiltersProps {
  value?: OpenWorldGapsFilter;
  onChange: (filters: OpenWorldGapsFilter) => void;
}

const SEVERITY_OPTIONS: { value: GapSeverity | ''; label: string }[] = [
  { value: '', label: 'All Severities' },
  { value: 'Critical', label: 'Critical' },
  { value: 'High', label: 'High' },
  { value: 'Medium', label: 'Medium' },
  { value: 'Low', label: 'Low' },
];

const STATUS_OPTIONS: { value: GapStatus | ''; label: string }[] = [
  { value: '', label: 'All Statuses' },
  { value: 'Detected', label: 'Detected' },
  { value: 'Confirmed', label: 'Confirmed' },
  { value: 'InProgress', label: 'In Progress' },
  { value: 'Resolved', label: 'Resolved' },
  { value: 'Dismissed', label: 'Dismissed' },
];

const CATEGORY_OPTIONS: { value: GapCategory | ''; label: string }[] = [
  { value: '', label: 'All Categories' },
  { value: 'MissingKnowledge', label: 'Missing Knowledge' },
  { value: 'IncorrectPattern', label: 'Incorrect Pattern' },
  { value: 'ContextMismatch', label: 'Context Mismatch' },
  { value: 'ToolGap', label: 'Tool Gap' },
];

export function GapsFilters({ value, onChange }: GapsFiltersProps) {
  const handleSeverityChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const severity = e.target.value as GapSeverity | '';
    const newFilters: OpenWorldGapsFilter = { ...value };

    if (severity === '') {
      delete newFilters.severity;
    } else {
      newFilters.severity = severity;
    }

    onChange(newFilters);
  };

  const handleStatusChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const status = e.target.value as GapStatus | '';
    const newFilters: OpenWorldGapsFilter = { ...value };

    if (status === '') {
      delete newFilters.status;
    } else {
      newFilters.status = status;
    }

    onChange(newFilters);
  };

  const handleCategoryChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const category = e.target.value as GapCategory | '';
    const newFilters: OpenWorldGapsFilter = { ...value };

    if (category === '') {
      delete newFilters.category;
    } else {
      newFilters.category = category;
    }

    onChange(newFilters);
  };

  return (
    <div className="gaps-filters">
      <div className="filter-group">
        <label htmlFor="severity-filter">Severity</label>
        <select
          id="severity-filter"
          value={value?.severity ?? ''}
          onChange={handleSeverityChange}
        >
          {SEVERITY_OPTIONS.map((opt) => (
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
    </div>
  );
}
