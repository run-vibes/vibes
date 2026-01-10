import { LinePath } from '@visx/shape';
import { scaleLinear, scaleTime } from '@visx/scale';
import { AxisBottom, AxisLeft } from '@visx/axis';
import { Group } from '@visx/group';
import { curveMonotoneX } from '@visx/curve';
import './TrendChart.css';

export interface DataPoint {
  timestamp: string;
  value: number;
}

export interface Series {
  id: string;
  label: string;
  data: DataPoint[];
  color: string;
}

export interface TrendChartProps {
  series: Series[];
  width: number;
  height: number;
  showLegend?: boolean;
}

const margin = { top: 20, right: 20, bottom: 40, left: 50 };

export function TrendChart({
  series,
  width,
  height,
  showLegend = false,
}: TrendChartProps) {
  const innerWidth = width - margin.left - margin.right;
  const innerHeight = height - margin.top - margin.bottom;

  // Flatten all data points for scale calculation
  const allData = series.flatMap((s) => s.data);

  if (allData.length === 0) {
    return (
      <div className="trend-chart">
        <svg width={width} height={height} />
      </div>
    );
  }

  const timestamps = allData.map((d) => new Date(d.timestamp));
  const values = allData.map((d) => d.value);

  const xScale = scaleTime({
    domain: [Math.min(...timestamps.map((t) => t.getTime())), Math.max(...timestamps.map((t) => t.getTime()))],
    range: [0, innerWidth],
  });

  const minY = Math.min(...values);
  const maxY = Math.max(...values);
  const yPadding = (maxY - minY) * 0.1 || 1;

  const yScale = scaleLinear({
    domain: [minY - yPadding, maxY + yPadding],
    range: [innerHeight, 0],
  });

  return (
    <div className="trend-chart">
      <svg width={width} height={height}>
        <Group left={margin.left} top={margin.top}>
          <AxisBottom
            scale={xScale}
            top={innerHeight}
            stroke="var(--crt-border)"
            tickStroke="var(--crt-border)"
            tickLabelProps={() => ({
              fill: 'var(--crt-text-dim)',
              fontSize: 10,
              fontFamily: 'var(--crt-font-mono)',
              textAnchor: 'middle',
            })}
            numTicks={5}
          />
          <AxisLeft
            scale={yScale}
            stroke="var(--crt-border)"
            tickStroke="var(--crt-border)"
            tickLabelProps={() => ({
              fill: 'var(--crt-text-dim)',
              fontSize: 10,
              fontFamily: 'var(--crt-font-mono)',
              textAnchor: 'end',
              dx: '-0.25em',
              dy: '0.25em',
            })}
            numTicks={5}
          />
          {series.map((s) => (
            <LinePath
              key={s.id}
              className="trend-line"
              data={s.data}
              x={(d) => xScale(new Date(d.timestamp)) ?? 0}
              y={(d) => yScale(d.value) ?? 0}
              stroke={s.color}
              strokeWidth={2}
              curve={curveMonotoneX}
            />
          ))}
        </Group>
      </svg>
      {showLegend && series.length > 0 && (
        <div className="trend-chart__legend">
          {series.map((s) => (
            <div key={s.id} className="trend-chart__legend-item">
              <span
                className="trend-chart__legend-color"
                style={{ backgroundColor: s.color }}
              />
              <span className="trend-chart__legend-label">{s.label}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
