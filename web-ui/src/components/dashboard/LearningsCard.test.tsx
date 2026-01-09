/**
 * Tests for the LearningsCard component
 */
import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';

// Mock TanStack Router Link component
vi.mock('@tanstack/react-router', () => ({
  Link: ({ to, children }: { to: string; children: React.ReactNode }) => (
    <a href={to}>{children}</a>
  ),
}));

import { LearningsCard } from './LearningsCard';
import type { LearningSummary, LearningBrief } from '../../hooks/useDashboard';

// Helper to create a full LearningBrief with minimal required data
function makeLearning(overrides: { id: string; content: string; created_at: string }): LearningBrief {
  return {
    ...overrides,
    category: 'Correction',
    scope: { User: 'test' },
    status: 'active',
    estimated_value: 0.5,
  };
}

describe('LearningsCard', () => {
  it('renders title', () => {
    render(<LearningsCard />);
    expect(screen.getByText('Learnings')).toBeInTheDocument();
  });

  it('shows total and active counts', () => {
    const data: LearningSummary = {
      total: 42,
      active: 35,
      recent: [],
      by_category: {},
    };
    render(<LearningsCard data={data} />);

    expect(screen.getByText('Total')).toBeInTheDocument();
    expect(screen.getByText('42')).toBeInTheDocument();
    expect(screen.getByText('Active')).toBeInTheDocument();
    expect(screen.getByText('35')).toBeInTheDocument();
  });

  it('shows recent learnings', () => {
    const data: LearningSummary = {
      total: 10,
      active: 8,
      recent: [
        makeLearning({ id: '1', content: 'First learning', created_at: new Date().toISOString() }),
        makeLearning({ id: '2', content: 'Second learning', created_at: new Date().toISOString() }),
      ],
      by_category: {},
    };
    render(<LearningsCard data={data} />);

    expect(screen.getByText('Recent:')).toBeInTheDocument();
    expect(screen.getByText('First learning')).toBeInTheDocument();
    expect(screen.getByText('Second learning')).toBeInTheDocument();
  });

  it('shows relative time for recent learnings', () => {
    const oneHourAgo = new Date(Date.now() - 60 * 60 * 1000).toISOString();
    const data: LearningSummary = {
      total: 1,
      active: 1,
      recent: [makeLearning({ id: '1', content: 'Test', created_at: oneHourAgo })],
      by_category: {},
    };
    render(<LearningsCard data={data} />);

    expect(screen.getByText('1h ago')).toBeInTheDocument();
  });

  it('shows "just now" for very recent learnings', () => {
    const data: LearningSummary = {
      total: 1,
      active: 1,
      recent: [makeLearning({ id: '1', content: 'Test', created_at: new Date().toISOString() })],
      by_category: {},
    };
    render(<LearningsCard data={data} />);

    expect(screen.getByText('just now')).toBeInTheDocument();
  });

  it('limits displayed recent learnings to 5', () => {
    const data: LearningSummary = {
      total: 10,
      active: 10,
      recent: Array.from({ length: 10 }, (_, i) =>
        makeLearning({
          id: String(i),
          content: `Learning ${i}`,
          created_at: new Date().toISOString(),
        })
      ),
      by_category: {},
    };
    render(<LearningsCard data={data} />);

    // Should only show first 5
    expect(screen.getByText('Learning 0')).toBeInTheDocument();
    expect(screen.getByText('Learning 4')).toBeInTheDocument();
    expect(screen.queryByText('Learning 5')).not.toBeInTheDocument();
  });

  it('handles zero counts gracefully', () => {
    const data: LearningSummary = {
      total: 0,
      active: 0,
      recent: [],
      by_category: {},
    };
    render(<LearningsCard data={data} />);

    expect(screen.getAllByText('0')).toHaveLength(2);
  });

  it('renders link to learnings page', () => {
    render(<LearningsCard />);

    expect(screen.getByText('View →')).toBeInTheDocument();
    expect(screen.getByRole('link')).toHaveAttribute('href', '/groove/dashboard/learnings');
  });
});
