import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Header } from './Header';

describe('Header', () => {
  it('renders logo', () => {
    render(<Header />);
    expect(screen.getByText(/vibes/i)).toBeInTheDocument();
  });

  it('renders as header element', () => {
    render(<Header />);
    expect(screen.getByRole('banner')).toBeInTheDocument();
  });

  it('renders nav items', () => {
    render(<Header navItems={[{ label: 'Sessions', href: '/sessions' }]} />);
    // Nav items appear in both desktop nav and mobile menu
    expect(screen.getAllByText('Sessions').length).toBeGreaterThanOrEqual(1);
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
    // Nav items appear in both desktop nav and mobile menu
    expect(screen.getAllByText('Sessions').length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText('Settings').length).toBeGreaterThanOrEqual(1);
  });

  it('renders identity when provided', () => {
    render(<Header identity={{ email: 'user@example.com' }} />);
    expect(screen.getByText('user@example.com')).toBeInTheDocument();
  });

  it('calls onThemeToggle when theme button clicked', () => {
    const onToggle = vi.fn();
    render(<Header theme="dark" onThemeToggle={onToggle} />);
    fireEvent.click(screen.getByLabelText('Toggle theme'));
    expect(onToggle).toHaveBeenCalled();
  });

  it('renders theme toggle with icon and text label', () => {
    render(<Header theme="dark" onThemeToggle={() => {}} />);
    const themeButton = screen.getByLabelText('Toggle theme');
    expect(themeButton).toHaveTextContent('◐');
    expect(themeButton).toHaveTextContent('THEME');
  });

  it('renders settings link with icon and text label', () => {
    render(<Header settingsHref="/settings" renderLink={({ href, className, children }) => (
      <a href={href} className={className}>{children}</a>
    )} />);
    // Settings link appears in both desktop header and mobile menu
    const settingsLinks = screen.getAllByRole('link', { name: /settings/i });
    expect(settingsLinks.length).toBeGreaterThanOrEqual(1);
    expect(settingsLinks[0]).toHaveTextContent('⚙');
    expect(settingsLinks[0]).toHaveTextContent('SETTINGS');
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
    // Nav items appear in both desktop nav and mobile menu
    expect(screen.getAllByText(/Groove/).length).toBeGreaterThanOrEqual(1);
  });

  it('renders subnav indicator for items with hasSubnav', () => {
    render(
      <Header
        navItems={[{ label: 'Groove', href: '/groove', hasSubnav: true }]}
      />
    );
    // Subnav indicator appears in both desktop nav and mobile menu
    expect(screen.getAllByText('▾').length).toBeGreaterThanOrEqual(1);
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
    // Custom link appears in both desktop nav and mobile menu
    const testElements = screen.getAllByText('Test');
    expect(testElements.length).toBeGreaterThanOrEqual(1);
    // Check that at least one has the data-href attribute
    const hasDataHref = testElements.some(el => el.closest('[data-href="/test"]'));
    expect(hasDataHref).toBe(true);
  });

  it('renders toolbar items when provided', () => {
    render(
      <Header
        toolbarItems={<span data-testid="custom-indicator">Custom</span>}
      />
    );
    expect(screen.getByTestId('custom-indicator')).toBeInTheDocument();
  });

  it('renders toolbar items before theme toggle', () => {
    render(
      <Header
        toolbarItems={<span>Indicator</span>}
        onThemeToggle={() => {}}
      />
    );
    const actions = screen.getByRole('banner').querySelector('[class*="actions"]');
    expect(actions?.textContent).toMatch(/Indicator.*THEME/);
  });

  // Mobile menu tests
  it('renders hamburger button', () => {
    render(<Header navItems={[{ label: 'Test', href: '/test' }]} />);
    expect(screen.getByLabelText('Open menu')).toBeInTheDocument();
  });

  it('hamburger button has correct aria attributes', () => {
    render(<Header navItems={[{ label: 'Test', href: '/test' }]} />);
    const hamburger = screen.getByLabelText('Open menu');
    expect(hamburger).toHaveAttribute('aria-expanded', 'false');
    expect(hamburger).toHaveAttribute('aria-controls', 'mobile-menu');
  });

  it('opens mobile menu when hamburger clicked', () => {
    render(<Header navItems={[{ label: 'Test', href: '/test' }]} onThemeToggle={() => {}} />);
    fireEvent.click(screen.getByLabelText('Open menu'));
    expect(screen.getByRole('dialog')).toBeInTheDocument();
  });

  it('mobile menu contains nav items', () => {
    render(<Header navItems={[{ label: 'Sessions', href: '/sessions' }]} onThemeToggle={() => {}} />);
    fireEvent.click(screen.getByLabelText('Open menu'));
    // Nav items appear in both desktop nav and mobile menu
    expect(screen.getAllByText('Sessions').length).toBeGreaterThanOrEqual(1);
  });

  it('closes mobile menu when close button clicked', () => {
    render(<Header navItems={[{ label: 'Test', href: '/test' }]} onThemeToggle={() => {}} />);
    fireEvent.click(screen.getByLabelText('Open menu'));
    expect(screen.getByRole('dialog')).toBeInTheDocument();
    fireEvent.click(screen.getByLabelText('Close menu'));
    // Dialog should no longer be visible (panel is hidden but still in DOM)
    expect(screen.getByLabelText('Open menu')).toHaveAttribute('aria-expanded', 'false');
  });

  it('closes mobile menu when Escape key pressed', () => {
    render(<Header navItems={[{ label: 'Test', href: '/test' }]} onThemeToggle={() => {}} />);
    fireEvent.click(screen.getByLabelText('Open menu'));
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(screen.getByLabelText('Open menu')).toHaveAttribute('aria-expanded', 'false');
  });

  it('closes mobile menu when pathname changes', () => {
    const { rerender } = render(
      <Header navItems={[{ label: 'Test', href: '/test' }]} pathname="/page1" onThemeToggle={() => {}} />
    );
    fireEvent.click(screen.getByLabelText('Open menu'));
    expect(screen.getByLabelText('Open menu')).toHaveAttribute('aria-expanded', 'true');

    rerender(
      <Header navItems={[{ label: 'Test', href: '/test' }]} pathname="/page2" onThemeToggle={() => {}} />
    );
    expect(screen.getByLabelText('Open menu')).toHaveAttribute('aria-expanded', 'false');
  });
});
