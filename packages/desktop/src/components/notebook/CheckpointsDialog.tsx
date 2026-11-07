import { type FC, useCallback, useEffect, useMemo, useState } from 'react';
import { LuHistory, LuX } from 'react-icons/lu';
import {
  createNotebookCheckpoint,
  listNotebookCheckpoints,
  readNotebookCheckpoint,
  restoreNotebookCheckpoint,
} from '../../lib/kernelService';
import { diffLines } from '../../lib/textDiff';
import { computeCellDiffs } from '../../lib/notebookCellDiff';
import { fileNameFromPath } from '../../lib/filePath';
import { IconButton } from '../shared';

type CheckpointsDialogProps = {
  open: boolean;
  filePath: string | null;
  currentContent: string;
  onRestored: () => void;
  onStatus: (status: string) => void;
  onClose: () => void;
};

const formatCheckpointName = (path: string): string => {
  const name = fileNameFromPath(path);
  const match = name.match(/checkpoint-(\d+)\.mars$/);
  if (!match) return name;
  const millis = Number(match[1]);
  const date = new Date(millis);
  return Number.isNaN(date.getTime()) ? name : date.toLocaleString();
};

const CheckpointsDialog: FC<CheckpointsDialogProps> = ({
  open,
  filePath,
  currentContent,
  onRestored,
  onStatus,
  onClose,
}) => {
  const [checkpoints, setCheckpoints] = useState<string[]>([]);
  const [busy, setBusy] = useState(false);
  const [diffTarget, setDiffTarget] = useState<string | null>(null);
  const [diffContent, setDiffContent] = useState<string | null>(null);
  const [selectedCellDiff, setSelectedCellDiff] = useState<number | null>(null);

  const refresh = useCallback(async () => {
    if (!filePath) {
      setCheckpoints([]);
      return;
    }
    try {
      setCheckpoints(await listNotebookCheckpoints(filePath));
    } catch {
      setCheckpoints([]);
    }
  }, [filePath]);

  useEffect(() => {
    if (open) void refresh();
  }, [open, refresh]);

  useEffect(() => {
    if (!open) {
      setDiffTarget(null);
      setDiffContent(null);
    }
  }, [open]);

  if (!open) return null;

  const create = async () => {
    if (!filePath) return;
    setBusy(true);
    try {
      await createNotebookCheckpoint(filePath);
      onStatus('Checkpoint created');
      await refresh();
    } catch (error) {
      onStatus(error instanceof Error ? error.message : 'Checkpoint failed');
    } finally {
      setBusy(false);
    }
  };

  const restore = async (checkpointPath: string) => {
    if (!filePath) return;
    setBusy(true);
    try {
      await restoreNotebookCheckpoint(filePath, checkpointPath);
      onStatus('Checkpoint restored');
      onRestored();
      onClose();
    } catch (error) {
      onStatus(error instanceof Error ? error.message : 'Restore failed');
    } finally {
      setBusy(false);
    }
  };

  const showDiff = async (checkpointPath: string) => {
    if (!filePath) return;
    setBusy(true);
    try {
      const content = await readNotebookCheckpoint(filePath, checkpointPath);
      setDiffTarget(checkpointPath);
      setDiffContent(content);
      setSelectedCellDiff(null);
    } catch (error) {
      onStatus(error instanceof Error ? error.message : 'Diff failed');
    } finally {
      setBusy(false);
    }
  };

  const diffRows = useMemo(
    () => (diffContent === null ? null : diffLines(diffContent, currentContent)),
    [diffContent, currentContent],
  );
  const cellDiffs = useMemo(
    () => (diffContent === null ? [] : computeCellDiffs(diffContent, currentContent)),
    [diffContent, currentContent],
  );
  const activeDiffRows =
    selectedCellDiff !== null ? (cellDiffs[selectedCellDiff]?.rows ?? diffRows) : diffRows;
  const changedCount = diffRows?.filter((row) => row.type !== 'same').length ?? 0;

  return (
    <div
      className='fixed inset-0 z-50 flex items-center justify-center bg-black/60'
      role='dialog'
      aria-modal='true'
      aria-label='Notebook checkpoints'>
      <div
        className={`rounded-xl border border-neutral-700 bg-neutral-900 p-4 shadow-2xl ${
          diffRows ? 'w-full max-w-3xl' : 'w-full max-w-md'
        }`}>
        <div className='mb-3 flex items-center justify-between'>
          <h2 className='flex items-center gap-2 text-sm font-semibold text-neutral-100'>
            <LuHistory aria-hidden='true' /> Checkpoints
          </h2>
          <IconButton
            label='Close'
            icon={<LuX aria-hidden='true' className='h-4 w-4' />}
            onClick={onClose}
          />
        </div>
        {!filePath ? (
          <p className='text-sm text-neutral-500'>Save the notebook before creating checkpoints.</p>
        ) : (
          <>
            <button
              type='button'
              disabled={busy}
              className='mb-3 w-full rounded-md bg-orange-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-orange-500 disabled:opacity-50'
              onClick={() => void create()}>
              Create checkpoint now
            </button>
            <ul className='max-h-64 space-y-1 overflow-y-auto'>
              {checkpoints.length === 0 ? (
                <li className='text-xs text-neutral-600'>No checkpoints yet.</li>
              ) : (
                [...checkpoints].reverse().map((checkpoint) => (
                  <li
                    key={checkpoint}
                    className='flex items-center justify-between gap-2 rounded-md bg-neutral-950 px-2 py-1.5 text-sm'>
                    <span className='text-neutral-300'>{formatCheckpointName(checkpoint)}</span>
                    <span className='flex items-center gap-1'>
                      <button
                        type='button'
                        disabled={busy}
                        className='rounded px-2 py-0.5 text-xs text-neutral-400 hover:bg-neutral-800 disabled:opacity-50'
                        onClick={() => void showDiff(checkpoint)}>
                        Diff
                      </button>
                      <button
                        type='button'
                        disabled={busy}
                        className='rounded px-2 py-0.5 text-xs text-orange-400 hover:bg-neutral-800 disabled:opacity-50'
                        onClick={() => void restore(checkpoint)}>
                        Restore
                      </button>
                    </span>
                  </li>
                ))
              )}
            </ul>
            {diffRows ? (
              <div className='mt-3'>
                <div className='mb-1 flex items-center justify-between text-xs text-neutral-500'>
                  <span>{diffTarget ? formatCheckpointName(diffTarget) : ''} → current</span>
                  <span>{changedCount} changed line(s)</span>
                </div>
                {cellDiffs.length > 0 ? (
                  <ul className='mb-2 flex flex-wrap gap-1' aria-label='Changed cells'>
                    {cellDiffs.map((cellDiff, index) => (
                      <li key={cellDiff.cellId}>
                        <button
                          type='button'
                          className={`rounded border px-1.5 py-0.5 text-[10px] ${
                            selectedCellDiff === index
                              ? 'border-orange-700 bg-orange-950/50 text-orange-200'
                              : 'border-neutral-700 text-neutral-400 hover:bg-neutral-800'
                          }`}
                          onClick={() =>
                            setSelectedCellDiff((current) => (current === index ? null : index))
                          }>
                          Cell {cellDiff.index + 1} · {cellDiff.cellType} · +{cellDiff.added} -
                          {cellDiff.removed}
                        </button>
                      </li>
                    ))}
                  </ul>
                ) : null}
                {selectedCellDiff !== null ? (
                  <button
                    type='button'
                    className='mb-1 text-[10px] text-neutral-500 underline hover:text-neutral-300'
                    onClick={() => setSelectedCellDiff(null)}>
                    Show full notebook diff
                  </button>
                ) : null}
                <pre className='max-h-72 overflow-auto rounded-md border border-neutral-800 bg-neutral-950 p-2 font-mono text-xs leading-5'>
                  {(activeDiffRows ?? []).map((row, index) => (
                    <div
                      key={index}
                      className={
                        row.type === 'add'
                          ? 'bg-green-950/60 text-green-300'
                          : row.type === 'del'
                            ? 'bg-red-950/60 text-red-300'
                            : 'text-neutral-500'
                      }>
                      <span className='mr-2 inline-block w-3 select-none'>
                        {row.type === 'add' ? '+' : row.type === 'del' ? '-' : ' '}
                      </span>
                      {row.text || ' '}
                    </div>
                  ))}
                </pre>
              </div>
            ) : null}
          </>
        )}
      </div>
    </div>
  );
};

CheckpointsDialog.displayName = 'CheckpointsDialog';
export default CheckpointsDialog;
