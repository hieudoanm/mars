import type { FC } from 'react';
import type { LabJob } from '../../lib/kernelService';
import { Badge } from '../shared';

type LabJobListProps = {
  jobs: LabJob[];
  onSelect: (jobId: string) => void;
  compareIds?: Set<string>;
  onToggleCompare?: (jobId: string) => void;
};

const statusTone = (status: string): 'success' | 'warning' | 'danger' | 'neutral' => {
  if (status === 'completed') return 'success';
  if (status === 'failed' || status === 'cancelled') return 'danger';
  if (status === 'queued' || status === 'training' || status === 'exporting') return 'warning';
  return 'neutral';
};

const LabJobList: FC<LabJobListProps> = ({ jobs, onSelect, compareIds, onToggleCompare }) => {
  const comparing = onToggleCompare !== undefined;
  if (jobs.length === 0) {
    return (
      <p className='rounded-lg border border-dashed border-neutral-800 p-4 text-sm text-neutral-500'>
        No training jobs yet.
      </p>
    );
  }
  return (
    <ul className='space-y-2'>
      {jobs.map((job) => {
        const selected = compareIds?.has(job.id) ?? false;
        return (
          <li key={job.id} className='flex items-stretch gap-2'>
            {comparing && (
              <input
                type='checkbox'
                aria-label={`Compare ${job.name ?? job.algorithmId}`}
                checked={selected}
                onChange={() => onToggleCompare(job.id)}
                className='mt-3 size-4 shrink-0 accent-orange-600'
              />
            )}
            <button
              type='button'
              onClick={() => onSelect(job.id)}
              className={`w-full rounded-lg border px-3 py-2 text-left transition-colors hover:border-neutral-700 hover:bg-neutral-900 ${
                selected ? 'border-orange-700 bg-orange-950/30' : 'border-neutral-800 bg-neutral-900/60'
              }`}>
              <div className='flex items-center gap-2'>
                <span className='truncate text-sm text-neutral-200'>
                  {job.name ?? job.algorithmId}
                </span>
                <Badge tone={statusTone(job.status)}>{job.status}</Badge>
                <span className='ml-auto text-xs text-neutral-500'>{job.taskType}</span>
              </div>
              <p className='mt-1 text-xs text-neutral-500'>
                {job.algorithmId} · {job.featureColumns.length} features
                {job.labelColumn ? ` · label: ${job.labelColumn}` : ''} ·{' '}
                {new Date(job.createdAt).toLocaleString()}
              </p>
              {job.errorMessage && (
                <p className='mt-1 truncate text-xs text-error'>{job.errorMessage}</p>
              )}
            </button>
          </li>
        );
      })}
    </ul>
  );
};

LabJobList.displayName = 'LabJobList';
export default LabJobList;
