import { describe, test, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { SessionCard } from './SessionCard';

// Mock TanStack Router Link
vi.mock('@tanstack/react-router', () => ({
  Link: ({ to, params, children, className }: { to: string; params?: Record<string, string>; children: React.ReactNode; className?: string }) => {
    // Interpolate params into the URL
    let href = to;
    if (params) {
      for (const [key, value] of Object.entries(params)) {
        href = href.replace(`$${key}`, value);
      }
    }
    return <a href={href} className={className}>{children}</a>;
  },
}));

describe('SessionCard', () => {
  test('links to /sessions/$sessionId', () => {
    render(
      <SessionCard
        id="test-session-id-12345"
        state="idle"
        createdAt={Date.now()}
      />
    );

    const link = screen.getByRole('link');
    expect(link).toHaveAttribute('href', '/sessions/test-session-id-12345');
  });

  test('applies session-active class for processing state', () => {
    render(
      <SessionCard
        id="test-session-id"
        state="processing"
        createdAt={Date.now()}
      />
    );

    const link = screen.getByRole('link');
    expect(link.className).toContain('session-active');
  });

  test('applies session-active class for waiting state', () => {
    render(
      <SessionCard
        id="test-session-id"
        state="waiting_permission"
        createdAt={Date.now()}
      />
    );

    const link = screen.getByRole('link');
    expect(link.className).toContain('session-active');
  });

  test('applies session-inactive class for idle state', () => {
    render(
      <SessionCard
        id="test-session-id"
        state="idle"
        createdAt={Date.now()}
      />
    );

    const link = screen.getByRole('link');
    expect(link.className).toContain('session-inactive');
  });

  test('applies session-inactive class for finished state', () => {
    render(
      <SessionCard
        id="test-session-id"
        state="finished"
        createdAt={Date.now()}
      />
    );

    const link = screen.getByRole('link');
    expect(link.className).toContain('session-inactive');
  });

  test('renders status dot with correct class', () => {
    const { container } = render(
      <SessionCard
        id="test-session-id"
        state="processing"
        createdAt={Date.now()}
      />
    );

    const statusDot = container.querySelector('.status-dot');
    expect(statusDot).toBeInTheDocument();
    expect(statusDot?.className).toContain('status-dot-processing');
  });
});
