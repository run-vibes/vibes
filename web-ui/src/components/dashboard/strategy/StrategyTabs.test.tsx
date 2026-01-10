/**
 * Tests for StrategyTabs component
 */
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { StrategyTabs } from './StrategyTabs';

describe('StrategyTabs', () => {
  it('renders both tab options', () => {
    render(<StrategyTabs activeTab="distributions" onTabChange={() => {}} />);

    expect(screen.getByRole('tab', { name: /distributions/i })).toBeInTheDocument();
    expect(screen.getByRole('tab', { name: /overrides/i })).toBeInTheDocument();
  });

  it('marks active tab as selected', () => {
    render(<StrategyTabs activeTab="distributions" onTabChange={() => {}} />);

    const distributionsTab = screen.getByRole('tab', { name: /distributions/i });
    const overridesTab = screen.getByRole('tab', { name: /overrides/i });

    expect(distributionsTab).toHaveAttribute('aria-selected', 'true');
    expect(overridesTab).toHaveAttribute('aria-selected', 'false');
  });

  it('calls onTabChange when clicking inactive tab', () => {
    const onTabChange = vi.fn();
    render(<StrategyTabs activeTab="distributions" onTabChange={onTabChange} />);

    fireEvent.click(screen.getByRole('tab', { name: /overrides/i }));

    expect(onTabChange).toHaveBeenCalledWith('overrides');
  });

  it('does not call onTabChange when clicking active tab', () => {
    const onTabChange = vi.fn();
    render(<StrategyTabs activeTab="distributions" onTabChange={onTabChange} />);

    fireEvent.click(screen.getByRole('tab', { name: /distributions/i }));

    expect(onTabChange).not.toHaveBeenCalled();
  });

  it('applies correct styling to active tab', () => {
    render(<StrategyTabs activeTab="overrides" onTabChange={() => {}} />);

    const overridesTab = screen.getByRole('tab', { name: /overrides/i });
    expect(overridesTab).toHaveClass('strategy-tabs__tab--active');
  });
});
