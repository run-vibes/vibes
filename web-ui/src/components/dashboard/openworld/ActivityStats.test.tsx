/**
 * Tests for ActivityStats component
 */
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';

import { ActivityStats } from './ActivityStats';
import type { ActivitySummary } from '../../../hooks/useDashboard';

const mockSummary: ActivitySummary = {
  outcomes_total: 42,
  negative_rate: 0.15,
  avg_exploration_bonus: 0.25,
};

describe('ActivityStats', () => {
  it('shows loading state', () => {
    render(<ActivityStats isLoading />);
    expect(screen.getByText('Loading...')).toBeInTheDocument();
  });

  it('renders outcomes metric', () => {
    render(<ActivityStats summary={mockSummary} />);
    expect(screen.getByTestId('outcomes-metric')).toHaveTextContent('42');
  });

  it('renders negative rate as percentage', () => {
    render(<ActivityStats summary={mockSummary} />);
    expect(screen.getByTestId('negative-metric')).toHaveTextContent('15%');
  });

  it('renders exploration bonus with plus sign', () => {
    render(<ActivityStats summary={mockSummary} />);
    expect(screen.getByTestId('exploration-metric')).toHaveTextContent('+0.25');
  });

  it('shows default values when no summary', () => {
    render(<ActivityStats />);
    expect(screen.getByTestId('outcomes-metric')).toHaveTextContent('0');
    expect(screen.getByTestId('negative-metric')).toHaveTextContent('0%');
    expect(screen.getByTestId('exploration-metric')).toHaveTextContent('+0.00');
  });

  it('shows live indicator when isLive is true', () => {
    render(<ActivityStats summary={mockSummary} isLive />);
    expect(screen.getByTestId('live-indicator')).toBeInTheDocument();
    expect(screen.getByText('LIVE')).toBeInTheDocument();
  });

  it('hides live indicator when isLive is false', () => {
    render(<ActivityStats summary={mockSummary} isLive={false} />);
    expect(screen.queryByTestId('live-indicator')).not.toBeInTheDocument();
  });
});
