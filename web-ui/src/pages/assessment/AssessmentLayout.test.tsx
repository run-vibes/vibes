import { describe, test, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';

// Track current pathname for dynamic mock
let currentPathname = '/groove/assessment';

// Helper to determine if a link should be active based on TanStack Router logic
function isLinkActive(to: string, exact: boolean): boolean {
  if (exact) {
    return currentPathname === to || currentPathname === `${to}/`;
  }
  return currentPathname.startsWith(to);
}

// Mock TanStack Router with activeOptions support
vi.mock('@tanstack/react-router', () => ({
  Link: ({
    to,
    children,
    className,
    activeOptions,
    activeProps,
    inactiveProps,
  }: {
    to: string;
    children: React.ReactNode;
    className?: string;
    activeOptions?: { exact?: boolean };
    activeProps?: { className?: string };
    inactiveProps?: { className?: string };
  }) => {
    const exact = activeOptions?.exact ?? false;
    const isActive = isLinkActive(to, exact);
    const finalClassName = isActive ? activeProps?.className : inactiveProps?.className || className;
    return <a href={to} className={finalClassName}>{children}</a>;
  },
  Outlet: () => <div data-testid="outlet">Outlet Content</div>,
}));

// Import after mock setup
import { AssessmentLayout } from './AssessmentLayout';

describe('AssessmentLayout', () => {
  beforeEach(() => {
    currentPathname = '/groove/assessment';
  });

  test('renders subnav with all assessment tabs', () => {
    render(<AssessmentLayout />);

    expect(screen.getByRole('link', { name: /stream/i })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /status/i })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /history/i })).toBeInTheDocument();
  });

  test('renders outlet for child routes', () => {
    render(<AssessmentLayout />);

    expect(screen.getByTestId('outlet')).toBeInTheDocument();
  });

  test('stream tab links to /groove/assessment', () => {
    render(<AssessmentLayout />);

    const streamLink = screen.getByRole('link', { name: /stream/i });
    expect(streamLink).toHaveAttribute('href', '/groove/assessment');
  });

  test('status tab links to /groove/assessment/status', () => {
    render(<AssessmentLayout />);

    const statusLink = screen.getByRole('link', { name: /status/i });
    expect(statusLink).toHaveAttribute('href', '/groove/assessment/status');
  });

  test('history tab links to /groove/assessment/history', () => {
    render(<AssessmentLayout />);

    const historyLink = screen.getByRole('link', { name: /history/i });
    expect(historyLink).toHaveAttribute('href', '/groove/assessment/history');
  });

  describe('active tab highlighting', () => {
    test('stream tab is active on /groove/assessment', () => {
      currentPathname = '/groove/assessment';
      render(<AssessmentLayout />);

      const streamLink = screen.getByRole('link', { name: /stream/i });
      const statusLink = screen.getByRole('link', { name: /status/i });
      const historyLink = screen.getByRole('link', { name: /history/i });

      expect(streamLink).toHaveClass('active');
      expect(statusLink).not.toHaveClass('active');
      expect(historyLink).not.toHaveClass('active');
    });

    test('status tab is active on /groove/assessment/status', () => {
      currentPathname = '/groove/assessment/status';
      render(<AssessmentLayout />);

      const streamLink = screen.getByRole('link', { name: /stream/i });
      const statusLink = screen.getByRole('link', { name: /status/i });
      const historyLink = screen.getByRole('link', { name: /history/i });

      expect(streamLink).not.toHaveClass('active');
      expect(statusLink).toHaveClass('active');
      expect(historyLink).not.toHaveClass('active');
    });

    test('history tab is active on /groove/assessment/history', () => {
      currentPathname = '/groove/assessment/history';
      render(<AssessmentLayout />);

      const streamLink = screen.getByRole('link', { name: /stream/i });
      const statusLink = screen.getByRole('link', { name: /status/i });
      const historyLink = screen.getByRole('link', { name: /history/i });

      expect(streamLink).not.toHaveClass('active');
      expect(statusLink).not.toHaveClass('active');
      expect(historyLink).toHaveClass('active');
    });
  });
});
