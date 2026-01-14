import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { Card } from './Card';

// Helper to check if element has a CSS module class (handles hashed names)
const hasModuleClass = (element: HTMLElement, className: string): boolean => {
  return Array.from(element.classList).some(cls =>
    cls.includes(className) || cls.startsWith(`_${className}_`)
  );
};

describe('Card', () => {
  it('renders children', () => {
    render(<Card>Content</Card>);
    expect(screen.getByText('Content')).toBeInTheDocument();
  });

  it('renders title when provided', () => {
    render(<Card title="Settings">Content</Card>);
    expect(screen.getByText('Settings')).toBeInTheDocument();
  });

  it('applies default variant', () => {
    const { container } = render(<Card>Content</Card>);
    expect(hasModuleClass(container.firstChild as HTMLElement, 'default')).toBe(true);
  });

  it('applies elevated variant', () => {
    const { container } = render(<Card variant="elevated">Content</Card>);
    expect(hasModuleClass(container.firstChild as HTMLElement, 'elevated')).toBe(true);
  });

  it('applies inset variant', () => {
    const { container } = render(<Card variant="inset">Content</Card>);
    expect(hasModuleClass(container.firstChild as HTMLElement, 'inset')).toBe(true);
  });

  it('applies crt variant', () => {
    const { container } = render(<Card variant="crt">Content</Card>);
    expect(hasModuleClass(container.firstChild as HTMLElement, 'crt')).toBe(true);
  });

  it('renders actions when provided', () => {
    render(
      <Card title="Settings" actions={<button>Action</button>}>
        Content
      </Card>
    );
    expect(screen.getByRole('button', { name: 'Action' })).toBeInTheDocument();
  });

  it('applies noPadding modifier', () => {
    const { container } = render(<Card noPadding>Content</Card>);
    expect(hasModuleClass(container.firstChild as HTMLElement, 'noPadding')).toBe(true);
  });

  it('merges custom className', () => {
    const { container } = render(<Card className="custom-class">Content</Card>);
    expect(container.firstChild).toHaveClass('custom-class');
  });

  it('passes through additional props', () => {
    render(<Card data-testid="custom-card" aria-label="Custom">Content</Card>);
    expect(screen.getByTestId('custom-card')).toBeInTheDocument();
    expect(screen.getByTestId('custom-card')).toHaveAttribute('aria-label', 'Custom');
  });

  it('renders footer when provided', () => {
    const { container } = render(
      <Card title="Health" footer={<a href="#">View →</a>}>
        Content
      </Card>
    );
    expect(screen.getByRole('link', { name: 'View →' })).toBeInTheDocument();
    // Footer should be in a separate div with footer class, outside content
    const footerElement = container.querySelector('[class*="footer"]');
    expect(footerElement).toBeInTheDocument();
    expect(footerElement).toContainElement(screen.getByRole('link'));
  });

  it('does not render footer div when footer not provided', () => {
    const { container } = render(<Card title="Settings">Content</Card>);
    const footerElement = container.querySelector('[class*="footer"]');
    expect(footerElement).not.toBeInTheDocument();
  });
});
