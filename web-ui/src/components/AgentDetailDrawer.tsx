import { Button, Badge, Metric } from '@vibes/design-system';
import type { AgentInfo } from '../lib/types';
import { getAgentStatusVariant } from '../lib/types';
import './AgentDetailDrawer.css';

interface AgentDetailDrawerProps {
  agent: AgentInfo;
  onClose: () => void;
  onPause: () => void;
  onResume: () => void;
  onCancel: () => void;
  onStop: () => void;
}

const statusBadgeMap = {
  idle: 'idle',
  running: 'accent',
  paused: 'warning',
  waiting_for_input: 'warning',
  failed: 'error',
} as const;

const statusLabels = {
  idle: 'Idle',
  running: 'Running',
  paused: 'Paused',
  waiting_for_input: 'Waiting for Input',
  failed: 'Failed',
};

const typeLabels = {
  AdHoc: 'Ad-hoc',
  Background: 'Background',
  Subagent: 'Subagent',
  Interactive: 'Interactive',
};

function formatDuration(secs: number, nanos: number): string {
  const totalSeconds = Math.floor(secs + nanos / 1e9);
  if (totalSeconds < 60) return `${totalSeconds}s`;
  const minutes = Math.floor(totalSeconds / 60);
  if (minutes < 60) return `${minutes}m ${totalSeconds % 60}s`;
  const hours = Math.floor(minutes / 60);
  return `${hours}h ${minutes % 60}m`;
}

function formatTokens(tokens: number): string {
  if (tokens < 1000) return tokens.toString();
  return `${(tokens / 1000).toFixed(1)}k`;
}

export function AgentDetailDrawer({
  agent,
  onClose,
  onPause,
  onResume,
  onCancel,
  onStop,
}: AgentDetailDrawerProps) {
  const status = getAgentStatusVariant(agent.status);
  const metrics = agent.current_task_metrics;

  const getStatusDetail = () => {
    const s = agent.status;
    // Unit variants (like 'idle') are strings, data variants are objects
    if (typeof s === 'string') return null;
    if ('running' in s) {
      return { label: 'Task ID', value: s.running.task.slice(0, 8) };
    }
    if ('paused' in s) {
      return { label: 'Reason', value: s.paused.reason };
    }
    if ('waiting_for_input' in s) {
      return { label: 'Prompt', value: s.waiting_for_input.prompt };
    }
    if ('failed' in s) {
      return { label: 'Error', value: s.failed.error };
    }
    return null;
  };

  const statusDetail = getStatusDetail();

  return (
    <div className="agent-drawer-backdrop" onClick={onClose}>
      <div className="agent-drawer" onClick={(e) => e.stopPropagation()}>
        <div className="agent-drawer-header">
          <div className="agent-drawer-header-left">
            <h2 className="agent-drawer-title">{agent.name}</h2>
            <Badge status={statusBadgeMap[status]}>{statusLabels[status]}</Badge>
          </div>
          <button
            type="button"
            className="agent-drawer-close"
            onClick={onClose}
            aria-label="Close"
          >
            ×
          </button>
        </div>

        <div className="agent-drawer-content">
          {/* Agent Info Section */}
          <section className="agent-drawer-section">
            <h3 className="agent-drawer-section-title">AGENT INFO</h3>
            <dl className="agent-drawer-info">
              <div className="agent-drawer-info-row">
                <dt>ID</dt>
                <dd className="agent-drawer-mono">{agent.id}</dd>
              </div>
              <div className="agent-drawer-info-row">
                <dt>Type</dt>
                <dd>{typeLabels[agent.agent_type]}</dd>
              </div>
              <div className="agent-drawer-info-row">
                <dt>Model</dt>
                <dd className="agent-drawer-mono">{agent.context.model.id || 'unknown'}</dd>
              </div>
              <div className="agent-drawer-info-row">
                <dt>Location</dt>
                <dd>{agent.context.location.type}</dd>
              </div>
            </dl>
          </section>

          {/* Status Detail */}
          {statusDetail && (
            <section className="agent-drawer-section">
              <h3 className="agent-drawer-section-title">STATUS DETAIL</h3>
              <dl className="agent-drawer-info">
                <div className="agent-drawer-info-row">
                  <dt>{statusDetail.label}</dt>
                  <dd className="agent-drawer-mono">{statusDetail.value}</dd>
                </div>
              </dl>
            </section>
          )}

          {/* Metrics */}
          {metrics && (
            <section className="agent-drawer-section">
              <h3 className="agent-drawer-section-title">METRICS</h3>
              <div className="agent-drawer-metrics">
                <Metric
                  label="Duration"
                  value={formatDuration(metrics.duration.secs, metrics.duration.nanos)}
                />
                <Metric
                  label="Tokens"
                  value={formatTokens(metrics.tokens_used)}
                />
                <Metric
                  label="Tool Calls"
                  value={metrics.tool_calls.toString()}
                />
                <Metric
                  label="Iterations"
                  value={metrics.iterations.toString()}
                />
              </div>
            </section>
          )}

          {/* Context */}
          <section className="agent-drawer-section">
            <h3 className="agent-drawer-section-title">CONTEXT</h3>
            <dl className="agent-drawer-info">
              <div className="agent-drawer-info-row">
                <dt>Tools</dt>
                <dd>
                  {agent.context.tools.length > 0
                    ? agent.context.tools.map((t) => t.id).join(', ')
                    : 'None'}
                </dd>
              </div>
              <div className="agent-drawer-info-row">
                <dt>Permissions</dt>
                <dd>
                  {[
                    agent.context.permissions.filesystem && 'Filesystem',
                    agent.context.permissions.network && 'Network',
                    agent.context.permissions.shell && 'Shell',
                  ]
                    .filter(Boolean)
                    .join(', ') || 'None'}
                </dd>
              </div>
              {agent.context.resource_limits.max_tokens && (
                <div className="agent-drawer-info-row">
                  <dt>Max Tokens</dt>
                  <dd>{agent.context.resource_limits.max_tokens.toLocaleString()}</dd>
                </div>
              )}
              {agent.context.resource_limits.max_tool_calls && (
                <div className="agent-drawer-info-row">
                  <dt>Max Tool Calls</dt>
                  <dd>{agent.context.resource_limits.max_tool_calls}</dd>
                </div>
              )}
            </dl>
          </section>
        </div>

        {/* Actions */}
        <div className="agent-drawer-actions">
          {status === 'running' && (
            <>
              <Button variant="secondary" onClick={onPause}>
                Pause
              </Button>
              <Button variant="secondary" onClick={onCancel}>
                Cancel Task
              </Button>
            </>
          )}
          {status === 'paused' && (
            <Button variant="secondary" onClick={onResume}>
              Resume
            </Button>
          )}
          <Button variant="primary" onClick={onStop}>
            Stop Agent
          </Button>
        </div>
      </div>
    </div>
  );
}
