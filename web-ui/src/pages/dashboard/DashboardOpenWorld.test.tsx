/**
 * Tests for DashboardOpenWorld page
 */
import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { DashboardOpenWorld } from './DashboardOpenWorld';

// Create a fresh QueryClient for each test
function createTestQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });
}

function renderWithQueryClient(ui: React.ReactElement) {
  const queryClient = createTestQueryClient();
  return render(
    <QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>
  );
}

describe('DashboardOpenWorld', () => {
  describe('tab rendering', () => {
    it('renders all four tabs', () => {
      renderWithQueryClient(<DashboardOpenWorld />);

      expect(screen.getByRole('button', { name: /novelty/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /gaps/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /solutions/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /activity/i })).toBeInTheDocument();
    });

    it('shows Novelty tab as active by default', () => {
      renderWithQueryClient(<DashboardOpenWorld />);

      const noveltyTab = screen.getByRole('button', { name: /novelty/i });
      expect(noveltyTab).toHaveClass('active');
    });
  });

  describe('tab switching', () => {
    it('switches to Gaps tab when clicked', () => {
      renderWithQueryClient(<DashboardOpenWorld />);

      const gapsTab = screen.getByRole('button', { name: /gaps/i });
      fireEvent.click(gapsTab);

      expect(gapsTab).toHaveClass('active');
      // Gaps tab shows split panel with filters and detail panel
      expect(screen.getByLabelText('Severity')).toBeInTheDocument();
      expect(screen.getByText('Gap Detail')).toBeInTheDocument();
    });

    it('switches to Solutions tab when clicked', () => {
      renderWithQueryClient(<DashboardOpenWorld />);

      const solutionsTab = screen.getByRole('button', { name: /solutions/i });
      fireEvent.click(solutionsTab);

      expect(solutionsTab).toHaveClass('active');
      expect(screen.getByRole('heading', { name: /suggested solutions/i })).toBeInTheDocument();
    });

    it('switches to Activity tab when clicked', () => {
      renderWithQueryClient(<DashboardOpenWorld />);

      const activityTab = screen.getByRole('button', { name: /activity/i });
      fireEvent.click(activityTab);

      expect(activityTab).toHaveClass('active');
      expect(screen.getByRole('heading', { name: /response activity/i })).toBeInTheDocument();
    });

    it('switches back to Novelty tab', () => {
      renderWithQueryClient(<DashboardOpenWorld />);

      // Switch away first
      fireEvent.click(screen.getByRole('button', { name: /gaps/i }));

      // Switch back
      const noveltyTab = screen.getByRole('button', { name: /novelty/i });
      fireEvent.click(noveltyTab);

      expect(noveltyTab).toHaveClass('active');
      // Should show the Novelty Detection panel
      expect(screen.getByText('Novelty Detection')).toBeInTheDocument();
    });
  });

  describe('Novelty tab content', () => {
    it('renders NoveltyStats component', () => {
      renderWithQueryClient(<DashboardOpenWorld />);

      // Should render the Novelty Detection panel (loading or with data)
      expect(screen.getByText('Novelty Detection')).toBeInTheDocument();
    });

    it('renders ClusterList component', () => {
      renderWithQueryClient(<DashboardOpenWorld />);

      // Should render the Recent Clusters panel (loading or with data)
      expect(screen.getByText('Recent Clusters')).toBeInTheDocument();
    });
  });

  describe('Gaps tab content', () => {
    it('shows loading state initially', () => {
      renderWithQueryClient(<DashboardOpenWorld />);
      fireEvent.click(screen.getByRole('button', { name: /gaps/i }));

      // Without backend, hooks stay in loading state
      expect(screen.getByText(/loading gaps/i)).toBeInTheDocument();
    });
  });

  describe('Solutions tab content', () => {
    it('shows loading state initially', () => {
      renderWithQueryClient(<DashboardOpenWorld />);
      fireEvent.click(screen.getByRole('button', { name: /solutions/i }));

      // Without backend, hooks stay in loading state
      expect(screen.getByText(/loading solutions/i)).toBeInTheDocument();
    });
  });

  describe('Activity tab content', () => {
    it('shows stats cards for outcomes, negative rate, and exploration', () => {
      renderWithQueryClient(<DashboardOpenWorld />);
      fireEvent.click(screen.getByRole('button', { name: /activity/i }));

      expect(screen.getByText('Outcomes')).toBeInTheDocument();
      expect(screen.getByText('Negative')).toBeInTheDocument();
      expect(screen.getByText('Exploration')).toBeInTheDocument();
    });

    it('shows empty state for activity feed', () => {
      renderWithQueryClient(<DashboardOpenWorld />);
      fireEvent.click(screen.getByRole('button', { name: /activity/i }));

      expect(screen.getByText(/no recent activity/i)).toBeInTheDocument();
    });
  });
});
