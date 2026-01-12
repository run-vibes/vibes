import { useState } from 'react';
import { NoveltyStats, ClusterList, GapsList, GapDetail } from '../../components/dashboard/openworld';
import {
  useOpenWorldOverview,
  useOpenWorldGaps,
  useOpenWorldGapDetail,
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
        {activeTab === 'solutions' && <SolutionsPlaceholder />}
        {activeTab === 'activity' && <ActivityPlaceholder />}
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

function SolutionsPlaceholder() {
  return (
    <div className="dashboard-openworld__placeholder">
      <h3>Suggested Solutions</h3>
      <p>Review and apply solutions for capability gaps.</p>
      <div className="dashboard-openworld__empty-state">
        <span className="dashboard-openworld__empty-icon">◇</span>
        <span>No solutions pending review</span>
      </div>
    </div>
  );
}

function ActivityPlaceholder() {
  return (
    <div className="dashboard-openworld__placeholder">
      <h3>Response Activity</h3>
      <p>Live feed of graduated response actions.</p>
      <div className="dashboard-openworld__placeholder-grid">
        <div className="dashboard-openworld__stat-card">
          <span className="dashboard-openworld__stat-label">Outcomes</span>
          <span className="dashboard-openworld__stat-value">0</span>
        </div>
        <div className="dashboard-openworld__stat-card">
          <span className="dashboard-openworld__stat-label">Negative</span>
          <span className="dashboard-openworld__stat-value">0%</span>
        </div>
        <div className="dashboard-openworld__stat-card">
          <span className="dashboard-openworld__stat-label">Exploration</span>
          <span className="dashboard-openworld__stat-value">+0.00</span>
        </div>
      </div>
      <div className="dashboard-openworld__empty-state">
        <span className="dashboard-openworld__empty-icon">●</span>
        <span>No recent activity</span>
      </div>
    </div>
  );
}
