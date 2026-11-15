import { type FC, useMemo } from 'react';
import { LuCheck, LuCircleAlert, LuPause, LuX } from 'react-icons/lu';
import type { NotebookCell } from '../../types/types';
import type { RunSummary } from '../../hooks/useKernelExecution';
import { Dialog, IconButton } from '../shared';
import { cellLanguage } from '../../lib/cellLanguage';

type RunSummaryDialogProps = {
  open: boolean;
  summary: RunSummary | null;
  cells: NotebookCell[];
  notebookLanguage: string;
  onClose: () => void;
};

const STATUS_META = {
  success: { icon: LuCheck, className: 'text-emerald-400', label: 'succeeded' },
  error: { icon: LuCircleAlert, className: 'text-red-400', label: 'failed' },
  interrupted: { icon: LuPause, className: 'text-amber-400', label: 'interrupted' },
} as const;

const formatMs = (ms: number): string =>
  ms >= 1000 ? `${(ms / 1000).toFixed(2)}s` : `${Math.round(ms)}ms`;

const cellPreview = (source: string): string => {
  const first = source.split('\n').find((line) => line.trim().length > 0) ?? '';
  return first.length > 60 ? `${first.slice(0, 57)}…` : first;
};

/// Modal showing the most recent batch run: per-cell status and duration,
/// sorted slowest-first, with aggregate totals.
const RunSummaryDialog: FC<RunSummaryDialogProps> = ({
  open,
  summary,
  cells,
  notebookLanguage,
  onClose,
}) => {
  const cellIndex = useMemo(() => {
    const map = new Map<string, number>();
    cells.forEach((cell, index) => map.set(cell.id, index));
    return map;
  }, [cells]);

  if (!open || !summary) return null;

  const entries = [...summary.entries].sort((a, b) => b.durationMs - a.durationMs);
  const counts = entries.reduce(
    (acc, entry) => {
      acc[entry.status] += 1;
      return acc;
    },
    { success: 0, error: 0, interrupted: 0 } as Record<RunSummary['entries'][number]['status'], number>,
  );
  const slowest = entries[0];

  return (
    <Dialog labelledBy='run-summary-title' className='max-w-2xl'>
      <div className='mb-3 flex items-center justify-between'>
        <h2 id='run-summary-title' className='text-sm font-semibold text-neutral-100'>
          Run summary
        </h2>
        <IconButton
          label='Close'
          icon={<LuX aria-hidden='true' className='h-4 w-4' />}
          onClick={onClose}
        />
      </div>
      <p className='mb-3 text-xs text-neutral-400'>
        {entries.length} cells · {counts.success} succeeded
        {counts.error > 0 ? ` · ${counts.error} failed` : ''}
        {counts.interrupted > 0 ? ` · ${counts.interrupted} interrupted` : ''} · execution{' '}
        {formatMs(summary.executionMs)} · wall {formatMs(summary.wallMs)} · finished{' '}
        {new Date(summary.finishedAt).toLocaleTimeString()}
      </p>
      {slowest && entries.length > 1 ? (
        <p className='mb-3 text-xs text-neutral-500'>
          Slowest cell:{' '}
          <span className='text-neutral-300'>
            #{(cellIndex.get(slowest.cellId) ?? 0) + 1} · {formatMs(slowest.durationMs)}
          </span>
        </p>
      ) : null}
      <ul className='max-h-80 space-y-1 overflow-y-auto'>
        {entries.map((entry) => {
          const meta = STATUS_META[entry.status];
          const Icon = meta.icon;
          const index = cellIndex.get(entry.cellId);
          const cell = index !== undefined ? cells[index] : undefined;
          const language = cell ? cellLanguage(cell, notebookLanguage) : '?';
          return (
            <li
              key={entry.cellId}
              className='flex items-center gap-2 rounded-md bg-neutral-800/60 px-2 py-1.5 text-xs'>
              <Icon aria-hidden='true' className={`h-3.5 w-3.5 shrink-0 ${meta.className}`} />
              <span className='w-10 shrink-0 text-neutral-500'>
                #{index !== undefined ? index + 1 : '?'}
              </span>
              <span className='w-14 shrink-0 text-neutral-400'>{language}</span>
              <span className='min-w-0 flex-1 truncate text-neutral-300'>
                {cell ? cellPreview(cell.source) : entry.cellId}
              </span>
              <span className='shrink-0 text-neutral-500'>{meta.label}</span>
              <span className='w-16 shrink-0 text-right text-neutral-400'>
                {formatMs(entry.durationMs)}
              </span>
            </li>
          );
        })}
      </ul>
    </Dialog>
  );
};

RunSummaryDialog.displayName = 'RunSummaryDialog';
export default RunSummaryDialog;
