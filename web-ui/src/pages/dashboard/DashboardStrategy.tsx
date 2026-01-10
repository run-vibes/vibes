import { useState } from 'react';
import {
  useDashboardStrategyDistributions,
  useDashboardStrategyOverrides,
} from '../../hooks/useDashboard';
import {
  StrategyTabs,
  type StrategyTab,
} from '../../components/dashboard/strategy/StrategyTabs';
import { DistributionCard } from '../../components/dashboard/strategy/DistributionCard';
import { OverridesList, type OverrideFilter } from '../../components/dashboard/strategy/OverridesList';
import './DashboardStrategy.css';

export function DashboardStrategy() {
  const [activeTab, setActiveTab] = useState<StrategyTab>('distributions');
  const [overrideFilter, setOverrideFilter] = useState<OverrideFilter>('all');

  const distributionsQuery = useDashboardStrategyDistributions();
  const overridesQuery = useDashboardStrategyOverrides();

  const isLoading =
    activeTab === 'distributions' ? distributionsQuery.isLoading : overridesQuery.isLoading;
  const isError =
    activeTab === 'distributions' ? distributionsQuery.isError : overridesQuery.isError;

  // Filter overrides based on selected filter
  const filteredOverrides = overridesQuery.data?.overrides.filter((o) => {
    if (overrideFilter === 'specialized') return o.is_specialized;
    if (overrideFilter === 'inheriting') return !o.is_specialized;
    return true;
  });

  const specializedCount = overridesQuery.data?.overrides.filter((o) => o.is_specialized).length ?? 0;
  const totalOverrides = overridesQuery.data?.overrides.length ?? 0;

  return (
    <div className="dashboard-page dashboard-strategy">
      <StrategyTabs activeTab={activeTab} onTabChange={setActiveTab} />

      {isLoading && <p className="dashboard-strategy__loading">Loading...</p>}

      {isError && (
        <p className="dashboard-strategy__error">
          Error loading data. Please try again.
        </p>
      )}

      {!isLoading && !isError && activeTab === 'distributions' && distributionsQuery.data && (
        <div className="dashboard-strategy__distributions">
          <header className="dashboard-strategy__header">
            <h3>Category Distributions</h3>
            <span className="dashboard-strategy__summary">
              {distributionsQuery.data.specialized_count} of {distributionsQuery.data.total_learnings} learnings specialized
            </span>
          </header>
          <div className="dashboard-strategy__grid">
            {distributionsQuery.data.distributions.map((dist) => (
              <DistributionCard key={dist.category_key} distribution={dist} />
            ))}
          </div>
        </div>
      )}

      {!isLoading && !isError && activeTab === 'overrides' && overridesQuery.data && (
        <OverridesList
          overrides={filteredOverrides ?? []}
          specializedCount={specializedCount}
          totalCount={totalOverrides}
          filter={overrideFilter}
          onFilterChange={setOverrideFilter}
        />
      )}
    </div>
  );
}
