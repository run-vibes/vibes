/**
 * Tests for OverridesList and OverrideItem components
 */
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { OverrideItem } from './OverrideItem';
import { OverridesList } from './OverridesList';
import type { LearningOverrideEntry } from '../../../hooks/useDashboard';

const mockSpecializedOverride: LearningOverrideEntry = {
  learning_id: 'learn-123',
  content: 'Use snake_case for variables',
  session_count: 47,
  is_specialized: true,
  base_category: 'Correction + Interactive',
  override_weights: [
    { strategy: 'Prefix', weight: 0.81 },
    { strategy: 'EarlyContext', weight: 0.45 },
  ],
};

const mockInheritingOverride: LearningOverrideEntry = {
  learning_id: 'learn-456',
  content: 'Prefer async/await',
  session_count: 12,
  is_specialized: false,
  base_category: 'Pattern + Interactive',
  sessions_to_specialize: 8,
};

describe('OverrideItem', () => {
  it('renders learning content', () => {
    render(<OverrideItem override={mockSpecializedOverride} />);

    expect(screen.getByText('Use snake_case for variables')).toBeInTheDocument();
  });

  it('shows session count', () => {
    render(<OverrideItem override={mockSpecializedOverride} />);

    expect(screen.getByText(/47 sessions/i)).toBeInTheDocument();
  });

  it('shows specialized status for specialized overrides', () => {
    render(<OverrideItem override={mockSpecializedOverride} />);

    expect(screen.getByText(/specialized/i)).toBeInTheDocument();
  });

  it('shows inheriting status for non-specialized overrides', () => {
    render(<OverrideItem override={mockInheritingOverride} />);

    expect(screen.getByText(/inheriting/i)).toBeInTheDocument();
  });

  it('shows sessions needed to specialize for inheriting', () => {
    render(<OverrideItem override={mockInheritingOverride} />);

    expect(screen.getByText(/8 more sessions/i)).toBeInTheDocument();
  });

  it('shows base category', () => {
    render(<OverrideItem override={mockSpecializedOverride} />);

    expect(screen.getByText(/Correction \+ Interactive/i)).toBeInTheDocument();
  });

  it('can expand to show weights when specialized', () => {
    render(<OverrideItem override={mockSpecializedOverride} />);

    fireEvent.click(screen.getByRole('button'));

    expect(screen.getByText('Prefix')).toBeInTheDocument();
    expect(screen.getByText('81%')).toBeInTheDocument();
  });

  it('does not expand for non-specialized overrides', () => {
    render(<OverrideItem override={mockInheritingOverride} />);

    // Should not have an expand button
    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });
});

describe('OverridesList', () => {
  const mockOverrides = [mockSpecializedOverride, mockInheritingOverride];

  it('renders all overrides', () => {
    render(<OverridesList overrides={mockOverrides} />);

    expect(screen.getByText('Use snake_case for variables')).toBeInTheDocument();
    expect(screen.getByText('Prefer async/await')).toBeInTheDocument();
  });

  it('shows filter dropdown', () => {
    render(<OverridesList overrides={mockOverrides} />);

    expect(screen.getByRole('combobox', { name: /filter/i })).toBeInTheDocument();
  });

  it('calls onFilterChange when filter changes', () => {
    const onFilterChange = vi.fn();
    render(<OverridesList overrides={mockOverrides} onFilterChange={onFilterChange} />);

    fireEvent.change(screen.getByRole('combobox', { name: /filter/i }), {
      target: { value: 'specialized' },
    });

    expect(onFilterChange).toHaveBeenCalledWith('specialized');
  });

  it('shows specialized count', () => {
    render(
      <OverridesList
        overrides={mockOverrides}
        specializedCount={1}
        totalCount={2}
      />
    );

    expect(screen.getByText(/1 of 2/i)).toBeInTheDocument();
  });

  it('shows empty state when no overrides', () => {
    render(<OverridesList overrides={[]} />);

    expect(screen.getByText(/no overrides/i)).toBeInTheDocument();
  });
});
