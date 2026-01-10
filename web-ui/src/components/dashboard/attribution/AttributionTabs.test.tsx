/**
 * Tests for AttributionTabs component
 */
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { AttributionTabs } from './AttributionTabs';

describe('AttributionTabs', () => {
  it('renders both tab options', () => {
    render(<AttributionTabs activeTab="leaderboard" onTabChange={() => {}} />);

    expect(screen.getByRole('tab', { name: /leaderboard/i })).toBeInTheDocument();
    expect(screen.getByRole('tab', { name: /timeline/i })).toBeInTheDocument();
  });

  it('marks active tab as selected', () => {
    render(<AttributionTabs activeTab="leaderboard" onTabChange={() => {}} />);

    const leaderboardTab = screen.getByRole('tab', { name: /leaderboard/i });
    const timelineTab = screen.getByRole('tab', { name: /timeline/i });

    expect(leaderboardTab).toHaveAttribute('aria-selected', 'true');
    expect(timelineTab).toHaveAttribute('aria-selected', 'false');
  });

  it('calls onTabChange when clicking inactive tab', () => {
    const onTabChange = vi.fn();
    render(<AttributionTabs activeTab="leaderboard" onTabChange={onTabChange} />);

    fireEvent.click(screen.getByRole('tab', { name: /timeline/i }));

    expect(onTabChange).toHaveBeenCalledWith('timeline');
  });

  it('does not call onTabChange when clicking active tab', () => {
    const onTabChange = vi.fn();
    render(<AttributionTabs activeTab="leaderboard" onTabChange={onTabChange} />);

    fireEvent.click(screen.getByRole('tab', { name: /leaderboard/i }));

    expect(onTabChange).not.toHaveBeenCalled();
  });

  it('applies correct styling to active tab', () => {
    render(<AttributionTabs activeTab="timeline" onTabChange={() => {}} />);

    const timelineTab = screen.getByRole('tab', { name: /timeline/i });
    expect(timelineTab).toHaveClass('attribution-tabs__tab--active');
  });
});
