import { useState } from 'react';
import '../../tokens/index.css';
import { Header } from './Header';

export default {
  title: 'Compositions/Header',
};

const defaultNavItems = [
  { label: 'SESSIONS', href: '/sessions', isActive: true },
  { label: 'FIREHOSE', href: '/firehose' },
  { label: 'MODELS', href: '/models' },
  { label: 'GROOVE', href: '/groove', isGroove: true, hasSubnav: true },
];

export const Default = () => (
  <Header
    navItems={defaultNavItems}
    onThemeToggle={() => {}}
    settingsHref="/settings"
  />
);

export const WithIdentity = () => (
  <Header
    navItems={defaultNavItems}
    identity={{ email: 'user@example.com' }}
    onThemeToggle={() => {}}
    settingsHref="/settings"
  />
);

export const WithGrooveActive = () => (
  <Header
    navItems={[
      { label: 'SESSIONS', href: '/sessions' },
      { label: 'FIREHOSE', href: '/firehose' },
      { label: 'MODELS', href: '/models' },
      { label: 'GROOVE', href: '/groove', isGroove: true, isActive: true, hasSubnav: true },
    ]}
    identity={{ email: 'user@example.com' }}
    onThemeToggle={() => {}}
    settingsHref="/settings"
  />
);

export const WithThemeToggle = () => {
  const [theme, setTheme] = useState<'dark' | 'light'>('dark');

  return (
    <div data-theme={theme} style={{ background: 'var(--screen)', minHeight: '100vh' }}>
      <Header
        navItems={defaultNavItems}
        theme={theme}
        onThemeToggle={() => setTheme(t => t === 'dark' ? 'light' : 'dark')}
        settingsHref="/settings"
      />
    </div>
  );
};

export const FullFeatured = () => (
  <Header
    navItems={defaultNavItems}
    identity={{ email: 'admin@example.com', provider: 'cloudflare' }}
    theme="dark"
    onThemeToggle={() => {}}
    settingsHref="/settings"
  />
);

// Mobile viewport stories
export const MobileCollapsed = () => (
  <div style={{ maxWidth: '430px', margin: '0 auto', border: '1px solid var(--border)' }}>
    <Header
      navItems={defaultNavItems}
      onThemeToggle={() => {}}
      settingsHref="/settings"
    />
    <p style={{ padding: 'var(--space-4)', color: 'var(--text-dim)', fontSize: 'var(--font-size-sm)' }}>
      ↑ On mobile (≤768px), only the logo and hamburger are visible.
      Click the hamburger to open the slide-out menu.
    </p>
  </div>
);
MobileCollapsed.meta = {
  width: 430,
};

export const MobileMenuOpen = () => {
  const [isOpen, setIsOpen] = useState(true);

  return (
    <div style={{ maxWidth: '430px', margin: '0 auto', border: '1px solid var(--border)', minHeight: '100vh', position: 'relative' }}>
      <Header
        navItems={defaultNavItems}
        onThemeToggle={() => {}}
        settingsHref="/settings"
      />
      <p style={{ padding: 'var(--space-4)', color: 'var(--text-dim)', fontSize: 'var(--font-size-sm)' }}>
        The mobile menu is open. Try closing it with the ✕ button, clicking the overlay, or pressing Escape.
      </p>
    </div>
  );
};
MobileMenuOpen.meta = {
  width: 430,
};

export const TabletView = () => (
  <div style={{ maxWidth: '768px', margin: '0 auto', border: '1px solid var(--border)' }}>
    <Header
      navItems={defaultNavItems}
      onThemeToggle={() => {}}
      settingsHref="/settings"
    />
    <p style={{ padding: 'var(--space-4)', color: 'var(--text-dim)', fontSize: 'var(--font-size-sm)' }}>
      At exactly 768px, the mobile view activates. Resize the viewport to see the transition.
    </p>
  </div>
);
TabletView.meta = {
  width: 768,
};
