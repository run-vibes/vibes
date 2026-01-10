/**
 * Tests for SystemStatusBanner and SubsystemCard components
 */
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { SystemStatusBanner } from './SystemStatusBanner';
import { SubsystemCard } from './SubsystemCard';
import type { ComponentHealth } from '../../../hooks/useDashboard';

describe('SystemStatusBanner', () => {
  it('renders operational status', () => {
    render(<SystemStatusBanner status="ok" />);

    expect(screen.getByText(/operational/i)).toBeInTheDocument();
  });

  it('renders degraded status', () => {
    render(<SystemStatusBanner status="degraded" />);

    expect(screen.getByText(/degraded/i)).toBeInTheDocument();
  });

  it('renders error status', () => {
    render(<SystemStatusBanner status="error" />);

    expect(screen.getByText(/error/i)).toBeInTheDocument();
  });

  it('shows last check timestamp when provided', () => {
    render(<SystemStatusBanner status="ok" lastCheck="2026-01-09T14:30:00Z" />);

    expect(screen.getByText(/last check/i)).toBeInTheDocument();
  });

  it('applies correct class for ok status', () => {
    const { container } = render(<SystemStatusBanner status="ok" />);

    expect(container.firstChild).toHaveClass('status-banner--ok');
  });

  it('applies correct class for degraded status', () => {
    const { container } = render(<SystemStatusBanner status="degraded" />);

    expect(container.firstChild).toHaveClass('status-banner--degraded');
  });

  it('applies correct class for error status', () => {
    const { container } = render(<SystemStatusBanner status="error" />);

    expect(container.firstChild).toHaveClass('status-banner--error');
  });
});

describe('SubsystemCard', () => {
  const mockHealthOk: ComponentHealth = {
    status: 'ok',
    coverage: 0.82,
    last_activity: '2026-01-09T14:30:00Z',
    item_count: 47,
  };

  const mockHealthDegraded: ComponentHealth = {
    status: 'degraded',
    coverage: 0.45,
    last_activity: '2026-01-09T14:00:00Z',
  };

  const mockHealthError: ComponentHealth = {
    status: 'error',
    coverage: 0,
  };

  it('renders subsystem name', () => {
    render(<SubsystemCard name="Assessment" health={mockHealthOk} />);

    expect(screen.getByText('Assessment')).toBeInTheDocument();
  });

  it('shows status indicator for ok', () => {
    render(<SubsystemCard name="Assessment" health={mockHealthOk} />);

    expect(screen.getByText('●')).toBeInTheDocument();
  });

  it('shows status indicator for degraded', () => {
    render(<SubsystemCard name="Extraction" health={mockHealthDegraded} />);

    expect(screen.getByText('◐')).toBeInTheDocument();
  });

  it('shows status indicator for error', () => {
    render(<SubsystemCard name="Attribution" health={mockHealthError} />);

    expect(screen.getByText('○')).toBeInTheDocument();
  });

  it('displays coverage percentage', () => {
    render(<SubsystemCard name="Assessment" health={mockHealthOk} />);

    expect(screen.getByText('82%')).toBeInTheDocument();
  });

  it('displays item count when provided', () => {
    render(<SubsystemCard name="Extraction" health={mockHealthOk} />);

    expect(screen.getByText(/47/)).toBeInTheDocument();
  });

  it('displays last activity when provided', () => {
    render(<SubsystemCard name="Assessment" health={mockHealthOk} />);

    expect(screen.getByText(/last activity/i)).toBeInTheDocument();
  });

  it('shows warning message when provided', () => {
    render(
      <SubsystemCard
        name="Attribution"
        health={mockHealthDegraded}
        warning="Processing delayed"
      />
    );

    expect(screen.getByText('Processing delayed')).toBeInTheDocument();
  });
});
