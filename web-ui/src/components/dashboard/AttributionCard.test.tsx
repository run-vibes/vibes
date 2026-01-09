/**
 * Tests for the AttributionCard component
 */
import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';

// Mock TanStack Router Link component
vi.mock('@tanstack/react-router', () => ({
  Link: ({ to, children }: { to: string; children: React.ReactNode }) => (
    <a href={to}>{children}</a>
  ),
}));

import { AttributionCard } from './AttributionCard';
import type { AttributionSummary } from '../../hooks/useDashboard';

describe('AttributionCard', () => {
  it('renders title', () => {
    render(<AttributionCard />);
    expect(screen.getByText('Attribution')).toBeInTheDocument();
  });

  it('shows top contributors with estimated values', () => {
    const data: AttributionSummary = {
      top_contributors: [
        { learning_id: '1', content: 'First contributor', estimated_value: 1.5, confidence: 0.9 },
        { learning_id: '2', content: 'Second contributor', estimated_value: 0.75, confidence: 0.8 },
      ],
      under_review_count: 0,
      negative_count: 0,
    };
    render(<AttributionCard data={data} />);

    expect(screen.getByText('Top Contributors:')).toBeInTheDocument();
    expect(screen.getByText('First contributor')).toBeInTheDocument();
    expect(screen.getByText('+1.50')).toBeInTheDocument();
    expect(screen.getByText('Second contributor')).toBeInTheDocument();
    expect(screen.getByText('+0.75')).toBeInTheDocument();
  });

  it('shows ranking numbers for contributors', () => {
    const data: AttributionSummary = {
      top_contributors: [
        { learning_id: '1', content: 'First', estimated_value: 2.0, confidence: 0.95 },
        { learning_id: '2', content: 'Second', estimated_value: 1.0, confidence: 0.85 },
        { learning_id: '3', content: 'Third', estimated_value: 0.5, confidence: 0.75 },
      ],
      under_review_count: 0,
      negative_count: 0,
    };
    render(<AttributionCard data={data} />);

    expect(screen.getByText('1.')).toBeInTheDocument();
    expect(screen.getByText('2.')).toBeInTheDocument();
    expect(screen.getByText('3.')).toBeInTheDocument();
  });

  it('limits displayed contributors to 3', () => {
    const data: AttributionSummary = {
      top_contributors: Array.from({ length: 5 }, (_, i) => ({
        learning_id: String(i),
        content: `Contributor ${i}`,
        estimated_value: 1.0,
        confidence: 0.8,
      })),
      under_review_count: 0,
      negative_count: 0,
    };
    render(<AttributionCard data={data} />);

    expect(screen.getByText('Contributor 0')).toBeInTheDocument();
    expect(screen.getByText('Contributor 2')).toBeInTheDocument();
    expect(screen.queryByText('Contributor 3')).not.toBeInTheDocument();
  });

  it('shows under review warning', () => {
    const data: AttributionSummary = {
      top_contributors: [],
      under_review_count: 3,
      negative_count: 0,
    };
    render(<AttributionCard data={data} />);

    expect(screen.getByText('3 learnings under review')).toBeInTheDocument();
    expect(screen.getByText('⚠')).toBeInTheDocument();
  });

  it('shows singular form for single under review', () => {
    const data: AttributionSummary = {
      top_contributors: [],
      under_review_count: 1,
      negative_count: 0,
    };
    render(<AttributionCard data={data} />);

    expect(screen.getByText('1 learning under review')).toBeInTheDocument();
  });

  it('shows negative impact warning', () => {
    const data: AttributionSummary = {
      top_contributors: [],
      under_review_count: 0,
      negative_count: 2,
    };
    render(<AttributionCard data={data} />);

    expect(screen.getByText('2 with negative impact')).toBeInTheDocument();
  });

  it('shows both warnings when applicable', () => {
    const data: AttributionSummary = {
      top_contributors: [],
      under_review_count: 2,
      negative_count: 1,
    };
    render(<AttributionCard data={data} />);

    expect(screen.getByText('2 learnings under review')).toBeInTheDocument();
    expect(screen.getByText('1 with negative impact')).toBeInTheDocument();
  });

  it('shows empty state when no contributors', () => {
    const data: AttributionSummary = {
      top_contributors: [],
      under_review_count: 0,
      negative_count: 0,
    };
    render(<AttributionCard data={data} />);

    expect(screen.getByText('No data yet')).toBeInTheDocument();
  });

  it('renders link to attribution page', () => {
    render(<AttributionCard />);

    expect(screen.getByText('View →')).toBeInTheDocument();
    expect(screen.getByRole('link')).toHaveAttribute('href', '/groove/dashboard/attribution');
  });
});
