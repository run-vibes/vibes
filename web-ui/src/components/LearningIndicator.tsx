import { useState } from 'react';
import './LearningIndicator.css';

export type IndicatorState = 'idle' | 'active' | 'error';

export interface LearningIndicatorProps {
  state: IndicatorState;
  onClick?: () => void;
}

function getStatusText(state: IndicatorState): string {
  switch (state) {
    case 'idle':
      return 'Groove: Idle';
    case 'active':
      return 'Groove: Learning...';
    case 'error':
      return 'Groove: Error';
  }
}

function getAriaLabel(state: IndicatorState): string {
  switch (state) {
    case 'idle':
      return 'Learning indicator: idle';
    case 'active':
      return 'Learning indicator: actively learning';
    case 'error':
      return 'Learning indicator: error occurred';
  }
}

export function LearningIndicator({ state, onClick }: LearningIndicatorProps) {
  const [showTooltip, setShowTooltip] = useState(false);

  return (
    <button
      className={`learning-indicator learning-indicator--${state}`}
      onClick={onClick}
      onMouseEnter={() => setShowTooltip(true)}
      onMouseLeave={() => setShowTooltip(false)}
      aria-label={getAriaLabel(state)}
      type="button"
    >
      <span className="learning-indicator__icon">🧠</span>
      {showTooltip && (
        <div className="learning-indicator__tooltip">
          {getStatusText(state)}
        </div>
      )}
    </button>
  );
}
