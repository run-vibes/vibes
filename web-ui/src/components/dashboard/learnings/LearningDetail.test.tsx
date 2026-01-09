/**
 * Tests for LearningDetail component
 */
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { LearningDetail } from './LearningDetail';
import type { LearningDetailData } from '../../../hooks/useDashboard';

const mockLearning: LearningDetailData = {
  data_type: 'learning_detail',
  id: 'test-123',
  content: 'Always use semantic HTML elements for accessibility',
  category: 'Pattern',
  scope: { Project: 'vibes' },
  status: 'active',
  estimated_value: 0.75,
  confidence: 0.85,
  times_injected: 42,
  activation_rate: 0.68,
  session_count: 15,
  created_at: '2024-01-15T10:30:00Z',
  source_session: 'session-abc',
  extraction_method: 'explicit_instruction',
};

describe('LearningDetail', () => {
  it('renders learning content', () => {
    render(<LearningDetail data={mockLearning} />);

    expect(screen.getByText(mockLearning.content)).toBeInTheDocument();
  });

  it('shows category badge', () => {
    render(<LearningDetail data={mockLearning} />);

    expect(screen.getByText('Pattern')).toBeInTheDocument();
  });

  it('shows status badge', () => {
    render(<LearningDetail data={mockLearning} />);

    expect(screen.getByText('Active')).toBeInTheDocument();
  });

  it('displays metrics', () => {
    render(<LearningDetail data={mockLearning} />);

    expect(screen.getByText('Value')).toBeInTheDocument();
    expect(screen.getByText('+0.75')).toBeInTheDocument();
    expect(screen.getByText('Confidence')).toBeInTheDocument();
    expect(screen.getByText('85%')).toBeInTheDocument();
    expect(screen.getByText('Sessions')).toBeInTheDocument();
    expect(screen.getByText('15')).toBeInTheDocument();
  });

  it('shows injection stats', () => {
    render(<LearningDetail data={mockLearning} />);

    expect(screen.getByText('Times Injected')).toBeInTheDocument();
    expect(screen.getByText('42')).toBeInTheDocument();
    expect(screen.getByText('Activation Rate')).toBeInTheDocument();
    expect(screen.getByText('68%')).toBeInTheDocument();
  });

  it('shows scope information', () => {
    render(<LearningDetail data={mockLearning} />);

    expect(screen.getByText('Scope')).toBeInTheDocument();
    expect(screen.getByText('Project: vibes')).toBeInTheDocument();
  });

  it('shows source information', () => {
    render(<LearningDetail data={mockLearning} />);

    expect(screen.getByText('Source')).toBeInTheDocument();
    expect(screen.getByText('explicit_instruction')).toBeInTheDocument();
  });

  it('shows created date', () => {
    render(<LearningDetail data={mockLearning} />);

    expect(screen.getByText('Created')).toBeInTheDocument();
    // Date formatting may vary, just check it's present
    expect(screen.getByText(/2024/)).toBeInTheDocument();
  });

  it('shows empty state when no data', () => {
    render(<LearningDetail />);

    expect(screen.getByText('Select a learning to view details')).toBeInTheDocument();
  });

  it('shows loading state', () => {
    render(<LearningDetail isLoading />);

    expect(screen.getByText('Loading...')).toBeInTheDocument();
  });
});
