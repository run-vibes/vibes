import type { InjectionStrategy } from '../../../hooks/useDashboard';
import './StrategyBar.css';

export interface StrategyBarProps {
  strategy: InjectionStrategy;
  weight: number;
}

function formatPercent(value: number): string {
  return `${Math.round(value * 100)}%`;
}

export function StrategyBar({ strategy, weight }: StrategyBarProps) {
  const percent = Math.round(weight * 100);

  return (
    <div className="strategy-bar">
      <span className="strategy-bar__label">{strategy}</span>
      <div className="strategy-bar__track">
        <div
          className={`strategy-bar__fill strategy-bar__fill--${strategy.toLowerCase()}`}
          role="progressbar"
          aria-valuenow={percent}
          aria-valuemin={0}
          aria-valuemax={100}
          style={{ width: `${percent}%` }}
        />
      </div>
      <span className="strategy-bar__value">{formatPercent(weight)}</span>
    </div>
  );
}
