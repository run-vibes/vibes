import { Panel, Metric } from '@vibes/design-system';
import type { OpenWorldGapDetailData, SolutionBrief } from '../../../hooks/useDashboard';
import { GapSeverityBadge } from './GapSeverityBadge';
import './GapDetail.css';

export interface GapDetailProps {
  data?: OpenWorldGapDetailData;
  isLoading?: boolean;
}

function formatTimeAgo(dateStr: string): string {
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
  const diffDays = Math.floor(diffHours / 24);

  if (diffHours < 1) return 'just now';
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffDays === 1) return 'yesterday';
  return `${diffDays}d ago`;
}

const CATEGORY_LABELS: Record<string, string> = {
  MissingKnowledge: 'Missing Knowledge',
  IncorrectPattern: 'Incorrect Pattern',
  ContextMismatch: 'Context Mismatch',
  ToolGap: 'Tool Gap',
};

const STATUS_LABELS: Record<string, string> = {
  Detected: 'Detected',
  Confirmed: 'Confirmed',
  InProgress: 'In Progress',
  Resolved: 'Resolved',
  Dismissed: 'Dismissed',
};

export function GapDetail({ data, isLoading }: GapDetailProps) {
  if (isLoading) {
    return (
      <div className="gap-detail">
        <Panel variant="crt" title="Gap Detail" className="gap-detail__panel">
          <p className="gap-detail__loading">Loading...</p>
        </Panel>
      </div>
    );
  }

  if (!data) {
    return (
      <div className="gap-detail">
        <Panel variant="crt" title="Gap Detail" className="gap-detail__panel">
          <div className="gap-detail__empty">
            <span className="gap-detail__empty-icon">←</span>
            <span>Select a gap to view details</span>
          </div>
        </Panel>
      </div>
    );
  }

  const {
    id,
    category,
    severity,
    status,
    context_pattern,
    failure_count,
    first_seen,
    last_seen,
    suggested_solutions,
  } = data;

  return (
    <div className="gap-detail">
      <Panel variant="crt" title="Gap Detail" className="gap-detail__panel">
        <div className="gap-detail__header">
          <GapSeverityBadge severity={severity} />
          <span className="gap-detail__id">{id.slice(0, 8)}</span>
        </div>

        <div className="gap-detail__info">
          <div className="gap-detail__row">
            <span className="gap-detail__label">Category</span>
            <span className="gap-detail__value">{CATEGORY_LABELS[category] || category}</span>
          </div>
          <div className="gap-detail__row">
            <span className="gap-detail__label">Status</span>
            <span className="gap-detail__value gap-detail__status">{STATUS_LABELS[status] || status}</span>
          </div>
          <div className="gap-detail__row">
            <span className="gap-detail__label">First Seen</span>
            <span className="gap-detail__value">{formatTimeAgo(first_seen)}</span>
          </div>
          <div className="gap-detail__row">
            <span className="gap-detail__label">Last Seen</span>
            <span className="gap-detail__value">{formatTimeAgo(last_seen)}</span>
          </div>
        </div>

        <div className="gap-detail__metrics">
          <Metric label="Failures" value={String(failure_count)} />
          <Metric label="Solutions" value={String(suggested_solutions.length)} />
        </div>

        <div className="gap-detail__section">
          <h4 className="gap-detail__section-title">Context Pattern</h4>
          <pre className="gap-detail__context">{context_pattern}</pre>
        </div>

        {suggested_solutions.length > 0 && (
          <div className="gap-detail__section">
            <h4 className="gap-detail__section-title">Suggested Solutions</h4>
            <ul className="gap-detail__solutions">
              {suggested_solutions.map((solution, index) => (
                <SolutionItem key={index} solution={solution} />
              ))}
            </ul>
          </div>
        )}
      </Panel>
    </div>
  );
}

function SolutionItem({ solution }: { solution: SolutionBrief }) {
  const { action_type, description, confidence, applied } = solution;

  return (
    <li className={`solution-item ${applied ? 'solution-item--applied' : ''}`}>
      <div className="solution-item__header">
        <span className="solution-item__type">{action_type}</span>
        <span className="solution-item__confidence">{Math.round(confidence * 100)}%</span>
        {applied && <span className="solution-item__applied">Applied</span>}
      </div>
      <p className="solution-item__description">{description}</p>
    </li>
  );
}
