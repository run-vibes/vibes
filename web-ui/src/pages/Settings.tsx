// web-ui/src/pages/Settings.tsx
import { useState, useEffect } from 'react';
import { Panel, Button, Text } from '@vibes/design-system';
import { NotificationSettings } from '../components/NotificationSettings';
import './Settings.css';

export function SettingsPage() {
  const [theme, setTheme] = useState<'dark' | 'light'>(() => {
    const saved = localStorage.getItem('vibes-theme');
    return (saved === 'light' || saved === 'dark') ? saved : 'dark';
  });

  const handleThemeChange = (newTheme: 'dark' | 'light') => {
    setTheme(newTheme);
    localStorage.setItem('vibes-theme', newTheme);
    document.documentElement.setAttribute('data-theme', newTheme);
  };

  // Sync theme with document
  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
  }, [theme]);

  const handleClearStorage = () => {
    if (confirm('Clear all local settings? This cannot be undone.')) {
      localStorage.clear();
      window.location.reload();
    }
  };

  return (
    <div className="settings-page">
      <div className="settings-header">
        <h1>Settings</h1>
        <Text intensity="dim">Configure your vibes experience</Text>
      </div>

      <div className="settings-content">
        <Panel title="Appearance" className="settings-panel">
          <div className="setting-row">
            <div className="setting-info">
              <div className="setting-label">Theme</div>
              <Text intensity="dim">Choose between dark and light mode</Text>
            </div>
            <div className="setting-control">
              <div className="theme-toggle">
                <button
                  className={`theme-option ${theme === 'dark' ? 'active' : ''}`}
                  onClick={() => handleThemeChange('dark')}
                >
                  🌙 Dark
                </button>
                <button
                  className={`theme-option ${theme === 'light' ? 'active' : ''}`}
                  onClick={() => handleThemeChange('light')}
                >
                  ☀️ Light
                </button>
              </div>
            </div>
          </div>
        </Panel>

        <Panel title="Notifications" className="settings-panel">
          <NotificationSettings />
        </Panel>

        <Panel title="Data & Storage" className="settings-panel">
          <div className="setting-row">
            <div className="setting-info">
              <div className="setting-label">Clear Local Data</div>
              <Text intensity="dim">Remove all cached settings and preferences</Text>
            </div>
            <div className="setting-control">
              <Button variant="secondary" onClick={handleClearStorage}>
                Clear Storage
              </Button>
            </div>
          </div>
        </Panel>

        <Panel title="About" className="settings-panel">
          <div className="about-content">
            <div className="about-logo">vibes</div>
            <Text intensity="dim">Remote control for your Claude Code sessions</Text>
            <div className="about-links">
              <a href="https://github.com/anthropics/vibes" target="_blank" rel="noopener noreferrer">
                GitHub
              </a>
              <span>•</span>
              <a href="https://github.com/anthropics/vibes/issues" target="_blank" rel="noopener noreferrer">
                Report Issue
              </a>
            </div>
          </div>
        </Panel>
      </div>
    </div>
  );
}
