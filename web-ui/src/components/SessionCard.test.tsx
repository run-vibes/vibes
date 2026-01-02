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
});
