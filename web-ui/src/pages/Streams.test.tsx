import { describe, test, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { StreamsPage } from './Streams';

// Mock the hooks
vi.mock('../hooks/useWebSocket', () => ({
  useWebSocket: () => ({ isConnected: true }),
}));

vi.mock('../hooks/useTunnelStatus', () => ({
  useTunnelStatus: () => ({
    data: { state: 'connected', url: 'https://tunnel.example.com' },
  }),
}));

vi.mock('../hooks/useFirehose', () => ({
  useFirehose: () => ({ events: [], isConnected: true }),
}));

// Mock TanStack Router Link
vi.mock('@tanstack/react-router', () => ({
  Link: ({ to, children, className }: { to: string; children: React.ReactNode; className?: string }) => (
    <a href={to} className={className}>{children}</a>
  ),
}));

describe('StreamsPage', () => {
  test('does not have a Status card', () => {
    render(<StreamsPage />);

    // Status should NOT be present as a card title
    const cards = screen.getAllByRole('heading', { level: 3 });
    const cardTitles = cards.map(h => h.textContent);

    expect(cardTitles).not.toContain('Status');
  });

  test('has Firehose, Debug Console, and Sessions cards', () => {
    render(<StreamsPage />);

    // Check for card titles (h3 elements)
    const cards = screen.getAllByRole('heading', { level: 3 });
    const cardTitles = cards.map(h => h.textContent);

    expect(cardTitles).toContain('Firehose');
    expect(cardTitles).toContain('Debug Console');
    expect(cardTitles).toContain('Sessions');
  });

  test('Sessions card links to /sessions', () => {
    render(<StreamsPage />);

    const sessionsLink = screen.getByRole('link', { name: /sessions/i });
    expect(sessionsLink).toHaveAttribute('href', '/sessions');
  });
});
