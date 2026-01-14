import { Link } from '@tanstack/react-router';
import { Card, Metric } from '@vibes/design-system';
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
    <Card
      variant="crt"
      title="Learnings"
      className="dashboard-card"
      footer={
        <Link to="/groove/dashboard/learnings" className="card-footer-link">
          View →
        </Link>
      }
    >
      <div className="dashboard-card__metrics">
        <Metric label="Total" value={total} />
        <Metric label="Active" value={active} />
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
    </Card>
  );
}
