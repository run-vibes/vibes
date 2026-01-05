import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { SessionCard } from './SessionCard';

describe('SessionCard', () => {
  const defaultProps = {
    id: 'sess-abc123',
    status: 'processing' as const,
    updatedAt: new Date('2024-01-01T12:00:00Z'),
  };

  it('renders session id', () => {
    render(<SessionCard {...defaultProps} />);
    expect(screen.getByText('sess-abc123')).toBeInTheDocument();
  });

  it('renders session name when provided', () => {
    render(<SessionCard {...defaultProps} name="auth-refactor" />);
    expect(screen.getByText('auth-refactor')).toBeInTheDocument();
  });

  it('renders status badge', () => {
    render(<SessionCard {...defaultProps} status="processing" />);
    expect(screen.getByText('processing')).toBeInTheDocument();
  });

  it('renders different statuses correctly', () => {
    const { rerender } = render(<SessionCard {...defaultProps} status="idle" />);
    expect(screen.getByText('idle')).toBeInTheDocument();

    rerender(<SessionCard {...defaultProps} status="waiting" />);
    expect(screen.getByText('waiting')).toBeInTheDocument();

    rerender(<SessionCard {...defaultProps} status="finished" />);
    expect(screen.getByText('finished')).toBeInTheDocument();

    rerender(<SessionCard {...defaultProps} status="failed" />);
    expect(screen.getByText('failed')).toBeInTheDocument();
  });

  it('renders subscriber count', () => {
    render(<SessionCard {...defaultProps} subscribers={3} />);
    expect(screen.getByText('3')).toBeInTheDocument();
  });

  it('defaults to 0 subscribers', () => {
    render(<SessionCard {...defaultProps} />);
    expect(screen.getByText('0')).toBeInTheDocument();
  });

  it('calls onClick when clicked', () => {
    const onClick = vi.fn();
    render(<SessionCard {...defaultProps} onClick={onClick} />);
    fireEvent.click(screen.getByRole('article'));
    expect(onClick).toHaveBeenCalled();
  });

  it('renders as article element', () => {
    render(<SessionCard {...defaultProps} />);
    expect(screen.getByRole('article')).toBeInTheDocument();
  });

  it('merges custom className', () => {
    render(<SessionCard {...defaultProps} className="custom-class" />);
    expect(screen.getByRole('article')).toHaveClass('custom-class');
  });

  it('passes through additional props', () => {
    render(<SessionCard {...defaultProps} data-testid="custom-card" aria-label="Session" />);
    expect(screen.getByTestId('custom-card')).toBeInTheDocument();
    expect(screen.getByTestId('custom-card')).toHaveAttribute('aria-label', 'Session');
  });

  describe('active state styling', () => {
    it('applies active class for processing status', () => {
      render(<SessionCard {...defaultProps} status="processing" />);
      const article = screen.getByRole('article');
      expect(article.className).toContain('active');
    });

    it('applies active class for waiting status', () => {
      render(<SessionCard {...defaultProps} status="waiting" />);
      const article = screen.getByRole('article');
      expect(article.className).toContain('active');
    });

    it('applies inactive class for idle status', () => {
      render(<SessionCard {...defaultProps} status="idle" />);
      const article = screen.getByRole('article');
      expect(article.className).toContain('inactive');
    });

    it('applies inactive class for finished status', () => {
      render(<SessionCard {...defaultProps} status="finished" />);
      const article = screen.getByRole('article');
      expect(article.className).toContain('inactive');
    });
  });

  describe('duration and event count', () => {
    it('renders duration when provided', () => {
      render(<SessionCard {...defaultProps} duration={3600} />);
      expect(screen.getByText('1h')).toBeInTheDocument();
    });

    it('formats duration in minutes', () => {
      render(<SessionCard {...defaultProps} duration={300} />);
      expect(screen.getByText('5m')).toBeInTheDocument();
    });

    it('formats duration in hours and minutes', () => {
      render(<SessionCard {...defaultProps} duration={5400} />);
      expect(screen.getByText('1h 30m')).toBeInTheDocument();
    });

    it('renders event count when provided', () => {
      render(<SessionCard {...defaultProps} eventCount={42} />);
      expect(screen.getByText('42 events')).toBeInTheDocument();
    });

    it('renders both duration and event count', () => {
      render(<SessionCard {...defaultProps} duration={600} eventCount={15} />);
      expect(screen.getByText('10m')).toBeInTheDocument();
      expect(screen.getByText('15 events')).toBeInTheDocument();
    });
  });

  describe('quick actions', () => {
    it('renders action buttons', () => {
      const actions = [
        { icon: 'X', label: 'Close', onClick: vi.fn() },
        { icon: 'R', label: 'Refresh', onClick: vi.fn() },
      ];
      render(<SessionCard {...defaultProps} actions={actions} />);
      expect(screen.getByRole('button', { name: 'Close' })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: 'Refresh' })).toBeInTheDocument();
    });

    it('calls action onClick when clicked', () => {
      const onClick = vi.fn();
      const actions = [{ icon: 'X', label: 'Close', onClick }];
      render(<SessionCard {...defaultProps} actions={actions} />);
      fireEvent.click(screen.getByRole('button', { name: 'Close' }));
      expect(onClick).toHaveBeenCalled();
    });

    it('stops propagation when action is clicked', () => {
      const cardClick = vi.fn();
      const actionClick = vi.fn();
      const actions = [{ icon: 'X', label: 'Close', onClick: actionClick }];
      render(<SessionCard {...defaultProps} actions={actions} onClick={cardClick} />);
      fireEvent.click(screen.getByRole('button', { name: 'Close' }));
      expect(actionClick).toHaveBeenCalled();
      expect(cardClick).not.toHaveBeenCalled();
    });
  });
});
