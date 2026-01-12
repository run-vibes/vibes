/**
 * Tests for SolutionsList component
 */
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';

import { SolutionsList } from './SolutionsList';
import type { SolutionEntry } from '../../../hooks/useDashboard';

const mockSolutions: SolutionEntry[] = [
  {
    id: 'sol-001',
    gap_id: 'gap-001',
    gap_context: 'User asks about TypeScript generics',
    action_type: 'AddKnowledge',
    description: 'Add documentation about TypeScript generics',
    confidence: 0.85,
    status: 'Pending',
    created_at: new Date().toISOString(),
  },
  {
    id: 'sol-002',
    gap_id: 'gap-002',
    gap_context: 'Project structure not recognized',
    action_type: 'UpdatePattern',
    description: 'Update pattern matching for monorepo structures',
    confidence: 0.72,
    status: 'Applied',
    created_at: new Date(Date.now() - 1000 * 60 * 60 * 24).toISOString(),
    updated_at: new Date().toISOString(),
  },
  {
    id: 'sol-003',
    gap_id: 'gap-003',
    gap_context: 'Rare edge case in parsing',
    action_type: 'ContextAdjustment',
    description: 'Adjust context window for edge cases',
    confidence: 0.45,
    status: 'Dismissed',
    created_at: new Date(Date.now() - 1000 * 60 * 60 * 48).toISOString(),
  },
  {
    id: 'sol-004',
    gap_id: 'gap-001',
    gap_context: 'User asks about TypeScript generics',
    action_type: 'AddTool',
    description: 'Add TypeScript analysis tool',
    confidence: 0.92,
    status: 'Pending',
    created_at: new Date().toISOString(),
  },
];

describe('SolutionsList', () => {
  const defaultProps = {
    onApply: vi.fn(),
    onDismiss: vi.fn(),
  };

  it('shows loading state', () => {
    render(<SolutionsList {...defaultProps} isLoading />);
    expect(screen.getByText('Loading solutions...')).toBeInTheDocument();
  });

  it('shows empty state when no solutions', () => {
    render(<SolutionsList {...defaultProps} solutions={[]} total={0} />);
    expect(screen.getByTestId('solutions-list-empty')).toBeInTheDocument();
    expect(screen.getByText('No solutions pending review')).toBeInTheDocument();
  });

  it('renders solution count', () => {
    render(<SolutionsList {...defaultProps} solutions={mockSolutions} total={4} />);
    expect(screen.getByText('4 solutions')).toBeInTheDocument();
  });

  it('groups solutions by status', () => {
    render(<SolutionsList {...defaultProps} solutions={mockSolutions} total={4} />);

    // Should have all three groups
    const pendingGroup = screen.getByTestId('group-pending');
    const appliedGroup = screen.getByTestId('group-applied');
    const dismissedGroup = screen.getByTestId('group-dismissed');

    expect(pendingGroup).toBeInTheDocument();
    expect(appliedGroup).toBeInTheDocument();
    expect(dismissedGroup).toBeInTheDocument();

    // Group titles (check within each group element)
    expect(pendingGroup.querySelector('h4')).toHaveTextContent('Pending Review');
    expect(appliedGroup.querySelector('h4')).toHaveTextContent('Applied');
    expect(dismissedGroup.querySelector('h4')).toHaveTextContent('Dismissed');
  });

  it('renders pending solutions with action buttons', () => {
    render(<SolutionsList {...defaultProps} solutions={mockSolutions} total={4} />);

    // Pending solutions should have Apply/Dismiss buttons
    expect(screen.getByTestId('apply-sol-001')).toBeInTheDocument();
    expect(screen.getByTestId('dismiss-sol-001')).toBeInTheDocument();
    expect(screen.getByTestId('apply-sol-004')).toBeInTheDocument();
    expect(screen.getByTestId('dismiss-sol-004')).toBeInTheDocument();
  });

  it('calls onApply when Apply button is clicked', () => {
    const onApply = vi.fn();
    render(<SolutionsList {...defaultProps} solutions={mockSolutions} total={4} onApply={onApply} />);

    fireEvent.click(screen.getByTestId('apply-sol-001'));
    expect(onApply).toHaveBeenCalledWith('sol-001');
  });

  it('calls onDismiss when Dismiss button is clicked', () => {
    const onDismiss = vi.fn();
    render(
      <SolutionsList {...defaultProps} solutions={mockSolutions} total={4} onDismiss={onDismiss} />
    );

    fireEvent.click(screen.getByTestId('dismiss-sol-001'));
    expect(onDismiss).toHaveBeenCalledWith('sol-001');
  });

  it('disables buttons and shows loading for solution being actioned', () => {
    render(
      <SolutionsList {...defaultProps} solutions={mockSolutions} total={4} actionLoading="sol-001" />
    );

    const applyBtn = screen.getByTestId('apply-sol-001');
    expect(applyBtn).toBeDisabled();
    expect(applyBtn).toHaveTextContent('...');
  });

  it('shows solution descriptions', () => {
    render(<SolutionsList {...defaultProps} solutions={mockSolutions} total={4} />);

    expect(screen.getByText('Add documentation about TypeScript generics')).toBeInTheDocument();
    expect(
      screen.getByText('Update pattern matching for monorepo structures')
    ).toBeInTheDocument();
  });

  it('shows confidence badges', () => {
    render(<SolutionsList {...defaultProps} solutions={mockSolutions} total={4} />);

    // Check for confidence values (displayed as percentages)
    expect(screen.getByText('85%')).toBeInTheDocument();
    expect(screen.getByText('72%')).toBeInTheDocument();
    expect(screen.getByText('92%')).toBeInTheDocument();
  });

  it('shows action type labels', () => {
    render(<SolutionsList {...defaultProps} solutions={mockSolutions} total={4} />);

    expect(screen.getByText('Add Knowledge')).toBeInTheDocument();
    expect(screen.getByText('Update Pattern')).toBeInTheDocument();
    expect(screen.getByText('Add Tool')).toBeInTheDocument();
  });

  it('only shows groups that have solutions', () => {
    const pendingOnly: SolutionEntry[] = [mockSolutions[0], mockSolutions[3]];
    render(<SolutionsList {...defaultProps} solutions={pendingOnly} total={2} />);

    expect(screen.getByTestId('group-pending')).toBeInTheDocument();
    expect(screen.queryByTestId('group-applied')).not.toBeInTheDocument();
    expect(screen.queryByTestId('group-dismissed')).not.toBeInTheDocument();
  });
});
