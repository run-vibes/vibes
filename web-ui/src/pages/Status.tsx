import { useTunnelStatus } from '../hooks/useTunnelStatus';

export function StatusPage() {
  const { data: tunnel, isLoading, error } = useTunnelStatus();

  return (
    <div style={{ padding: '2rem' }}>
      <h1>Status</h1>

      <section style={{ marginTop: '2rem' }}>
        <h2>Tunnel</h2>

        {isLoading && <p>Loading...</p>}
        {error && <p style={{ color: 'red' }}>Error loading status</p>}

        {tunnel && (
          <dl style={{ display: 'grid', gridTemplateColumns: 'auto 1fr', gap: '0.5rem 1rem' }}>
            <dt>State</dt>
            <dd>
              <StatusBadge state={tunnel.state} />
            </dd>

            <dt>Mode</dt>
            <dd>{tunnel.mode || 'Not configured'}</dd>

            {tunnel.url && (
              <>
                <dt>URL</dt>
                <dd>
                  <a href={tunnel.url} target="_blank" rel="noopener noreferrer">
                    {tunnel.url}
                  </a>
                </dd>
              </>
            )}

            {tunnel.tunnel_name && (
              <>
                <dt>Tunnel Name</dt>
                <dd>{tunnel.tunnel_name}</dd>
              </>
            )}

            {tunnel.error && (
              <>
                <dt>Error</dt>
                <dd style={{ color: 'red' }}>{tunnel.error}</dd>
              </>
            )}
          </dl>
        )}
      </section>
    </div>
  );
}

function StatusBadge({ state }: { state: string }) {
  const colors: Record<string, string> = {
    disabled: '#9CA3AF',
    starting: '#F59E0B',
    connected: '#10B981',
    reconnecting: '#F59E0B',
    failed: '#EF4444',
    stopped: '#9CA3AF',
  };

  return (
    <span
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap: '0.5rem',
        padding: '0.25rem 0.75rem',
        borderRadius: '9999px',
        backgroundColor: `${colors[state]}20`,
        color: colors[state],
        fontSize: '0.875rem',
        fontWeight: 500,
      }}
    >
      <span style={{ fontSize: '0.5rem' }}>●</span>
      {state}
    </span>
  );
}
