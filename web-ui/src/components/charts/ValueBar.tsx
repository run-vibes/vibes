import './ValueBar.css';

export interface ValueBarProps {
  value: number; // -1 to +1
  showValue?: boolean;
}

function formatValue(value: number): string {
  const sign = value >= 0 ? '+' : '';
  return `${sign}${value.toFixed(2)}`;
}

export function ValueBar({ value, showValue = false }: ValueBarProps) {
  const clampedValue = Math.max(-1, Math.min(1, value));
  const isPositive = clampedValue >= 0;

  // Width is 50% of the bar (half the track) times the absolute value
  const width = Math.abs(clampedValue) * 50;

  // Position: positive values start at 50% (center), negative values end at 50%
  const left = isPositive ? 50 : 50 - width;

  return (
    <div className="value-bar">
      <div className="value-bar__track">
        <div className="value-bar__zero" />
        <div
          className={`value-bar__fill ${isPositive ? 'value-bar__fill--positive' : 'value-bar__fill--negative'}`}
          style={{
            left: `${left}%`,
            width: `${width}%`,
          }}
        />
      </div>
      {showValue && (
        <span className="value-bar__value">{formatValue(clampedValue)}</span>
      )}
    </div>
  );
}
