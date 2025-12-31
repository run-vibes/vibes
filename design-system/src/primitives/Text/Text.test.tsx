import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Text } from './Text';

// Helper to check if element has a CSS module class (handles hashed names)
const hasModuleClass = (element: HTMLElement, className: string): boolean => {
  return Array.from(element.classList).some(cls =>
    cls.includes(className) || cls.startsWith(`_${className}_`)
  );
};

describe('Text', () => {
  it('renders children', () => {
    render(<Text>Hello world</Text>);
    expect(screen.getByText('Hello world')).toBeInTheDocument();
  });

  it('applies normal intensity by default', () => {
    render(<Text>Normal</Text>);
    expect(hasModuleClass(screen.getByText('Normal'), 'normal')).toBe(true);
  });

  it('applies high intensity', () => {
    render(<Text intensity="high">Important</Text>);
    expect(hasModuleClass(screen.getByText('Important'), 'high')).toBe(true);
  });

  it('applies dim intensity', () => {
    render(<Text intensity="dim">Metadata</Text>);
    expect(hasModuleClass(screen.getByText('Metadata'), 'dim')).toBe(true);
  });

  it('applies mono font', () => {
    render(<Text mono>code</Text>);
    expect(hasModuleClass(screen.getByText('code'), 'mono')).toBe(true);
  });

  it('renders as different elements', () => {
    render(<Text as="h1">Heading</Text>);
    expect(screen.getByRole('heading', { level: 1 })).toBeInTheDocument();
  });

  it('applies size classes', () => {
    render(<Text size="lg">Large</Text>);
    expect(hasModuleClass(screen.getByText('Large'), 'lg')).toBe(true);
  });

  it('merges custom className', () => {
    render(<Text className="custom-class">Custom</Text>);
    expect(screen.getByText('Custom')).toHaveClass('custom-class');
  });

  it('passes through additional props', () => {
    render(<Text data-testid="custom-text" aria-label="Custom">Text</Text>);
    expect(screen.getByTestId('custom-text')).toBeInTheDocument();
    expect(screen.getByTestId('custom-text')).toHaveAttribute('aria-label', 'Custom');
  });

  it('renders as span by default', () => {
    const { container } = render(<Text>Default</Text>);
    expect(container.querySelector('span')).toBeInTheDocument();
  });
});
