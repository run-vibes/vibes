// design-system/src/tokens/Colors.stories.tsx
import './index.css';

export default {
  title: 'Tokens/Colors',
};

const colorGroups = {
  Backgrounds: [
    { name: '--color-bg-base', label: 'Base' },
    { name: '--color-bg-surface', label: 'Surface' },
    { name: '--color-bg-elevated', label: 'Elevated' },
    { name: '--color-bg-overlay', label: 'Overlay' },
  ],
  Text: [
    { name: '--color-text-primary', label: 'Primary' },
    { name: '--color-text-secondary', label: 'Secondary' },
    { name: '--color-text-dim', label: 'Dim' },
    { name: '--color-text-inverse', label: 'Inverse' },
  ],
  Semantic: [
    { name: '--color-accent', label: 'Accent (Amber)' },
    { name: '--color-success', label: 'Success' },
    { name: '--color-warning', label: 'Warning (same as accent)' },
    { name: '--color-error', label: 'Error' },
    { name: '--color-info', label: 'Info' },
  ],
  Plugins: [
    { name: '--color-groove', label: 'groove Gold' },
  ],
  Borders: [
    { name: '--color-border-subtle', label: 'Subtle' },
    { name: '--color-border', label: 'Default' },
    { name: '--color-border-strong', label: 'Strong' },
  ],
  Interactive: [
    { name: '--color-hover', label: 'Hover' },
    { name: '--color-active', label: 'Active' },
    { name: '--color-focus', label: 'Focus' },
  ],
};

function ColorSwatch({ name, label }: { name: string; label: string }) {
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: '1rem', marginBottom: '0.5rem' }}>
      <div
        style={{
          width: '3rem',
          height: '3rem',
          backgroundColor: `var(${name})`,
          borderRadius: '4px',
          border: '1px solid var(--color-border)',
        }}
      />
      <div>
        <div style={{ color: 'var(--color-text-primary)', fontWeight: 500 }}>{label}</div>
        <code style={{ color: 'var(--color-text-dim)', fontSize: '0.75rem' }}>{name}</code>
      </div>
    </div>
  );
}

export const Colors = () => (
  <div style={{ padding: '2rem', backgroundColor: 'var(--color-bg-base)', minHeight: '100vh' }}>
    {Object.entries(colorGroups).map(([group, colors]) => (
      <div key={group} style={{ marginBottom: '2rem' }}>
        <h2 style={{ color: 'var(--color-text-primary)', marginBottom: '1rem' }}>{group}</h2>
        {colors.map((color) => (
          <ColorSwatch key={color.name} {...color} />
        ))}
      </div>
    ))}
  </div>
);
