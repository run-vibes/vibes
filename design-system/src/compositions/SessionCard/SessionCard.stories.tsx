import '../../tokens/index.css';
import { SessionCard, SessionAction } from './SessionCard';

export default {
  title: 'Compositions/SessionCard',
};

export const Default = () => (
  <div style={{ padding: '2rem', backgroundColor: 'var(--color-bg-base)', maxWidth: '400px' }}>
    <SessionCard
      id="sess-abc123"
      name="auth-refactor"
      status="processing"
      subscribers={2}
      updatedAt={new Date(Date.now() - 1000 * 60 * 5)}
    />
  </div>
);

export const WithoutName = () => (
  <div style={{ padding: '2rem', backgroundColor: 'var(--color-bg-base)', maxWidth: '400px' }}>
    <SessionCard
      id="sess-xyz789"
      status="idle"
      subscribers={0}
      updatedAt={new Date()}
    />
  </div>
);

export const AllStatuses = () => (
  <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem', padding: '2rem', backgroundColor: 'var(--color-bg-base)', maxWidth: '400px' }}>
    <SessionCard id="sess-1" status="idle" subscribers={0} updatedAt={new Date()} />
    <SessionCard id="sess-2" name="feature-work" status="processing" subscribers={1} updatedAt={new Date()} />
    <SessionCard id="sess-3" name="bug-fix" status="waiting" subscribers={2} updatedAt={new Date()} />
    <SessionCard id="sess-4" status="finished" subscribers={1} updatedAt={new Date(Date.now() - 1000 * 60 * 60)} />
    <SessionCard id="sess-5" name="failed-task" status="failed" subscribers={0} updatedAt={new Date(Date.now() - 1000 * 60 * 60 * 24)} />
  </div>
);

export const WithDurationAndEvents = () => (
  <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem', padding: '2rem', backgroundColor: 'var(--color-bg-base)', maxWidth: '450px' }}>
    <SessionCard
      id="sess-1"
      name="long-running-task"
      status="processing"
      subscribers={3}
      updatedAt={new Date(Date.now() - 1000 * 60 * 45)}
      duration={2700}
      eventCount={156}
    />
    <SessionCard
      id="sess-2"
      name="quick-fix"
      status="finished"
      subscribers={1}
      updatedAt={new Date(Date.now() - 1000 * 60 * 60 * 2)}
      duration={180}
      eventCount={12}
    />
  </div>
);

export const WithQuickActions = () => {
  const actions: SessionAction[] = [
    {
      icon: (
        <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
          <path d="M7 0a7 7 0 100 14A7 7 0 007 0zm3.5 7.7H7.7v2.8H6.3V7.7H3.5V6.3h2.8V3.5h1.4v2.8h2.8v1.4z" />
        </svg>
      ),
      label: 'View details',
      onClick: () => console.log('View clicked'),
    },
    {
      icon: (
        <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
          <path d="M12.95 1.05a3.6 3.6 0 00-5.1 0L7 1.9l-.85-.85a3.6 3.6 0 00-5.1 5.1l5.1 5.1a1.2 1.2 0 001.7 0l5.1-5.1a3.6 3.6 0 000-5.1z" />
        </svg>
      ),
      label: 'Terminate session',
      onClick: () => console.log('Terminate clicked'),
    },
  ];

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem', padding: '2rem', backgroundColor: 'var(--color-bg-base)', maxWidth: '450px' }}>
      <p style={{ color: 'var(--text-dim)', fontFamily: 'var(--font-mono)', fontSize: 'var(--text-sm)', marginBottom: '0.5rem' }}>
        Hover over cards to see action buttons
      </p>
      <SessionCard
        id="sess-active"
        name="feature-development"
        status="processing"
        subscribers={2}
        updatedAt={new Date(Date.now() - 1000 * 60 * 15)}
        duration={900}
        eventCount={45}
        actions={actions}
      />
      <SessionCard
        id="sess-waiting"
        name="code-review"
        status="waiting"
        subscribers={1}
        updatedAt={new Date(Date.now() - 1000 * 60 * 5)}
        duration={300}
        eventCount={8}
        actions={actions}
      />
    </div>
  );
};

export const ActiveVsInactive = () => (
  <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem', padding: '2rem', backgroundColor: 'var(--color-bg-base)', maxWidth: '450px' }}>
    <h3 style={{ color: 'var(--phosphor)', fontFamily: 'var(--font-display)', fontSize: 'var(--text-lg)', marginBottom: '0.5rem' }}>
      Active Sessions (with phosphor glow)
    </h3>
    <SessionCard
      id="sess-active-1"
      name="debugging-auth"
      status="processing"
      subscribers={2}
      updatedAt={new Date()}
      duration={1200}
      eventCount={67}
    />
    <SessionCard
      id="sess-active-2"
      name="waiting-for-input"
      status="waiting"
      subscribers={1}
      updatedAt={new Date(Date.now() - 1000 * 60 * 2)}
      duration={120}
      eventCount={4}
    />

    <h3 style={{ color: 'var(--text-dim)', fontFamily: 'var(--font-display)', fontSize: 'var(--text-lg)', marginTop: '1.5rem', marginBottom: '0.5rem' }}>
      Inactive Sessions (dimmed)
    </h3>
    <SessionCard
      id="sess-inactive-1"
      name="completed-task"
      status="finished"
      subscribers={0}
      updatedAt={new Date(Date.now() - 1000 * 60 * 60)}
      duration={3600}
      eventCount={234}
    />
    <SessionCard
      id="sess-inactive-2"
      status="idle"
      subscribers={0}
      updatedAt={new Date(Date.now() - 1000 * 60 * 60 * 24)}
    />
  </div>
);
