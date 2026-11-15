import type { FC, ReactNode } from 'react';
import { BarChart, Histogram, LineChart, ScatterPlot } from './charts/primitives';

const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === 'object' && value !== null && !Array.isArray(value);

const numArray = (value: unknown): number[] | null =>
  Array.isArray(value) && value.every((v) => typeof v === 'number') ? (value as number[]) : null;

const ChartCard: FC<{ title: string; children: ReactNode }> = ({ title, children }) => (
  <figure className='rounded-lg border border-neutral-800 bg-neutral-900/60 p-3'>
    <figcaption className='mb-2 text-xs font-medium text-neutral-400'>{title}</figcaption>
    {children}
  </figure>
);

const ConfusionMatrix: FC<{ matrix: number[][]; classes: string[] }> = ({ matrix, classes }) => {
  if (matrix.length === 0) return null;
  const max = Math.max(...matrix.flat(), 1);
  return (
    <div className='overflow-x-auto'>
      <table className='text-xs'>
        <thead>
          <tr>
            <th className='p-1 text-neutral-500'>true \ pred</th>
            {classes.map((c) => (
              <th key={c} className='p-1 text-neutral-400'>
                {c}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {matrix.map((row, i) => (
            <tr key={classes[i] ?? i}>
              <td className='p-1 pr-2 text-neutral-400'>{classes[i] ?? i}</td>
              {row.map((cell, j) => (
                <td
                  key={j}
                  className='p-1 text-center text-neutral-200'
                  style={{ backgroundColor: `rgba(234,88,12,${0.15 + (cell / max) * 0.6})` }}>
                  {cell}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
};

const FeatureImportance: FC<{ value: unknown }> = ({ value }) => {
  if (!isRecord(value)) return null;
  const features = Array.isArray(value.feature) ? value.feature.map(String) : null;
  const importance = numArray(value.importance);
  if (!features || !importance || features.length !== importance.length) return null;
  const method = typeof value.method === 'string' ? value.method : null;
  return (
    <ChartCard title={`Feature importance${method ? ` (${method})` : ''}`}>
      <BarChart labels={features} values={importance} horizontal />
    </ChartCard>
  );
};

const MlpProgress: FC<{ value: unknown }> = ({ value }) => {
  if (!isRecord(value)) return null;
  const epoch = numArray(value.epoch);
  const loss = numArray(value.loss);
  if (!epoch || !loss || epoch.length !== loss.length) return null;
  return (
    <ChartCard title='Training loss'>
      <LineChart x={epoch} y={loss} xLabel='epoch' yLabel='loss' />
    </ChartCard>
  );
};

const Projection: FC<{ value: unknown }> = ({ value }) => {
  if (!isRecord(value)) return null;
  const x = numArray(value.x);
  const y = numArray(value.y);
  const cluster = numArray(value.cluster);
  if (!x || !y || !cluster || x.length !== y.length) return null;
  return (
    <ChartCard title='Cluster projection (PCA)'>
      <ScatterPlot x={x} y={y} groups={cluster} />
    </ChartCard>
  );
};

const HistogramCard: FC<{ title: string; value: unknown }> = ({ title, value }) => {
  if (!isRecord(value)) return null;
  const edges = numArray(value.edges);
  const counts = numArray(value.counts);
  if (!edges || !counts) return null;
  const threshold = typeof value.threshold === 'number' ? value.threshold : null;
  return (
    <ChartCard title={title}>
      <Histogram edges={edges} counts={counts} threshold={threshold} />
    </ChartCard>
  );
};

const ScorePoints: FC<{ value: unknown }> = ({ value }) => {
  if (!isRecord(value)) return null;
  const score = numArray(value.score);
  const anomaly = Array.isArray(value.anomaly) ? (value.anomaly as boolean[]) : null;
  const index = numArray(value.index);
  if (!score || !anomaly || score.length !== anomaly.length) return null;
  const x = index ?? score.map((_, i) => i);
  return (
    <ChartCard title='Anomaly scores'>
      <ScatterPlot x={x} y={score} highlight={anomaly} />
    </ChartCard>
  );
};

const FqiProgress: FC<{ value: unknown }> = ({ value }) => {
  if (!Array.isArray(value)) return null;
  const points = value.filter(isRecord);
  const iteration = points.map((p) => (typeof p.iteration === 'number' ? p.iteration : NaN));
  const target = points.map((p) => (typeof p.meanTarget === 'number' ? p.meanTarget : NaN));
  if (iteration.some(Number.isNaN) || target.some(Number.isNaN)) return null;
  return (
    <ChartCard title='FQI progress'>
      <LineChart x={iteration} y={target} xLabel='iteration' yLabel='mean target' />
    </ChartCard>
  );
};

const RegressionPoints: FC<{ value: unknown }> = ({ value }) => {
  if (!isRecord(value)) return null;
  const predicted = numArray(value.predicted);
  const actual = numArray(value.actual);
  if (!predicted || !actual || predicted.length !== actual.length) return null;
  return (
    <ChartCard title='Predicted vs actual'>
      <ScatterPlot x={predicted} y={actual} />
    </ChartCard>
  );
};

const Residuals: FC<{ value: unknown }> = ({ value }) => {
  if (!isRecord(value)) return null;
  const predicted = numArray(value.predicted);
  const residual = numArray(value.residual);
  if (!predicted || !residual || predicted.length !== residual.length) return null;
  return (
    <ChartCard title='Residuals'>
      <ScatterPlot x={predicted} y={residual} />
    </ChartCard>
  );
};

const ClusterSizes: FC<{ value: unknown }> = ({ value }) => {
  const sizes = numArray(value);
  if (!sizes) return null;
  return (
    <ChartCard title='Cluster sizes'>
      <BarChart labels={sizes.map((_, i) => `c${i}`)} values={sizes} />
    </ChartCard>
  );
};

type LabChartsProps = {
  charts: Record<string, unknown> | null;
};

const LabCharts: FC<LabChartsProps> = ({ charts }) => {
  if (!charts || Object.keys(charts).length === 0) return null;
  const classes = Array.isArray(charts.classes) ? charts.classes.map(String) : [];
  const matrix = Array.isArray(charts.confusionMatrix)
    ? (charts.confusionMatrix as number[][])
    : null;

  return (
    <div className='grid gap-3 sm:grid-cols-2'>
      {matrix && (
        <ChartCard title='Confusion matrix'>
          <ConfusionMatrix matrix={matrix} classes={classes} />
        </ChartCard>
      )}
      <FeatureImportance value={charts.featureImportance} />
      <MlpProgress value={charts.mlpProgress} />
      <RegressionPoints value={charts.regressionPoints} />
      <Residuals value={charts.residuals} />
      <Projection value={charts.projection} />
      <ClusterSizes value={charts.clusterSizes} />
      <HistogramCard title='Score histogram' value={charts.scoreHistogram} />
      <HistogramCard title='Q-value histogram' value={charts.qValueHistogram} />
      <HistogramCard title='TD-error histogram' value={charts.tdErrorHistogram} />
      <ScorePoints value={charts.scorePoints} />
      <FqiProgress value={charts.fqiProgress} />
    </div>
  );
};

LabCharts.displayName = 'LabCharts';
export default LabCharts;
