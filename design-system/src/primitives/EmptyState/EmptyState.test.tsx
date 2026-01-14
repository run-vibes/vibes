import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { EmptyState } from './EmptyState';

// Helper to check if element has a CSS module class (handles hashed names)
const hasModuleClass = (element: HTMLElement, className: string): boolean => {
  return Array.from(element.classList).some(cls =>
    cls.includes(className) || cls.startsWith(`_${className}_`)
  );
};

describe('EmptyState', () => {
  it('renders message', () => {
    render(<EmptyState message="No items found" />);
    expect(screen.getByText('No items found')).toBeInTheDocument();
  });

  it('renders hint when provided', () => {
    render(<EmptyState message="No items" hint="Try adding some items" />);
    expect(screen.getByText('Try adding some items')).toBeInTheDocument();
  });

  it('renders icon when provided', () => {
    render(<EmptyState message="Empty" icon={<span data-testid="icon">📦</span>} />);
    expect(screen.getByTestId('icon')).toBeInTheDocument();
  });

  it('renders action when provided', () => {
    render(<EmptyState message="Empty" action={<button>Add item</button>} />);
    expect(screen.getByRole('button', { name: 'Add item' })).toBeInTheDocument();
  });

  it('applies size variant classes', () => {
    const { container, rerender } = render(<EmptyState message="Empty" size="sm" />);
    expect(hasModuleClass(container.firstChild as HTMLElement, 'sm')).toBe(true);

    rerender(<EmptyState message="Empty" size="lg" />);
    expect(hasModuleClass(container.firstChild as HTMLElement, 'lg')).toBe(true);
  });

  it('applies custom className', () => {
    const { container } = render(<EmptyState message="Empty" className="custom-class" />);
    expect(container.firstChild).toHaveClass('custom-class');
  });
});
