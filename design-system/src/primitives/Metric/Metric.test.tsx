import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Metric } from './Metric';

// Helper to check if element has a CSS module class (handles hashed names)
const hasModuleClass = (element: HTMLElement, className: string): boolean => {
  return Array.from(element.classList).some(cls =>
    cls.includes(className) || cls.startsWith(`_${className}_`)
  );
};

describe('Metric', () => {
  it('renders label and value', () => {
    render(<Metric label="Success Rate" value="94.2%" />);
    expect(screen.getByText('Success Rate')).toBeInTheDocument();
    expect(screen.getByText('94.2%')).toBeInTheDocument();
  });

  it('renders numeric values', () => {
    render(<Metric label="Sessions" value={42} />);
    expect(screen.getByText('Sessions')).toBeInTheDocument();
    expect(screen.getByText('42')).toBeInTheDocument();
  });

  it('renders ReactNode values', () => {
    render(<Metric label="Status" value={<strong>Active</strong>} />);
    expect(screen.getByText('Active').tagName).toBe('STRONG');
  });

  it('applies default md size (no extra class)', () => {
    render(<Metric label="Test" value="123" data-testid="metric" />);
    const metric = screen.getByTestId('metric');
    expect(hasModuleClass(metric, 'sm')).toBe(false);
    expect(hasModuleClass(metric, 'lg')).toBe(false);
    expect(hasModuleClass(metric, 'xl')).toBe(false);
  });

  it('applies sm size', () => {
    render(<Metric label="Small" value="123" size="sm" data-testid="metric" />);
    expect(hasModuleClass(screen.getByTestId('metric'), 'sm')).toBe(true);
  });

  it('applies lg size', () => {
    render(<Metric label="Large" value="123" size="lg" data-testid="metric" />);
    expect(hasModuleClass(screen.getByTestId('metric'), 'lg')).toBe(true);
  });

  it('applies xl size', () => {
    render(<Metric label="Extra Large" value="123" size="xl" data-testid="metric" />);
    expect(hasModuleClass(screen.getByTestId('metric'), 'xl')).toBe(true);
  });

  it('passes through additional props', () => {
    render(<Metric label="Test" value="123" data-testid="custom-metric" aria-label="Custom" />);
    const metric = screen.getByTestId('custom-metric');
    expect(metric).toHaveAttribute('aria-label', 'Custom');
  });

  it('merges custom className', () => {
    render(<Metric label="Test" value="123" className="custom-class" data-testid="metric" />);
    expect(screen.getByTestId('metric')).toHaveClass('custom-class');
  });

  it('has correct structure with label and value spans', () => {
    render(<Metric label="Test Label" value="Test Value" data-testid="metric" />);
    const metric = screen.getByTestId('metric');
    const spans = metric.querySelectorAll('span');
    expect(spans).toHaveLength(2);
    expect(spans[0]).toHaveTextContent('Test Label');
    expect(spans[1]).toHaveTextContent('Test Value');
  });
});
