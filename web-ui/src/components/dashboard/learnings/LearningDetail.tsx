import type { LearningDetailData, Scope } from '../../../hooks/useDashboard';
import { LearningStatusBadge } from './LearningStatusBadge';
import { ValueBar } from './ValueBar';
import { LearningActions } from './LearningActions';
import './LearningDetail.css';

export interface LearningDetailProps {
  data?: LearningDetailData;
  isLoading?: boolean;
  onActionComplete?: () => void;
}

function formatScope(scope: Scope): string {
  if (scope.Project) return `Project: ${scope.Project}`;
  if (scope.User) return `User: ${scope.User}`;
  if (scope.Enterprise) return `Enterprise: ${scope.Enterprise}`;
  return 'Unknown';
}

function formatDate(dateStr: string): string {
  const date = new Date(dateStr);
  return date.toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

function formatPercent(value: number): string {
  return `${Math.round(value * 100)}%`;
}

function formatValue(value: number): string {
  const sign = value >= 0 ? '+' : '';
  return `${sign}${value.toFixed(2)}`;
}

export function LearningDetail({ data, isLoading, onActionComplete }: LearningDetailProps) {
  if (isLoading) {
    return (
      <div className="learning-detail learning-detail--empty">
        <p className="learning-detail__placeholder">Loading...</p>
      </div>
    );
  }

  if (!data) {
    return (
      <div className="learning-detail learning-detail--empty">
        <p className="learning-detail__placeholder">Select a learning to view details</p>
      </div>
    );
  }

  return (
    <div className="learning-detail">
      <header className="learning-detail__header">
        <div className="learning-detail__badges">
          <span className="learning-detail__category">{data.category}</span>
          <LearningStatusBadge status={data.status} />
        </div>
      </header>

      <section className="learning-detail__content">
        <p>{data.content}</p>
      </section>

      <section className="learning-detail__metrics">
        <h4 className="section-title">Metrics</h4>
        <div className="metrics-grid">
          <div className="metric">
            <span className="metric__label">Value</span>
            <span className="metric__value">{formatValue(data.estimated_value)}</span>
            <ValueBar value={data.estimated_value} />
          </div>
          <div className="metric">
            <span className="metric__label">Confidence</span>
            <span className="metric__value">{formatPercent(data.confidence)}</span>
          </div>
          <div className="metric">
            <span className="metric__label">Sessions</span>
            <span className="metric__value">{data.session_count}</span>
          </div>
        </div>
      </section>

      <section className="learning-detail__injection">
        <h4 className="section-title">Injection Stats</h4>
        <div className="metrics-grid">
          <div className="metric">
            <span className="metric__label">Times Injected</span>
            <span className="metric__value">{data.times_injected}</span>
          </div>
          <div className="metric">
            <span className="metric__label">Activation Rate</span>
            <span className="metric__value">{formatPercent(data.activation_rate)}</span>
          </div>
        </div>
      </section>

      <section className="learning-detail__info">
        <h4 className="section-title">Information</h4>
        <dl className="info-list">
          <div className="info-item">
            <dt>Scope</dt>
            <dd>{formatScope(data.scope)}</dd>
          </div>
          <div className="info-item">
            <dt>Source</dt>
            <dd>{data.extraction_method}</dd>
          </div>
          <div className="info-item">
            <dt>Created</dt>
            <dd>{formatDate(data.created_at)}</dd>
          </div>
          {data.source_session && (
            <div className="info-item">
              <dt>Source Session</dt>
              <dd className="session-id">{data.source_session}</dd>
            </div>
          )}
        </dl>
      </section>

      <section className="learning-detail__actions">
        <LearningActions
          learningId={data.id}
          status={data.status}
          onActionComplete={onActionComplete}
        />
      </section>
    </div>
  );
}
