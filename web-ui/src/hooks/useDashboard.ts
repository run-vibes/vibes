import { useQuery } from '@tanstack/react-query';

// ============================================================================
// Types (matching Rust dashboard types)
// ============================================================================

export type TrendDirection = 'rising' | 'stable' | 'falling';
export type SystemStatus = 'ok' | 'degraded' | 'error';
export type LearningStatus = 'active' | 'disabled' | 'deprecated' | 'under_review';
export type ActivityType = 'assessment' | 'extraction' | 'attribution' | 'strategy';

export interface Scope {
  User?: string;
  Project?: string;
  Enterprise?: string;
}

export type LearningCategory =
  | 'Correction'
  | 'Workflow'
  | 'Preference'
  | 'Pattern'
  | 'Optimization'
  | 'Automation';

// ─── Overview Types ──────────────────────────────────────────────────────────

export interface TrendSummary {
  sparkline_data: number[];
  improvement_percent: number;
  trend_direction: TrendDirection;
  session_count: number;
  period_days: number;
}

export interface LearningBrief {
  id: string;
  content: string;
  category: LearningCategory;
  scope: Scope;
  status: LearningStatus;
  estimated_value: number;
  created_at: string;
}

export interface LearningSummary {
  total: number;
  active: number;
  recent: LearningBrief[];
  by_category: Record<string, number>;
}

export interface ContributorBrief {
  learning_id: string;
  content: string;
  estimated_value: number;
  confidence: number;
}

export interface AttributionSummary {
  top_contributors: ContributorBrief[];
  under_review_count: number;
  negative_count: number;
}

export interface HealthSummary {
  overall_status: SystemStatus;
  assessment_coverage: number;
  ablation_coverage: number;
  last_activity?: string;
}

export interface OverviewData {
  data_type: 'overview';
  trends: TrendSummary;
  learnings: LearningSummary;
  attribution: AttributionSummary;
  health: HealthSummary;
}

// ─── Learnings Types ─────────────────────────────────────────────────────────

export interface LearningsData {
  data_type: 'learnings';
  learnings: LearningBrief[];
  total: number;
}

export interface LearningDetailData {
  data_type: 'learning_detail';
  id: string;
  content: string;
  category: LearningCategory;
  scope: Scope;
  status: LearningStatus;
  estimated_value: number;
  confidence: number;
  times_injected: number;
  activation_rate: number;
  session_count: number;
  created_at: string;
  source_session?: string;
  extraction_method: string;
}

// ─── Attribution Types ───────────────────────────────────────────────────────

export interface AttributionEntry {
  learning_id: string;
  content: string;
  estimated_value: number;
  confidence: number;
  session_count: number;
  status: LearningStatus;
}

export interface AblationCoverage {
  coverage_percent: number;
  completed: number;
  in_progress: number;
  pending: number;
}

export interface AttributionData {
  data_type: 'attribution';
  top_contributors: AttributionEntry[];
  negative_impact: AttributionEntry[];
  ablation_coverage: AblationCoverage;
}

// ─── Session Timeline Types ─────────────────────────────────────────────────

export type SessionOutcome = 'positive' | 'negative' | 'neutral';

export interface ActivatedLearning {
  learning_id: string;
  content: string;
  contribution: number;
}

export interface SessionTimelineEntry {
  session_id: string;
  timestamp: string;
  score: number;
  activated_learnings: ActivatedLearning[];
  outcome: SessionOutcome;
}

export interface SessionTimelineData {
  data_type: 'session_timeline';
  sessions: SessionTimelineEntry[];
}

// ─── Strategy Types ──────────────────────────────────────────────────────────

export type InjectionStrategy =
  | 'Prefix'
  | 'EarlyContext'
  | 'MidContext'
  | 'JustBeforeQuery'
  | 'Deferral';

export interface StrategyWeight {
  strategy: InjectionStrategy;
  weight: number;
}

export interface CategoryDistribution {
  category_key: string;
  label: string;
  session_count: number;
  weights: StrategyWeight[];
}

export interface StrategyDistributionsData {
  data_type: 'strategy_distributions';
  distributions: CategoryDistribution[];
  specialized_count: number;
  total_learnings: number;
}

export interface LearningOverrideEntry {
  learning_id: string;
  content: string;
  session_count: number;
  is_specialized: boolean;
  base_category: string;
  override_weights?: StrategyWeight[];
  sessions_to_specialize?: number;
}

export interface StrategyOverridesData {
  data_type: 'strategy_overrides';
  overrides: LearningOverrideEntry[];
}

// ─── Health Types ────────────────────────────────────────────────────────────

export interface ComponentHealth {
  status: SystemStatus;
  coverage: number;
  last_activity?: string;
  item_count?: number;
}

export interface AdaptiveParamStatus {
  name: string;
  current_value: number;
  confidence: number;
  trend: TrendDirection;
}

export interface ActivityEntry {
  timestamp: string;
  message: string;
  activity_type: ActivityType;
}

// Types for health page components
export type ParamTrend = 'up' | 'down' | 'stable';
export type EventType = 'extraction' | 'attribution' | 'strategy' | 'error';

export interface AdaptiveParam {
  name: string;
  current: number;
  mean: number;
  trend: ParamTrend;
  sparklineData?: number[];
}

export interface ActivityEvent {
  id: string;
  type: EventType;
  description: string;
  timestamp: string;
}

export interface HealthData {
  data_type: 'health';
  overall_status: SystemStatus;
  assessment: ComponentHealth;
  extraction: ComponentHealth;
  attribution: ComponentHealth;
  adaptive_params: AdaptiveParamStatus[];
  recent_activity: ActivityEntry[];
}

// ─── Filter Types ────────────────────────────────────────────────────────────

export interface LearningsFilter {
  scope?: Scope;
  category?: LearningCategory;
  status?: LearningStatus;
}

// ============================================================================
// Hooks
// ============================================================================

export function useDashboardOverview() {
  return useQuery<OverviewData>({
    queryKey: ['dashboard', 'overview'],
    queryFn: async () => {
      const response = await fetch('/api/groove/dashboard/overview');
      if (!response.ok) {
        throw new Error('Failed to fetch dashboard overview');
      }
      return response.json();
    },
    refetchInterval: 30000, // Poll every 30 seconds
  });
}

export function useDashboardLearnings(filters?: LearningsFilter) {
  const params = new URLSearchParams();
  if (filters?.scope) {
    const [key, value] = Object.entries(filters.scope)[0];
    params.set('scope', `${key}:${value}`);
  }
  if (filters?.category) {
    params.set('category', filters.category);
  }
  if (filters?.status) {
    params.set('status', filters.status);
  }

  const queryString = params.toString();
  const url = queryString
    ? `/api/groove/dashboard/learnings?${queryString}`
    : '/api/groove/dashboard/learnings';

  return useQuery<LearningsData>({
    queryKey: ['dashboard', 'learnings', filters],
    queryFn: async () => {
      const response = await fetch(url);
      if (!response.ok) {
        throw new Error('Failed to fetch dashboard learnings');
      }
      return response.json();
    },
    refetchInterval: 30000,
  });
}

export function useDashboardLearningDetail(id?: string) {
  return useQuery<LearningDetailData>({
    queryKey: ['dashboard', 'learning', id],
    queryFn: async () => {
      const response = await fetch(`/api/groove/dashboard/learnings/${id}`);
      if (!response.ok) {
        throw new Error('Failed to fetch learning detail');
      }
      return response.json();
    },
    enabled: !!id,
  });
}

export function useDashboardAttribution(days?: number) {
  const params = days ? `?days=${days}` : '';

  return useQuery<AttributionData>({
    queryKey: ['dashboard', 'attribution', days],
    queryFn: async () => {
      const response = await fetch(`/api/groove/dashboard/attribution${params}`);
      if (!response.ok) {
        throw new Error('Failed to fetch dashboard attribution');
      }
      return response.json();
    },
    refetchInterval: 30000,
  });
}

export function useDashboardSessionTimeline(days?: number, outcome?: SessionOutcome) {
  const params = new URLSearchParams();
  if (days) params.set('days', String(days));
  if (outcome) params.set('outcome', outcome);
  const queryString = params.toString();
  const url = queryString
    ? `/api/groove/dashboard/session-timeline?${queryString}`
    : '/api/groove/dashboard/session-timeline';

  return useQuery<SessionTimelineData>({
    queryKey: ['dashboard', 'session-timeline', days, outcome],
    queryFn: async () => {
      const response = await fetch(url);
      if (!response.ok) {
        throw new Error('Failed to fetch session timeline');
      }
      return response.json();
    },
    refetchInterval: 30000,
  });
}

export function useDashboardHealth() {
  return useQuery<HealthData>({
    queryKey: ['dashboard', 'health'],
    queryFn: async () => {
      const response = await fetch('/api/groove/dashboard/health');
      if (!response.ok) {
        throw new Error('Failed to fetch dashboard health');
      }
      return response.json();
    },
    refetchInterval: 10000, // Poll more frequently for health
  });
}

export function useDashboardStrategyDistributions() {
  return useQuery<StrategyDistributionsData>({
    queryKey: ['dashboard', 'strategy', 'distributions'],
    queryFn: async () => {
      const response = await fetch('/api/groove/dashboard/strategy/distributions');
      if (!response.ok) {
        throw new Error('Failed to fetch strategy distributions');
      }
      return response.json();
    },
    refetchInterval: 30000,
  });
}

export function useDashboardStrategyOverrides() {
  return useQuery<StrategyOverridesData>({
    queryKey: ['dashboard', 'strategy', 'overrides'],
    queryFn: async () => {
      const response = await fetch('/api/groove/dashboard/strategy/overrides');
      if (!response.ok) {
        throw new Error('Failed to fetch strategy overrides');
      }
      return response.json();
    },
    refetchInterval: 30000,
  });
}

// ============================================================================
// OpenWorld Types
// ============================================================================

export type GapCategory = 'MissingKnowledge' | 'IncorrectPattern' | 'ContextMismatch' | 'ToolGap';
export type GapSeverity = 'Low' | 'Medium' | 'High' | 'Critical';
export type GapStatus = 'Detected' | 'Confirmed' | 'InProgress' | 'Resolved' | 'Dismissed';

export interface GapCounts {
  low: number;
  medium: number;
  high: number;
  critical: number;
  total: number;
}

export interface HookStatsData {
  outcomes_processed: number;
  negative_outcomes: number;
  low_confidence_outcomes: number;
  exploration_adjustments: number;
  gaps_created: number;
}

export interface ClusterBrief {
  id: string;
  member_count: number;
  category_hint?: string;
  created_at: string;
  last_seen: string;
}

export interface OpenWorldOverviewData {
  data_type: 'open_world_overview';
  novelty_threshold: number;
  pending_outliers: number;
  cluster_count: number;
  gap_counts: GapCounts;
  hook_stats: HookStatsData;
  recent_clusters?: ClusterBrief[];
}

export interface GapBrief {
  id: string;
  category: GapCategory;
  severity: GapSeverity;
  status: GapStatus;
  context_pattern: string;
  failure_count: number;
  first_seen: string;
  last_seen: string;
  solution_count: number;
}

export interface OpenWorldGapsData {
  data_type: 'open_world_gaps';
  gaps: GapBrief[];
  total: number;
}

export interface SolutionBrief {
  action_type: string;
  description: string;
  confidence: number;
  applied: boolean;
}

export interface OpenWorldGapDetailData {
  data_type: 'open_world_gap_detail';
  id: string;
  category: GapCategory;
  severity: GapSeverity;
  status: GapStatus;
  context_pattern: string;
  failure_count: number;
  first_seen: string;
  last_seen: string;
  suggested_solutions: SolutionBrief[];
}

export type SolutionStatus = 'Pending' | 'Applied' | 'Dismissed';

export interface SolutionEntry {
  id: string;
  gap_id: string;
  gap_context: string;
  action_type: string;
  description: string;
  confidence: number;
  status: SolutionStatus;
  created_at: string;
  updated_at?: string;
}

export interface OpenWorldSolutionsData {
  data_type: 'open_world_solutions';
  solutions: SolutionEntry[];
  total: number;
}

export type OpenWorldEventType =
  | 'novelty_detected'
  | 'cluster_updated'
  | 'gap_created'
  | 'gap_status_changed'
  | 'solution_generated'
  | 'strategy_feedback';

export interface OpenWorldActivityEntry {
  timestamp: string;
  event_type: OpenWorldEventType;
  message: string;
  gap_id?: string;
  learning_id?: string;
}

export interface ActivitySummary {
  outcomes_total: number;
  negative_rate: number;
  avg_exploration_bonus: number;
}

export interface OpenWorldActivityData {
  data_type: 'open_world_activity';
  events: OpenWorldActivityEntry[];
  summary: ActivitySummary;
}

// ============================================================================
// OpenWorld Hooks
// ============================================================================

export interface OpenWorldGapsFilter {
  status?: GapStatus;
  severity?: GapSeverity;
  category?: GapCategory;
}

export function useOpenWorldOverview() {
  return useQuery<OpenWorldOverviewData>({
    queryKey: ['dashboard', 'openworld', 'overview'],
    queryFn: async () => {
      const response = await fetch('/api/groove/dashboard/openworld/overview');
      if (!response.ok) {
        throw new Error('Failed to fetch openworld overview');
      }
      return response.json();
    },
    refetchInterval: 10000, // Poll every 10 seconds
  });
}

export function useOpenWorldGaps(filters?: OpenWorldGapsFilter) {
  const params = new URLSearchParams();
  if (filters?.status) params.set('status', filters.status);
  if (filters?.severity) params.set('severity', filters.severity);
  if (filters?.category) params.set('category', filters.category);
  const queryString = params.toString();
  const url = queryString
    ? `/api/groove/dashboard/openworld/gaps?${queryString}`
    : '/api/groove/dashboard/openworld/gaps';

  return useQuery<OpenWorldGapsData>({
    queryKey: ['dashboard', 'openworld', 'gaps', filters],
    queryFn: async () => {
      const response = await fetch(url);
      if (!response.ok) {
        throw new Error('Failed to fetch openworld gaps');
      }
      return response.json();
    },
    refetchInterval: 30000,
  });
}

export function useOpenWorldGapDetail(id?: string) {
  return useQuery<OpenWorldGapDetailData>({
    queryKey: ['dashboard', 'openworld', 'gap', id],
    queryFn: async () => {
      const response = await fetch(`/api/groove/dashboard/openworld/gaps/${id}`);
      if (!response.ok) {
        throw new Error('Failed to fetch gap detail');
      }
      return response.json();
    },
    enabled: !!id,
  });
}

export function useOpenWorldSolutions() {
  return useQuery<OpenWorldSolutionsData>({
    queryKey: ['dashboard', 'openworld', 'solutions'],
    queryFn: async () => {
      const response = await fetch('/api/groove/dashboard/openworld/solutions');
      if (!response.ok) {
        throw new Error('Failed to fetch openworld solutions');
      }
      return response.json();
    },
    refetchInterval: 30000,
  });
}

export function useOpenWorldActivity() {
  return useQuery<OpenWorldActivityData>({
    queryKey: ['dashboard', 'openworld', 'activity'],
    queryFn: async () => {
      const response = await fetch('/api/groove/dashboard/openworld/activity');
      if (!response.ok) {
        throw new Error('Failed to fetch openworld activity');
      }
      return response.json();
    },
    refetchInterval: 10000, // Poll more frequently for activity
  });
}
