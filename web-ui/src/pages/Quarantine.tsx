import { useState } from 'react';
import {
  useQuarantineList,
  useQuarantineStats,
  useTrustLevels,
  usePolicy,
  type QuarantinedLearningSummary,
} from '../hooks/useGroove';

// ============================================================================
// Stats Card Component
// ============================================================================

function StatCard({ label, value, color }: { label: string; value: number; color: string }) {
  return (
    <div
      style={{
        padding: '1rem',
        backgroundColor: 'var(--bg-secondary, #1E1E1E)',
        borderRadius: '0.5rem',
        textAlign: 'center',
      }}
    >
      <div style={{ fontSize: '2rem', fontWeight: 'bold', color }}>{value}</div>
      <div style={{ fontSize: '0.875rem', color: 'var(--text-muted, #9CA3AF)' }}>{label}</div>
    </div>
  );
}

// ============================================================================
// Trust Badge Component
// ============================================================================

function TrustBadge({ level }: { level: string }) {
  const colors: Record<string, { bg: string; text: string }> = {
    Local: { bg: '#10B98120', text: '#10B981' },
    PrivateCloud: { bg: '#6366F120', text: '#6366F1' },
    OrganizationVerified: { bg: '#3B82F620', text: '#3B82F6' },
    OrganizationUnverified: { bg: '#8B5CF620', text: '#8B5CF6' },
    PublicVerified: { bg: '#F59E0B20', text: '#F59E0B' },
    PublicUnverified: { bg: '#EF444420', text: '#EF4444' },
    Quarantined: { bg: '#6B728020', text: '#6B7280' },
  };

  const style = colors[level] || colors.Quarantined;

  return (
    <span
      style={{
        display: 'inline-block',
        padding: '0.25rem 0.5rem',
        borderRadius: '0.25rem',
        backgroundColor: style.bg,
        color: style.text,
        fontSize: '0.75rem',
        fontWeight: 500,
      }}
    >
      {level}
    </span>
  );
}

// ============================================================================
// Quarantine Item Component
// ============================================================================

function QuarantineItem({ item }: { item: QuarantinedLearningSummary }) {
  return (
    <div
      style={{
        padding: '1rem',
        backgroundColor: 'var(--bg-secondary, #1E1E1E)',
        borderRadius: '0.5rem',
        display: 'flex',
        flexDirection: 'column',
        gap: '0.5rem',
      }}
    >
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
        <div style={{ flex: 1 }}>
          <div style={{ fontWeight: 500 }}>{item.description}</div>
          <div style={{ fontSize: '0.875rem', color: 'var(--text-muted, #9CA3AF)' }}>
            ID: {item.id}
          </div>
        </div>
        <TrustBadge level={item.trust_level} />
      </div>
      <div style={{ display: 'flex', gap: '1rem', fontSize: '0.875rem' }}>
        <div>
          <span style={{ color: 'var(--text-muted, #9CA3AF)' }}>Reason:</span> {item.reason}
        </div>
        <div>
          <span style={{ color: 'var(--text-muted, #9CA3AF)' }}>Quarantined:</span>{' '}
          {new Date(item.quarantined_at).toLocaleDateString()}
        </div>
      </div>
      {item.pending_review && (
        <div style={{ display: 'flex', gap: '0.5rem', marginTop: '0.5rem' }}>
          <button
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: '#10B981',
              color: 'white',
              border: 'none',
              borderRadius: '0.25rem',
              cursor: 'pointer',
            }}
          >
            Approve
          </button>
          <button
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: '#EF4444',
              color: 'white',
              border: 'none',
              borderRadius: '0.25rem',
              cursor: 'pointer',
            }}
          >
            Reject
          </button>
          <button
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: '#6B7280',
              color: 'white',
              border: 'none',
              borderRadius: '0.25rem',
              cursor: 'pointer',
            }}
          >
            Escalate
          </button>
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

  if (isLoading) return <p>Loading trust levels...</p>;
  if (!data) return null;

  return (
    <section style={{ marginTop: '2rem' }}>
      <h2>Trust Levels</h2>
      <div
        style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))',
          gap: '0.5rem',
          marginTop: '1rem',
        }}
      >
        {data.levels.map((level) => (
          <div
            key={level.name}
            style={{
              padding: '0.75rem',
              backgroundColor: 'var(--bg-secondary, #1E1E1E)',
              borderRadius: '0.5rem',
            }}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <TrustBadge level={level.name} />
              <span style={{ fontSize: '0.875rem', fontWeight: 'bold' }}>{level.score}</span>
            </div>
            <div style={{ fontSize: '0.75rem', color: 'var(--text-muted, #9CA3AF)', marginTop: '0.5rem' }}>
              {level.description}
            </div>
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

  if (isLoading) return <p>Loading policy...</p>;
  if (!data) return null;

  return (
    <section style={{ marginTop: '2rem' }}>
      <h2 style={{ cursor: 'pointer' }} onClick={() => setExpanded(!expanded)}>
        Security Policy {expanded ? '▼' : '▶'}
      </h2>
      {expanded && (
        <div
          style={{
            backgroundColor: 'var(--bg-secondary, #1E1E1E)',
            borderRadius: '0.5rem',
            padding: '1rem',
            marginTop: '1rem',
          }}
        >
          <h3>Injection</h3>
          <dl style={{ display: 'grid', gridTemplateColumns: 'auto 1fr', gap: '0.25rem 1rem' }}>
            <dt>Block Quarantined</dt>
            <dd>{data.injection.block_quarantined ? 'Yes' : 'No'}</dd>
            <dt>Allow Personal</dt>
            <dd>{data.injection.allow_personal_injection ? 'Yes' : 'No'}</dd>
            <dt>Allow Unverified</dt>
            <dd>{data.injection.allow_unverified_injection ? 'Yes' : 'No'}</dd>
          </dl>

          <h3 style={{ marginTop: '1rem' }}>Quarantine</h3>
          <dl style={{ display: 'grid', gridTemplateColumns: 'auto 1fr', gap: '0.25rem 1rem' }}>
            <dt>Reviewers</dt>
            <dd>{data.quarantine.reviewers.join(', ') || 'None'}</dd>
            <dt>Visible To</dt>
            <dd>{data.quarantine.visible_to.join(', ') || 'None'}</dd>
            <dt>Auto-delete After</dt>
            <dd>{data.quarantine.auto_delete_after_days ? `${data.quarantine.auto_delete_after_days} days` : 'Never'}</dd>
          </dl>

          <h3 style={{ marginTop: '1rem' }}>Import/Export</h3>
          <dl style={{ display: 'grid', gridTemplateColumns: 'auto 1fr', gap: '0.25rem 1rem' }}>
            <dt>Import from File</dt>
            <dd>{data.import_export.allow_import_from_file ? 'Yes' : 'No'}</dd>
            <dt>Import from URL</dt>
            <dd>{data.import_export.allow_import_from_url ? 'Yes' : 'No'}</dd>
            <dt>Export Personal</dt>
            <dd>{data.import_export.allow_export_personal ? 'Yes' : 'No'}</dd>
            <dt>Export Enterprise</dt>
            <dd>{data.import_export.allow_export_enterprise ? 'Yes' : 'No'}</dd>
          </dl>

          <h3 style={{ marginTop: '1rem' }}>Audit</h3>
          <dl style={{ display: 'grid', gridTemplateColumns: 'auto 1fr', gap: '0.25rem 1rem' }}>
            <dt>Enabled</dt>
            <dd>{data.audit.enabled ? 'Yes' : 'No'}</dd>
            <dt>Retention</dt>
            <dd>{data.audit.retention_days} days</dd>
          </dl>
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
    <div style={{ padding: '2rem' }}>
      <h1>Groove Security</h1>
      <p style={{ color: 'var(--text-muted, #9CA3AF)' }}>
        Manage quarantined learnings and security policy
      </p>

      {/* Stats Grid */}
      <section style={{ marginTop: '2rem' }}>
        <h2>Quarantine Queue</h2>
        {statsLoading ? (
          <p>Loading stats...</p>
        ) : stats ? (
          <div
            style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fit, minmax(120px, 1fr))',
              gap: '1rem',
              marginTop: '1rem',
            }}
          >
            <StatCard label="Total" value={stats.total} color="#6366F1" />
            <StatCard label="Pending" value={stats.pending_review} color="#F59E0B" />
            <StatCard label="Approved" value={stats.approved} color="#10B981" />
            <StatCard label="Rejected" value={stats.rejected} color="#EF4444" />
            <StatCard label="Escalated" value={stats.escalated} color="#8B5CF6" />
          </div>
        ) : null}
      </section>

      {/* Quarantine List */}
      <section style={{ marginTop: '2rem' }}>
        <h2>Pending Review</h2>
        {listLoading ? (
          <p>Loading quarantined items...</p>
        ) : quarantined && quarantined.items.length > 0 ? (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', marginTop: '1rem' }}>
            {quarantined.items.map((item) => (
              <QuarantineItem key={item.id} item={item} />
            ))}
          </div>
        ) : (
          <div
            style={{
              padding: '2rem',
              backgroundColor: 'var(--bg-secondary, #1E1E1E)',
              borderRadius: '0.5rem',
              textAlign: 'center',
              marginTop: '1rem',
            }}
          >
            <p style={{ color: 'var(--text-muted, #9CA3AF)' }}>
              No items in quarantine queue
            </p>
          </div>
        )}
      </section>

      {/* Trust Levels */}
      <TrustLevelsSection />

      {/* Policy */}
      <PolicySection />
    </div>
  );
}
