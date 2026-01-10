import type { AdaptiveParam, ParamTrend } from '../../../hooks/useDashboard';
import { Sparkline } from '../../charts/Sparkline';
import './AdaptiveParamsTable.css';

export interface AdaptiveParamsTableProps {
  params: AdaptiveParam[];
}

function getTrendIndicator(trend: ParamTrend): string {
  switch (trend) {
    case 'up':
      return '↑';
    case 'down':
      return '↓';
    case 'stable':
      return '→';
  }
}

export function AdaptiveParamsTable({ params }: AdaptiveParamsTableProps) {
  if (params.length === 0) {
    return (
      <div className="adaptive-params-table adaptive-params-table--empty">
        No parameters available
      </div>
    );
  }

  const hasSparklines = params.some((p) => p.sparklineData && p.sparklineData.length > 0);

  return (
    <table className="adaptive-params-table">
      <thead>
        <tr>
          <th>Name</th>
          <th>Current</th>
          <th>Mean</th>
          {hasSparklines && <th>History</th>}
          <th>Trend</th>
        </tr>
      </thead>
      <tbody>
        {params.map((param) => (
          <tr key={param.name}>
            <td className="adaptive-params-table__name">{param.name}</td>
            <td className="adaptive-params-table__value">{param.current}</td>
            <td className="adaptive-params-table__value">{param.mean}</td>
            {hasSparklines && (
              <td className="adaptive-params-table__sparkline">
                {param.sparklineData && param.sparklineData.length > 0 && (
                  <Sparkline
                    data={param.sparklineData}
                    width={60}
                    height={20}
                    color={param.trend === 'up' ? 'var(--crt-success)' : param.trend === 'down' ? 'var(--crt-error)' : 'var(--crt-text-dim)'}
                  />
                )}
              </td>
            )}
            <td className={`adaptive-params-table__trend adaptive-params-table__trend--${param.trend}`}>
              {getTrendIndicator(param.trend)}
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
