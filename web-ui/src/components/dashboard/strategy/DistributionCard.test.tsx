/**
 * Tests for DistributionCard and StrategyBar components
 */
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { StrategyBar } from './StrategyBar';
import { DistributionCard } from './DistributionCard';
import type { CategoryDistribution, StrategyWeight } from '../../../hooks/useDashboard';

const mockWeights: StrategyWeight[] = [
  { strategy: 'Prefix', weight: 0.72 },
  { strategy: 'EarlyContext', weight: 0.31 },
  { strategy: 'MidContext', weight: 0.08 },
  { strategy: 'JustBeforeQuery', weight: 0.41 },
  { strategy: 'Deferral', weight: 0.15 },
];

const mockDistribution: CategoryDistribution = {
  category_key: 'correction_interactive',
  label: 'Correction + Interactive',
  session_count: 245,
  weights: mockWeights,
};

describe('StrategyBar', () => {
  it('renders the strategy name', () => {
    render(<StrategyBar strategy="Prefix" weight={0.72} />);

    expect(screen.getByText('Prefix')).toBeInTheDocument();
  });

  it('shows weight as percentage', () => {
    render(<StrategyBar strategy="Prefix" weight={0.72} />);

    expect(screen.getByText('72%')).toBeInTheDocument();
  });

  it('renders progress bar with correct width', () => {
    render(<StrategyBar strategy="EarlyContext" weight={0.31} />);

    const bar = screen.getByRole('progressbar');
    expect(bar).toHaveAttribute('aria-valuenow', '31');
  });

  it('handles zero weight', () => {
    render(<StrategyBar strategy="Deferral" weight={0} />);

    expect(screen.getByText('0%')).toBeInTheDocument();
  });

  it('handles 100% weight', () => {
    render(<StrategyBar strategy="Prefix" weight={1} />);

    expect(screen.getByText('100%')).toBeInTheDocument();
  });
});

describe('DistributionCard', () => {
  it('renders category label', () => {
    render(<DistributionCard distribution={mockDistribution} />);

    expect(screen.getByText('Correction + Interactive')).toBeInTheDocument();
  });

  it('shows session count', () => {
    render(<DistributionCard distribution={mockDistribution} />);

    expect(screen.getByText(/245 sessions/i)).toBeInTheDocument();
  });

  it('renders all strategy bars', () => {
    render(<DistributionCard distribution={mockDistribution} />);

    expect(screen.getByText('Prefix')).toBeInTheDocument();
    expect(screen.getByText('EarlyContext')).toBeInTheDocument();
    expect(screen.getByText('MidContext')).toBeInTheDocument();
    expect(screen.getByText('JustBeforeQuery')).toBeInTheDocument();
    expect(screen.getByText('Deferral')).toBeInTheDocument();
  });

  it('renders weight percentages', () => {
    render(<DistributionCard distribution={mockDistribution} />);

    expect(screen.getByText('72%')).toBeInTheDocument();
    expect(screen.getByText('31%')).toBeInTheDocument();
    expect(screen.getByText('8%')).toBeInTheDocument();
  });

  it('has accessible heading', () => {
    render(<DistributionCard distribution={mockDistribution} />);

    expect(screen.getByRole('heading', { name: /correction \+ interactive/i })).toBeInTheDocument();
  });
});
