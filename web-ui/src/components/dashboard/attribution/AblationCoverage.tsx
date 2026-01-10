import type { AblationCoverage as AblationCoverageType } from '../../../hooks/useDashboard';
import './AblationCoverage.css';

export interface AblationCoverageProps {
  coverage: AblationCoverageType;
}

export function AblationCoverage({ coverage }: AblationCoverageProps) {
  return (
    <section className="ablation-coverage">
      <h4 className="ablation-coverage__title">Ablation Coverage</h4>

      <div className="ablation-coverage__bar-container">
        <div
          className="ablation-coverage__bar"
          role="progressbar"
          aria-valuenow={coverage.coverage_percent}
          aria-valuemin={0}
          aria-valuemax={100}
        >
          <div
            className="ablation-coverage__bar-fill"
            style={{ width: `${coverage.coverage_percent}%` }}
          />
        </div>
        <span className="ablation-coverage__percent">{coverage.coverage_percent}%</span>
      </div>

      <div className="ablation-coverage__stats">
        <span className="ablation-coverage__stat ablation-coverage__stat--completed">
          {coverage.completed} completed
        </span>
        <span className="ablation-coverage__stat ablation-coverage__stat--progress">
          {coverage.in_progress} in progress
        </span>
        <span className="ablation-coverage__stat ablation-coverage__stat--pending">
          {coverage.pending} pending
        </span>
      </div>
    </section>
  );
}
