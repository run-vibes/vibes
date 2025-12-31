import '../../tokens/index.css';
import { SessionCard } from './SessionCard';

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
