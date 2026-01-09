/**
 * Tests for LearningsList component
 */
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { LearningsList } from './LearningsList';
import type { LearningBrief } from '../../../hooks/useDashboard';

// Helper to create a full LearningBrief
function makeLearning(overrides: Partial<LearningBrief> & { id: string; content: string }): LearningBrief {
  return {
    category: 'Correction',
    scope: { User: 'test' },
    status: 'active',
    estimated_value: 0.5,
    created_at: new Date().toISOString(),
    ...overrides,
  };
}

describe('LearningsList', () => {
  const mockLearnings: LearningBrief[] = [
    makeLearning({ id: '1', content: 'First learning', estimated_value: 0.8 }),
    makeLearning({ id: '2', content: 'Second learning', estimated_value: -0.3, status: 'under_review' }),
    makeLearning({ id: '3', content: 'Third learning', estimated_value: 0.5, category: 'Pattern' }),
  ];

  it('renders list of learnings', () => {
    render(<LearningsList learnings={mockLearnings} onSelect={vi.fn()} />);

    expect(screen.getByText('First learning')).toBeInTheDocument();
    expect(screen.getByText('Second learning')).toBeInTheDocument();
    expect(screen.getByText('Third learning')).toBeInTheDocument();
  });

  it('shows category badge for each learning', () => {
    render(<LearningsList learnings={mockLearnings} onSelect={vi.fn()} />);

    expect(screen.getAllByText('Correction')).toHaveLength(2);
    expect(screen.getByText('Pattern')).toBeInTheDocument();
  });

  it('shows status badge for each learning', () => {
    render(<LearningsList learnings={mockLearnings} onSelect={vi.fn()} />);

    expect(screen.getAllByText('Active')).toHaveLength(2);
    expect(screen.getByText('Under Review')).toBeInTheDocument();
  });

  it('calls onSelect when learning is clicked', () => {
    const onSelect = vi.fn();
    render(<LearningsList learnings={mockLearnings} onSelect={onSelect} />);

    fireEvent.click(screen.getByText('First learning'));

    expect(onSelect).toHaveBeenCalledWith('1');
  });

  it('highlights selected learning', () => {
    render(
      <LearningsList
        learnings={mockLearnings}
        selectedId="2"
        onSelect={vi.fn()}
      />
    );

    const items = screen.getAllByRole('listitem');
    expect(items[1]).toHaveClass('learning-item--selected');
  });

  it('shows empty state when no learnings', () => {
    render(<LearningsList learnings={[]} onSelect={vi.fn()} />);

    expect(screen.getByText('No learnings found')).toBeInTheDocument();
  });

  it('shows value bar for each learning', () => {
    render(<LearningsList learnings={mockLearnings} onSelect={vi.fn()} />);

    const valueBars = screen.getAllByTestId('value-bar');
    expect(valueBars).toHaveLength(3);
  });

  it('displays loading state', () => {
    render(<LearningsList learnings={[]} isLoading onSelect={vi.fn()} />);

    expect(screen.getByText('Loading learnings...')).toBeInTheDocument();
  });
});
