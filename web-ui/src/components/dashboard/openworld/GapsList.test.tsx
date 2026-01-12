/**
 * Tests for GapsList component
 */
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';

import { GapsList } from './GapsList';
import type { GapBrief, OpenWorldGapsFilter } from '../../../hooks/useDashboard';

const mockGaps: GapBrief[] = [
  {
    id: 'gap-001',
    category: 'MissingKnowledge',
    severity: 'High',
    status: 'Detected',
    context_pattern: 'User asks about advanced TypeScript features',
    failure_count: 5,
    first_seen: new Date(Date.now() - 1000 * 60 * 60 * 24).toISOString(),
    last_seen: new Date(Date.now() - 1000 * 60 * 30).toISOString(),
    solution_count: 2,
  },
  {
    id: 'gap-002',
    category: 'ContextMismatch',
    severity: 'Critical',
    status: 'Confirmed',
    context_pattern: 'Project structure not recognized',
    failure_count: 12,
    first_seen: new Date(Date.now() - 1000 * 60 * 60 * 48).toISOString(),
    last_seen: new Date().toISOString(),
    solution_count: 0,
  },
];

describe('GapsList', () => {
  const defaultProps = {
    onFiltersChange: vi.fn(),
    onSelectGap: vi.fn(),
  };

  it('renders gap count', () => {
    render(<GapsList {...defaultProps} gaps={mockGaps} total={2} />);
    expect(screen.getByText('2 gaps')).toBeInTheDocument();
  });

  it('renders gap items', () => {
    render(<GapsList {...defaultProps} gaps={mockGaps} total={2} />);
    expect(screen.getByText('User asks about advanced TypeScript features')).toBeInTheDocument();
    expect(screen.getByText('Project structure not recognized')).toBeInTheDocument();
  });

  it('renders severity badges', () => {
    render(<GapsList {...defaultProps} gaps={mockGaps} total={2} />);
    expect(screen.getByTestId('severity-high')).toBeInTheDocument();
    expect(screen.getByTestId('severity-critical')).toBeInTheDocument();
  });

  it('shows loading state', () => {
    render(<GapsList {...defaultProps} isLoading />);
    expect(screen.getByText('Loading...')).toBeInTheDocument();
    expect(screen.getByText('Loading gaps...')).toBeInTheDocument();
  });

  it('shows empty state when no gaps', () => {
    render(<GapsList {...defaultProps} gaps={[]} total={0} />);
    expect(screen.getByTestId('gaps-list-empty')).toBeInTheDocument();
    expect(screen.getByText('No capability gaps detected')).toBeInTheDocument();
  });

  it('calls onSelectGap when gap is clicked', () => {
    const onSelectGap = vi.fn();
    render(<GapsList {...defaultProps} gaps={mockGaps} total={2} onSelectGap={onSelectGap} />);

    fireEvent.click(screen.getByTestId('gap-gap-001'));
    expect(onSelectGap).toHaveBeenCalledWith('gap-001');
  });

  it('highlights selected gap', () => {
    render(<GapsList {...defaultProps} gaps={mockGaps} total={2} selectedId="gap-001" />);
    const selectedItem = screen.getByTestId('gap-gap-001');
    expect(selectedItem).toHaveClass('gap-item--selected');
  });

  it('renders filter dropdowns', () => {
    render(<GapsList {...defaultProps} gaps={mockGaps} total={2} />);
    expect(screen.getByLabelText('Severity')).toBeInTheDocument();
    expect(screen.getByLabelText('Status')).toBeInTheDocument();
    expect(screen.getByLabelText('Category')).toBeInTheDocument();
  });

  it('calls onFiltersChange when filter changes', () => {
    const onFiltersChange = vi.fn();
    render(<GapsList {...defaultProps} gaps={mockGaps} total={2} onFiltersChange={onFiltersChange} />);

    fireEvent.change(screen.getByLabelText('Severity'), { target: { value: 'Critical' } });
    expect(onFiltersChange).toHaveBeenCalledWith({ severity: 'Critical' });
  });

  it('displays current filter values', () => {
    const filters: OpenWorldGapsFilter = { severity: 'High', status: 'Confirmed' };
    render(<GapsList {...defaultProps} gaps={mockGaps} total={2} filters={filters} />);

    expect(screen.getByLabelText('Severity')).toHaveValue('High');
    expect(screen.getByLabelText('Status')).toHaveValue('Confirmed');
  });
});
