import type { FC } from 'react';
import type { LabJobDetail } from '../../lib/kernelService';
import { Badge } from '../shared';

type LabCompareProps = {
  details: LabJobDetail[];
};

const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === 'object' && value !== null && !Array.isArray(value);

const flatten = (value: unknown, prefix: string, out: Map<string, string>) => {
  if (isRecord(value)) {
    for (const [key, entry] of Object.entries(value)) {
      flatten(entry, prefix ? `${prefix}.${key}` : key, out);
    }
    return;
  }
  if (Array.isArray(value)) {
    out.set(prefix, `[${value.length} items]`);
    return;
  }
  out.set(
    prefix,
    typeof value === 'number' ? value.toPrecision(6) : String(value ?? '—'),
  );
};

const LabCompare: FC<LabCompareProps> = ({ details }) => {
  if (details.length === 0) {
    return <p className='p-6 text-sm text-neutral-500'>Loading jobs…</p>;
  }
  const rows = new Map<string, string[]>();
  details.forEach((detail, col) => {
    const flat = new Map<string, string>();
    if (detail.metrics) flatten(detail.metrics.metrics, '', flat);
    for (const [key, value] of flat) {
      const cells = rows.get(key) ?? details.map(() => '—');
      cells[col] = value;
      rows.set(key, cells);
    }
  });

  return (
    <div className='mx-auto max-w-5xl overflow-x-auto p-6'>
      <table className='w-full border-collapse text-sm'>
        <thead>
          <tr className='border-b border-neutral-700'>
            <th className='py-2 pr-4 text-left text-xs font-medium text-neutral-400'>Metric</th>
            {details.map((detail) => (
              <th key={detail.job.id} className='py-2 pr-4 text-left'>
                <div className='text-xs font-medium text-neutral-200'>
                  {detail.job.name ?? detail.job.algorithmId}
                </div>
                <Badge tone={detail.job.status === 'completed' ? 'success' : 'warning'}>
                  {detail.job.status}
                </Badge>
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {[...rows.entries()].map(([key, cells]) => (
            <tr key={key} className='border-b border-neutral-800/60'>
              <td className='py-1.5 pr-4 font-mono text-xs text-neutral-400'>{key}</td>
              {cells.map((cell, i) => (
                <td key={i} className='py-1.5 pr-4 text-neutral-200'>
                  {cell}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
      {rows.size === 0 && (
        <p className='mt-4 text-sm text-neutral-500'>Selected jobs have no metrics to compare.</p>
      )}
    </div>
  );
};

LabCompare.displayName = 'LabCompare';
export default LabCompare;
