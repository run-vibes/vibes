// web-ui/src/pages/Quarantine.tsx
import { useState } from 'react';
import { Text, Button } from '@vibes/design-system';
import {
  useQuarantineList,
  useQuarantineStats,
  useTrustLevels,
  usePolicy,
  type QuarantinedLearningSummary,
} from '../hooks/useGroove';
import './Quarantine.css';

// ============================================================================
// Stats Card Component
// ============================================================================

function StatCard({ label, value, variant }: { label: string; value: number; variant: string }) {
  return (
    <div className="stat-card">
      <div className={`stat-value ${variant}`}>{value}</div>
      <div className="stat-label">{label}</div>
    </div>
  );
}

// ============================================================================
// Trust Badge Component
// ============================================================================

function TrustBadge({ level }: { level: string }) {
  const classMap: Record<string, string> = {
    Local: 'local',
    PrivateCloud: 'private-cloud',
    OrganizationVerified: 'org-verified',
    OrganizationUnverified: 'org-unverified',
    PublicVerified: 'public-verified',
    PublicUnverified: 'public-unverified',
    Quarantined: 'quarantined',
  };

  const className = classMap[level] || 'quarantined';

  return <span className={`trust-badge ${className}`}>{level}</span>;
}

// ============================================================================
// Quarantine Item Component
// ============================================================================

function QuarantineItem({ item }: { item: QuarantinedLearningSummary }) {
  return (
    <div className="quarantine-item">
      <div className="quarantine-item-header">
        <div className="quarantine-item-info">
          <div className="quarantine-item-description">{item.description}</div>
          <div className="quarantine-item-id">ID: {item.id}</div>
        </div>
        <TrustBadge level={item.trust_level} />
      </div>
      <dl className="quarantine-item-meta">
        <dt>Reason:</dt>
        <dd>{item.reason}</dd>
        <dt>Quarantined:</dt>
        <dd>{new Date(item.quarantined_at).toLocaleDateString()}</dd>
      </dl>
      {item.pending_review && (
        <div className="quarantine-item-actions">
          <Button variant="primary">Approve</Button>
          <Button variant="secondary">Reject</Button>
          <Button variant="ghost">Escalate</Button>
        </div>
      )}
    </div>
  );
}

// ============================================================================
// Trust Levels Section
// ============================================================================

function TrustLevelsSection() {
  const { data, isLoading } = useTrustLevels();

  if (isLoading) return <Text intensity="dim">Loading trust levels...</Text>;
  if (!data) return null;

  return (
    <section>
      <h2>TRUST LEVELS</h2>
      <div className="trust-levels-grid">
        {data.levels.map((level) => (
          <div key={level.name} className="trust-level-card">
            <div className="trust-level-header">
              <TrustBadge level={level.name} />
              <span className="trust-level-score">{level.score}</span>
            </div>
            <div className="trust-level-description">{level.description}</div>
          </div>
        ))}
      </div>
    </section>
  );
}

// ============================================================================
// Policy Section
// ============================================================================

function PolicySection() {
  const { data, isLoading } = usePolicy();
  const [expanded, setExpanded] = useState(false);

  if (isLoading) return <Text intensity="dim">Loading policy...</Text>;
  if (!data) return null;

  return (
    <section>
      <h2 className="policy-header" onClick={() => setExpanded(!expanded)}>
        SECURITY POLICY {expanded ? '▼' : '▶'}
      </h2>
      {expanded && (
        <div className="policy-content">
          <div className="policy-section">
            <h3>Injection</h3>
            <dl>
              <dt>Block Quarantined</dt>
              <dd>{data.injection.block_quarantined ? 'Yes' : 'No'}</dd>
              <dt>Allow Personal</dt>
              <dd>{data.injection.allow_personal_injection ? 'Yes' : 'No'}</dd>
              <dt>Allow Unverified</dt>
              <dd>{data.injection.allow_unverified_injection ? 'Yes' : 'No'}</dd>
            </dl>
          </div>

          <div className="policy-section">
            <h3>Quarantine</h3>
            <dl>
              <dt>Reviewers</dt>
              <dd>{data.quarantine.reviewers.join(', ') || 'None'}</dd>
              <dt>Visible To</dt>
              <dd>{data.quarantine.visible_to.join(', ') || 'None'}</dd>
              <dt>Auto-delete After</dt>
              <dd>{data.quarantine.auto_delete_after_days ? `${data.quarantine.auto_delete_after_days} days` : 'Never'}</dd>
            </dl>
          </div>

          <div className="policy-section">
            <h3>Import/Export</h3>
            <dl>
              <dt>Import from File</dt>
              <dd>{data.import_export.allow_import_from_file ? 'Yes' : 'No'}</dd>
              <dt>Import from URL</dt>
              <dd>{data.import_export.allow_import_from_url ? 'Yes' : 'No'}</dd>
              <dt>Export Personal</dt>
              <dd>{data.import_export.allow_export_personal ? 'Yes' : 'No'}</dd>
              <dt>Export Enterprise</dt>
              <dd>{data.import_export.allow_export_enterprise ? 'Yes' : 'No'}</dd>
            </dl>
          </div>

          <div className="policy-section">
            <h3>Audit</h3>
            <dl>
              <dt>Enabled</dt>
              <dd>{data.audit.enabled ? 'Yes' : 'No'}</dd>
              <dt>Retention</dt>
              <dd>{data.audit.retention_days} days</dd>
            </dl>
          </div>
        </div>
      )}
    </section>
  );
}

// ============================================================================
// Main Page Component
// ============================================================================

export function QuarantinePage() {
  const { data: stats, isLoading: statsLoading } = useQuarantineStats();
  const { data: quarantined, isLoading: listLoading } = useQuarantineList();

  return (
    <div className="quarantine-page">
      {/* Header */}
      <div className="quarantine-header">
        <div className="quarantine-header-left">
          <h1 className="quarantine-title">SECURITY</h1>
        </div>
        <div className="quarantine-header-right">
          {/* Future: Add actions like refresh button */}
        </div>
      </div>

      {/* Content */}
      <div className="quarantine-content">
        {/* Stats Grid */}
        <section>
          <h2>QUARANTINE QUEUE</h2>
          {statsLoading ? (
            <Text intensity="dim">Loading stats...</Text>
          ) : stats ? (
            <div className="stats-grid">
              <StatCard label="Total" value={stats.total} variant="total" />
              <StatCard label="Pending" value={stats.pending_review} variant="pending" />
              <StatCard label="Approved" value={stats.approved} variant="approved" />
              <StatCard label="Rejected" value={stats.rejected} variant="rejected" />
              <StatCard label="Escalated" value={stats.escalated} variant="escalated" />
            </div>
          ) : null}
        </section>

        {/* Quarantine List */}
        <section>
          <h2>PENDING REVIEW</h2>
          {listLoading ? (
            <Text intensity="dim">Loading quarantined items...</Text>
          ) : quarantined && quarantined.items.length > 0 ? (
            <div className="quarantine-list">
              {quarantined.items.map((item) => (
                <QuarantineItem key={item.id} item={item} />
              ))}
            </div>
          ) : (
            <div className="empty-state">
              <Text intensity="dim">No items in quarantine queue</Text>
            </div>
          )}
        </section>

        {/* Trust Levels */}
        <TrustLevelsSection />

        {/* Policy */}
        <PolicySection />
      </div>
    </div>
  );
}
