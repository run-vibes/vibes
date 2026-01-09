/**
 * Tests for LearningStatusBadge component
 */
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { LearningStatusBadge } from './LearningStatusBadge';

describe('LearningStatusBadge', () => {
  it('renders active status with green color', () => {
    render(<LearningStatusBadge status="active" />);

    const badge = screen.getByText('Active');
    expect(badge).toBeInTheDocument();
    expect(badge).toHaveClass('status-badge--active');
  });

  it('renders disabled status with gray color', () => {
    render(<LearningStatusBadge status="disabled" />);

    const badge = screen.getByText('Disabled');
    expect(badge).toBeInTheDocument();
    expect(badge).toHaveClass('status-badge--disabled');
  });

  it('renders under_review status with yellow color', () => {
    render(<LearningStatusBadge status="under_review" />);

    const badge = screen.getByText('Under Review');
    expect(badge).toBeInTheDocument();
    expect(badge).toHaveClass('status-badge--under_review');
  });

  it('renders deprecated status with red color', () => {
    render(<LearningStatusBadge status="deprecated" />);

    const badge = screen.getByText('Deprecated');
    expect(badge).toBeInTheDocument();
    expect(badge).toHaveClass('status-badge--deprecated');
  });

  it('applies small variant class', () => {
    render(<LearningStatusBadge status="active" size="small" />);

    const badge = screen.getByText('Active');
    expect(badge).toHaveClass('status-badge--small');
  });
});
