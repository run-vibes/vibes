import { Link } from '@tanstack/react-router';
import { Card } from '@vibes/design-system';
import type { AttributionSummary, ContributorBrief } from '../../hooks/useDashboard';
import './DashboardCards.css';

export interface AttributionCardProps {
  data?: AttributionSummary;
}

export function AttributionCard({ data }: AttributionCardProps) {
  const topContributors = data?.top_contributors ?? [];
  const underReviewCount = data?.under_review_count ?? 0;
  const negativeCount = data?.negative_count ?? 0;

  return (
    <Card
      variant="crt"
      title="Attribution"
      className="dashboard-card"
      footer={
        <Link to="/groove/dashboard/attribution" className="card-footer-link">
          View →
        </Link>
      }
    >
      <div className="dashboard-card__list">
        <span className="list-label">Top Contributors:</span>
        {topContributors.length > 0 ? (
          <ol className="contributor-list">
            {topContributors.slice(0, 3).map((contributor: ContributorBrief, index: number) => (
              <li key={contributor.learning_id} className="contributor-item">
                <span className="contributor-rank">{index + 1}.</span>
                <span className="contributor-content">{contributor.content}</span>
                <span className="contributor-value">+{contributor.estimated_value.toFixed(2)}</span>
              </li>
            ))}
          </ol>
        ) : (
          <p className="empty-text">No data yet</p>
        )}
      </div>

      {(underReviewCount > 0 || negativeCount > 0) && (
        <div className="dashboard-card__warnings">
          {underReviewCount > 0 && (
            <span className="warning-item">
              <span className="warning-icon">⚠</span>
              {underReviewCount} learning{underReviewCount !== 1 ? 's' : ''} under review
            </span>
          )}
          {negativeCount > 0 && (
            <span className="warning-item warning-item--negative">
              <span className="warning-icon">!</span>
              {negativeCount} with negative impact
            </span>
          )}
        </div>
      )}
    </Card>
  );
}
