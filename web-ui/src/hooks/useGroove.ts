import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';

// ============================================================================
// Types
// ============================================================================

export interface TrustLevelInfo {
  name: string;
  score: number;
  description: string;
}

export interface TrustHierarchyResponse {
  levels: TrustLevelInfo[];
}

export interface PermissionFlags {
  can_create: boolean;
  can_read: boolean;
  can_modify: boolean;
  can_delete: boolean;
  can_publish: boolean;
  can_review: boolean;
  can_admin: boolean;
}

export interface RolePermissionsResponse {
  role: string;
  permissions: PermissionFlags;
}

export interface PolicyResponse {
  injection: {
    block_quarantined: boolean;
    allow_personal_injection: boolean;
    allow_unverified_injection: boolean;
  };
  quarantine: {
    reviewers: string[];
    visible_to: string[];
    auto_delete_after_days: number | null;
  };
  import_export: {
    allow_import_from_file: boolean;
    allow_import_from_url: boolean;
    allowed_import_sources: string[];
    allow_export_personal: boolean;
    allow_export_enterprise: boolean;
  };
  audit: {
    enabled: boolean;
    retention_days: number;
  };
}

export interface QuarantinedLearningSummary {
  id: string;
  description: string;
  trust_level: string;
  reason: string;
  quarantined_at: string;
  pending_review: boolean;
}

export interface QuarantineListResponse {
  items: QuarantinedLearningSummary[];
  total: number;
}

export interface QuarantineStatsResponse {
  total: number;
  pending_review: number;
  approved: number;
  rejected: number;
  escalated: number;
}

export interface ReviewRequest {
  outcome: 'approve' | 'reject' | 'escalate';
  notes?: string;
}

export interface ReviewResponse {
  outcome: string;
  restored: boolean;
  deleted: boolean;
}

// ============================================================================
// Hooks
// ============================================================================

export function usePolicy() {
  return useQuery<PolicyResponse>({
    queryKey: ['groove', 'policy'],
    queryFn: async () => {
      const response = await fetch('/api/groove/policy');
      if (!response.ok) {
        throw new Error('Failed to fetch policy');
      }
      return response.json();
    },
  });
}

export function useTrustLevels() {
  return useQuery<TrustHierarchyResponse>({
    queryKey: ['groove', 'trust-levels'],
    queryFn: async () => {
      const response = await fetch('/api/groove/trust/levels');
      if (!response.ok) {
        throw new Error('Failed to fetch trust levels');
      }
      return response.json();
    },
  });
}

export function useRolePermissions(role: string) {
  return useQuery<RolePermissionsResponse>({
    queryKey: ['groove', 'role', role],
    queryFn: async () => {
      const response = await fetch(`/api/groove/trust/role/${role}`);
      if (!response.ok) {
        throw new Error('Failed to fetch role permissions');
      }
      return response.json();
    },
    enabled: !!role,
  });
}

export function useQuarantineList() {
  return useQuery<QuarantineListResponse>({
    queryKey: ['groove', 'quarantine', 'list'],
    queryFn: async () => {
      const response = await fetch('/api/groove/quarantine');
      if (!response.ok) {
        throw new Error('Failed to fetch quarantine list');
      }
      return response.json();
    },
    refetchInterval: 30000, // Poll every 30 seconds
  });
}

export function useQuarantineStats() {
  return useQuery<QuarantineStatsResponse>({
    queryKey: ['groove', 'quarantine', 'stats'],
    queryFn: async () => {
      const response = await fetch('/api/groove/quarantine/stats');
      if (!response.ok) {
        throw new Error('Failed to fetch quarantine stats');
      }
      return response.json();
    },
    refetchInterval: 30000, // Poll every 30 seconds
  });
}

export function useReviewQuarantine() {
  const queryClient = useQueryClient();

  return useMutation<ReviewResponse, Error, { id: string; request: ReviewRequest }>({
    mutationFn: async ({ id, request }) => {
      const response = await fetch(`/api/groove/quarantine/${id}/review`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(request),
      });
      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to review');
      }
      return response.json();
    },
    onSuccess: () => {
      // Invalidate quarantine queries to refresh the data
      queryClient.invalidateQueries({ queryKey: ['groove', 'quarantine'] });
    },
  });
}
