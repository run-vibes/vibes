import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Header } from './Header';

describe('Header', () => {
  it('renders logo', () => {
    render(<Header />);
    expect(screen.getByText(/vibes/)).toBeInTheDocument();
  });

  it('renders as header element', () => {
    render(<Header />);
    expect(screen.getByRole('banner')).toBeInTheDocument();
  });

  it('renders nav items', () => {
    render(<Header navItems={[{ label: 'Sessions', href: '/sessions' }]} />);
    expect(screen.getByText('Sessions')).toBeInTheDocument();
  });

  it('renders multiple nav items', () => {
    render(
      <Header
        navItems={[
          { label: 'Sessions', href: '/sessions' },
          { label: 'Settings', href: '/settings' },
        ]}
      />
    );
    expect(screen.getByText('Sessions')).toBeInTheDocument();
    expect(screen.getByText('Settings')).toBeInTheDocument();
  });

  it('renders identity when provided', () => {
    render(<Header identity={{ email: 'user@example.com' }} />);
    expect(screen.getByText('user@example.com')).toBeInTheDocument();
  });

  it('renders local badge when isLocal', () => {
    render(<Header isLocal />);
    expect(screen.getByText('Local')).toBeInTheDocument();
  });

  it('calls onThemeToggle when theme button clicked', () => {
    const onToggle = vi.fn();
    render(<Header theme="dark" onThemeToggle={onToggle} />);
    fireEvent.click(screen.getByLabelText('Toggle theme'));
    expect(onToggle).toHaveBeenCalled();
  });

  it('does not render theme toggle when onThemeToggle not provided', () => {
    render(<Header />);
    expect(screen.queryByLabelText('Toggle theme')).not.toBeInTheDocument();
  });

  it('renders groove nav item with groove styling', () => {
    render(
      <Header
        navItems={[{ label: 'Groove', href: '/groove', isGroove: true }]}
      />
    );
    expect(screen.getByText(/Groove/)).toBeInTheDocument();
  });

  it('merges custom className', () => {
    render(<Header className="custom-class" />);
    expect(screen.getByRole('banner')).toHaveClass('custom-class');
  });

  it('passes through additional props', () => {
    render(<Header data-testid="custom-header" aria-label="Main navigation" />);
    expect(screen.getByTestId('custom-header')).toBeInTheDocument();
    expect(screen.getByTestId('custom-header')).toHaveAttribute('aria-label', 'Main navigation');
  });

  it('uses custom renderLink when provided', () => {
    const CustomLink = ({ href, className, children }: { href: string; className: string; children: React.ReactNode }) => (
      <span data-href={href} className={className}>{children}</span>
    );
    render(
      <Header
        navItems={[{ label: 'Test', href: '/test' }]}
        renderLink={CustomLink}
      />
    );
    expect(screen.getByText('Test')).toHaveAttribute('data-href', '/test');
  });
});
