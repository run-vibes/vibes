/**
 * Tests for SessionTimeline and SessionTimelineItem components
 */
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { SessionTimelineItem } from './SessionTimelineItem';
import { SessionTimeline } from './SessionTimeline';
import type { SessionTimelineEntry, ActivatedLearning } from '../../../hooks/useDashboard';

const mockActivatedLearning: ActivatedLearning = {
  learning_id: 'learn-123',
  content: 'Use semantic HTML',
  contribution: 0.15,
};

// Use relative dates for "Today" and "Yesterday" tests
const today = new Date();
const yesterday = new Date(today);
yesterday.setDate(yesterday.getDate() - 1);

const mockSession: SessionTimelineEntry = {
  session_id: 'session-abc',
  timestamp: today.toISOString(),
  score: 0.82,
  activated_learnings: [
    mockActivatedLearning,
    { learning_id: 'learn-456', content: 'Prefer async/await', contribution: 0.08 },
  ],
  outcome: 'positive',
};

const mockNegativeSession: SessionTimelineEntry = {
  session_id: 'session-def',
  timestamp: yesterday.toISOString(),
  score: 0.45,
  activated_learnings: [
    { learning_id: 'learn-789', content: 'Avoid console.log', contribution: -0.12 },
  ],
  outcome: 'negative',
};

describe('SessionTimelineItem', () => {
  it('renders session id', () => {
    render(<SessionTimelineItem session={mockSession} />);

    expect(screen.getByText(/session-abc/i)).toBeInTheDocument();
  });

  it('shows session score', () => {
    render(<SessionTimelineItem session={mockSession} />);

    expect(screen.getByText('0.82')).toBeInTheDocument();
  });

  it('shows score progress bar', () => {
    render(<SessionTimelineItem session={mockSession} />);

    expect(screen.getByRole('progressbar')).toBeInTheDocument();
  });

  it('shows activated learnings count', () => {
    render(<SessionTimelineItem session={mockSession} />);

    expect(screen.getByText(/2 learnings/i)).toBeInTheDocument();
  });

  it('expands to show learning details on click', () => {
    render(<SessionTimelineItem session={mockSession} />);

    fireEvent.click(screen.getByRole('button'));

    expect(screen.getByText('Use semantic HTML')).toBeInTheDocument();
    expect(screen.getByText('Prefer async/await')).toBeInTheDocument();
  });

  it('shows learning contribution values when expanded', () => {
    render(<SessionTimelineItem session={mockSession} />);

    fireEvent.click(screen.getByRole('button'));

    expect(screen.getByText('+0.15')).toBeInTheDocument();
    expect(screen.getByText('+0.08')).toBeInTheDocument();
  });

  it('indicates negative outcome sessions', () => {
    render(<SessionTimelineItem session={mockNegativeSession} />);

    expect(screen.getByText(/negative/i)).toBeInTheDocument();
  });

  it('calls onClick when session clicked', () => {
    const onClick = vi.fn();
    render(<SessionTimelineItem session={mockSession} onClick={onClick} />);

    fireEvent.click(screen.getByRole('button'));

    expect(onClick).toHaveBeenCalledWith('session-abc');
  });
});

describe('SessionTimeline', () => {
  const mockSessions = [mockSession, mockNegativeSession];

  it('renders timeline container', () => {
    render(<SessionTimeline sessions={mockSessions} />);

    expect(screen.getByRole('list')).toBeInTheDocument();
  });

  it('groups sessions by day', () => {
    render(<SessionTimeline sessions={mockSessions} />);

    expect(screen.getByText('Today')).toBeInTheDocument();
    expect(screen.getByText('Yesterday')).toBeInTheDocument();
  });

  it('renders all sessions', () => {
    render(<SessionTimeline sessions={mockSessions} />);

    expect(screen.getByText(/session-abc/i)).toBeInTheDocument();
    expect(screen.getByText(/session-def/i)).toBeInTheDocument();
  });

  it('shows period selector', () => {
    render(<SessionTimeline sessions={mockSessions} />);

    expect(screen.getByRole('combobox', { name: /period/i })).toBeInTheDocument();
  });

  it('calls onPeriodChange when period changes', () => {
    const onPeriodChange = vi.fn();
    render(<SessionTimeline sessions={mockSessions} onPeriodChange={onPeriodChange} />);

    fireEvent.change(screen.getByRole('combobox', { name: /period/i }), {
      target: { value: '30' },
    });

    expect(onPeriodChange).toHaveBeenCalledWith(30);
  });

  it('shows outcome filter', () => {
    render(<SessionTimeline sessions={mockSessions} />);

    expect(screen.getByRole('combobox', { name: /outcome/i })).toBeInTheDocument();
  });

  it('filters by outcome when filter changes', () => {
    const onOutcomeFilter = vi.fn();
    render(<SessionTimeline sessions={mockSessions} onOutcomeFilter={onOutcomeFilter} />);

    fireEvent.change(screen.getByRole('combobox', { name: /outcome/i }), {
      target: { value: 'negative' },
    });

    expect(onOutcomeFilter).toHaveBeenCalledWith('negative');
  });

  it('shows empty state when no sessions', () => {
    render(<SessionTimeline sessions={[]} />);

    expect(screen.getByText(/no sessions/i)).toBeInTheDocument();
  });
});
