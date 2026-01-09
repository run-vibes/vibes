/**
 * Tests for ValueBar component
 */
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ValueBar } from './ValueBar';

describe('ValueBar', () => {
  it('renders positive value with green color', () => {
    render(<ValueBar value={0.8} />);

    const bar = screen.getByTestId('value-bar');
    expect(bar).toHaveClass('value-bar--positive');
  });

  it('renders negative value with red color', () => {
    render(<ValueBar value={-0.5} />);

    const bar = screen.getByTestId('value-bar');
    expect(bar).toHaveClass('value-bar--negative');
  });

  it('renders neutral value', () => {
    render(<ValueBar value={0} />);

    const bar = screen.getByTestId('value-bar');
    expect(bar).toHaveClass('value-bar--neutral');
  });

  it('clamps value to -1 to +1 range', () => {
    const { rerender } = render(<ValueBar value={2.5} />);
    let fill = screen.getByTestId('value-fill');
    // Should be 100% (clamped to 1)
    expect(fill.style.width).toBe('100%');

    rerender(<ValueBar value={-1.5} />);
    fill = screen.getByTestId('value-fill');
    // Should be 100% (clamped to -1)
    expect(fill.style.width).toBe('100%');
  });

  it('displays value text when showValue is true', () => {
    render(<ValueBar value={0.75} showValue />);

    expect(screen.getByText('+0.75')).toBeInTheDocument();
  });

  it('formats negative values correctly', () => {
    render(<ValueBar value={-0.3} showValue />);

    expect(screen.getByText('-0.30')).toBeInTheDocument();
  });

  it('calculates fill width based on absolute value', () => {
    render(<ValueBar value={0.5} />);

    const fill = screen.getByTestId('value-fill');
    expect(fill.style.width).toBe('50%');
  });
});
