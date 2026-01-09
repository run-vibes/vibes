/**
 * Tests for LearningsFilters component
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { LearningsFilters } from './LearningsFilters';
import type { LearningsFilter } from '../../../hooks/useDashboard';

describe('LearningsFilters', () => {
  const mockOnChange = vi.fn();

  beforeEach(() => {
    mockOnChange.mockClear();
  });

  it('renders all filter dropdowns', () => {
    render(<LearningsFilters onChange={mockOnChange} />);

    expect(screen.getByLabelText('Scope')).toBeInTheDocument();
    expect(screen.getByLabelText('Category')).toBeInTheDocument();
    expect(screen.getByLabelText('Status')).toBeInTheDocument();
    expect(screen.getByLabelText('Sort')).toBeInTheDocument();
  });

  it('calls onChange when scope filter changes', () => {
    render(<LearningsFilters onChange={mockOnChange} />);

    const scopeSelect = screen.getByLabelText('Scope');
    fireEvent.change(scopeSelect, { target: { value: 'project' } });

    expect(mockOnChange).toHaveBeenCalledWith(
      expect.objectContaining({ scope: { Project: expect.any(String) } })
    );
  });

  it('calls onChange when category filter changes', () => {
    render(<LearningsFilters onChange={mockOnChange} />);

    const categorySelect = screen.getByLabelText('Category');
    fireEvent.change(categorySelect, { target: { value: 'Correction' } });

    expect(mockOnChange).toHaveBeenCalledWith(
      expect.objectContaining({ category: 'Correction' })
    );
  });

  it('calls onChange when status filter changes', () => {
    render(<LearningsFilters onChange={mockOnChange} />);

    const statusSelect = screen.getByLabelText('Status');
    fireEvent.change(statusSelect, { target: { value: 'active' } });

    expect(mockOnChange).toHaveBeenCalledWith(
      expect.objectContaining({ status: 'active' })
    );
  });

  it('supports controlled filter values', () => {
    const filters: LearningsFilter = {
      category: 'Pattern',
      status: 'active',
    };

    render(<LearningsFilters value={filters} onChange={mockOnChange} />);

    expect(screen.getByLabelText('Category')).toHaveValue('Pattern');
    expect(screen.getByLabelText('Status')).toHaveValue('active');
  });

  it('clears filter when "All" is selected', () => {
    const filters: LearningsFilter = {
      category: 'Pattern',
    };

    render(<LearningsFilters value={filters} onChange={mockOnChange} />);

    const categorySelect = screen.getByLabelText('Category');
    fireEvent.change(categorySelect, { target: { value: '' } });

    expect(mockOnChange).toHaveBeenCalledWith(
      expect.not.objectContaining({ category: expect.anything() })
    );
  });

  it('renders scope options', () => {
    render(<LearningsFilters onChange={mockOnChange} />);

    const scopeSelect = screen.getByLabelText('Scope');
    expect(scopeSelect).toContainElement(screen.getByText('All Scopes'));
    expect(scopeSelect).toContainElement(screen.getByText('Project'));
    expect(scopeSelect).toContainElement(screen.getByText('User'));
    expect(scopeSelect).toContainElement(screen.getByText('Global'));
  });

  it('renders category options', () => {
    render(<LearningsFilters onChange={mockOnChange} />);

    const categorySelect = screen.getByLabelText('Category');
    expect(categorySelect).toContainElement(screen.getByText('All Categories'));
    expect(categorySelect).toContainElement(screen.getByText('Correction'));
    expect(categorySelect).toContainElement(screen.getByText('Workflow'));
    expect(categorySelect).toContainElement(screen.getByText('Pattern'));
  });

  it('renders status options', () => {
    render(<LearningsFilters onChange={mockOnChange} />);

    const statusSelect = screen.getByLabelText('Status');
    expect(statusSelect).toContainElement(screen.getByText('All Statuses'));
    expect(statusSelect).toContainElement(screen.getByText('Active'));
    expect(statusSelect).toContainElement(screen.getByText('Disabled'));
    expect(statusSelect).toContainElement(screen.getByText('Under Review'));
  });
});
