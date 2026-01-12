/**
 * Tests for DashboardOpenWorld page
 */
import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { DashboardOpenWorld } from './DashboardOpenWorld';

describe('DashboardOpenWorld', () => {
  describe('tab rendering', () => {
    it('renders all four tabs', () => {
      render(<DashboardOpenWorld />);

      expect(screen.getByRole('button', { name: /novelty/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /gaps/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /solutions/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /activity/i })).toBeInTheDocument();
    });

    it('shows Novelty tab as active by default', () => {
      render(<DashboardOpenWorld />);

      const noveltyTab = screen.getByRole('button', { name: /novelty/i });
      expect(noveltyTab).toHaveClass('active');
    });
  });

  describe('tab switching', () => {
    it('switches to Gaps tab when clicked', () => {
      render(<DashboardOpenWorld />);

      const gapsTab = screen.getByRole('button', { name: /gaps/i });
      fireEvent.click(gapsTab);

      expect(gapsTab).toHaveClass('active');
      expect(screen.getByRole('heading', { name: /capability gaps/i })).toBeInTheDocument();
    });

    it('switches to Solutions tab when clicked', () => {
      render(<DashboardOpenWorld />);

      const solutionsTab = screen.getByRole('button', { name: /solutions/i });
      fireEvent.click(solutionsTab);

      expect(solutionsTab).toHaveClass('active');
      expect(screen.getByRole('heading', { name: /suggested solutions/i })).toBeInTheDocument();
    });

    it('switches to Activity tab when clicked', () => {
      render(<DashboardOpenWorld />);

      const activityTab = screen.getByRole('button', { name: /activity/i });
      fireEvent.click(activityTab);

      expect(activityTab).toHaveClass('active');
      expect(screen.getByRole('heading', { name: /response activity/i })).toBeInTheDocument();
    });

    it('switches back to Novelty tab', () => {
      render(<DashboardOpenWorld />);

      // Switch away first
      fireEvent.click(screen.getByRole('button', { name: /gaps/i }));

      // Switch back
      const noveltyTab = screen.getByRole('button', { name: /novelty/i });
      fireEvent.click(noveltyTab);

      expect(noveltyTab).toHaveClass('active');
      expect(screen.getByRole('heading', { name: /novelty detection/i })).toBeInTheDocument();
    });
  });

  describe('Novelty tab content', () => {
    it('shows stats cards for threshold, pending, and clusters', () => {
      render(<DashboardOpenWorld />);

      expect(screen.getByText('Threshold')).toBeInTheDocument();
      expect(screen.getByText('Pending')).toBeInTheDocument();
      expect(screen.getByText('Clusters')).toBeInTheDocument();
    });

    it('shows placeholder values', () => {
      render(<DashboardOpenWorld />);

      expect(screen.getByText('0.85')).toBeInTheDocument();
    });
  });

  describe('Gaps tab content', () => {
    it('shows empty state when no gaps', () => {
      render(<DashboardOpenWorld />);
      fireEvent.click(screen.getByRole('button', { name: /gaps/i }));

      expect(screen.getByText(/no capability gaps detected/i)).toBeInTheDocument();
    });
  });

  describe('Solutions tab content', () => {
    it('shows empty state when no solutions', () => {
      render(<DashboardOpenWorld />);
      fireEvent.click(screen.getByRole('button', { name: /solutions/i }));

      expect(screen.getByText(/no solutions pending review/i)).toBeInTheDocument();
    });
  });

  describe('Activity tab content', () => {
    it('shows stats cards for outcomes, negative rate, and exploration', () => {
      render(<DashboardOpenWorld />);
      fireEvent.click(screen.getByRole('button', { name: /activity/i }));

      expect(screen.getByText('Outcomes')).toBeInTheDocument();
      expect(screen.getByText('Negative')).toBeInTheDocument();
      expect(screen.getByText('Exploration')).toBeInTheDocument();
    });

    it('shows empty state for activity feed', () => {
      render(<DashboardOpenWorld />);
      fireEvent.click(screen.getByRole('button', { name: /activity/i }));

      expect(screen.getByText(/no recent activity/i)).toBeInTheDocument();
    });
  });
});
