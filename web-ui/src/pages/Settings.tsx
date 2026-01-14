// web-ui/src/pages/Settings.tsx
import { useState, useEffect } from 'react';
import { Button, Card } from '@vibes/design-system';
import { NotificationSettings } from '../components/NotificationSettings';
import { useTunnelStatus } from '../hooks/useTunnelStatus';
import { useCrtEffects } from '../hooks/useCrtEffects';
import { useGrooveSettings } from '../hooks/useGrooveSettings';
import { useModels } from '../hooks/useModels';
import { useWebSocket } from '../hooks/useWebSocket';
import './Settings.css';

export function SettingsPage() {
  const { data: tunnel, isLoading: tunnelLoading, error: tunnelError } = useTunnelStatus();
  const { enabled: crtEffectsEnabled, setEffects: setCrtEffects } = useCrtEffects();
  const { settings: grooveSettings, updateSetting: updateGrooveSetting } = useGrooveSettings();
  const { send, addMessageHandler, isConnected } = useWebSocket();
  const { providers, credentials } = useModels({ send, addMessageHandler, isConnected });
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
        <h1>SETTINGS</h1>
      </div>

      <div className="settings-content">
        <Card variant="crt" title="APPEARANCE" className="settings-panel">
          <div className="setting-row">
            <div className="setting-info">
              <div className="setting-label">Theme</div>
              <span className="setting-description">Choose between dark and light mode</span>
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
              <span className="setting-description">Enable retro CRT scanlines and vignette</span>
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
        </Card>

        <Card variant="crt" title="NOTIFICATIONS" className="settings-panel">
          <NotificationSettings />
        </Card>

        <Card variant="crt" title="GROOVE" className="settings-panel">
          <div className="setting-row">
            <div className="setting-info">
              <div className="setting-label">Learning Indicator</div>
              <span className="setting-description">Show learning status indicator in header</span>
            </div>
            <div className="setting-control">
              <div className="theme-toggle">
                <button
                  className={`theme-option ${grooveSettings.showLearningIndicator ? 'active' : ''}`}
                  onClick={() => updateGrooveSetting('showLearningIndicator', true)}
                >
                  On
                </button>
                <button
                  className={`theme-option ${!grooveSettings.showLearningIndicator ? 'active' : ''}`}
                  onClick={() => updateGrooveSetting('showLearningIndicator', false)}
                >
                  Off
                </button>
              </div>
            </div>
          </div>
          <div className="setting-row">
            <div className="setting-info">
              <div className="setting-label">Dashboard Auto-Refresh</div>
              <span className="setting-description">Automatically refresh dashboard data</span>
            </div>
            <div className="setting-control">
              <div className="theme-toggle">
                <button
                  className={`theme-option ${grooveSettings.dashboardAutoRefresh ? 'active' : ''}`}
                  onClick={() => updateGrooveSetting('dashboardAutoRefresh', true)}
                >
                  On
                </button>
                <button
                  className={`theme-option ${!grooveSettings.dashboardAutoRefresh ? 'active' : ''}`}
                  onClick={() => updateGrooveSetting('dashboardAutoRefresh', false)}
                >
                  Off
                </button>
              </div>
            </div>
          </div>
        </Card>

        <Card variant="crt" title="CREDENTIALS" className="settings-panel">
          {providers.length === 0 ? (
            <div className="setting-row">
              <span className="setting-description">No providers configured</span>
            </div>
          ) : (
            providers.map((provider) => {
              const cred = credentials.find((c) => c.provider === provider);
              return (
                <div key={provider} className="setting-row">
                  <div className="setting-info">
                    <div className="setting-label">{provider}</div>
                    <span className="setting-description">
                      {cred ? `Configured via ${cred.source}` : 'Not configured'}
                    </span>
                  </div>
                  <div className="setting-control">
                    <CredentialStatusBadge configured={!!cred} source={cred?.source} />
                  </div>
                </div>
              );
            })
          )}
        </Card>

        <Card variant="crt" title="TUNNEL" className="settings-panel">
          {tunnelLoading && <span className="setting-description">Loading...</span>}
          {tunnelError && <span className="setting-error">Error loading tunnel status</span>}
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
                  <span className="setting-value">{tunnel.mode || 'Not configured'}</span>
                </div>
              </div>
              {tunnel.url && (
                <div className="setting-row">
                  <div className="setting-info">
                    <div className="setting-label">URL</div>
                  </div>
                  <div className="setting-control">
                    <a href={tunnel.url} target="_blank" rel="noopener noreferrer" className="setting-link">
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
                    <span className="setting-value">{tunnel.tunnel_name}</span>
                  </div>
                </div>
              )}
              {tunnel.error && (
                <div className="setting-row">
                  <div className="setting-info">
                    <div className="setting-label">Error</div>
                  </div>
                  <div className="setting-control">
                    <span className="setting-error">{tunnel.error}</span>
                  </div>
                </div>
              )}
            </div>
          )}
        </Card>

        <Card variant="crt" title="DATA & STORAGE" className="settings-panel">
          <div className="setting-row">
            <div className="setting-info">
              <div className="setting-label">Clear Local Data</div>
              <span className="setting-description">Remove all cached settings and preferences</span>
            </div>
            <div className="setting-control">
              {showClearConfirm ? (
                <div className="confirm-dialog">
                  <span className="setting-description">Clear all settings?</span>
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
        </Card>

        <Card variant="crt" title="ABOUT" className="settings-panel">
          <div className="about-content">
            <div className="about-logo">vibes</div>
            <span className="setting-description">Remote control for your Claude Code sessions</span>
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
        </Card>
      </div>
    </div>
  );
}

function CredentialStatusBadge({ configured, source }: { configured: boolean; source?: string }) {
  if (!configured) {
    return (
      <span
        className="credential-badge"
        style={{
          backgroundColor: 'var(--status-disabled-subtle)',
          color: 'var(--status-disabled)',
        }}
      >
        Not configured
      </span>
    );
  }

  return (
    <span
      className="credential-badge"
      style={{
        backgroundColor: 'var(--status-connected-subtle)',
        color: 'var(--status-connected)',
      }}
    >
      <span className="credential-badge-dot">●</span>
      {source || 'configured'}
    </span>
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
      className="tunnel-badge"
      style={{
        backgroundColor: style.bgColor,
        color: style.color,
      }}
    >
      <span className="tunnel-badge-dot">●</span>
      {state}
    </span>
  );
}
