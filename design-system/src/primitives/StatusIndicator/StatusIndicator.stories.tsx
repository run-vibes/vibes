import '../../tokens/index.css';
import { StatusIndicator } from './StatusIndicator';

export default {
  title: 'Primitives/StatusIndicator',
};

export const States = () => (
  <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem', padding: '2rem', backgroundColor: 'var(--color-bg-base)' }}>
    <StatusIndicator state="live" label="Live" />
    <StatusIndicator state="paused" label="Paused" />
    <StatusIndicator state="offline" label="Offline" />
    <StatusIndicator state="error" label="Error" />
  </div>
);

export const DashboardStates = () => (
  <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem', padding: '2rem', backgroundColor: 'var(--screen)' }}>
    <StatusIndicator state="ok" label="OK" />
    <StatusIndicator state="degraded" label="Degraded" />
    <StatusIndicator state="error" label="Error" />
  </div>
);

export const DotsOnly = () => (
  <div style={{ display: 'flex', gap: '1rem', padding: '2rem', backgroundColor: 'var(--color-bg-base)', alignItems: 'center' }}>
    <StatusIndicator state="live" />
    <StatusIndicator state="paused" />
    <StatusIndicator state="offline" />
    <StatusIndicator state="error" />
  </div>
);

export const InContext = () => (
  <div style={{ padding: '2rem', backgroundColor: 'var(--color-bg-base)' }}>
    <div style={{
      display: 'flex',
      alignItems: 'center',
      gap: 'var(--space-3)',
      padding: 'var(--space-3) var(--space-4)',
      backgroundColor: 'var(--color-bg-surface)',
      borderRadius: 'var(--radius-md)',
      border: '1px solid var(--color-border)'
    }}>
      <StatusIndicator state="live" />
      <span style={{
        fontFamily: 'var(--font-mono)',
        fontSize: 'var(--text-sm)',
        color: 'var(--color-text-primary)'
      }}>
        Session: abc123
      </span>
    </div>
  </div>
);
