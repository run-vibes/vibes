import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { StatusIndicator } from './StatusIndicator';

// Helper to check if element has a CSS module class (handles hashed names)
const hasModuleClass = (element: HTMLElement, className: string): boolean => {
  return Array.from(element.classList).some(cls =>
    cls.includes(className) || cls.startsWith(`_${className}_`)
  );
};

describe('StatusIndicator', () => {
  it('renders a dot', () => {
    const { container } = render(<StatusIndicator state="live" />);
    const dot = container.querySelector('span');
    expect(dot).toBeInTheDocument();
    expect(hasModuleClass(dot!, 'dot')).toBe(true);
  });

  it('applies live state', () => {
    const { container } = render(<StatusIndicator state="live" />);
    expect(hasModuleClass(container.firstChild as HTMLElement, 'live')).toBe(true);
  });

  it('applies paused state', () => {
    const { container } = render(<StatusIndicator state="paused" />);
    expect(hasModuleClass(container.firstChild as HTMLElement, 'paused')).toBe(true);
  });

  it('applies offline state', () => {
    const { container } = render(<StatusIndicator state="offline" />);
    expect(hasModuleClass(container.firstChild as HTMLElement, 'offline')).toBe(true);
  });

  it('applies error state', () => {
    const { container } = render(<StatusIndicator state="error" />);
    expect(hasModuleClass(container.firstChild as HTMLElement, 'error')).toBe(true);
  });

  it('applies ok state (dashboard alias)', () => {
    const { container } = render(<StatusIndicator state="ok" />);
    expect(hasModuleClass(container.firstChild as HTMLElement, 'ok')).toBe(true);
  });

  it('applies degraded state (dashboard alias)', () => {
    const { container } = render(<StatusIndicator state="degraded" />);
    expect(hasModuleClass(container.firstChild as HTMLElement, 'degraded')).toBe(true);
  });

  it('renders label when provided', () => {
    render(<StatusIndicator state="live" label="Connected" />);
    expect(screen.getByText('Connected')).toBeInTheDocument();
  });

  it('does not render label when not provided', () => {
    const { container } = render(<StatusIndicator state="live" />);
    expect(container.querySelectorAll('span')).toHaveLength(1); // just the dot
  });

  it('merges custom className', () => {
    const { container } = render(<StatusIndicator state="live" className="custom-class" />);
    expect(container.firstChild).toHaveClass('custom-class');
  });

  it('passes through additional props', () => {
    render(<StatusIndicator state="live" data-testid="custom-indicator" aria-label="Status" />);
    expect(screen.getByTestId('custom-indicator')).toBeInTheDocument();
    expect(screen.getByTestId('custom-indicator')).toHaveAttribute('aria-label', 'Status');
  });
});
