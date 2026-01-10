import './AttributionTabs.css';

export type AttributionTab = 'leaderboard' | 'timeline';

export interface AttributionTabsProps {
  activeTab: AttributionTab;
  onTabChange: (tab: AttributionTab) => void;
}

export function AttributionTabs({ activeTab, onTabChange }: AttributionTabsProps) {
  const handleClick = (tab: AttributionTab) => {
    if (tab !== activeTab) {
      onTabChange(tab);
    }
  };

  return (
    <div className="attribution-tabs" role="tablist">
      <button
        type="button"
        role="tab"
        className={`attribution-tabs__tab ${activeTab === 'leaderboard' ? 'attribution-tabs__tab--active' : ''}`}
        aria-selected={activeTab === 'leaderboard'}
        onClick={() => handleClick('leaderboard')}
      >
        Leaderboard
      </button>
      <button
        type="button"
        role="tab"
        className={`attribution-tabs__tab ${activeTab === 'timeline' ? 'attribution-tabs__tab--active' : ''}`}
        aria-selected={activeTab === 'timeline'}
        onClick={() => handleClick('timeline')}
      >
        Session Timeline
      </button>
    </div>
  );
}
