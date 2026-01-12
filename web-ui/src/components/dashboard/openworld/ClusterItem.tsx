import type { ClusterBrief } from '../../../hooks/useDashboard';
import './ClusterList.css';

export interface ClusterItemProps {
  cluster: ClusterBrief;
}

function formatTimeAgo(dateStr: string): string {
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMinutes = Math.floor(diffMs / (1000 * 60));
  const diffHours = Math.floor(diffMinutes / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffMinutes < 1) return 'just now';
  if (diffMinutes < 60) return `${diffMinutes}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffDays === 1) return 'yesterday';
  return `${diffDays}d ago`;
}

export function ClusterItem({ cluster }: ClusterItemProps) {
  const { id, member_count, category_hint, created_at } = cluster;

  return (
    <tr className="cluster-item" data-testid={`cluster-${id}`}>
      <td className="cluster-item__id">{id.slice(0, 8)}</td>
      <td className="cluster-item__category">{category_hint || '—'}</td>
      <td className="cluster-item__members">{member_count}</td>
      <td className="cluster-item__age">{formatTimeAgo(created_at)}</td>
    </tr>
  );
}
