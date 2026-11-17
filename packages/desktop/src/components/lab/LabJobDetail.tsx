import type { FC, ReactNode } from 'react';
import type { LabJobDetail as LabJobDetailData } from '../../lib/kernelService';
import { Badge, Button } from '../shared';
import LabCharts from './LabCharts';

type LabJobDetailProps = {
  detail: LabJobDetailData | null;
  onDownload: (jobId: string, filename: string) => void;
};

const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === 'object' && value !== null && !Array.isArray(value);

const formatValue = (value: unknown): string => {
  if (typeof value === 'number') return Number.isInteger(value) ? String(value) : value.toFixed(4);
  if (typeof value === 'boolean' || value === null || value === undefined) return String(value);
  if (Array.isArray(value)) return `[${value.length} items]`;
  if (isRecord(value)) return `{${Object.keys(value).length} keys}`;
  return String(value);
};

const MetricTable: FC<{ metrics: Record<string, unknown> }> = ({ metrics }) => {
  const rows: ReactNode[] = [];
  const walk = (value: unknown, prefix: string, depth: number) => {
    if (isRecord(value)) {
      for (const [key, entry] of Object.entries(value)) {
        const path = prefix ? `${prefix}.${key}` : key;
        if (isRecord(entry) || (Array.isArray(entry) && entry.length > 0 && isRecord(entry[0]))) {
          walk(entry, path, depth + 1);
        } else {
          rows.push(
            <tr key={path} className='border-b border-neutral-800/60 last:border-0'>
              <td className='py-1.5 pr-4 font-mono text-xs text-neutral-400'>{path}</td>
              <td className='py-1.5 text-sm text-neutral-200'>{formatValue(entry)}</td>
            </tr>,
          );
        }
      }
    }
  };
  walk(metrics, '', 0);
  if (rows.length === 0) return <p className='text-sm text-neutral-500'>No metrics recorded.</p>;
  return (
    <table className='w-full'>
      <tbody>{rows}</tbody>
    </table>
  );
};

const LabJobDetail: FC<LabJobDetailProps> = ({ detail, onDownload }) => {
  if (!detail) {
    return <p className='p-6 text-sm text-neutral-500'>Loading job…</p>;
  }
  const { job, metrics, artefacts } = detail;
  return (
    <div className='mx-auto max-w-3xl space-y-5 p-6'>
      <section className='rounded-lg border border-neutral-800 bg-neutral-900/60 p-4'>
        <div className='flex items-center gap-2'>
          <h2 className='text-sm font-semibold text-neutral-200'>{job.name ?? job.algorithmId}</h2>
          <Badge
            tone={
              job.status === 'completed'
                ? 'success'
                : job.status === 'failed'
                  ? 'danger'
                  : 'warning'
            }>
            {job.status}
          </Badge>
        </div>
        <dl className='mt-3 grid grid-cols-2 gap-x-6 gap-y-1 text-xs sm:grid-cols-3'>
          <div>
            <dt className='text-neutral-500'>Task</dt>
            <dd className='text-neutral-300'>{job.taskType}</dd>
          </div>
          <div>
            <dt className='text-neutral-500'>Algorithm</dt>
            <dd className='text-neutral-300'>{job.algorithmId}</dd>
          </div>
          <div>
            <dt className='text-neutral-500'>Created</dt>
            <dd className='text-neutral-300'>{new Date(job.createdAt).toLocaleString()}</dd>
          </div>
          <div>
            <dt className='text-neutral-500'>Features</dt>
            <dd className='text-neutral-300'>{job.featureColumns.join(', ') || '—'}</dd>
          </div>
          <div>
            <dt className='text-neutral-500'>Label</dt>
            <dd className='text-neutral-300'>{job.labelColumn ?? '—'}</dd>
          </div>
          <div>
            <dt className='text-neutral-500'>Split / seed</dt>
            <dd className='text-neutral-300'>
              {job.splitRatio} / {job.randomSeed}
            </dd>
          </div>
        </dl>
        {job.errorMessage && (
          <p className='bg-error/10 text-error mt-3 rounded-md px-3 py-2 text-xs'>
            {job.errorMessage}
          </p>
        )}
      </section>

      {metrics && (
        <section className='rounded-lg border border-neutral-800 bg-neutral-900/60 p-4'>
          <h3 className='mb-2 text-sm font-medium text-neutral-300'>Metrics</h3>
          <MetricTable metrics={metrics.metrics} />
          <div className='mt-4'>
            <LabCharts charts={metrics.charts} />
          </div>
        </section>
      )}

      {artefacts.length > 0 && (
        <section className='rounded-lg border border-neutral-800 bg-neutral-900/60 p-4'>
          <h3 className='mb-2 text-sm font-medium text-neutral-300'>Artefacts</h3>
          <ul className='space-y-2'>
            {artefacts.map((artefact) => (
              <li key={artefact.id} className='flex items-center gap-3'>
                <span className='text-sm text-neutral-200'>{artefact.filename}</span>
                <Badge tone='neutral'>{artefact.format}</Badge>
                <span className='text-xs text-neutral-500'>
                  {(artefact.byteSize / 1024).toFixed(1)} KB
                </span>
                {artefact.format === 'onnx' && (
                  <Button
                    size='sm'
                    variant='secondary'
                    className='ml-auto'
                    onClick={() => onDownload(job.id, artefact.filename)}>
                    Download ONNX
                  </Button>
                )}
              </li>
            ))}
          </ul>
        </section>
      )}
    </div>
  );
};

LabJobDetail.displayName = 'LabJobDetail';
export default LabJobDetail;
