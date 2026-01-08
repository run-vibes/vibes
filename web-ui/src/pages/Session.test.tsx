import { describe, test, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Session } from './Session';

// Mock TanStack Router
vi.mock('@tanstack/react-router', () => ({
  useParams: () => ({ sessionId: 'test-session-id' }),
  Link: ({ to, children, className }: { to: string; children: React.ReactNode; className?: string }) => (
    <a href={to} className={className}>{children}</a>
  ),
}));

// Mock the WebSocket hook
vi.mock('../hooks/useWebSocket', () => ({
  useWebSocket: () => ({
    isConnected: false,
    addMessageHandler: () => () => {},
    send: vi.fn(),
  }),
}));

// Mock the Terminal component to avoid xterm.js complexity
vi.mock('../components/Terminal', () => ({
  SessionTerminal: vi.fn(() => null),
}));

describe('Session', () => {
  test('back link goes to /sessions', () => {
    render(<Session />);

    const backLink = screen.getByRole('link', { name: /back/i });
    expect(backLink).toHaveAttribute('href', '/sessions');
  });
});
