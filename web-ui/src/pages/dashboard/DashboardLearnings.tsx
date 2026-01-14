import { useState, useCallback } from 'react';
import { PageHeader } from '@vibes/design-system';
import { useDashboardLearnings, useDashboardLearningDetail } from '../../hooks';
import type { LearningsFilter } from '../../hooks/useDashboard';
import { LearningsFilters, type SortOption } from '../../components/dashboard/learnings/LearningsFilters';
import { LearningsList } from '../../components/dashboard/learnings/LearningsList';
import { LearningDetail } from '../../components/dashboard/learnings/LearningDetail';
import './DashboardLearnings.css';

export function DashboardLearnings() {
  const [filters, setFilters] = useState<LearningsFilter>({});
  const [sortBy, setSortBy] = useState<SortOption>('recency');
  const [selectedId, setSelectedId] = useState<string | undefined>();

  const {
    data: learningsData,
    isLoading: learningsLoading,
    isError: learningsError,
  } = useDashboardLearnings(filters);

  const {
    data: detailData,
    isLoading: detailLoading,
  } = useDashboardLearningDetail(selectedId);

  const handleFilterChange = useCallback((newFilters: LearningsFilter) => {
    setFilters(newFilters);
    setSelectedId(undefined); // Clear selection when filters change
  }, []);

  const handleSortChange = useCallback((newSort: SortOption) => {
    setSortBy(newSort);
  }, []);

  const handleSelect = useCallback((id: string) => {
    setSelectedId(id);
  }, []);

  // Sort learnings client-side for now
  const sortedLearnings = [...(learningsData?.learnings ?? [])].sort((a, b) => {
    switch (sortBy) {
      case 'value':
        return b.estimated_value - a.estimated_value;
      case 'recency':
        return new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
      default:
        return 0;
    }
  });

  if (learningsError) {
    return (
      <div className="dashboard-page dashboard-learnings">
        <PageHeader title="LEARNINGS" />
        <p className="error-text">Failed to load learnings. Please try again.</p>
      </div>
    );
  }

  return (
    <div className="dashboard-page dashboard-learnings">
      <PageHeader title="LEARNINGS" />
      <div className="dashboard-learnings__layout">
        {/* Left Panel: Filters + List */}
        <div className="dashboard-learnings__left">
          <LearningsFilters
            value={filters}
            sortBy={sortBy}
            onChange={handleFilterChange}
            onSortChange={handleSortChange}
          />
          <LearningsList
            learnings={sortedLearnings}
            selectedId={selectedId}
            isLoading={learningsLoading}
            onSelect={handleSelect}
          />
        </div>

        {/* Right Panel: Detail */}
        <div className="dashboard-learnings__right">
          <LearningDetail
            data={detailData}
            isLoading={detailLoading}
          />
        </div>
      </div>
    </div>
  );
}
