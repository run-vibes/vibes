/**
 * Tests for the TrendCard component
 */
import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';

// Mock TanStack Router Link component
vi.mock('@tanstack/react-router', () => ({
  Link: ({ to, children }: { to: string; children: React.ReactNode }) => (
    <a href={to}>{children}</a>
  ),
}));

import { TrendCard } from './TrendCard';

describe('TrendCard', () => {
  it('renders title and primary metric', () => {
    render(
      <TrendCard
        title="Session Trends"
        primaryValue="12%"
        primaryLabel="improvement"
        trendDirection="rising"
      />
    );

    expect(screen.getByText('Session Trends')).toBeInTheDocument();
    expect(screen.getByText('12%')).toBeInTheDocument();
    expect(screen.getByText('improvement')).toBeInTheDocument();
  });

  it('shows rising trend indicator', () => {
    render(
      <TrendCard
        title="Test"
        primaryValue="10"
        primaryLabel="sessions"
        trendDirection="rising"
      />
    );

    expect(screen.getByText('↑')).toBeInTheDocument();
  });

  it('shows falling trend indicator', () => {
    render(
      <TrendCard
        title="Test"
        primaryValue="10"
        primaryLabel="sessions"
        trendDirection="falling"
      />
    );

    expect(screen.getByText('↓')).toBeInTheDocument();
  });

  it('shows stable trend indicator', () => {
    render(
      <TrendCard
        title="Test"
        primaryValue="10"
        primaryLabel="sessions"
        trendDirection="stable"
      />
    );

    expect(screen.getByText('→')).toBeInTheDocument();
  });

  it('renders secondary metrics when provided', () => {
    render(
      <TrendCard
        title="Test"
        primaryValue="10"
        primaryLabel="sessions"
        trendDirection="stable"
        secondaryMetrics={[
          { label: 'Total', value: '100' },
          { label: 'Active', value: '85' },
        ]}
      />
    );

    expect(screen.getByText('Total')).toBeInTheDocument();
    expect(screen.getByText('100')).toBeInTheDocument();
    expect(screen.getByText('Active')).toBeInTheDocument();
    expect(screen.getByText('85')).toBeInTheDocument();
  });

  it('renders sparkline placeholder', () => {
    render(
      <TrendCard
        title="Test"
        primaryValue="10"
        primaryLabel="sessions"
        trendDirection="rising"
        sparklineData={[1, 2, 3, 4, 5]}
      />
    );

    // Sparkline renders as a placeholder for now
    expect(screen.getByTestId('sparkline-placeholder')).toBeInTheDocument();
  });

  it('renders link in card footer when href is provided', () => {
    const { container } = render(
      <TrendCard
        title="Test"
        primaryValue="10"
        primaryLabel="sessions"
        trendDirection="stable"
        href="/details"
      />
    );

    const link = screen.getByRole('link');
    expect(link).toHaveTextContent('View →');
    expect(link).toHaveAttribute('href', '/details');

    // Link should be in Card's footer
    const footerElement = container.querySelector('[class*="footer"]');
    expect(footerElement).toBeInTheDocument();
    expect(footerElement).toContainElement(link);
  });

  it('applies correct CSS class for trend direction', () => {
    const { container } = render(
      <TrendCard
        title="Test"
        primaryValue="10"
        primaryLabel="sessions"
        trendDirection="rising"
      />
    );

    expect(container.querySelector('.trend-indicator--rising')).toBeInTheDocument();
  });
});
