import { forwardRef, HTMLAttributes, MouseEvent, ReactNode } from 'react';
import { Badge } from '../../primitives/Badge';
import styles from './AgentCard.module.css';

export type AgentStatusVariant = 'idle' | 'running' | 'paused' | 'waiting_for_input' | 'failed';
export type AgentTypeVariant = 'AdHoc' | 'Background' | 'Subagent' | 'Interactive';

export interface AgentAction {
  icon: ReactNode;
  label: string;
  onClick: (e: MouseEvent<HTMLButtonElement>) => void;
}

export interface AgentCardProps extends HTMLAttributes<HTMLElement> {
  id: string;
  name: string;
  agentType: AgentTypeVariant;
  status: AgentStatusVariant;
  /** Current task description if running */
  currentTask?: string;
  /** Model being used */
  model?: string;
  /** Metrics - tokens used */
  tokensUsed?: number;
  /** Metrics - tool calls */
  toolCalls?: number;
  /** Metrics - duration in seconds */
  duration?: number;
  /** Quick action buttons shown on hover */
  actions?: AgentAction[];
  /** URL to navigate to - renders as anchor when provided */
  href?: string;
}

const statusBadgeMap = {
  idle: 'idle',
  running: 'accent',
  paused: 'warning',
  waiting_for_input: 'warning',
  failed: 'error',
} as const;

const statusLabels: Record<AgentStatusVariant, string> = {
  idle: 'Idle',
  running: 'Running',
  paused: 'Paused',
  waiting_for_input: 'Waiting',
  failed: 'Failed',
};

const typeLabels: Record<AgentTypeVariant, string> = {
  AdHoc: 'Ad-hoc',
  Background: 'Background',
  Subagent: 'Subagent',
  Interactive: 'Interactive',
};

function isActiveStatus(status: AgentStatusVariant): boolean {
  return status === 'running' || status === 'waiting_for_input';
}

export const AgentCard = forwardRef<HTMLElement, AgentCardProps>(
  (
    {
      id,
      name,
      agentType,
      status,
      currentTask,
      model,
      tokensUsed,
      toolCalls,
      duration,
      actions,
      href,
      className = '',
      onClick,
      ...props
    },
    ref
  ) => {
    const isActive = isActiveStatus(status);
    const classes = [
      styles.card,
      isActive ? styles.active : styles.inactive,
      className,
    ]
      .filter(Boolean)
      .join(' ');

    const durationStr = duration !== undefined ? formatDuration(duration) : undefined;

    const Tag = href ? 'a' : 'article';

    return (
      <Tag
        ref={ref as React.Ref<HTMLAnchorElement & HTMLElement>}
        className={classes}
        onClick={onClick}
        href={href}
        {...props}
      >
        <div className={styles.header}>
          <div className={styles.titleSection}>
            <span className={`${styles.statusDot} ${styles[status]}`} />
            <h3 className={styles.title}>{name}</h3>
          </div>
          {actions && actions.length > 0 && (
            <div className={styles.actions}>
              {actions.map((action, i) => (
                <button
                  key={i}
                  type="button"
                  className={styles.actionButton}
                  onClick={(e) => {
                    e.stopPropagation();
                    action.onClick(e);
                  }}
                  aria-label={action.label}
                  title={action.label}
                >
                  {action.icon}
                </button>
              ))}
            </div>
          )}
        </div>

        <div className={styles.id}>{id.slice(0, 8)}</div>

        {currentTask && (
          <p className={styles.task}>{currentTask}</p>
        )}

        <div className={styles.meta}>
          <div className={styles.metaLeft}>
            <span className={styles.typeBadge}>{typeLabels[agentType]}</span>
            {model && <span className={styles.model}>{model}</span>}
          </div>
          <div className={styles.metaRight}>
            {(durationStr || tokensUsed !== undefined || toolCalls !== undefined) && (
              <div className={styles.badgeGroup}>
                {durationStr && <span className={styles.metaBadge}>{durationStr}</span>}
                {tokensUsed !== undefined && (
                  <span className={styles.metaBadge}>{formatTokens(tokensUsed)}</span>
                )}
                {toolCalls !== undefined && (
                  <span className={styles.metaBadge}>{toolCalls} calls</span>
                )}
              </div>
            )}
            <Badge status={statusBadgeMap[status]}>{statusLabels[status]}</Badge>
          </div>
        </div>
      </Tag>
    );
  }
);

AgentCard.displayName = 'AgentCard';

function formatDuration(seconds: number): string {
  if (seconds < 60) return `${seconds}s`;
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.floor(minutes / 60);
  const remainingMinutes = minutes % 60;
  if (remainingMinutes === 0) return `${hours}h`;
  return `${hours}h ${remainingMinutes}m`;
}

function formatTokens(tokens: number): string {
  if (tokens < 1000) return `${tokens} tok`;
  return `${(tokens / 1000).toFixed(1)}k tok`;
}
