import type { GapSeverity } from '../../../hooks/useDashboard';
import './GapSeverityBadge.css';

export interface GapSeverityBadgeProps {
  severity: GapSeverity;
}

const SEVERITY_LABELS: Record<GapSeverity, string> = {
  Low: 'LOW',
  Medium: 'MED',
  High: 'HIGH',
  Critical: 'CRIT',
};

export function GapSeverityBadge({ severity }: GapSeverityBadgeProps) {
  return (
    <span
      className={`gap-severity-badge gap-severity-badge--${severity.toLowerCase()}`}
      data-testid={`severity-${severity.toLowerCase()}`}
    >
      {SEVERITY_LABELS[severity]}
    </span>
  );
}
