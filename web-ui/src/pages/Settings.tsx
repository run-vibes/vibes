// web-ui/src/pages/Settings.tsx
import { useState, useEffect } from 'react';
import { Panel, Button, Text } from '@vibes/design-system';
import { NotificationSettings } from '../components/NotificationSettings';
import { useTunnelStatus } from '../hooks/useTunnelStatus';
import { useCrtEffects } from '../hooks/useCrtEffects';
import './Settings.css';

export function SettingsPage() {
  const { data: tunnel, isLoading: tunnelLoading, error: tunnelError } = useTunnelStatus();
  const { enabled: crtEffectsEnabled, setEffects: setCrtEffects } = useCrtEffects();
  const [theme, setTheme] = useState<'dark' | 'light'>(() => {
    const saved = localStorage.getItem('vibes-theme');
    return (saved === 'light' || saved === 'dark') ? saved : 'dark';
  });
  const [showClearConfirm, setShowClearConfirm] = useState(false);

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
    setShowClearConfirm(true);
  };

  const confirmClearStorage = () => {
    localStorage.clear();
    window.location.reload();
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
                  Dark
                </button>
                <button
                  className={`theme-option ${theme === 'light' ? 'active' : ''}`}
                  onClick={() => handleThemeChange('light')}
                >
                  Light
                </button>
              </div>
            </div>
          </div>
          <div className="setting-row">
            <div className="setting-info">
              <div className="setting-label">CRT Effects</div>
              <Text intensity="dim">Enable retro CRT scanlines and vignette</Text>
            </div>
            <div className="setting-control">
              <div className="theme-toggle">
                <button
                  className={`theme-option ${crtEffectsEnabled ? 'active' : ''}`}
                  onClick={() => setCrtEffects(true)}
                >
                  On
                </button>
                <button
                  className={`theme-option ${!crtEffectsEnabled ? 'active' : ''}`}
                  onClick={() => setCrtEffects(false)}
                >
                  Off
                </button>
              </div>
            </div>
          </div>
        </Panel>

        <Panel title="Notifications" className="settings-panel">
          <NotificationSettings />
        </Panel>

        <Panel title="Tunnel" className="settings-panel">
          {tunnelLoading && <Text intensity="dim">Loading...</Text>}
          {tunnelError && <Text intensity="dim" style={{ color: 'var(--color-error)' }}>Error loading tunnel status</Text>}
          {tunnel && (
            <div className="tunnel-status">
              <div className="setting-row">
                <div className="setting-info">
                  <div className="setting-label">State</div>
                </div>
                <div className="setting-control">
                  <TunnelStatusBadge state={tunnel.state} />
                </div>
              </div>
              <div className="setting-row">
                <div className="setting-info">
                  <div className="setting-label">Mode</div>
                </div>
                <div className="setting-control">
                  <Text intensity="dim">{tunnel.mode || 'Not configured'}</Text>
                </div>
              </div>
              {tunnel.url && (
                <div className="setting-row">
                  <div className="setting-info">
                    <div className="setting-label">URL</div>
                  </div>
                  <div className="setting-control">
                    <a href={tunnel.url} target="_blank" rel="noopener noreferrer">
                      {tunnel.url}
                    </a>
                  </div>
                </div>
              )}
              {tunnel.tunnel_name && (
                <div className="setting-row">
                  <div className="setting-info">
                    <div className="setting-label">Tunnel Name</div>
                  </div>
                  <div className="setting-control">
                    <Text intensity="dim">{tunnel.tunnel_name}</Text>
                  </div>
                </div>
              )}
              {tunnel.error && (
                <div className="setting-row">
                  <div className="setting-info">
                    <div className="setting-label">Error</div>
                  </div>
                  <div className="setting-control">
                    <Text intensity="dim" style={{ color: 'var(--color-error)' }}>{tunnel.error}</Text>
                  </div>
                </div>
              )}
            </div>
          )}
        </Panel>

        <Panel title="Data & Storage" className="settings-panel">
          <div className="setting-row">
            <div className="setting-info">
              <div className="setting-label">Clear Local Data</div>
              <Text intensity="dim">Remove all cached settings and preferences</Text>
            </div>
            <div className="setting-control">
              {showClearConfirm ? (
                <div className="confirm-dialog">
                  <Text intensity="dim">Clear all settings?</Text>
                  <div className="confirm-buttons">
                    <Button variant="secondary" onClick={() => setShowClearConfirm(false)}>
                      Cancel
                    </Button>
                    <Button variant="primary" onClick={confirmClearStorage}>
                      Confirm
                    </Button>
                  </div>
                </div>
              ) : (
                <Button variant="secondary" onClick={handleClearStorage}>
                  Clear Storage
                </Button>
              )}
            </div>
          </div>
        </Panel>

        <Panel title="About" className="settings-panel">
          <div className="about-content">
            <div className="about-logo">vibes</div>
            <Text intensity="dim">Remote control for your Claude Code sessions</Text>
            <div className="about-links">
              <a href="https://github.com/run-vibes/vibes" target="_blank" rel="noopener noreferrer">
                GitHub
              </a>
              <span>•</span>
              <a href="https://github.com/run-vibes/vibes/issues" target="_blank" rel="noopener noreferrer">
                Report Issue
              </a>
            </div>
          </div>
        </Panel>
      </div>
    </div>
  );
}

function TunnelStatusBadge({ state }: { state: string }) {
  // Map tunnel states to design system status tokens
  const stateStyles: Record<string, { color: string; bgColor: string }> = {
    disabled: { color: 'var(--status-disabled)', bgColor: 'var(--status-disabled-subtle)' },
    starting: { color: 'var(--status-starting)', bgColor: 'var(--status-starting-subtle)' },
    connected: { color: 'var(--status-connected)', bgColor: 'var(--status-connected-subtle)' },
    reconnecting: { color: 'var(--status-starting)', bgColor: 'var(--status-starting-subtle)' },
    failed: { color: 'var(--status-failed)', bgColor: 'var(--status-failed-subtle)' },
    stopped: { color: 'var(--status-disabled)', bgColor: 'var(--status-disabled-subtle)' },
  };

  const style = stateStyles[state] || stateStyles.disabled;

  return (
    <span
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap: 'var(--space-2)',
        padding: 'var(--space-1) var(--space-3)',
        borderRadius: 'var(--radius-full)',
        backgroundColor: style.bgColor,
        color: style.color,
        fontSize: 'var(--font-size-sm)',
        fontWeight: 500,
      }}
    >
      <span style={{ fontSize: '0.5rem' }}>●</span>
      {state}
    </span>
  );
}
