import { Panel } from '@vibes/design-system';
import type { ClusterBrief } from '../../../hooks/useDashboard';
import { ClusterItem } from './ClusterItem';
import './ClusterList.css';

export interface ClusterListProps {
  clusters?: ClusterBrief[];
  isLoading?: boolean;
}

export function ClusterList({ clusters, isLoading }: ClusterListProps) {
  if (isLoading) {
    return (
      <Panel variant="crt" title="Recent Clusters" className="cluster-list">
        <p className="cluster-list__empty">Loading...</p>
      </Panel>
    );
  }

  if (!clusters || clusters.length === 0) {
    return (
      <Panel variant="crt" title="Recent Clusters" className="cluster-list">
        <div className="cluster-list__empty" data-testid="cluster-list-empty">
          <span className="cluster-list__empty-icon">○</span>
          <span>No clusters formed yet</span>
        </div>
      </Panel>
    );
  }

  return (
    <Panel variant="crt" title="Recent Clusters" className="cluster-list">
      <table className="cluster-list__table" data-testid="cluster-list-table">
        <thead>
          <tr>
            <th>ID</th>
            <th>Category</th>
            <th>Members</th>
            <th>Age</th>
          </tr>
        </thead>
        <tbody>
          {clusters.map((cluster) => (
            <ClusterItem key={cluster.id} cluster={cluster} />
          ))}
        </tbody>
      </table>
    </Panel>
  );
}
