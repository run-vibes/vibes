import { describe, test, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';

// Track current pathname for dynamic mock
let currentPathname = '/groove/dashboard';

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
import { DashboardLayout } from './DashboardLayout';

describe('DashboardLayout', () => {
  beforeEach(() => {
    currentPathname = '/groove/dashboard';
  });

  test('renders subnav with all dashboard tabs', () => {
    render(<DashboardLayout />);

    expect(screen.getByRole('link', { name: /overview/i })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /learnings/i })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /attribution/i })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /strategy/i })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /health/i })).toBeInTheDocument();
  });

  test('renders outlet for child routes', () => {
    render(<DashboardLayout />);

    expect(screen.getByTestId('outlet')).toBeInTheDocument();
  });

  test('overview tab links to /groove/dashboard/overview', () => {
    render(<DashboardLayout />);

    const link = screen.getByRole('link', { name: /overview/i });
    expect(link).toHaveAttribute('href', '/groove/dashboard/overview');
  });

  test('learnings tab links to /groove/dashboard/learnings', () => {
    render(<DashboardLayout />);

    const link = screen.getByRole('link', { name: /learnings/i });
    expect(link).toHaveAttribute('href', '/groove/dashboard/learnings');
  });

  test('attribution tab links to /groove/dashboard/attribution', () => {
    render(<DashboardLayout />);

    const link = screen.getByRole('link', { name: /attribution/i });
    expect(link).toHaveAttribute('href', '/groove/dashboard/attribution');
  });

  test('strategy tab links to /groove/dashboard/strategy', () => {
    render(<DashboardLayout />);

    const link = screen.getByRole('link', { name: /strategy/i });
    expect(link).toHaveAttribute('href', '/groove/dashboard/strategy');
  });

  test('health tab links to /groove/dashboard/health', () => {
    render(<DashboardLayout />);

    const link = screen.getByRole('link', { name: /health/i });
    expect(link).toHaveAttribute('href', '/groove/dashboard/health');
  });

  describe('active tab highlighting', () => {
    test('overview tab is active on /groove/dashboard/overview', () => {
      currentPathname = '/groove/dashboard/overview';
      render(<DashboardLayout />);

      const overviewLink = screen.getByRole('link', { name: /overview/i });
      const learningsLink = screen.getByRole('link', { name: /learnings/i });
      const attributionLink = screen.getByRole('link', { name: /attribution/i });
      const strategyLink = screen.getByRole('link', { name: /strategy/i });
      const healthLink = screen.getByRole('link', { name: /health/i });

      expect(overviewLink).toHaveClass('active');
      expect(learningsLink).not.toHaveClass('active');
      expect(attributionLink).not.toHaveClass('active');
      expect(strategyLink).not.toHaveClass('active');
      expect(healthLink).not.toHaveClass('active');
    });

    test('learnings tab is active on /groove/dashboard/learnings', () => {
      currentPathname = '/groove/dashboard/learnings';
      render(<DashboardLayout />);

      const overviewLink = screen.getByRole('link', { name: /overview/i });
      const learningsLink = screen.getByRole('link', { name: /learnings/i });
      const attributionLink = screen.getByRole('link', { name: /attribution/i });
      const strategyLink = screen.getByRole('link', { name: /strategy/i });
      const healthLink = screen.getByRole('link', { name: /health/i });

      expect(overviewLink).not.toHaveClass('active');
      expect(learningsLink).toHaveClass('active');
      expect(attributionLink).not.toHaveClass('active');
      expect(strategyLink).not.toHaveClass('active');
      expect(healthLink).not.toHaveClass('active');
    });

    test('attribution tab is active on /groove/dashboard/attribution', () => {
      currentPathname = '/groove/dashboard/attribution';
      render(<DashboardLayout />);

      const overviewLink = screen.getByRole('link', { name: /overview/i });
      const learningsLink = screen.getByRole('link', { name: /learnings/i });
      const attributionLink = screen.getByRole('link', { name: /attribution/i });
      const strategyLink = screen.getByRole('link', { name: /strategy/i });
      const healthLink = screen.getByRole('link', { name: /health/i });

      expect(overviewLink).not.toHaveClass('active');
      expect(learningsLink).not.toHaveClass('active');
      expect(attributionLink).toHaveClass('active');
      expect(strategyLink).not.toHaveClass('active');
      expect(healthLink).not.toHaveClass('active');
    });

    test('strategy tab is active on /groove/dashboard/strategy', () => {
      currentPathname = '/groove/dashboard/strategy';
      render(<DashboardLayout />);

      const overviewLink = screen.getByRole('link', { name: /overview/i });
      const learningsLink = screen.getByRole('link', { name: /learnings/i });
      const attributionLink = screen.getByRole('link', { name: /attribution/i });
      const strategyLink = screen.getByRole('link', { name: /strategy/i });
      const healthLink = screen.getByRole('link', { name: /health/i });

      expect(overviewLink).not.toHaveClass('active');
      expect(learningsLink).not.toHaveClass('active');
      expect(attributionLink).not.toHaveClass('active');
      expect(strategyLink).toHaveClass('active');
      expect(healthLink).not.toHaveClass('active');
    });

    test('health tab is active on /groove/dashboard/health', () => {
      currentPathname = '/groove/dashboard/health';
      render(<DashboardLayout />);

      const overviewLink = screen.getByRole('link', { name: /overview/i });
      const learningsLink = screen.getByRole('link', { name: /learnings/i });
      const attributionLink = screen.getByRole('link', { name: /attribution/i });
      const strategyLink = screen.getByRole('link', { name: /strategy/i });
      const healthLink = screen.getByRole('link', { name: /health/i });

      expect(overviewLink).not.toHaveClass('active');
      expect(learningsLink).not.toHaveClass('active');
      expect(attributionLink).not.toHaveClass('active');
      expect(strategyLink).not.toHaveClass('active');
      expect(healthLink).toHaveClass('active');
    });
  });
});
