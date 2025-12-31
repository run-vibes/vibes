import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Panel } from './Panel';

// Helper to check if element has a CSS module class (handles hashed names)
const hasModuleClass = (element: HTMLElement, className: string): boolean => {
  return Array.from(element.classList).some(cls =>
    cls.includes(className) || cls.startsWith(`_${className}_`)
  );
};

describe('Panel', () => {
  it('renders children', () => {
    render(<Panel>Content</Panel>);
    expect(screen.getByText('Content')).toBeInTheDocument();
  });

  it('renders title when provided', () => {
    render(<Panel title="Settings">Content</Panel>);
    expect(screen.getByText('Settings')).toBeInTheDocument();
  });

  it('applies default variant', () => {
    const { container } = render(<Panel>Content</Panel>);
    expect(hasModuleClass(container.firstChild as HTMLElement, 'default')).toBe(true);
  });

  it('applies elevated variant', () => {
    const { container } = render(<Panel variant="elevated">Content</Panel>);
    expect(hasModuleClass(container.firstChild as HTMLElement, 'elevated')).toBe(true);
  });

  it('applies inset variant', () => {
    const { container } = render(<Panel variant="inset">Content</Panel>);
    expect(hasModuleClass(container.firstChild as HTMLElement, 'inset')).toBe(true);
  });

  it('renders actions when provided', () => {
    render(
      <Panel title="Settings" actions={<button>Action</button>}>
        Content
      </Panel>
    );
    expect(screen.getByRole('button', { name: 'Action' })).toBeInTheDocument();
  });

  it('applies noPadding modifier', () => {
    const { container } = render(<Panel noPadding>Content</Panel>);
    expect(hasModuleClass(container.firstChild as HTMLElement, 'noPadding')).toBe(true);
  });

  it('merges custom className', () => {
    const { container } = render(<Panel className="custom-class">Content</Panel>);
    expect(container.firstChild).toHaveClass('custom-class');
  });

  it('passes through additional props', () => {
    render(<Panel data-testid="custom-panel" aria-label="Custom">Content</Panel>);
    expect(screen.getByTestId('custom-panel')).toBeInTheDocument();
    expect(screen.getByTestId('custom-panel')).toHaveAttribute('aria-label', 'Custom');
  });
});
