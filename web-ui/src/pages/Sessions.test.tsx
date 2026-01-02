import { describe, test, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Sessions } from './Sessions';

// Mock TanStack Router
vi.mock('@tanstack/react-router', () => ({
  useNavigate: () => vi.fn(),
  Link: ({ to, children }: { to: string; children: React.ReactNode }) => (
    <a href={to}>{children}</a>
  ),
}));

// Mock the hooks
vi.mock('../hooks', () => ({
  useSessionList: () => ({
    sessions: [],
    isLoading: false,
    isCreating: false,
    error: null,
    refresh: vi.fn(),
    killSession: vi.fn(),
    createSession: vi.fn(),
  }),
  useWebSocket: () => ({
    send: vi.fn(),
    addMessageHandler: () => () => {},
    isConnected: true,
    connectionState: 'connected',
  }),
}));

describe('Sessions', () => {
  test('renders Sessions heading', () => {
    render(<Sessions />);
    expect(screen.getByRole('heading', { name: /sessions/i })).toBeInTheDocument();
  });
});
