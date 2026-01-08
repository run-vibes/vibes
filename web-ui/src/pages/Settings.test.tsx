import { describe, test, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { SettingsPage } from './Settings';

// Mock the tunnel status hook
vi.mock('../hooks/useTunnelStatus', () => ({
  useTunnelStatus: () => ({
    data: {
      state: 'connected',
      mode: 'quick',
      url: 'https://tunnel.example.com',
      tunnel_name: null,
      error: null,
    },
    isLoading: false,
    error: null,
  }),
}));

// Mock the CRT effects hook
vi.mock('../hooks/useCrtEffects', () => ({
  useCrtEffects: () => ({
    enabled: true,
    setEffects: vi.fn(),
  }),
}));

describe('SettingsPage', () => {
  describe('Tunnel section', () => {
    test('displays tunnel status', () => {
      render(<SettingsPage />);

      // Should have a Tunnel section
      expect(screen.getByText('TUNNEL')).toBeInTheDocument();
      expect(screen.getByText('connected')).toBeInTheDocument();
    });

    test('displays tunnel URL when available', () => {
      render(<SettingsPage />);

      const tunnelLink = screen.getByRole('link', { name: /tunnel\.example\.com/i });
      expect(tunnelLink).toHaveAttribute('href', 'https://tunnel.example.com');
    });
  });

  describe('About section', () => {
    test('GitHub link points to run-vibes/vibes', () => {
      render(<SettingsPage />);

      const githubLink = screen.getByRole('link', { name: /github/i });
      expect(githubLink).toHaveAttribute('href', 'https://github.com/run-vibes/vibes');
    });

    test('Report Issue link points to run-vibes/vibes/issues', () => {
      render(<SettingsPage />);

      const issueLink = screen.getByRole('link', { name: /report issue/i });
      expect(issueLink).toHaveAttribute('href', 'https://github.com/run-vibes/vibes/issues');
    });
  });
});
