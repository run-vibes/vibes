import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { Tabs } from './Tabs';

// Helper to check if element has a CSS module class (handles hashed names)
const hasModuleClass = (element: HTMLElement, className: string): boolean => {
  return Array.from(element.classList).some(cls =>
    cls.includes(className) || cls.startsWith(`_${className}_`)
  );
};

describe('Tabs', () => {
  it('renders tabs with correct role', () => {
    render(
      <Tabs value="one" onChange={() => {}}>
        <Tabs.Tab value="one">Tab One</Tabs.Tab>
        <Tabs.Tab value="two">Tab Two</Tabs.Tab>
      </Tabs>
    );
    expect(screen.getByRole('tablist')).toBeInTheDocument();
    expect(screen.getAllByRole('tab')).toHaveLength(2);
  });

  it('renders tab children', () => {
    render(
      <Tabs value="one" onChange={() => {}}>
        <Tabs.Tab value="one">First Tab</Tabs.Tab>
        <Tabs.Tab value="two">Second Tab</Tabs.Tab>
      </Tabs>
    );
    expect(screen.getByText('First Tab')).toBeInTheDocument();
    expect(screen.getByText('Second Tab')).toBeInTheDocument();
  });

  it('marks active tab with aria-selected', () => {
    render(
      <Tabs value="two" onChange={() => {}}>
        <Tabs.Tab value="one">Tab One</Tabs.Tab>
        <Tabs.Tab value="two">Tab Two</Tabs.Tab>
      </Tabs>
    );
    expect(screen.getByText('Tab One')).toHaveAttribute('aria-selected', 'false');
    expect(screen.getByText('Tab Two')).toHaveAttribute('aria-selected', 'true');
  });

  it('applies active class to selected tab', () => {
    render(
      <Tabs value="one" onChange={() => {}}>
        <Tabs.Tab value="one">Tab One</Tabs.Tab>
        <Tabs.Tab value="two">Tab Two</Tabs.Tab>
      </Tabs>
    );
    expect(hasModuleClass(screen.getByText('Tab One'), 'active')).toBe(true);
    expect(hasModuleClass(screen.getByText('Tab Two'), 'active')).toBe(false);
  });

  it('calls onChange when tab is clicked', () => {
    const handleChange = vi.fn();
    render(
      <Tabs value="one" onChange={handleChange}>
        <Tabs.Tab value="one">Tab One</Tabs.Tab>
        <Tabs.Tab value="two">Tab Two</Tabs.Tab>
      </Tabs>
    );
    fireEvent.click(screen.getByText('Tab Two'));
    expect(handleChange).toHaveBeenCalledWith('two');
  });

  it('calls onChange with correct value for each tab', () => {
    const handleChange = vi.fn();
    render(
      <Tabs value="one" onChange={handleChange}>
        <Tabs.Tab value="first">First</Tabs.Tab>
        <Tabs.Tab value="second">Second</Tabs.Tab>
        <Tabs.Tab value="third">Third</Tabs.Tab>
      </Tabs>
    );

    fireEvent.click(screen.getByText('Third'));
    expect(handleChange).toHaveBeenCalledWith('third');

    fireEvent.click(screen.getByText('First'));
    expect(handleChange).toHaveBeenCalledWith('first');
  });

  it('passes through additional props to tab', () => {
    render(
      <Tabs value="one" onChange={() => {}}>
        <Tabs.Tab value="one" data-testid="custom-tab" aria-label="Custom">Tab</Tabs.Tab>
      </Tabs>
    );
    expect(screen.getByTestId('custom-tab')).toBeInTheDocument();
    expect(screen.getByRole('tab')).toHaveAttribute('aria-label', 'Custom');
  });

  it('merges custom className on Tabs', () => {
    render(
      <Tabs value="one" onChange={() => {}} className="custom-tabs">
        <Tabs.Tab value="one">Tab</Tabs.Tab>
      </Tabs>
    );
    expect(screen.getByRole('tablist')).toHaveClass('custom-tabs');
  });

  it('merges custom className on Tab', () => {
    render(
      <Tabs value="one" onChange={() => {}}>
        <Tabs.Tab value="one" className="custom-tab">Tab</Tabs.Tab>
      </Tabs>
    );
    expect(screen.getByRole('tab')).toHaveClass('custom-tab');
  });

  it('has type="button" on tabs', () => {
    render(
      <Tabs value="one" onChange={() => {}}>
        <Tabs.Tab value="one">Tab</Tabs.Tab>
      </Tabs>
    );
    expect(screen.getByRole('tab')).toHaveAttribute('type', 'button');
  });

  it('can disable individual tabs', () => {
    const handleChange = vi.fn();
    render(
      <Tabs value="one" onChange={handleChange}>
        <Tabs.Tab value="one">Tab One</Tabs.Tab>
        <Tabs.Tab value="two" disabled>Tab Two</Tabs.Tab>
      </Tabs>
    );
    expect(screen.getByText('Tab Two')).toBeDisabled();
    fireEvent.click(screen.getByText('Tab Two'));
    expect(handleChange).not.toHaveBeenCalled();
  });

  it('throws error when Tab is used outside Tabs', () => {
    const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {});
    expect(() => {
      render(<Tabs.Tab value="orphan">Orphan</Tabs.Tab>);
    }).toThrow('Tabs.Tab must be used within a Tabs component');
    consoleError.mockRestore();
  });
});
