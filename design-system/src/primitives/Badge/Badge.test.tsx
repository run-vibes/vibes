import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Badge } from './Badge';

// Helper to check if element has a CSS module class
const hasModuleClass = (element: HTMLElement, className: string): boolean => {
  return Array.from(element.classList).some(cls =>
    cls.includes(className) || cls.startsWith(`_${className}_`)
  );
};

describe('Badge', () => {
  it('renders children', () => {
    render(<Badge>Connected</Badge>);
    expect(screen.getByText('Connected')).toBeInTheDocument();
  });

  it('applies idle status by default', () => {
    render(<Badge>Idle</Badge>);
    expect(hasModuleClass(screen.getByText('Idle'), 'idle')).toBe(true);
  });

  it('applies success status', () => {
    render(<Badge status="success">Connected</Badge>);
    expect(hasModuleClass(screen.getByText('Connected'), 'success')).toBe(true);
  });

  it('applies warning status', () => {
    render(<Badge status="warning">Processing</Badge>);
    expect(hasModuleClass(screen.getByText('Processing'), 'warning')).toBe(true);
  });

  it('applies error status', () => {
    render(<Badge status="error">Failed</Badge>);
    expect(hasModuleClass(screen.getByText('Failed'), 'error')).toBe(true);
  });

  it('applies info status', () => {
    render(<Badge status="info">Info</Badge>);
    expect(hasModuleClass(screen.getByText('Info'), 'info')).toBe(true);
  });

  it('applies accent status', () => {
    render(<Badge status="accent">Accent</Badge>);
    expect(hasModuleClass(screen.getByText('Accent'), 'accent')).toBe(true);
  });
});
