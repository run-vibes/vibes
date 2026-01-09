import { Link } from '@tanstack/react-router';
import type { LearningBrief, LearningSummary } from '../../hooks/useDashboard';
import './DashboardCards.css';

export interface LearningsCardProps {
  data?: LearningSummary;
}

function formatTimeAgo(dateStr: string): string {
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffHours = Math.floor(diffMs / (1000 * 60 * 60));

  if (diffHours < 1) return 'just now';
  if (diffHours < 24) return `${diffHours}h ago`;

  const diffDays = Math.floor(diffHours / 24);
  if (diffDays === 1) return 'yesterday';
  return `${diffDays}d ago`;
}

export function LearningsCard({ data }: LearningsCardProps) {
  const total = data?.total ?? 0;
  const active = data?.active ?? 0;
  const recent = data?.recent ?? [];

  return (
    <div className="dashboard-card">
      <h3 className="dashboard-card__title">Learnings</h3>

      <div className="dashboard-card__metrics">
        <div className="metric">
          <span className="metric__label">Total</span>
          <span className="metric__value">{total}</span>
        </div>
        <div className="metric">
          <span className="metric__label">Active</span>
          <span className="metric__value">{active}</span>
        </div>
      </div>

      {recent.length > 0 && (
        <div className="dashboard-card__list">
          <span className="list-label">Recent:</span>
          <ul className="recent-list">
            {recent.slice(0, 5).map((learning: LearningBrief) => (
              <li key={learning.id} className="recent-item">
                <span className="recent-item__content">{learning.content}</span>
                <span className="recent-item__time">{formatTimeAgo(learning.created_at)}</span>
              </li>
            ))}
          </ul>
        </div>
      )}

      <Link to="/groove/dashboard/learnings" className="dashboard-card__link">
        View →
      </Link>
    </div>
  );
}
