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
});
