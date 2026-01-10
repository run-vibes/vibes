import { useState, useCallback } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { ConfirmDialog } from '../ConfirmDialog';
import type { LearningStatus, LearningsData, LearningDetailData } from '../../../hooks/useDashboard';
import './LearningActions.css';

export interface LearningActionsProps {
  learningId: string;
  status: LearningStatus;
  onActionComplete?: () => void;
  onError?: (error: Error) => void;
}

type ActionType = 'disable' | 'delete' | null;

interface MutationContext {
  previousLearnings?: LearningsData;
  previousDetail?: LearningDetailData;
}

export function LearningActions({
  learningId,
  status,
  onActionComplete,
  onError,
}: LearningActionsProps) {
  const [confirmAction, setConfirmAction] = useState<ActionType>(null);
  const queryClient = useQueryClient();

  const handleError = useCallback(
    (error: Error, context?: MutationContext) => {
      // Rollback on error
      if (context?.previousLearnings) {
        queryClient.setQueryData(['dashboard', 'learnings'], context.previousLearnings);
      }
      if (context?.previousDetail) {
        queryClient.setQueryData(['dashboard', 'learning', learningId], context.previousDetail);
      }
      // Notify parent
      onError?.(error);
      // Log for debugging
      console.error('Learning action failed:', error.message);
    },
    [queryClient, learningId, onError]
  );

  const invalidateQueries = useCallback(() => {
    queryClient.invalidateQueries({ queryKey: ['dashboard', 'learnings'] });
    queryClient.invalidateQueries({ queryKey: ['dashboard', 'learning', learningId] });
    onActionComplete?.();
  }, [queryClient, learningId, onActionComplete]);

  // Optimistically update learning status in cache
  const updateStatusOptimistically = useCallback(
    (newStatus: LearningStatus): MutationContext => {
      // Cancel any outgoing refetches
      queryClient.cancelQueries({ queryKey: ['dashboard', 'learnings'] });
      queryClient.cancelQueries({ queryKey: ['dashboard', 'learning', learningId] });

      // Snapshot previous values
      const previousLearnings = queryClient.getQueryData<LearningsData>(['dashboard', 'learnings']);
      const previousDetail = queryClient.getQueryData<LearningDetailData>([
        'dashboard',
        'learning',
        learningId,
      ]);

      // Optimistically update learnings list
      if (previousLearnings) {
        queryClient.setQueryData<LearningsData>(['dashboard', 'learnings'], {
          ...previousLearnings,
          learnings: previousLearnings.learnings.map((l) =>
            l.id === learningId ? { ...l, status: newStatus } : l
          ),
        });
      }

      // Optimistically update detail
      if (previousDetail) {
        queryClient.setQueryData<LearningDetailData>(['dashboard', 'learning', learningId], {
          ...previousDetail,
          status: newStatus,
        });
      }

      return { previousLearnings, previousDetail };
    },
    [queryClient, learningId]
  );

  // Remove learning optimistically from cache
  const removeOptimistically = useCallback((): MutationContext => {
    // Cancel any outgoing refetches
    queryClient.cancelQueries({ queryKey: ['dashboard', 'learnings'] });
    queryClient.cancelQueries({ queryKey: ['dashboard', 'learning', learningId] });

    // Snapshot previous values
    const previousLearnings = queryClient.getQueryData<LearningsData>(['dashboard', 'learnings']);
    const previousDetail = queryClient.getQueryData<LearningDetailData>([
      'dashboard',
      'learning',
      learningId,
    ]);

    // Optimistically remove from list
    if (previousLearnings) {
      queryClient.setQueryData<LearningsData>(['dashboard', 'learnings'], {
        ...previousLearnings,
        learnings: previousLearnings.learnings.filter((l) => l.id !== learningId),
        total: previousLearnings.total - 1,
      });
    }

    // Remove detail from cache
    queryClient.removeQueries({ queryKey: ['dashboard', 'learning', learningId] });

    return { previousLearnings, previousDetail };
  }, [queryClient, learningId]);

  const enableMutation = useMutation({
    mutationFn: async () => {
      const response = await fetch(`/api/groove/learnings/${learningId}/enable`, {
        method: 'POST',
      });
      if (!response.ok) {
        throw new Error('Failed to enable learning');
      }
    },
    onMutate: () => updateStatusOptimistically('active'),
    onError: (error: Error, _variables, context) => handleError(error, context),
    onSettled: invalidateQueries,
  });

  const disableMutation = useMutation({
    mutationFn: async () => {
      const response = await fetch(`/api/groove/learnings/${learningId}/disable`, {
        method: 'POST',
      });
      if (!response.ok) {
        throw new Error('Failed to disable learning');
      }
    },
    onMutate: () => updateStatusOptimistically('disabled'),
    onError: (error: Error, _variables, context) => handleError(error, context),
    onSuccess: () => setConfirmAction(null),
    onSettled: invalidateQueries,
  });

  const deleteMutation = useMutation({
    mutationFn: async () => {
      const response = await fetch(`/api/groove/learnings/${learningId}`, {
        method: 'DELETE',
      });
      if (!response.ok) {
        throw new Error('Failed to delete learning');
      }
    },
    onMutate: removeOptimistically,
    onError: (error: Error, _variables, context) => handleError(error, context),
    onSuccess: () => setConfirmAction(null),
    onSettled: invalidateQueries,
  });

  const handleEnable = () => {
    enableMutation.mutate();
  };

  const handleDisableClick = () => {
    setConfirmAction('disable');
  };

  const handleDeleteClick = () => {
    setConfirmAction('delete');
  };

  const handleConfirm = () => {
    if (confirmAction === 'disable') {
      disableMutation.mutate();
    } else if (confirmAction === 'delete') {
      deleteMutation.mutate();
    }
  };

  const handleCancel = () => {
    setConfirmAction(null);
  };

  const isLoading =
    enableMutation.isPending ||
    disableMutation.isPending ||
    deleteMutation.isPending;

  const showEnableButton = status !== 'active';
  const showDisableButton = status === 'active';

  return (
    <div className="learning-actions">
      {showEnableButton && (
        <button
          type="button"
          className="learning-actions__button learning-actions__button--enable"
          onClick={handleEnable}
          disabled={isLoading}
        >
          Enable
        </button>
      )}

      {showDisableButton && (
        <button
          type="button"
          className="learning-actions__button learning-actions__button--disable"
          onClick={handleDisableClick}
          disabled={isLoading}
        >
          Disable
        </button>
      )}

      <button
        type="button"
        className="learning-actions__button learning-actions__button--delete"
        onClick={handleDeleteClick}
        disabled={isLoading}
      >
        Delete
      </button>

      <ConfirmDialog
        isOpen={confirmAction === 'disable'}
        title="Disable Learning"
        message="This learning won't be injected into future sessions. You can re-enable it later."
        confirmText="Disable"
        onConfirm={handleConfirm}
        onCancel={handleCancel}
        isLoading={disableMutation.isPending}
      />

      <ConfirmDialog
        isOpen={confirmAction === 'delete'}
        title="Delete Learning"
        message="This will permanently remove the learning. This action cannot be undone."
        confirmText="Delete"
        onConfirm={handleConfirm}
        onCancel={handleCancel}
        destructive
        isLoading={deleteMutation.isPending}
      />
    </div>
  );
}
