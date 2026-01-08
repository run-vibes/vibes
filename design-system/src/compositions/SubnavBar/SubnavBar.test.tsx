import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { SubnavBar } from './SubnavBar';

describe('SubnavBar', () => {
  it('renders when open', () => {
    render(<SubnavBar isOpen label="GROOVE" />);
    expect(screen.getByText('GROOVE')).toBeInTheDocument();
  });

  it('renders nav items', () => {
    render(
      <SubnavBar
        isOpen
        items={[
          { label: 'Security', href: '/groove/security' },
          { label: 'Assessment', href: '/groove/assessment' },
        ]}
      />
    );
    expect(screen.getByText('Security')).toBeInTheDocument();
    expect(screen.getByText('Assessment')).toBeInTheDocument();
  });

  it('renders item icons', () => {
    render(
      <SubnavBar
        isOpen
        items={[{ label: 'Security', href: '/groove/security', icon: '🛡' }]}
      />
    );
    expect(screen.getByText('🛡')).toBeInTheDocument();
  });

  it('applies active state to items', () => {
    render(
      <SubnavBar
        isOpen
        items={[
          { label: 'Security', href: '/groove/security', isActive: true },
          { label: 'Assessment', href: '/groove/assessment' },
        ]}
      />
    );
    // CSS modules hash class names, so we check for partial match
    const securityLink = screen.getByText('Security').closest('a');
    const assessmentLink = screen.getByText('Assessment').closest('a');
    expect(securityLink?.className).toMatch(/subnavItemActive/);
    expect(assessmentLink?.className).not.toMatch(/subnavItemActive/);
  });

  it('uses custom renderLink', () => {
    const CustomLink = ({ href, className, children }: { href: string; className: string; children: React.ReactNode }) => (
      <span data-href={href} className={className}>{children}</span>
    );
    render(
      <SubnavBar
        isOpen
        items={[{ label: 'Test', href: '/test' }]}
        renderLink={CustomLink}
      />
    );
    expect(screen.getByText('Test')).toHaveAttribute('data-href', '/test');
  });

  it('applies plugin-specific class', () => {
    const { container } = render(<SubnavBar isOpen plugin="groove" />);
    expect((container.firstChild as HTMLElement)?.className).toMatch(/pluginGroove/);
  });

  it('merges custom className', () => {
    const { container } = render(<SubnavBar isOpen className="custom-class" />);
    expect(container.firstChild).toHaveClass('custom-class');
  });
});
