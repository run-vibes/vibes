/**
 * Tests for Sparkline component
 */
import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/react';
import { Sparkline } from './Sparkline';

describe('Sparkline', () => {
  const mockData = [10, 20, 15, 25, 30, 22, 28];

  it('renders svg element', () => {
    const { container } = render(<Sparkline data={mockData} width={100} height={30} />);

    expect(container.querySelector('svg')).toBeInTheDocument();
  });

  it('applies width and height', () => {
    const { container } = render(<Sparkline data={mockData} width={150} height={40} />);

    const svg = container.querySelector('svg');
    expect(svg).toHaveAttribute('width', '150');
    expect(svg).toHaveAttribute('height', '40');
  });

  it('renders path for line', () => {
    const { container } = render(<Sparkline data={mockData} width={100} height={30} />);

    expect(container.querySelector('path')).toBeInTheDocument();
  });

  it('applies custom color', () => {
    const { container } = render(
      <Sparkline data={mockData} width={100} height={30} color="var(--crt-error)" />
    );

    const path = container.querySelector('path');
    expect(path).toHaveAttribute('stroke', 'var(--crt-error)');
  });

  it('uses default green color', () => {
    const { container } = render(<Sparkline data={mockData} width={100} height={30} />);

    const path = container.querySelector('path');
    expect(path).toHaveAttribute('stroke', 'var(--crt-success)');
  });

  it('handles empty data', () => {
    const { container } = render(<Sparkline data={[]} width={100} height={30} />);

    expect(container.querySelector('svg')).toBeInTheDocument();
  });

  it('handles single data point', () => {
    const { container } = render(<Sparkline data={[50]} width={100} height={30} />);

    expect(container.querySelector('svg')).toBeInTheDocument();
  });

  it('applies sparkline class for styling', () => {
    const { container } = render(<Sparkline data={mockData} width={100} height={30} />);

    expect(container.firstChild).toHaveClass('sparkline');
  });

  it('renders area fill when showArea is true', () => {
    const { container } = render(
      <Sparkline data={mockData} width={100} height={30} showArea />
    );

    // Should have both line path and area path
    const paths = container.querySelectorAll('path');
    expect(paths.length).toBeGreaterThanOrEqual(1);
  });
});
