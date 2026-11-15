import type { FC } from 'react';

export const CHART_COLORS = [
  '#ea580c',
  '#38bdf8',
  '#4ade80',
  '#fbbf24',
  '#f87171',
  '#c084fc',
  '#2dd4bf',
  '#f472b6',
];

const W = 560;
const H = 200;
const PAD = { top: 12, right: 12, bottom: 24, left: 44 };

const scale = (values: number[], min?: number, max?: number) => {
  const lo = min ?? Math.min(...values);
  const hi = max ?? Math.max(...values);
  const span = hi - lo || 1;
  return (v: number) => PAD.top + (1 - (v - lo) / span) * (H - PAD.top - PAD.bottom);
};

const xScale = (n: number) =>
  (i: number) => PAD.left + (n <= 1 ? 0 : (i / (n - 1)) * (W - PAD.left - PAD.right));

const niceTicks = (min: number, max: number, count = 4): number[] => {
  const span = max - min || 1;
  const step = span / count;
  const mag = 10 ** Math.floor(Math.log10(step));
  const norm = step / mag;
  const nice = norm >= 5 ? 5 : norm >= 2 ? 2 : 1;
  const unit = nice * mag;
  const start = Math.ceil(min / unit) * unit;
  const ticks: number[] = [];
  for (let v = start; v <= max + unit * 0.5; v += unit) ticks.push(v);
  return ticks.slice(0, 6);
};

const Axes: FC<{ ymin: number; ymax: number; xLabels?: [string, string] }> = ({
  ymin,
  ymax,
  xLabels,
}) => {
  const ticks = niceTicks(ymin, ymax);
  const y = scale([ymin, ymax]);
  return (
    <g className='stroke-neutral-700' strokeWidth={1}>
      {ticks.map((t) => (
        <g key={t}>
          <line x1={PAD.left} x2={W - PAD.right} y1={y(t)} y2={y(t)} strokeDasharray='2 4' />
          <text x={PAD.left - 6} y={y(t) + 3} textAnchor='end' className='fill-neutral-500' fontSize={9}>
            {Math.abs(t) >= 1000 ? `${(t / 1000).toFixed(1)}k` : t.toFixed(Math.abs(t) < 10 ? 1 : 0)}
          </text>
        </g>
      ))}
      {xLabels && (
        <>
          <text x={PAD.left} y={H - 8} className='fill-neutral-500' fontSize={9}>
            {xLabels[0]}
          </text>
          <text x={W - PAD.right} y={H - 8} textAnchor='end' className='fill-neutral-500' fontSize={9}>
            {xLabels[1]}
          </text>
        </>
      )}
    </g>
  );
};

export const LineChart: FC<{
  x: number[];
  y: number[];
  xLabel?: string;
  yLabel?: string;
  height?: number;
}> = ({ x, y, xLabel = 'index', yLabel, height = H }) => {
  if (x.length !== y.length || x.length === 0) return null;
  const xs = xScale(x.length);
  const ys = scale(y);
  const points = x.map((_, i) => `${xs(i).toFixed(1)},${ys(y[i]).toFixed(1)}`).join(' ');
  const ymin = Math.min(...y);
  const ymax = Math.max(...y);
  return (
    <svg viewBox={`0 0 ${W} ${height}`} className='w-full' role='img' aria-label={yLabel ?? 'line chart'}>
      <Axes ymin={ymin} ymax={ymax} xLabels={[xLabel, xLabel]} />
      <polyline points={points} fill='none' stroke='#ea580c' strokeWidth={1.5} />
    </svg>
  );
};

export const BarChart: FC<{
  labels: string[];
  values: number[];
  horizontal?: boolean;
}> = ({ labels, values, horizontal = false }) => {
  if (values.length === 0) return null;
  const max = Math.max(...values.map(Math.abs), 1e-9);
  if (horizontal) {
    const rowH = Math.min(28, (H - PAD.top - PAD.bottom) / values.length);
    return (
      <svg viewBox={`0 0 ${W} ${Math.max(H, values.length * rowH + PAD.top + PAD.bottom)}`} className='w-full' role='img'>
        {values.map((v, i) => {
          const w = (Math.abs(v) / max) * (W - PAD.left - PAD.right - 90);
          return (
            <g key={labels[i] ?? i}>
              <text x={PAD.left - 6} y={PAD.top + i * rowH + rowH / 2 + 3} textAnchor='end' className='fill-neutral-400' fontSize={10}>
                {(labels[i] ?? String(i)).slice(0, 18)}
              </text>
              <rect x={PAD.left} y={PAD.top + i * rowH + 4} width={w} height={rowH - 8} rx={2} fill='#ea580c' opacity={0.85} />
              <text x={PAD.left + w + 6} y={PAD.top + i * rowH + rowH / 2 + 3} className='fill-neutral-300' fontSize={10}>
                {v.toFixed(3)}
              </text>
            </g>
          );
        })}
      </svg>
    );
  }
  const bw = (W - PAD.left - PAD.right) / values.length;
  const ys = scale(values);
  return (
    <svg viewBox={`0 0 ${W} ${H}`} className='w-full' role='img'>
      <Axes ymin={Math.min(0, ...values)} ymax={Math.max(...values)} />
      {values.map((v, i) => (
        <g key={labels[i] ?? i}>
          <rect
            x={PAD.left + i * bw + bw * 0.15}
            y={ys(Math.max(v, 0))}
            width={bw * 0.7}
            height={Math.abs(ys(v) - ys(0))}
            rx={2}
            fill='#ea580c'
            opacity={0.85}
          />
          <text x={PAD.left + i * bw + bw / 2} y={H - 10} textAnchor='middle' className='fill-neutral-500' fontSize={8}>
            {(labels[i] ?? String(i)).slice(0, 10)}
          </text>
        </g>
      ))}
    </svg>
  );
};

export const ScatterPlot: FC<{
  x: number[];
  y: number[];
  groups?: number[];
  highlight?: boolean[];
}> = ({ x, y, groups, highlight }) => {
  if (x.length === 0) return null;
  const xPix = (v: number) =>
    PAD.left +
    ((v - Math.min(...x)) / (Math.max(...x) - Math.min(...x) || 1)) * (W - PAD.left - PAD.right);
  const yPix = (v: number) =>
    PAD.top +
    (1 - (v - Math.min(...y)) / (Math.max(...y) - Math.min(...y) || 1)) * (H - PAD.top - PAD.bottom);
  return (
    <svg viewBox={`0 0 ${W} ${H}`} className='w-full' role='img' aria-label='scatter plot'>
      <Axes ymin={Math.min(...y)} ymax={Math.max(...y)} />
      {x.map((vx, i) => {
        const g = groups?.[i];
        const color = highlight?.[i] ? '#f87171' : g !== undefined ? CHART_COLORS[g % CHART_COLORS.length] : '#38bdf8';
        return <circle key={i} cx={xPix(vx)} cy={yPix(y[i])} r={2.5} fill={color} opacity={0.7} />;
      })}
    </svg>
  );
};

export const Histogram: FC<{ edges: number[]; counts: number[]; threshold?: number | null }> = ({
  edges,
  counts,
  threshold,
}) => {
  if (edges.length < 2 || counts.length === 0) return null;
  const maxCount = Math.max(...counts, 1);
  const bw = (W - PAD.left - PAD.right) / counts.length;
  const xPix = (v: number) => {
    const min = edges[0];
    const max = edges[edges.length - 1];
    return PAD.left + ((v - min) / (max - min || 1)) * (W - PAD.left - PAD.right);
  };
  return (
    <svg viewBox={`0 0 ${W} ${H}`} className='w-full' role='img' aria-label='histogram'>
      <Axes ymin={0} ymax={maxCount} />
      {counts.map((c, i) => {
        const h = (c / maxCount) * (H - PAD.top - PAD.bottom);
        return (
          <rect
            key={i}
            x={PAD.left + i * bw}
            y={H - PAD.bottom - h}
            width={Math.max(bw - 1, 1)}
            height={h}
            fill='#38bdf8'
            opacity={0.8}
          />
        );
      })}
      {threshold !== null && threshold !== undefined && (
        <line
          x1={xPix(threshold)}
          x2={xPix(threshold)}
          y1={PAD.top}
          y2={H - PAD.bottom}
          stroke='#f87171'
          strokeWidth={1.5}
          strokeDasharray='4 3'
        />
      )}
    </svg>
  );
};
