/**
 * Tests for GapDetail component
 */
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';

import { GapDetail } from './GapDetail';
import type { OpenWorldGapDetailData } from '../../../hooks/useDashboard';

const mockGapData: OpenWorldGapDetailData = {
  data_type: 'open_world_gap_detail',
  id: 'gap-001-abcd-efgh',
  category: 'MissingKnowledge',
  severity: 'High',
  status: 'Detected',
  context_pattern: 'User asks about advanced TypeScript features like conditional types',
  failure_count: 5,
  first_seen: new Date(Date.now() - 1000 * 60 * 60 * 24).toISOString(),
  last_seen: new Date(Date.now() - 1000 * 60 * 30).toISOString(),
  suggested_solutions: [
    {
      action_type: 'AddKnowledge',
      description: 'Add documentation about TypeScript conditional types',
      confidence: 0.85,
      applied: false,
    },
    {
      action_type: 'UpdatePattern',
      description: 'Improve pattern matching for TypeScript queries',
      confidence: 0.72,
      applied: true,
    },
  ],
};

describe('GapDetail', () => {
  it('shows loading state', () => {
    render(<GapDetail isLoading />);
    expect(screen.getByText('Loading...')).toBeInTheDocument();
  });

  it('shows empty state when no data', () => {
    render(<GapDetail />);
    expect(screen.getByText('Select a gap to view details')).toBeInTheDocument();
  });

  it('renders gap ID', () => {
    render(<GapDetail data={mockGapData} />);
    expect(screen.getByText('gap-001-')).toBeInTheDocument();
  });

  it('renders severity badge', () => {
    render(<GapDetail data={mockGapData} />);
    expect(screen.getByTestId('severity-high')).toBeInTheDocument();
  });

  it('renders category', () => {
    render(<GapDetail data={mockGapData} />);
    expect(screen.getByText('Missing Knowledge')).toBeInTheDocument();
  });

  it('renders status', () => {
    render(<GapDetail data={mockGapData} />);
    expect(screen.getByText('Detected')).toBeInTheDocument();
  });

  it('renders context pattern', () => {
    render(<GapDetail data={mockGapData} />);
    expect(
      screen.getByText('User asks about advanced TypeScript features like conditional types')
    ).toBeInTheDocument();
  });

  it('renders failure count metric', () => {
    render(<GapDetail data={mockGapData} />);
    expect(screen.getByText('Failures')).toBeInTheDocument();
    expect(screen.getByText('5')).toBeInTheDocument();
  });

  it('renders solutions count metric', () => {
    render(<GapDetail data={mockGapData} />);
    expect(screen.getByText('Solutions')).toBeInTheDocument();
    expect(screen.getByText('2')).toBeInTheDocument();
  });

  it('renders suggested solutions', () => {
    render(<GapDetail data={mockGapData} />);
    expect(screen.getByText('Suggested Solutions')).toBeInTheDocument();
    expect(
      screen.getByText('Add documentation about TypeScript conditional types')
    ).toBeInTheDocument();
    expect(
      screen.getByText('Improve pattern matching for TypeScript queries')
    ).toBeInTheDocument();
  });

  it('renders solution types', () => {
    render(<GapDetail data={mockGapData} />);
    expect(screen.getByText('AddKnowledge')).toBeInTheDocument();
    expect(screen.getByText('UpdatePattern')).toBeInTheDocument();
  });

  it('renders solution confidence percentages', () => {
    render(<GapDetail data={mockGapData} />);
    expect(screen.getByText('85%')).toBeInTheDocument();
    expect(screen.getByText('72%')).toBeInTheDocument();
  });

  it('shows applied badge for applied solutions', () => {
    render(<GapDetail data={mockGapData} />);
    expect(screen.getByText('Applied')).toBeInTheDocument();
  });

  it('hides solutions section when no solutions', () => {
    const dataWithNoSolutions = { ...mockGapData, suggested_solutions: [] };
    render(<GapDetail data={dataWithNoSolutions} />);
    expect(screen.queryByText('Suggested Solutions')).not.toBeInTheDocument();
  });

  it('renders time ago for first seen', () => {
    render(<GapDetail data={mockGapData} />);
    expect(screen.getByText('First Seen')).toBeInTheDocument();
    // Time format depends on current time, just check the label exists
  });

  it('renders time ago for last seen', () => {
    render(<GapDetail data={mockGapData} />);
    expect(screen.getByText('Last Seen')).toBeInTheDocument();
  });
});
