import { LinePath, AreaClosed } from '@visx/shape';
import { scaleLinear } from '@visx/scale';
import { curveMonotoneX } from '@visx/curve';
import './Sparkline.css';

export interface SparklineProps {
  data: number[];
  width: number;
  height: number;
  color?: string;
  showArea?: boolean;
}

export function Sparkline({
  data,
  width,
  height,
  color = 'var(--crt-success)',
  showArea = false,
}: SparklineProps) {
  if (data.length === 0) {
    return (
      <div className="sparkline">
        <svg width={width} height={height} />
      </div>
    );
  }

  const padding = 2;
  const innerWidth = width - padding * 2;
  const innerHeight = height - padding * 2;

  const xScale = scaleLinear({
    domain: [0, Math.max(1, data.length - 1)],
    range: [padding, innerWidth + padding],
  });

  const minY = Math.min(...data);
  const maxY = Math.max(...data);
  const yRange = maxY - minY || 1;

  const yScale = scaleLinear({
    domain: [minY - yRange * 0.1, maxY + yRange * 0.1],
    range: [innerHeight + padding, padding],
  });

  const getX = (_: number, i: number) => xScale(i) ?? 0;
  const getY = (d: number) => yScale(d) ?? 0;

  return (
    <div className="sparkline">
      <svg width={width} height={height}>
        {showArea && (
          <AreaClosed
            data={data}
            x={getX}
            y={getY}
            yScale={yScale}
            curve={curveMonotoneX}
            fill={color}
            fillOpacity={0.15}
          />
        )}
        <LinePath
          data={data}
          x={getX}
          y={getY}
          stroke={color}
          strokeWidth={1.5}
          curve={curveMonotoneX}
        />
      </svg>
    </div>
  );
}
