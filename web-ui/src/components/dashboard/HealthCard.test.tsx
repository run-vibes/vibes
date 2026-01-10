/**
 * Tests for the HealthCard component
 */
import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';

// Mock TanStack Router Link component
vi.mock('@tanstack/react-router', () => ({
  Link: ({ to, children }: { to: string; children: React.ReactNode }) => (
    <a href={to}>{children}</a>
  ),
}));

import { HealthCard } from './HealthCard';
import type { HealthSummary } from '../../hooks/useDashboard';

describe('HealthCard', () => {
  it('renders title', () => {
    render(<HealthCard />);
    expect(screen.getByText('Health')).toBeInTheDocument();
  });

  it('shows OK status with indicator', () => {
    const data: HealthSummary = {
      overall_status: 'ok',
      assessment_coverage: 85,
      ablation_coverage: 70,
    };
    render(<HealthCard data={data} />);

    expect(screen.getByText('OK')).toBeInTheDocument();
    expect(screen.getByTestId('status-indicator')).toBeInTheDocument();
  });

  it('shows degraded status with indicator', () => {
    const data: HealthSummary = {
      overall_status: 'degraded',
      assessment_coverage: 50,
      ablation_coverage: 30,
    };
    render(<HealthCard data={data} />);

    expect(screen.getByText('Degraded')).toBeInTheDocument();
    expect(screen.getByTestId('status-indicator')).toBeInTheDocument();
  });

  it('shows error status with indicator', () => {
    const data: HealthSummary = {
      overall_status: 'error',
      assessment_coverage: 0,
      ablation_coverage: 0,
    };
    render(<HealthCard data={data} />);

    expect(screen.getByText('Error')).toBeInTheDocument();
    expect(screen.getByTestId('status-indicator')).toBeInTheDocument();
  });

  it('renders coverage metrics', () => {
    const data: HealthSummary = {
      overall_status: 'ok',
      assessment_coverage: 85,
      ablation_coverage: 70,
    };
    render(<HealthCard data={data} />);

    expect(screen.getByText('Assessment')).toBeInTheDocument();
    expect(screen.getByText('85%')).toBeInTheDocument();
    expect(screen.getByText('Ablation')).toBeInTheDocument();
    expect(screen.getByText('70%')).toBeInTheDocument();
  });

  it('shows last activity when provided', () => {
    const data: HealthSummary = {
      overall_status: 'ok',
      assessment_coverage: 85,
      ablation_coverage: 70,
      last_activity: '2024-01-15T10:30:00Z',
    };
    render(<HealthCard data={data} />);

    // Should show relative time
    expect(screen.getByText(/Last activity:/)).toBeInTheDocument();
  });

  it('shows empty state when no data', () => {
    render(<HealthCard />);
    expect(screen.getByText('No health data')).toBeInTheDocument();
  });

  it('renders link to health page', () => {
    const data: HealthSummary = {
      overall_status: 'ok',
      assessment_coverage: 85,
      ablation_coverage: 70,
    };
    render(<HealthCard data={data} />);

    expect(screen.getByText('View →')).toBeInTheDocument();
    expect(screen.getByRole('link')).toHaveAttribute('href', '/groove/dashboard/health');
  });
});
