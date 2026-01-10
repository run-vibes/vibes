/**
 * Tests for LearningActions component
 */
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { LearningActions } from './LearningActions';
import type { LearningStatus } from '../../../hooks/useDashboard';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch;

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });
  return function Wrapper({ children }: { children: React.ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );
  };
}

describe('LearningActions', () => {
  const defaultProps = {
    learningId: 'learning-123',
    status: 'active' as LearningStatus,
  };

  beforeEach(() => {
    mockFetch.mockReset();
  });

  describe('button visibility', () => {
    it('shows disable button when status is active', () => {
      render(<LearningActions {...defaultProps} status="active" />, {
        wrapper: createWrapper(),
      });

      expect(screen.getByRole('button', { name: /disable/i })).toBeInTheDocument();
      expect(screen.queryByRole('button', { name: /enable/i })).not.toBeInTheDocument();
    });

    it('shows enable button when status is disabled', () => {
      render(<LearningActions {...defaultProps} status="disabled" />, {
        wrapper: createWrapper(),
      });

      expect(screen.getByRole('button', { name: /enable/i })).toBeInTheDocument();
      expect(screen.queryByRole('button', { name: /disable/i })).not.toBeInTheDocument();
    });

    it('shows enable button when status is deprecated', () => {
      render(<LearningActions {...defaultProps} status="deprecated" />, {
        wrapper: createWrapper(),
      });

      expect(screen.getByRole('button', { name: /enable/i })).toBeInTheDocument();
      expect(screen.queryByRole('button', { name: /disable/i })).not.toBeInTheDocument();
    });

    it('shows enable button when status is under_review', () => {
      render(<LearningActions {...defaultProps} status="under_review" />, {
        wrapper: createWrapper(),
      });

      expect(screen.getByRole('button', { name: /enable/i })).toBeInTheDocument();
      expect(screen.queryByRole('button', { name: /disable/i })).not.toBeInTheDocument();
    });

    it('always shows delete button', () => {
      render(<LearningActions {...defaultProps} />, {
        wrapper: createWrapper(),
      });

      expect(screen.getByRole('button', { name: /delete/i })).toBeInTheDocument();
    });
  });

  describe('disable action', () => {
    it('opens confirmation dialog when disable clicked', () => {
      render(<LearningActions {...defaultProps} status="active" />, {
        wrapper: createWrapper(),
      });

      fireEvent.click(screen.getByRole('button', { name: /disable/i }));

      expect(screen.getByRole('dialog')).toBeInTheDocument();
      expect(screen.getByText(/won't be injected/i)).toBeInTheDocument();
    });

    it('calls API when disable confirmed', async () => {
      mockFetch.mockResolvedValueOnce({ ok: true });

      render(<LearningActions {...defaultProps} status="active" />, {
        wrapper: createWrapper(),
      });

      fireEvent.click(screen.getByRole('button', { name: /disable/i }));
      // The confirm button also says "Disable", so we get all and click the second one (in dialog)
      const disableButtons = screen.getAllByRole('button', { name: /disable/i });
      fireEvent.click(disableButtons[disableButtons.length - 1]);

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledWith(
          '/api/groove/learnings/learning-123/disable',
          expect.objectContaining({ method: 'POST' })
        );
      });
    });
  });

  describe('enable action', () => {
    it('calls API directly when enable clicked (no confirmation)', async () => {
      mockFetch.mockResolvedValueOnce({ ok: true });

      render(<LearningActions {...defaultProps} status="disabled" />, {
        wrapper: createWrapper(),
      });

      fireEvent.click(screen.getByRole('button', { name: /enable/i }));

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledWith(
          '/api/groove/learnings/learning-123/enable',
          expect.objectContaining({ method: 'POST' })
        );
      });
    });
  });

  describe('delete action', () => {
    it('opens confirmation dialog when delete clicked', () => {
      render(<LearningActions {...defaultProps} />, {
        wrapper: createWrapper(),
      });

      fireEvent.click(screen.getByRole('button', { name: /delete/i }));

      expect(screen.getByRole('dialog')).toBeInTheDocument();
      expect(screen.getByText(/permanently remove/i)).toBeInTheDocument();
    });

    it('calls API when delete confirmed', async () => {
      mockFetch.mockResolvedValueOnce({ ok: true });

      render(<LearningActions {...defaultProps} />, {
        wrapper: createWrapper(),
      });

      fireEvent.click(screen.getByRole('button', { name: /delete/i }));
      // The confirm button also says "Delete", so we get all and click the second one (in dialog)
      const deleteButtons = screen.getAllByRole('button', { name: /delete/i });
      fireEvent.click(deleteButtons[deleteButtons.length - 1]);

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledWith(
          '/api/groove/learnings/learning-123',
          expect.objectContaining({ method: 'DELETE' })
        );
      });
    });
  });

  describe('confirmation dialog', () => {
    it('closes dialog when cancel clicked', async () => {
      render(<LearningActions {...defaultProps} status="active" />, {
        wrapper: createWrapper(),
      });

      fireEvent.click(screen.getByRole('button', { name: /disable/i }));
      expect(screen.getByRole('dialog')).toBeInTheDocument();

      fireEvent.click(screen.getByRole('button', { name: /cancel/i }));

      await waitFor(() => {
        expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
      });
    });
  });

  describe('callbacks', () => {
    it('calls onActionComplete when action succeeds', async () => {
      mockFetch.mockResolvedValueOnce({ ok: true });
      const onActionComplete = vi.fn();

      render(
        <LearningActions
          {...defaultProps}
          status="disabled"
          onActionComplete={onActionComplete}
        />,
        { wrapper: createWrapper() }
      );

      fireEvent.click(screen.getByRole('button', { name: /enable/i }));

      await waitFor(() => {
        expect(onActionComplete).toHaveBeenCalled();
      });
    });

    it('calls onError when action fails', async () => {
      mockFetch.mockResolvedValueOnce({ ok: false });
      const onError = vi.fn();

      render(
        <LearningActions
          {...defaultProps}
          status="disabled"
          onError={onError}
        />,
        { wrapper: createWrapper() }
      );

      fireEvent.click(screen.getByRole('button', { name: /enable/i }));

      await waitFor(() => {
        expect(onError).toHaveBeenCalledWith(expect.any(Error));
      });
    });
  });
});
