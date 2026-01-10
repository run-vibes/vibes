/**
 * Tests for ProgressBar and ValueBar components
 */
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ProgressBar } from './ProgressBar';
import { ValueBar } from './ValueBar';

describe('ProgressBar', () => {
  it('renders progress bar container', () => {
    const { container } = render(<ProgressBar value={0.5} />);

    expect(container.querySelector('.progress-bar')).toBeInTheDocument();
  });

  it('shows percentage label', () => {
    render(<ProgressBar value={0.75} />);

    expect(screen.getByText('75%')).toBeInTheDocument();
  });

  it('shows custom label', () => {
    render(<ProgressBar value={0.5} label="Loading" />);

    expect(screen.getByText('Loading')).toBeInTheDocument();
  });

  it('applies fill width based on value', () => {
    const { container } = render(<ProgressBar value={0.6} />);

    const fill = container.querySelector('.progress-bar__fill');
    expect(fill).toHaveStyle({ width: '60%' });
  });

  it('clamps value at 0', () => {
    const { container } = render(<ProgressBar value={-0.5} />);

    const fill = container.querySelector('.progress-bar__fill');
    expect(fill).toHaveStyle({ width: '0%' });
  });

  it('clamps value at 100', () => {
    const { container } = render(<ProgressBar value={1.5} />);

    const fill = container.querySelector('.progress-bar__fill');
    expect(fill).toHaveStyle({ width: '100%' });
  });

  it('applies custom color', () => {
    const { container } = render(<ProgressBar value={0.5} color="var(--crt-error)" />);

    const fill = container.querySelector('.progress-bar__fill');
    expect(fill).toHaveStyle({ backgroundColor: 'var(--crt-error)' });
  });

  it('uses default success color', () => {
    const { container } = render(<ProgressBar value={0.5} />);

    const fill = container.querySelector('.progress-bar__fill');
    expect(fill).toHaveStyle({ backgroundColor: 'var(--crt-success)' });
  });
});

describe('ValueBar', () => {
  it('renders value bar container', () => {
    const { container } = render(<ValueBar value={0} />);

    expect(container.querySelector('.value-bar')).toBeInTheDocument();
  });

  it('shows zero marker', () => {
    const { container } = render(<ValueBar value={0} />);

    expect(container.querySelector('.value-bar__zero')).toBeInTheDocument();
  });

  it('positions fill for positive value', () => {
    const { container } = render(<ValueBar value={0.5} />);

    const fill = container.querySelector('.value-bar__fill');
    // Positive values fill from center (50%) to the right
    expect(fill).toHaveStyle({ left: '50%', width: '25%' });
  });

  it('positions fill for negative value', () => {
    const { container } = render(<ValueBar value={-0.5} />);

    const fill = container.querySelector('.value-bar__fill');
    // Negative values fill from left towards center
    expect(fill).toHaveStyle({ left: '25%', width: '25%' });
  });

  it('applies green color for positive values', () => {
    const { container } = render(<ValueBar value={0.5} />);

    const fill = container.querySelector('.value-bar__fill');
    expect(fill).toHaveClass('value-bar__fill--positive');
  });

  it('applies red color for negative values', () => {
    const { container } = render(<ValueBar value={-0.5} />);

    const fill = container.querySelector('.value-bar__fill');
    expect(fill).toHaveClass('value-bar__fill--negative');
  });

  it('handles zero value', () => {
    const { container } = render(<ValueBar value={0} />);

    const fill = container.querySelector('.value-bar__fill');
    expect(fill).toHaveStyle({ width: '0%' });
  });

  it('clamps value to -1 to 1 range', () => {
    const { container } = render(<ValueBar value={2} />);

    const fill = container.querySelector('.value-bar__fill');
    // Max positive (1) fills from 50% to 100%, width 50%
    expect(fill).toHaveStyle({ width: '50%' });
  });

  it('shows value label when showValue is true', () => {
    render(<ValueBar value={0.75} showValue />);

    expect(screen.getByText('+0.75')).toBeInTheDocument();
  });

  it('shows negative value with minus sign', () => {
    render(<ValueBar value={-0.25} showValue />);

    expect(screen.getByText('-0.25')).toBeInTheDocument();
  });
});
