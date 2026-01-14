/**
 * Tests for the StrategyCard component
 */
import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';

// Mock TanStack Router Link component
vi.mock('@tanstack/react-router', () => ({
  Link: ({ to, children }: { to: string; children: React.ReactNode }) => (
    <a href={to}>{children}</a>
  ),
}));

import { StrategyCard } from './StrategyCard';
import type { StrategyDistributionsData } from '../../hooks/useDashboard';

describe('StrategyCard', () => {
  it('renders title', () => {
    render(<StrategyCard />);
    expect(screen.getByText('Strategy')).toBeInTheDocument();
  });

  it('shows distribution and specialized counts', () => {
    const data: StrategyDistributionsData = {
      data_type: 'strategy_distributions',
      distributions: [
        { category_key: 'cat1', label: 'Category 1', session_count: 10, weights: [] },
        { category_key: 'cat2', label: 'Category 2', session_count: 5, weights: [] },
      ],
      specialized_count: 8,
      total_learnings: 15,
    };
    render(<StrategyCard data={data} />);

    expect(screen.getByText('Distributions')).toBeInTheDocument();
    expect(screen.getByText('2')).toBeInTheDocument();
    expect(screen.getByText('Specialized')).toBeInTheDocument();
    expect(screen.getByText('8/15')).toBeInTheDocument();
  });

  it('shows active categories with session counts', () => {
    const data: StrategyDistributionsData = {
      data_type: 'strategy_distributions',
      distributions: [
        { category_key: 'cat1', label: 'Frontend', session_count: 25, weights: [] },
        { category_key: 'cat2', label: 'Backend', session_count: 18, weights: [] },
      ],
      specialized_count: 5,
      total_learnings: 10,
    };
    render(<StrategyCard data={data} />);

    expect(screen.getByText('Active Categories:')).toBeInTheDocument();
    expect(screen.getByText('Frontend')).toBeInTheDocument();
    expect(screen.getByText('25 sessions')).toBeInTheDocument();
    expect(screen.getByText('Backend')).toBeInTheDocument();
    expect(screen.getByText('18 sessions')).toBeInTheDocument();
  });

  it('limits displayed categories to 3', () => {
    const data: StrategyDistributionsData = {
      data_type: 'strategy_distributions',
      distributions: Array.from({ length: 5 }, (_, i) => ({
        category_key: `cat${i}`,
        label: `Category ${i}`,
        session_count: 10 - i,
        weights: [],
      })),
      specialized_count: 0,
      total_learnings: 0,
    };
    render(<StrategyCard data={data} />);

    expect(screen.getByText('Category 0')).toBeInTheDocument();
    expect(screen.getByText('Category 2')).toBeInTheDocument();
    expect(screen.queryByText('Category 3')).not.toBeInTheDocument();
  });

  it('handles no distributions gracefully', () => {
    const data: StrategyDistributionsData = {
      data_type: 'strategy_distributions',
      distributions: [],
      specialized_count: 0,
      total_learnings: 0,
    };
    render(<StrategyCard data={data} />);

    expect(screen.getByText('0')).toBeInTheDocument();
    expect(screen.queryByText('Active Categories:')).not.toBeInTheDocument();
  });

  it('handles undefined data gracefully', () => {
    render(<StrategyCard />);

    expect(screen.getByText('Distributions')).toBeInTheDocument();
    expect(screen.getAllByText('0')).toHaveLength(1);
    expect(screen.getByText('0/0')).toBeInTheDocument();
  });

  it('renders link to strategy page in card footer', () => {
    const { container } = render(<StrategyCard />);

    const link = screen.getByRole('link');
    expect(link).toHaveTextContent('View →');
    expect(link).toHaveAttribute('href', '/groove/strategy');

    // Link should be in Card's footer
    const footerElement = container.querySelector('[class*="footer"]');
    expect(footerElement).toBeInTheDocument();
    expect(footerElement).toContainElement(link);
  });
});
