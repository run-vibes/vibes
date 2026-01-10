/**
 * Tests for Leaderboard and related components
 */
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { ContributorCard } from './ContributorCard';
import { NegativeImpact } from './NegativeImpact';
import { AblationCoverage } from './AblationCoverage';
import { Leaderboard } from './Leaderboard';
import type { AttributionEntry, AblationCoverage as AblationCoverageType } from '../../../hooks/useDashboard';

const mockContributor: AttributionEntry = {
  learning_id: 'learn-123',
  content: 'Use semantic HTML elements',
  estimated_value: 0.34,
  confidence: 0.87,
  session_count: 23,
  status: 'active',
};

const mockNegativeContributor: AttributionEntry = {
  learning_id: 'learn-456',
  content: 'Avoid console.log',
  estimated_value: -0.12,
  confidence: 0.65,
  session_count: 8,
  status: 'under_review',
};

const mockAblationCoverage: AblationCoverageType = {
  coverage_percent: 42,
  completed: 12,
  in_progress: 5,
  pending: 30,
};

describe('ContributorCard', () => {
  it('renders learning content', () => {
    render(<ContributorCard entry={mockContributor} rank={1} />);

    expect(screen.getByText('Use semantic HTML elements')).toBeInTheDocument();
  });

  it('shows rank number', () => {
    render(<ContributorCard entry={mockContributor} rank={1} />);

    expect(screen.getByText('1')).toBeInTheDocument();
  });

  it('displays value contribution with sign', () => {
    render(<ContributorCard entry={mockContributor} rank={1} />);

    expect(screen.getByText('+0.34')).toBeInTheDocument();
  });

  it('shows confidence percentage', () => {
    render(<ContributorCard entry={mockContributor} rank={1} />);

    expect(screen.getByText('87%')).toBeInTheDocument();
  });

  it('shows session count', () => {
    render(<ContributorCard entry={mockContributor} rank={1} />);

    expect(screen.getByText('23 sessions')).toBeInTheDocument();
  });

  it('includes value bar visualization', () => {
    render(<ContributorCard entry={mockContributor} rank={1} />);

    const valueBar = screen.getByRole('progressbar');
    expect(valueBar).toBeInTheDocument();
  });

  it('calls onClick when clicked', () => {
    const onClick = vi.fn();
    render(<ContributorCard entry={mockContributor} rank={1} onClick={onClick} />);

    fireEvent.click(screen.getByText('Use semantic HTML elements'));
    expect(onClick).toHaveBeenCalledWith('learn-123');
  });
});

describe('NegativeImpact', () => {
  it('renders warning section title', () => {
    render(<NegativeImpact entries={[mockNegativeContributor]} />);

    expect(screen.getByText('Negative Impact')).toBeInTheDocument();
  });

  it('shows negative learning content', () => {
    render(<NegativeImpact entries={[mockNegativeContributor]} />);

    expect(screen.getByText('Avoid console.log')).toBeInTheDocument();
  });

  it('displays negative value', () => {
    render(<NegativeImpact entries={[mockNegativeContributor]} />);

    expect(screen.getByText('-0.12')).toBeInTheDocument();
  });

  it('shows status badge', () => {
    render(<NegativeImpact entries={[mockNegativeContributor]} />);

    expect(screen.getByText(/under review/i)).toBeInTheDocument();
  });

  it('shows disable action button for active learnings', () => {
    const activeNegative = { ...mockNegativeContributor, status: 'active' as const };
    render(<NegativeImpact entries={[activeNegative]} />);

    expect(screen.getByRole('button', { name: /disable/i })).toBeInTheDocument();
  });

  it('hides disable button for non-active learnings', () => {
    render(<NegativeImpact entries={[mockNegativeContributor]} />);

    expect(screen.queryByRole('button', { name: /disable/i })).not.toBeInTheDocument();
  });

  it('returns null when no entries', () => {
    const { container } = render(<NegativeImpact entries={[]} />);

    expect(container.firstChild).toBeNull();
  });
});

describe('AblationCoverage', () => {
  it('renders progress bar', () => {
    render(<AblationCoverage coverage={mockAblationCoverage} />);

    expect(screen.getByRole('progressbar')).toBeInTheDocument();
  });

  it('shows coverage percentage', () => {
    render(<AblationCoverage coverage={mockAblationCoverage} />);

    expect(screen.getByText('42%')).toBeInTheDocument();
  });

  it('shows completed count', () => {
    render(<AblationCoverage coverage={mockAblationCoverage} />);

    expect(screen.getByText(/12 completed/i)).toBeInTheDocument();
  });

  it('shows in progress count', () => {
    render(<AblationCoverage coverage={mockAblationCoverage} />);

    expect(screen.getByText(/5 in progress/i)).toBeInTheDocument();
  });

  it('shows pending count', () => {
    render(<AblationCoverage coverage={mockAblationCoverage} />);

    expect(screen.getByText(/30 pending/i)).toBeInTheDocument();
  });
});

describe('Leaderboard', () => {
  const mockContributors = [
    mockContributor,
    { ...mockContributor, learning_id: 'learn-789', content: 'Prefer async/await', estimated_value: 0.28 },
  ];

  it('renders top contributors section', () => {
    render(
      <Leaderboard
        contributors={mockContributors}
        negativeImpact={[]}
        ablationCoverage={mockAblationCoverage}
      />
    );

    expect(screen.getByText('Top Contributors')).toBeInTheDocument();
  });

  it('renders all contributor cards', () => {
    render(
      <Leaderboard
        contributors={mockContributors}
        negativeImpact={[]}
        ablationCoverage={mockAblationCoverage}
      />
    );

    expect(screen.getByText('Use semantic HTML elements')).toBeInTheDocument();
    expect(screen.getByText('Prefer async/await')).toBeInTheDocument();
  });

  it('shows period selector', () => {
    render(
      <Leaderboard
        contributors={mockContributors}
        negativeImpact={[]}
        ablationCoverage={mockAblationCoverage}
      />
    );

    expect(screen.getByRole('combobox', { name: /period/i })).toBeInTheDocument();
  });

  it('calls onPeriodChange when period changes', () => {
    const onPeriodChange = vi.fn();
    render(
      <Leaderboard
        contributors={mockContributors}
        negativeImpact={[]}
        ablationCoverage={mockAblationCoverage}
        onPeriodChange={onPeriodChange}
      />
    );

    fireEvent.change(screen.getByRole('combobox', { name: /period/i }), {
      target: { value: '30' },
    });

    expect(onPeriodChange).toHaveBeenCalledWith(30);
  });

  it('shows negative impact section when present', () => {
    render(
      <Leaderboard
        contributors={mockContributors}
        negativeImpact={[mockNegativeContributor]}
        ablationCoverage={mockAblationCoverage}
      />
    );

    expect(screen.getByText('Negative Impact')).toBeInTheDocument();
  });

  it('shows ablation coverage section', () => {
    render(
      <Leaderboard
        contributors={mockContributors}
        negativeImpact={[]}
        ablationCoverage={mockAblationCoverage}
      />
    );

    expect(screen.getByText(/ablation coverage/i)).toBeInTheDocument();
  });

  it('shows empty state when no contributors', () => {
    render(
      <Leaderboard
        contributors={[]}
        negativeImpact={[]}
        ablationCoverage={mockAblationCoverage}
      />
    );

    expect(screen.getByText(/no attribution data/i)).toBeInTheDocument();
  });
});
