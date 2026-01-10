import { useState, useCallback } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { ConfirmDialog } from '../ConfirmDialog';
import type { LearningStatus } from '../../../hooks/useDashboard';
import './LearningActions.css';

export interface LearningActionsProps {
  learningId: string;
  status: LearningStatus;
  onActionComplete?: () => void;
}

type ActionType = 'disable' | 'delete' | null;

export function LearningActions({
  learningId,
  status,
  onActionComplete,
}: LearningActionsProps) {
  const [confirmAction, setConfirmAction] = useState<ActionType>(null);
  const queryClient = useQueryClient();

  const invalidateQueries = useCallback(() => {
    queryClient.invalidateQueries({ queryKey: ['dashboard', 'learnings'] });
    queryClient.invalidateQueries({ queryKey: ['dashboard', 'learning', learningId] });
    onActionComplete?.();
  }, [queryClient, learningId, onActionComplete]);

  const enableMutation = useMutation({
    mutationFn: async () => {
      const response = await fetch(`/api/groove/learnings/${learningId}/enable`, {
        method: 'POST',
      });
      if (!response.ok) {
        throw new Error('Failed to enable learning');
      }
    },
    onSuccess: invalidateQueries,
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
    onSuccess: () => {
      setConfirmAction(null);
      invalidateQueries();
    },
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
    onSuccess: () => {
      setConfirmAction(null);
      invalidateQueries();
    },
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
