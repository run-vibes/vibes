import './ProgressBar.css';

export interface ProgressBarProps {
  value: number; // 0 to 1
  label?: string;
  color?: string;
}

function formatPercent(value: number): string {
  return `${Math.round(value * 100)}%`;
}

export function ProgressBar({
  value,
  label,
  color = 'var(--crt-success)',
}: ProgressBarProps) {
  const clampedValue = Math.max(0, Math.min(1, value));
  const percent = clampedValue * 100;

  return (
    <div className="progress-bar">
      <div className="progress-bar__track">
        <div
          className="progress-bar__fill"
          style={{
            width: `${percent}%`,
            backgroundColor: color,
          }}
        />
      </div>
      <span className="progress-bar__label">
        {label ?? formatPercent(clampedValue)}
      </span>
    </div>
  );
}
