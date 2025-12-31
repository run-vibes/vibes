// design-system/src/tokens/Spacing.stories.tsx
import './index.css';
import { useState } from 'react';

export default {
  title: 'Tokens/Spacing',
};

// Spacing scale with pixel values for labels
const spaces: Array<{ token: number; px: number }> = [
  { token: 0, px: 0 },
  { token: 1, px: 4 },
  { token: 2, px: 8 },
  { token: 3, px: 12 },
  { token: 4, px: 16 },
  { token: 5, px: 20 },
  { token: 6, px: 24 },
  { token: 8, px: 32 },
  { token: 10, px: 40 },
  { token: 12, px: 48 },
  { token: 16, px: 64 },
];

const transitions = [
  { name: 'fast', duration: '100ms' },
  { name: 'normal', duration: '200ms' },
  { name: 'slow', duration: '300ms' },
];

function TransitionDemo({ name, duration }: { name: string; duration: string }) {
  const [active, setActive] = useState(false);

  return (
    <div style={{ textAlign: 'center' }}>
      <div
        style={{
          width: '3rem',
          height: '3rem',
          backgroundColor: active ? 'var(--color-accent)' : 'var(--color-bg-elevated)',
          border: '1px solid var(--color-border)',
          borderRadius: 'var(--radius-md)',
          transition: `var(--transition-${name})`,
          cursor: 'pointer',
        }}
        onMouseEnter={() => setActive(true)}
        onMouseLeave={() => setActive(false)}
      />
      <code style={{ color: 'var(--color-text-dim)', fontSize: 'var(--text-xs)', display: 'block', marginTop: '0.5rem' }}>
        --transition-{name}
      </code>
      <span style={{ color: 'var(--color-text-secondary)', fontSize: 'var(--text-xs)' }}>
        {duration}
      </span>
    </div>
  );
}

export const Spacing = () => (
  <div style={{ padding: '2rem', backgroundColor: 'var(--color-bg-base)', minHeight: '100vh' }}>
    <h2 style={{ color: 'var(--color-text-primary)', marginBottom: '2rem' }}>Spacing Scale</h2>

    {spaces.map(({ token, px }) => (
      <div key={token} style={{ display: 'flex', alignItems: 'center', marginBottom: '0.5rem' }}>
        <code style={{ color: 'var(--color-text-dim)', width: '8rem', fontSize: 'var(--text-xs)' }}>
          --space-{token}
        </code>
        <div
          style={{
            width: token === 0 ? '2px' : `var(--space-${token})`,
            height: '1.5rem',
            backgroundColor: token === 0 ? 'var(--color-border)' : 'var(--color-accent)',
            borderRadius: 'var(--radius-sm)',
          }}
        />
        <span style={{ color: 'var(--color-text-secondary)', marginLeft: '1rem', fontSize: 'var(--text-sm)' }}>
          {px}px
        </span>
      </div>
    ))}

    <h2 style={{ color: 'var(--color-text-primary)', margin: '2rem 0 1rem' }}>Border Radius</h2>
    <div style={{ display: 'flex', gap: '1rem' }}>
      {['sm', 'md', 'lg', 'full'].map((r) => (
        <div key={r} style={{ textAlign: 'center' }}>
          <div
            style={{
              width: '3rem',
              height: '3rem',
              backgroundColor: 'var(--color-accent)',
              borderRadius: `var(--radius-${r})`,
            }}
          />
          <code style={{ color: 'var(--color-text-dim)', fontSize: 'var(--text-xs)' }}>
            --radius-{r}
          </code>
        </div>
      ))}
    </div>

    <h2 style={{ color: 'var(--color-text-primary)', margin: '2rem 0 1rem' }}>Transitions</h2>
    <p style={{ color: 'var(--color-text-secondary)', fontSize: 'var(--text-sm)', marginBottom: '1rem' }}>
      Hover over the boxes to see transition speeds
    </p>
    <div style={{ display: 'flex', gap: '2rem' }}>
      {transitions.map((t) => (
        <TransitionDemo key={t.name} {...t} />
      ))}
    </div>
  </div>
);
