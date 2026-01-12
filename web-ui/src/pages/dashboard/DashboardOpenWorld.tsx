import { useState } from 'react';
import {
  NoveltyStats,
  ClusterList,
  GapsList,
  GapDetail,
  SolutionsList,
  ActivityStats,
  ActivityFeed,
} from '../../components/dashboard/openworld';
import { ConfirmDialog } from '../../components/dashboard/ConfirmDialog';
import {
  useOpenWorldOverview,
  useOpenWorldGaps,
  useOpenWorldGapDetail,
  useOpenWorldSolutions,
  useOpenWorldActivity,
  useApplySolution,
  useDismissSolution,
  type OpenWorldGapsFilter,
} from '../../hooks/useDashboard';
import './DashboardOpenWorld.css';

type OpenWorldTab = 'novelty' | 'gaps' | 'solutions' | 'activity';

const OPENWORLD_TABS: { id: OpenWorldTab; label: string }[] = [
  { id: 'novelty', label: 'Novelty' },
  { id: 'gaps', label: 'Gaps' },
  { id: 'solutions', label: 'Solutions' },
  { id: 'activity', label: 'Activity' },
];

export function DashboardOpenWorld() {
  const [activeTab, setActiveTab] = useState<OpenWorldTab>('novelty');

  return (
    <div className="dashboard-openworld">
      <nav className="dashboard-openworld__tabs">
        {OPENWORLD_TABS.map((tab) => (
          <button
            key={tab.id}
            className={`dashboard-openworld__tab ${activeTab === tab.id ? 'active' : ''}`}
            onClick={() => setActiveTab(tab.id)}
            type="button"
          >
            {tab.label}
          </button>
        ))}
      </nav>

      <div className="dashboard-openworld__content">
        {activeTab === 'novelty' && <NoveltyPanel />}
        {activeTab === 'gaps' && <GapsPanel />}
        {activeTab === 'solutions' && <SolutionsPanel />}
        {activeTab === 'activity' && <ActivityPanel />}
      </div>
    </div>
  );
}

function NoveltyPanel() {
  const { data, isLoading } = useOpenWorldOverview();

  return (
    <div className="dashboard-openworld__panel">
      <NoveltyStats data={data} isLoading={isLoading} />
      <ClusterList clusters={data?.recent_clusters} isLoading={isLoading} />
    </div>
  );
}

function GapsPanel() {
  const [filters, setFilters] = useState<OpenWorldGapsFilter>({});
  const [selectedGapId, setSelectedGapId] = useState<string | undefined>();

  const { data: gapsData, isLoading: gapsLoading } = useOpenWorldGaps(filters);
  const { data: detailData, isLoading: detailLoading } = useOpenWorldGapDetail(selectedGapId);

  return (
    <div className="dashboard-openworld__split-panel">
      <div className="dashboard-openworld__split-left">
        <GapsList
          gaps={gapsData?.gaps}
          total={gapsData?.total}
          filters={filters}
          selectedId={selectedGapId}
          isLoading={gapsLoading}
          onFiltersChange={setFilters}
          onSelectGap={setSelectedGapId}
        />
      </div>
      <div className="dashboard-openworld__split-right">
        <GapDetail data={detailData} isLoading={detailLoading && !!selectedGapId} />
      </div>
    </div>
  );
}

type ConfirmAction = { type: 'apply' | 'dismiss'; solutionId: string } | null;

function SolutionsPanel() {
  const { data, isLoading } = useOpenWorldSolutions();
  const applyMutation = useApplySolution();
  const dismissMutation = useDismissSolution();

  const [confirmAction, setConfirmAction] = useState<ConfirmAction>(null);
  const [actionLoading, setActionLoading] = useState<string | undefined>();

  const handleApply = (solutionId: string) => {
    setConfirmAction({ type: 'apply', solutionId });
  };

  const handleDismiss = (solutionId: string) => {
    setConfirmAction({ type: 'dismiss', solutionId });
  };

  const handleConfirm = async () => {
    if (!confirmAction) return;

    setActionLoading(confirmAction.solutionId);

    try {
      if (confirmAction.type === 'apply') {
        await applyMutation.mutateAsync(confirmAction.solutionId);
      } else {
        await dismissMutation.mutateAsync(confirmAction.solutionId);
      }
    } finally {
      setActionLoading(undefined);
      setConfirmAction(null);
    }
  };

  const handleCancel = () => {
    setConfirmAction(null);
  };

  return (
    <div className="dashboard-openworld__panel">
      <SolutionsList
        solutions={data?.solutions}
        total={data?.total}
        isLoading={isLoading}
        onApply={handleApply}
        onDismiss={handleDismiss}
        actionLoading={actionLoading}
      />

      <ConfirmDialog
        isOpen={confirmAction?.type === 'apply'}
        title="Apply Solution"
        message="This will apply the suggested solution. Are you sure you want to proceed?"
        confirmText="Apply"
        onConfirm={handleConfirm}
        onCancel={handleCancel}
        isLoading={applyMutation.isPending}
      />

      <ConfirmDialog
        isOpen={confirmAction?.type === 'dismiss'}
        title="Dismiss Solution"
        message="This solution will be marked as dismissed and won't be shown again."
        confirmText="Dismiss"
        onConfirm={handleConfirm}
        onCancel={handleCancel}
        isLoading={dismissMutation.isPending}
      />
    </div>
  );
}

function ActivityPanel() {
  const { data, isLoading, isFetching } = useOpenWorldActivity();

  // Show live indicator when we have data and are actively fetching updates
  const isLive = !isLoading && isFetching;

  return (
    <div className="dashboard-openworld__panel">
      <ActivityStats summary={data?.summary} isLoading={isLoading} isLive={isLive} />
      <ActivityFeed events={data?.events} isLoading={isLoading} />
    </div>
  );
}
