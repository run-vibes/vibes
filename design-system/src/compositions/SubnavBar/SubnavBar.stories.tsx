import { useState } from 'react';
import '../../tokens/index.css';
import { SubnavBar } from './SubnavBar';

export default {
  title: 'Compositions/SubnavBar',
};

const defaultItems = [
  { label: 'Status', href: '/groove/status', isActive: true },
  { label: 'Learnings', href: '/groove/learnings' },
  { label: 'Overrides', href: '/groove/overrides' },
  { label: 'Attribution', href: '/groove/attribution' },
];

const moreItems = [
  { label: 'Settings', href: '/groove/settings' },
  { label: 'History', href: '/groove/history' },
];

export const Default = () => (
  <div style={{ background: 'var(--screen)', padding: 'var(--space-4)' }}>
    <SubnavBar
      isOpen={true}
      label="GROOVE"
      items={defaultItems}
    />
  </div>
);

export const WithMoreMenu = () => (
  <div style={{ background: 'var(--screen)', padding: 'var(--space-4)' }}>
    <SubnavBar
      isOpen={true}
      label="GROOVE"
      items={defaultItems}
      moreItems={moreItems}
    />
  </div>
);

export const GroovePlugin = () => (
  <div style={{ background: 'var(--screen)', padding: 'var(--space-4)' }}>
    <SubnavBar
      isOpen={true}
      label="GROOVE"
      items={defaultItems}
      plugin="groove"
    />
  </div>
);

export const Collapsed = () => (
  <div style={{ background: 'var(--screen)', padding: 'var(--space-4)', border: '1px solid var(--border)' }}>
    <SubnavBar
      isOpen={false}
      label="GROOVE"
      items={defaultItems}
    />
    <p style={{ color: 'var(--text-dim)', fontSize: 'var(--font-size-sm)', marginTop: 'var(--space-4)' }}>
      SubnavBar is collapsed (isOpen=false). Height is 0.
    </p>
  </div>
);

export const DarkTheme = () => (
  <div data-theme="dark" style={{ background: 'var(--screen)', padding: 'var(--space-4)' }}>
    <SubnavBar
      isOpen={true}
      label="GROOVE"
      items={defaultItems}
      moreItems={moreItems}
      plugin="groove"
    />
  </div>
);

export const LightTheme = () => (
  <div data-theme="light" style={{ background: 'var(--screen)', padding: 'var(--space-4)' }}>
    <SubnavBar
      isOpen={true}
      label="GROOVE"
      items={defaultItems}
      moreItems={moreItems}
      plugin="groove"
    />
  </div>
);

export const ThemeComparison = () => {
  const [theme, setTheme] = useState<'dark' | 'light'>('dark');

  return (
    <div>
      <div style={{ marginBottom: 'var(--space-4)', padding: 'var(--space-2)' }}>
        <button
          onClick={() => setTheme(t => t === 'dark' ? 'light' : 'dark')}
          style={{
            padding: '8px 16px',
            background: '#333',
            color: '#fff',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
          }}
        >
          Toggle Theme (current: {theme})
        </button>
      </div>
      <div data-theme={theme} style={{ background: 'var(--screen)', padding: 'var(--space-4)', minHeight: '200px' }}>
        <SubnavBar
          isOpen={true}
          label="GROOVE"
          items={defaultItems}
          moreItems={moreItems}
          plugin="groove"
        />
        <p style={{ color: 'var(--text-dim)', marginTop: 'var(--space-6)', fontSize: 'var(--font-size-sm)' }}>
          Click the button above to toggle between dark and light themes.
          Notice how the SubnavBar adapts its background and hover states.
        </p>
      </div>
    </div>
  );
};

export const SideBySide = () => (
  <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 'var(--space-4)' }}>
    <div data-theme="dark" style={{ background: 'var(--screen)', padding: 'var(--space-4)' }}>
      <h3 style={{ color: 'var(--text)', marginBottom: 'var(--space-2)', fontFamily: 'var(--font-display)' }}>
        Dark Theme
      </h3>
      <SubnavBar
        isOpen={true}
        label="GROOVE"
        items={defaultItems}
        moreItems={moreItems}
        plugin="groove"
      />
    </div>
    <div data-theme="light" style={{ background: 'var(--screen)', padding: 'var(--space-4)' }}>
      <h3 style={{ color: 'var(--text)', marginBottom: 'var(--space-2)', fontFamily: 'var(--font-display)' }}>
        Light Theme
      </h3>
      <SubnavBar
        isOpen={true}
        label="GROOVE"
        items={defaultItems}
        moreItems={moreItems}
        plugin="groove"
      />
    </div>
  </div>
);
