import { describe, test, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { TunnelBadge } from './TunnelBadge';

// Mock TanStack Router Link
vi.mock('@tanstack/react-router', () => ({
  Link: ({ to, children }: { to: string; children: React.ReactNode }) => (
    <a href={to} data-testid="tunnel-link">{children}</a>
  ),
}));

// Mock the tunnel status hook
vi.mock('../hooks/useTunnelStatus', () => ({
  useTunnelStatus: () => ({
    data: { state: 'connected', url: 'https://tunnel.example.com' },
    isLoading: false,
  }),
}));

describe('TunnelBadge', () => {
  test('links to /settings', () => {
    render(<TunnelBadge />);

    const link = screen.getByTestId('tunnel-link');
    expect(link).toHaveAttribute('href', '/settings');
  });
});
