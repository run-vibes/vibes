import { useState } from 'react';
import {
  useDashboardAttribution,
  useDashboardSessionTimeline,
} from '../../hooks/useDashboard';
import type { SessionOutcome } from '../../hooks/useDashboard';
import {
  AttributionTabs,
  type AttributionTab,
} from '../../components/dashboard/attribution/AttributionTabs';
import { Leaderboard } from '../../components/dashboard/attribution/Leaderboard';
import { SessionTimeline } from '../../components/dashboard/attribution/SessionTimeline';
import './DashboardAttribution.css';

export function DashboardAttribution() {
  const [activeTab, setActiveTab] = useState<AttributionTab>('leaderboard');
  const [period, setPeriod] = useState(7);
  const [outcomeFilter, setOutcomeFilter] = useState<SessionOutcome | 'all'>('all');

  const attributionQuery = useDashboardAttribution(period);
  const timelineQuery = useDashboardSessionTimeline(
    period,
    outcomeFilter === 'all' ? undefined : outcomeFilter
  );

  const handlePeriodChange = (days: number) => {
    setPeriod(days);
  };

  const handleOutcomeFilter = (outcome: SessionOutcome | 'all') => {
    setOutcomeFilter(outcome);
  };

  const isLoading =
    activeTab === 'leaderboard' ? attributionQuery.isLoading : timelineQuery.isLoading;
  const isError =
    activeTab === 'leaderboard' ? attributionQuery.isError : timelineQuery.isError;

  return (
    <div className="dashboard-page dashboard-attribution">
      <AttributionTabs activeTab={activeTab} onTabChange={setActiveTab} />

      {isLoading && <p className="dashboard-attribution__loading">Loading...</p>}

      {isError && (
        <p className="dashboard-attribution__error">
          Error loading data. Please try again.
        </p>
      )}

      {!isLoading && !isError && activeTab === 'leaderboard' && attributionQuery.data && (
        <Leaderboard
          contributors={attributionQuery.data.top_contributors}
          negativeImpact={attributionQuery.data.negative_impact}
          ablationCoverage={attributionQuery.data.ablation_coverage}
          period={period}
          onPeriodChange={handlePeriodChange}
        />
      )}

      {!isLoading && !isError && activeTab === 'timeline' && timelineQuery.data && (
        <SessionTimeline
          sessions={timelineQuery.data.sessions}
          period={period}
          onPeriodChange={handlePeriodChange}
          onOutcomeFilter={handleOutcomeFilter}
        />
      )}
    </div>
  );
}
