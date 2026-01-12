/**
 * Tests for ActivityFeed component
 */
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';

import { ActivityFeed } from './ActivityFeed';
import type { OpenWorldActivityEntry } from '../../../hooks/useDashboard';

const mockEvents: OpenWorldActivityEntry[] = [
  {
    timestamp: new Date().toISOString(),
    event_type: 'novelty_detected',
    message: 'Detected novel pattern in user interaction',
  },
  {
    timestamp: new Date(Date.now() - 1000 * 60 * 5).toISOString(),
    event_type: 'gap_created',
    message: 'New capability gap identified',
    gap_id: 'gap-001-abcd1234',
  },
  {
    timestamp: new Date(Date.now() - 1000 * 60 * 30).toISOString(),
    event_type: 'solution_generated',
    message: 'Generated solution for gap',
    gap_id: 'gap-002-efgh5678',
    learning_id: 'learn-001-ijkl9012',
  },
];

describe('ActivityFeed', () => {
  it('shows loading state', () => {
    render(<ActivityFeed isLoading />);
    expect(screen.getByText('Loading events...')).toBeInTheDocument();
  });

  it('shows empty state when no events', () => {
    render(<ActivityFeed events={[]} />);
    expect(screen.getByTestId('activity-feed-empty')).toBeInTheDocument();
    expect(screen.getByText('No recent activity')).toBeInTheDocument();
  });

  it('renders event count', () => {
    render(<ActivityFeed events={mockEvents} />);
    expect(screen.getByText('3 events')).toBeInTheDocument();
  });

  it('renders event messages', () => {
    render(<ActivityFeed events={mockEvents} />);
    expect(screen.getByText('Detected novel pattern in user interaction')).toBeInTheDocument();
    expect(screen.getByText('New capability gap identified')).toBeInTheDocument();
    expect(screen.getByText('Generated solution for gap')).toBeInTheDocument();
  });

  it('renders event type badges', () => {
    render(<ActivityFeed events={mockEvents} />);
    expect(screen.getByTestId('event-type-novelty_detected')).toBeInTheDocument();
    expect(screen.getByTestId('event-type-gap_created')).toBeInTheDocument();
    expect(screen.getByTestId('event-type-solution_generated')).toBeInTheDocument();
  });

  it('shows gap references when present', () => {
    render(<ActivityFeed events={mockEvents} />);
    // Gap IDs are truncated to first 8 chars
    expect(screen.getByText('Gap: gap-001-')).toBeInTheDocument();
  });

  it('shows learning references when present', () => {
    render(<ActivityFeed events={mockEvents} />);
    // Learning IDs are truncated to first 8 chars
    expect(screen.getByText('Learning: learn-00')).toBeInTheDocument();
  });

  it('renders relative timestamps', () => {
    render(<ActivityFeed events={mockEvents} />);
    // First event should show "just now"
    expect(screen.getByText('just now')).toBeInTheDocument();
    // Second event (5 mins ago) should show "5m ago"
    expect(screen.getByText('5m ago')).toBeInTheDocument();
    // Third event (30 mins ago) should show "30m ago"
    expect(screen.getByText('30m ago')).toBeInTheDocument();
  });
});
