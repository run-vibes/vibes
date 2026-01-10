/**
 * Tests for AdaptiveParamsTable and RecentActivity components
 */
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { AdaptiveParamsTable } from './AdaptiveParamsTable';
import { RecentActivity } from './RecentActivity';
import type { AdaptiveParam, ActivityEvent } from '../../../hooks/useDashboard';

describe('AdaptiveParamsTable', () => {
  const mockParams: AdaptiveParam[] = [
    {
      name: 'extraction_threshold',
      current: 0.72,
      mean: 0.68,
      trend: 'up',
    },
    {
      name: 'strategy_confidence',
      current: 0.45,
      mean: 0.52,
      trend: 'down',
    },
    {
      name: 'attribution_decay',
      current: 0.85,
      mean: 0.85,
      trend: 'stable',
    },
  ];

  it('renders table headers', () => {
    render(<AdaptiveParamsTable params={mockParams} />);

    expect(screen.getByText('Name')).toBeInTheDocument();
    expect(screen.getByText('Current')).toBeInTheDocument();
    expect(screen.getByText('Mean')).toBeInTheDocument();
    expect(screen.getByText('Trend')).toBeInTheDocument();
  });

  it('displays parameter names', () => {
    render(<AdaptiveParamsTable params={mockParams} />);

    expect(screen.getByText('extraction_threshold')).toBeInTheDocument();
    expect(screen.getByText('strategy_confidence')).toBeInTheDocument();
    expect(screen.getByText('attribution_decay')).toBeInTheDocument();
  });

  it('shows current values', () => {
    render(<AdaptiveParamsTable params={mockParams} />);

    expect(screen.getByText('0.72')).toBeInTheDocument();
    expect(screen.getByText('0.45')).toBeInTheDocument();
  });

  it('shows mean values', () => {
    render(<AdaptiveParamsTable params={mockParams} />);

    expect(screen.getByText('0.68')).toBeInTheDocument();
    expect(screen.getByText('0.52')).toBeInTheDocument();
  });

  it('shows up trend indicator', () => {
    render(<AdaptiveParamsTable params={mockParams} />);

    expect(screen.getByText('↑')).toBeInTheDocument();
  });

  it('shows down trend indicator', () => {
    render(<AdaptiveParamsTable params={mockParams} />);

    expect(screen.getByText('↓')).toBeInTheDocument();
  });

  it('shows stable trend indicator', () => {
    render(<AdaptiveParamsTable params={mockParams} />);

    expect(screen.getByText('→')).toBeInTheDocument();
  });

  it('renders empty state when no params', () => {
    render(<AdaptiveParamsTable params={[]} />);

    expect(screen.getByText(/no parameters/i)).toBeInTheDocument();
  });
});

describe('RecentActivity', () => {
  const mockEvents: ActivityEvent[] = [
    {
      id: 'evt-1',
      type: 'extraction',
      description: 'Extracted 3 patterns from session abc123',
      timestamp: '2026-01-09T14:30:00Z',
    },
    {
      id: 'evt-2',
      type: 'attribution',
      description: 'Updated 5 learning scores',
      timestamp: '2026-01-09T14:25:00Z',
    },
    {
      id: 'evt-3',
      type: 'strategy',
      description: 'Recalculated weights for category X',
      timestamp: '2026-01-09T14:20:00Z',
    },
    {
      id: 'evt-4',
      type: 'error',
      description: 'Failed to process session xyz789',
      timestamp: '2026-01-09T14:15:00Z',
    },
  ];

  it('renders activity list', () => {
    render(<RecentActivity events={mockEvents} />);

    expect(screen.getByText(/Extracted 3 patterns/)).toBeInTheDocument();
    expect(screen.getByText(/Updated 5 learning scores/)).toBeInTheDocument();
  });

  it('shows event type icons', () => {
    render(<RecentActivity events={mockEvents} />);

    // Extraction icon
    expect(screen.getByText('⚡')).toBeInTheDocument();
    // Attribution icon
    expect(screen.getByText('📊')).toBeInTheDocument();
    // Strategy icon
    expect(screen.getByText('🎯')).toBeInTheDocument();
    // Error icon
    expect(screen.getByText('⚠️')).toBeInTheDocument();
  });

  it('displays timestamps', () => {
    const { container } = render(<RecentActivity events={mockEvents} />);

    // Check for time elements (format may vary by locale)
    const times = container.querySelectorAll('time');
    expect(times).toHaveLength(4);
  });

  it('shows empty state when no events', () => {
    render(<RecentActivity events={[]} />);

    expect(screen.getByText(/no recent activity/i)).toBeInTheDocument();
  });

  it('limits displayed events by maxItems prop', () => {
    render(<RecentActivity events={mockEvents} maxItems={2} />);

    expect(screen.getByText(/Extracted 3 patterns/)).toBeInTheDocument();
    expect(screen.getByText(/Updated 5 learning scores/)).toBeInTheDocument();
    expect(screen.queryByText(/Recalculated weights/)).not.toBeInTheDocument();
  });
});
