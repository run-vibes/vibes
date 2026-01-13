import '../../tokens/index.css';
import { Card } from './Card';
import { Button } from '../Button';

export default {
  title: 'Primitives/Card',
};

export const Default = () => (
  <div style={{ padding: '2rem', backgroundColor: 'var(--screen)' }}>
    <Card>
      <p style={{ margin: 0, color: 'var(--text)' }}>
        Default card with surface background.
      </p>
    </Card>
  </div>
);

export const WithTitle = () => (
  <div style={{ padding: '2rem', backgroundColor: 'var(--screen)' }}>
    <Card title="Session Details">
      <p style={{ margin: 0, color: 'var(--text)' }}>
        Card content with a title header.
      </p>
    </Card>
  </div>
);

export const WithActions = () => (
  <div style={{ padding: '2rem', backgroundColor: 'var(--screen)' }}>
    <Card title="Active Session" actions={<Button size="sm">Connect</Button>}>
      <p style={{ margin: 0, color: 'var(--text)' }}>
        Card with title and action button.
      </p>
    </Card>
  </div>
);

export const Variants = () => (
  <div style={{ padding: '2rem', backgroundColor: 'var(--screen)', display: 'flex', flexDirection: 'column', gap: '1rem' }}>
    <Card title="Default">
      <p style={{ margin: 0, color: 'var(--text-dim)' }}>
        Standard surface background with border.
      </p>
    </Card>
    <Card variant="elevated" title="Elevated">
      <p style={{ margin: 0, color: 'var(--text-dim)' }}>
        Elevated with stronger border and shadow.
      </p>
    </Card>
    <Card variant="inset" title="Inset">
      <p style={{ margin: 0, color: 'var(--text-dim)' }}>
        Inset for nested content areas.
      </p>
    </Card>
    <Card variant="crt" title="CRT">
      <p style={{ margin: 0, color: 'var(--text-dim)' }}>
        Sharp corners with phosphor glow title for dashboard cards.
      </p>
    </Card>
  </div>
);

export const CRTDashboard = () => (
  <div style={{ padding: '2rem', backgroundColor: 'var(--screen)', display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '1rem' }}>
    <Card variant="crt" title="Learnings">
      <p style={{ margin: 0, color: 'var(--text)' }}>
        Recent patterns discovered from sessions.
      </p>
    </Card>
    <Card variant="crt" title="Attribution">
      <p style={{ margin: 0, color: 'var(--text)' }}>
        Top contributors and impact metrics.
      </p>
    </Card>
    <Card variant="crt" title="Strategy">
      <p style={{ margin: 0, color: 'var(--text)' }}>
        Current distribution and overrides.
      </p>
    </Card>
    <Card variant="crt" title="Health">
      <p style={{ margin: 0, color: 'var(--text)' }}>
        System status and diagnostics.
      </p>
    </Card>
  </div>
);

export const NoPadding = () => (
  <div style={{ padding: '2rem', backgroundColor: 'var(--screen)' }}>
    <Card title="Terminal Output" noPadding>
      <div style={{
        backgroundColor: 'var(--surface)',
        padding: 'var(--space-4)',
        fontFamily: 'var(--font-mono)',
        fontSize: 'var(--font-size-sm)',
        color: 'var(--text-dim)'
      }}>
        $ vibes claude<br />
        Starting session...<br />
        Connected to daemon on localhost:3000
      </div>
    </Card>
  </div>
);
