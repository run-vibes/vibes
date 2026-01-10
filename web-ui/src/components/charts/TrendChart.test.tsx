/**
 * Tests for TrendChart component
 */
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { TrendChart } from './TrendChart';

describe('TrendChart', () => {
  const mockSeries = [
    {
      id: 'series-1',
      label: 'Score',
      data: [
        { timestamp: '2026-01-01T00:00:00Z', value: 10 },
        { timestamp: '2026-01-02T00:00:00Z', value: 20 },
        { timestamp: '2026-01-03T00:00:00Z', value: 15 },
      ],
      color: 'var(--crt-success)',
    },
  ];

  const multipleSeries = [
    {
      id: 'series-a',
      label: 'Positive',
      data: [
        { timestamp: '2026-01-01T00:00:00Z', value: 10 },
        { timestamp: '2026-01-02T00:00:00Z', value: 20 },
      ],
      color: 'var(--crt-success)',
    },
    {
      id: 'series-b',
      label: 'Negative',
      data: [
        { timestamp: '2026-01-01T00:00:00Z', value: 5 },
        { timestamp: '2026-01-02T00:00:00Z', value: 8 },
      ],
      color: 'var(--crt-error)',
    },
  ];

  it('renders svg element', () => {
    const { container } = render(
      <TrendChart series={mockSeries} width={400} height={200} />
    );

    expect(container.querySelector('svg')).toBeInTheDocument();
  });

  it('applies width and height', () => {
    const { container } = render(
      <TrendChart series={mockSeries} width={500} height={250} />
    );

    const svg = container.querySelector('svg');
    expect(svg).toHaveAttribute('width', '500');
    expect(svg).toHaveAttribute('height', '250');
  });

  it('renders line path for series', () => {
    const { container } = render(
      <TrendChart series={mockSeries} width={400} height={200} />
    );

    expect(container.querySelector('path')).toBeInTheDocument();
  });

  it('renders x-axis', () => {
    const { container } = render(
      <TrendChart series={mockSeries} width={400} height={200} />
    );

    // X-axis group should exist
    expect(container.querySelector('.visx-axis-bottom')).toBeInTheDocument();
  });

  it('renders y-axis', () => {
    const { container } = render(
      <TrendChart series={mockSeries} width={400} height={200} />
    );

    // Y-axis group should exist
    expect(container.querySelector('.visx-axis-left')).toBeInTheDocument();
  });

  it('renders legend when showLegend is true', () => {
    render(
      <TrendChart series={multipleSeries} width={400} height={200} showLegend />
    );

    expect(screen.getByText('Positive')).toBeInTheDocument();
    expect(screen.getByText('Negative')).toBeInTheDocument();
  });

  it('applies trend-chart class', () => {
    const { container } = render(
      <TrendChart series={mockSeries} width={400} height={200} />
    );

    expect(container.firstChild).toHaveClass('trend-chart');
  });

  it('handles empty series', () => {
    const { container } = render(
      <TrendChart series={[]} width={400} height={200} />
    );

    expect(container.querySelector('svg')).toBeInTheDocument();
  });

  it('renders multiple series', () => {
    const { container } = render(
      <TrendChart series={multipleSeries} width={400} height={200} />
    );

    // Should have multiple paths
    const paths = container.querySelectorAll('path.trend-line');
    expect(paths.length).toBe(2);
  });
});
