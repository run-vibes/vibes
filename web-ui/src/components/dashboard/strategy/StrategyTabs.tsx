import './StrategyTabs.css';

export type StrategyTab = 'distributions' | 'overrides';

export interface StrategyTabsProps {
  activeTab: StrategyTab;
  onTabChange: (tab: StrategyTab) => void;
}

export function StrategyTabs({ activeTab, onTabChange }: StrategyTabsProps) {
  const handleClick = (tab: StrategyTab) => {
    if (tab !== activeTab) {
      onTabChange(tab);
    }
  };

  return (
    <div className="strategy-tabs" role="tablist">
      <button
        type="button"
        role="tab"
        className={`strategy-tabs__tab ${activeTab === 'distributions' ? 'strategy-tabs__tab--active' : ''}`}
        aria-selected={activeTab === 'distributions'}
        onClick={() => handleClick('distributions')}
      >
        Distributions
      </button>
      <button
        type="button"
        role="tab"
        className={`strategy-tabs__tab ${activeTab === 'overrides' ? 'strategy-tabs__tab--active' : ''}`}
        aria-selected={activeTab === 'overrides'}
        onClick={() => handleClick('overrides')}
      >
        Learning Overrides
      </button>
    </div>
  );
}
