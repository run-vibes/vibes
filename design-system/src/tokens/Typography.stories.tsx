// design-system/src/tokens/Typography.stories.tsx
import './index.css';

export default {
  title: 'Tokens/Typography',
};

const samples = [
  { size: '--text-2xl', label: '2XL (24px)', sample: 'vibes groove' },
  { size: '--text-xl', label: 'XL (20px)', sample: 'Session Manager' },
  { size: '--text-lg', label: 'LG (18px)', sample: 'Event Stream' },
  { size: '--text-base', label: 'Base (16px)', sample: 'Body text for descriptions and content.' },
  { size: '--text-sm', label: 'SM (14px)', sample: 'Secondary text and metadata' },
  { size: '--text-xs', label: 'XS (12px)', sample: 'Timestamps and labels' },
];

export const Typography = () => (
  <div style={{ padding: '2rem', backgroundColor: 'var(--color-bg-base)', minHeight: '100vh' }}>
    <h2 style={{ color: 'var(--color-text-primary)', marginBottom: '2rem' }}>Typography Scale</h2>

    <div style={{ marginBottom: '3rem' }}>
      <h3 style={{ color: 'var(--color-text-secondary)', marginBottom: '1rem' }}>Monospace (Primary)</h3>
      {samples.map(({ size, label, sample }) => (
        <div key={size} style={{ marginBottom: '1.5rem' }}>
          <code style={{ color: 'var(--color-text-dim)', fontSize: 'var(--text-xs)' }}>{label}</code>
          <div style={{
            fontFamily: 'var(--font-mono)',
            fontSize: `var(${size})`,
            color: 'var(--color-text-primary)',
            marginTop: '0.25rem',
          }}>
            {sample}
          </div>
        </div>
      ))}
    </div>

    <div>
      <h3 style={{ color: 'var(--color-text-secondary)', marginBottom: '1rem' }}>Sans-serif (UI)</h3>
      {samples.slice(3).map(({ size, label, sample }) => (
        <div key={size} style={{ marginBottom: '1.5rem' }}>
          <code style={{ color: 'var(--color-text-dim)', fontSize: 'var(--text-xs)' }}>{label}</code>
          <div style={{
            fontFamily: 'var(--font-sans)',
            fontSize: `var(${size})`,
            color: 'var(--color-text-primary)',
            marginTop: '0.25rem',
          }}>
            {sample}
          </div>
        </div>
      ))}
    </div>
  </div>
);
