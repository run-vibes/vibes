import { useState } from 'react';
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
        {activeTab === 'novelty' && <NoveltyPlaceholder />}
        {activeTab === 'gaps' && <GapsPlaceholder />}
        {activeTab === 'solutions' && <SolutionsPlaceholder />}
        {activeTab === 'activity' && <ActivityPlaceholder />}
      </div>
    </div>
  );
}

function NoveltyPlaceholder() {
  return (
    <div className="dashboard-openworld__placeholder">
      <h3>Novelty Detection</h3>
      <p>Monitor adaptive threshold, pending outliers, and cluster formation.</p>
      <div className="dashboard-openworld__placeholder-grid">
        <div className="dashboard-openworld__stat-card">
          <span className="dashboard-openworld__stat-label">Threshold</span>
          <span className="dashboard-openworld__stat-value">0.85</span>
        </div>
        <div className="dashboard-openworld__stat-card">
          <span className="dashboard-openworld__stat-label">Pending</span>
          <span className="dashboard-openworld__stat-value">0</span>
        </div>
        <div className="dashboard-openworld__stat-card">
          <span className="dashboard-openworld__stat-label">Clusters</span>
          <span className="dashboard-openworld__stat-value">0</span>
        </div>
      </div>
    </div>
  );
}

function GapsPlaceholder() {
  return (
    <div className="dashboard-openworld__placeholder">
      <h3>Capability Gaps</h3>
      <p>Browse detected capability gaps by severity and status.</p>
      <div className="dashboard-openworld__empty-state">
        <span className="dashboard-openworld__empty-icon">○</span>
        <span>No capability gaps detected</span>
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
