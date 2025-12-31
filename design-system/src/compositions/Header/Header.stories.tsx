import { useState } from 'react';
import '../../tokens/index.css';
import { Header } from './Header';

export default {
  title: 'Compositions/Header',
};

export const Default = () => (
  <Header
    navItems={[
      { label: 'Sessions', href: '/sessions', isActive: true },
      { label: 'History', href: '/history' },
      { label: 'Settings', href: '/settings' },
    ]}
  />
);

export const WithIdentity = () => (
  <Header
    navItems={[
      { label: 'Sessions', href: '/sessions' },
      { label: 'History', href: '/history' },
    ]}
    identity={{ email: 'user@example.com' }}
  />
);

export const LocalMode = () => (
  <Header
    navItems={[
      { label: 'Sessions', href: '/sessions' },
    ]}
    isLocal
  />
);

export const WithGroove = () => (
  <Header
    navItems={[
      { label: 'Sessions', href: '/sessions' },
      { label: 'Groove', href: '/groove', isGroove: true },
    ]}
    identity={{ email: 'user@example.com' }}
  />
);

export const WithThemeToggle = () => {
  const [theme, setTheme] = useState<'dark' | 'light'>('dark');

  return (
    <div data-theme={theme}>
      <Header
        navItems={[
          { label: 'Sessions', href: '/sessions' },
        ]}
        theme={theme}
        onThemeToggle={() => setTheme(t => t === 'dark' ? 'light' : 'dark')}
      />
    </div>
  );
};

export const FullFeatured = () => (
  <Header
    navItems={[
      { label: 'Sessions', href: '/sessions', isActive: true },
      { label: 'History', href: '/history' },
      { label: 'Groove', href: '/groove', isGroove: true },
      { label: 'Settings', href: '/settings' },
    ]}
    identity={{ email: 'admin@example.com', provider: 'cloudflare' }}
    theme="dark"
    onThemeToggle={() => {}}
  />
);
