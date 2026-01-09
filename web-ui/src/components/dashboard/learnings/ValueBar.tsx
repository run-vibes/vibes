import './ValueBar.css';

export interface ValueBarProps {
  value: number; // -1 to +1 range
  showValue?: boolean;
}

export function ValueBar({ value, showValue = false }: ValueBarProps) {
  // Clamp value to -1 to +1
  const clampedValue = Math.max(-1, Math.min(1, value));
  const absValue = Math.abs(clampedValue);
  const fillWidth = `${absValue * 100}%`;

  const colorClass =
    clampedValue > 0.01
      ? 'value-bar--positive'
      : clampedValue < -0.01
        ? 'value-bar--negative'
        : 'value-bar--neutral';

  const formattedValue =
    clampedValue >= 0
      ? `+${clampedValue.toFixed(2)}`
      : clampedValue.toFixed(2);

  return (
    <div className={`value-bar ${colorClass}`} data-testid="value-bar">
      <div className="value-bar__track">
        <div
          className="value-bar__fill"
          style={{ width: fillWidth }}
          data-testid="value-fill"
        />
      </div>
      {showValue && <span className="value-bar__text">{formattedValue}</span>}
    </div>
  );
}
