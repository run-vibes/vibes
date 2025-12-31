import '../../tokens/index.css';
import { Panel } from './Panel';
import { Button } from '../Button';

export default {
  title: 'Primitives/Panel',
};

export const Default = () => (
  <div style={{ padding: '2rem', backgroundColor: 'var(--color-bg-base)' }}>
    <Panel>
      <p style={{ margin: 0, color: 'var(--color-text-primary)' }}>
        Default panel with surface background.
      </p>
    </Panel>
  </div>
);

export const WithTitle = () => (
  <div style={{ padding: '2rem', backgroundColor: 'var(--color-bg-base)' }}>
    <Panel title="Session Details">
      <p style={{ margin: 0, color: 'var(--color-text-primary)' }}>
        Panel content with a title header.
      </p>
    </Panel>
  </div>
);

export const WithActions = () => (
  <div style={{ padding: '2rem', backgroundColor: 'var(--color-bg-base)' }}>
    <Panel title="Active Session" actions={<Button size="sm">Connect</Button>}>
      <p style={{ margin: 0, color: 'var(--color-text-primary)' }}>
        Panel with title and action button.
      </p>
    </Panel>
  </div>
);

export const Variants = () => (
  <div style={{ padding: '2rem', backgroundColor: 'var(--color-bg-base)', display: 'flex', flexDirection: 'column', gap: '1rem' }}>
    <Panel title="Default">
      <p style={{ margin: 0, color: 'var(--color-text-secondary)' }}>
        Standard surface background with border.
      </p>
    </Panel>
    <Panel variant="elevated" title="Elevated">
      <p style={{ margin: 0, color: 'var(--color-text-secondary)' }}>
        Elevated with stronger border and shadow.
      </p>
    </Panel>
    <Panel variant="inset" title="Inset">
      <p style={{ margin: 0, color: 'var(--color-text-secondary)' }}>
        Inset for nested content areas.
      </p>
    </Panel>
  </div>
);

export const NoPadding = () => (
  <div style={{ padding: '2rem', backgroundColor: 'var(--color-bg-base)' }}>
    <Panel title="Terminal Output" noPadding>
      <div style={{
        backgroundColor: 'var(--color-bg-base)',
        padding: 'var(--space-4)',
        fontFamily: 'var(--font-mono)',
        fontSize: 'var(--text-sm)',
        color: 'var(--color-text-dim)'
      }}>
        $ vibes claude<br />
        Starting session...<br />
        Connected to daemon on localhost:3000
      </div>
    </Panel>
  </div>
);
