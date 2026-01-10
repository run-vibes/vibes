/**
 * Tests for LearningIndicator component
 */
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { LearningIndicator } from './LearningIndicator';

describe('LearningIndicator', () => {
  it('renders brain icon', () => {
    render(<LearningIndicator state="idle" />);

    expect(screen.getByText('🧠')).toBeInTheDocument();
  });

  it('applies idle class when state is idle', () => {
    const { container } = render(<LearningIndicator state="idle" />);

    expect(container.firstChild).toHaveClass('learning-indicator--idle');
  });

  it('applies active class when state is active', () => {
    const { container } = render(<LearningIndicator state="active" />);

    expect(container.firstChild).toHaveClass('learning-indicator--active');
  });

  it('applies error class when state is error', () => {
    const { container } = render(<LearningIndicator state="error" />);

    expect(container.firstChild).toHaveClass('learning-indicator--error');
  });

  it('shows tooltip on hover', () => {
    render(<LearningIndicator state="idle" />);

    const indicator = screen.getByRole('button');
    fireEvent.mouseEnter(indicator);

    expect(screen.getByText(/groove/i)).toBeInTheDocument();
  });

  it('shows idle status in tooltip', () => {
    render(<LearningIndicator state="idle" />);

    const indicator = screen.getByRole('button');
    fireEvent.mouseEnter(indicator);

    expect(screen.getByText(/idle/i)).toBeInTheDocument();
  });

  it('shows active status in tooltip', () => {
    render(<LearningIndicator state="active" />);

    const indicator = screen.getByRole('button');
    fireEvent.mouseEnter(indicator);

    expect(screen.getByText(/learning/i)).toBeInTheDocument();
  });

  it('shows error status in tooltip', () => {
    render(<LearningIndicator state="error" />);

    const indicator = screen.getByRole('button');
    fireEvent.mouseEnter(indicator);

    expect(screen.getByText(/error/i)).toBeInTheDocument();
  });

  it('calls onClick when clicked', () => {
    const onClick = vi.fn();
    render(<LearningIndicator state="idle" onClick={onClick} />);

    fireEvent.click(screen.getByRole('button'));

    expect(onClick).toHaveBeenCalled();
  });

  it('hides tooltip on mouse leave', () => {
    render(<LearningIndicator state="idle" />);

    const indicator = screen.getByRole('button');
    fireEvent.mouseEnter(indicator);
    fireEvent.mouseLeave(indicator);

    expect(screen.queryByText(/groove/i)).not.toBeInTheDocument();
  });

  it('has accessible button role', () => {
    render(<LearningIndicator state="idle" />);

    expect(screen.getByRole('button')).toBeInTheDocument();
  });

  it('has aria-label for accessibility', () => {
    render(<LearningIndicator state="active" />);

    expect(screen.getByRole('button')).toHaveAttribute('aria-label');
  });
});
