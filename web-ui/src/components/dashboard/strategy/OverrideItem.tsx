import { useState } from 'react';
import { StrategyBar } from './StrategyBar';
import type { LearningOverrideEntry } from '../../../hooks/useDashboard';
import './OverrideItem.css';

export interface OverrideItemProps {
  override: LearningOverrideEntry;
}

export function OverrideItem({ override }: OverrideItemProps) {
  const [expanded, setExpanded] = useState(false);

  const canExpand = override.is_specialized && override.override_weights;

  return (
    <div className={`override-item ${override.is_specialized ? 'override-item--specialized' : ''}`}>
      {canExpand ? (
        <button
          type="button"
          className="override-item__header override-item__header--expandable"
          onClick={() => setExpanded(!expanded)}
          aria-expanded={expanded}
        >
          <OverrideItemContent override={override} expanded={expanded} />
        </button>
      ) : (
        <div className="override-item__header">
          <OverrideItemContent override={override} expanded={false} />
        </div>
      )}

      {expanded && override.override_weights && (
        <div className="override-item__weights">
          {override.override_weights.map((w) => (
            <StrategyBar key={w.strategy} strategy={w.strategy} weight={w.weight} />
          ))}
        </div>
      )}
    </div>
  );
}

interface OverrideItemContentProps {
  override: LearningOverrideEntry;
  expanded: boolean;
}

function OverrideItemContent({ override, expanded }: OverrideItemContentProps) {
  return (
    <>
      <div className="override-item__main">
        <span className="override-item__content">{override.content}</span>
        <div className="override-item__meta">
          <span className="override-item__category">Base: {override.base_category}</span>
          <span className="override-item__sessions">{override.session_count} sessions</span>
        </div>
      </div>

      <div className="override-item__status">
        {override.is_specialized ? (
          <span className="override-item__badge override-item__badge--specialized">
            specialized
          </span>
        ) : (
          <>
            <span className="override-item__badge override-item__badge--inheriting">
              inheriting
            </span>
            {override.sessions_to_specialize !== undefined && (
              <span className="override-item__threshold">
                {override.sessions_to_specialize} more sessions needed
              </span>
            )}
          </>
        )}
      </div>

      {override.is_specialized && override.override_weights && (
        <span className="override-item__expand">{expanded ? '▼' : '▶'}</span>
      )}
    </>
  );
}
