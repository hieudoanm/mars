import { type FC } from 'react';
import { Badge } from '../shared';
import type { KernelInfo, RunSummary } from '../../hooks/useKernelExecution';

type NotebookStatusBarProps = {
  cellCount: number;
  isDirty: boolean;
  kernelInfo?: KernelInfo | null;
  isExecuting?: boolean;
  sessionId?: string;
  queueLength?: number;
  runSummary?: RunSummary | null;
  onShowRunSummary?: () => void;
  mode?: 'edit' | 'command';
};

const NotebookStatusBar: FC<NotebookStatusBarProps> = ({
  cellCount,
  isDirty,
  kernelInfo = null,
  isExecuting = false,
  sessionId = '',
  queueLength = 0,
  runSummary = null,
  onShowRunSummary,
  mode = 'command',
}) => {
  const shortSession = sessionId ? sessionId.slice(0, 8) : '';
  return (
    <footer
      data-component='NotebookStatusBar'
      className='flex gap-5 border-t border-neutral-800 bg-neutral-900 px-3 py-1.5 text-xs text-neutral-400'>
      <span>Rust 2024</span>
      <Badge
        dot
        dotTone={isExecuting ? 'warning' : 'success'}
        title={sessionId ? `Kernel session ${sessionId}` : undefined}>
        Kernel {isExecuting ? 'running' : (kernelInfo?.status.toLowerCase() ?? 'idle')}
        {kernelInfo ? ` · ${kernelInfo.executionCount} executions` : ''}
        {shortSession ? ` · ${shortSession}` : ''}
        {queueLength > 0 ? ` · ${queueLength} queued` : ''}
      </Badge>
      <span>{cellCount} cells</span>
      {runSummary ? (
        <button
          type='button'
          onClick={onShowRunSummary}
          title='Show run summary'
          className='cursor-pointer rounded px-1 hover:bg-neutral-800 hover:text-neutral-200'>
          Last run: {runSummary.entries.length} cells ·{' '}
          {runSummary.executionMs >= 1000
            ? `${(runSummary.executionMs / 1000).toFixed(1)}s`
            : `${Math.round(runSummary.executionMs)}ms`}
          {runSummary.entries.some((entry) => entry.status !== 'success') ? ' · issues' : ''}
        </button>
      ) : null}
      <Badge tone={mode === 'edit' ? 'success' : 'neutral'}>
        {mode === 'edit' ? 'Edit' : 'Command'}
      </Badge>
      <Badge tone={isDirty ? 'warning' : 'success'}>
        {isDirty ? 'Unsaved changes' : 'All changes saved'}
      </Badge>
    </footer>
  );
};
NotebookStatusBar.displayName = 'NotebookStatusBar';
export default NotebookStatusBar;
