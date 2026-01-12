/**
 * Tests for the NoveltyStats component
 */
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';

import { NoveltyStats } from './NoveltyStats';
import type { OpenWorldOverviewData } from '../../../hooks/useDashboard';

const mockData: OpenWorldOverviewData = {
  data_type: 'open_world_overview',
  novelty_threshold: 0.85,
  pending_outliers: 3,
  cluster_count: 12,
  gap_counts: {
    low: 2,
    medium: 1,
    high: 0,
    critical: 0,
    total: 3,
  },
  hook_stats: {
    outcomes_processed: 100,
    negative_outcomes: 10,
    low_confidence_outcomes: 5,
    exploration_adjustments: 2,
    gaps_created: 3,
  },
};

describe('NoveltyStats', () => {
  it('renders title', () => {
    render(<NoveltyStats data={mockData} />);
    expect(screen.getByText('Novelty Detection')).toBeInTheDocument();
  });

  it('renders threshold metric', () => {
    render(<NoveltyStats data={mockData} />);
    expect(screen.getByText('Threshold')).toBeInTheDocument();
    expect(screen.getByText('0.85')).toBeInTheDocument();
  });

  it('renders pending outliers metric', () => {
    render(<NoveltyStats data={mockData} />);
    expect(screen.getByText('Pending')).toBeInTheDocument();
    expect(screen.getByText('3')).toBeInTheDocument();
  });

  it('renders clusters metric', () => {
    render(<NoveltyStats data={mockData} />);
    expect(screen.getByText('Clusters')).toBeInTheDocument();
    expect(screen.getByText('12')).toBeInTheDocument();
  });

  it('shows loading state', () => {
    render(<NoveltyStats isLoading />);
    expect(screen.getByText('Loading...')).toBeInTheDocument();
  });

  it('shows empty state when no data', () => {
    render(<NoveltyStats />);
    expect(screen.getByText('No novelty data available')).toBeInTheDocument();
  });

  it('formats threshold to two decimal places', () => {
    const dataWithLongDecimal: OpenWorldOverviewData = {
      ...mockData,
      novelty_threshold: 0.8567890,
    };
    render(<NoveltyStats data={dataWithLongDecimal} />);
    expect(screen.getByText('0.86')).toBeInTheDocument();
  });
});
