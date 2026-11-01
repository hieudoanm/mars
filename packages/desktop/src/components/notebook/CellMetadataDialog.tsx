import { type FC, useEffect, useState } from 'react';
import type { NotebookCell } from '../../types/types';
import { Button, Dialog } from '../shared';

type CellMetadataDialogProps = {
  cell: NotebookCell | null;
  onClose: () => void;
  onSave: (metadata: Record<string, unknown>) => void;
};

const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === 'object' && value !== null && !Array.isArray(value);

/** Jupyter-style raw JSON editor for a cell's metadata object. */
const CellMetadataDialog: FC<CellMetadataDialogProps> = ({ cell, onClose, onSave }) => {
  const [draft, setDraft] = useState('');
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (cell) {
      setDraft(JSON.stringify(cell.metadata, null, 2));
      setError(null);
    }
  }, [cell]);

  if (!cell) return null;

  const save = () => {
    let parsed: unknown;
    try {
      parsed = JSON.parse(draft);
    } catch {
      setError('Invalid JSON');
      return;
    }
    if (!isRecord(parsed)) {
      setError('Metadata must be a JSON object');
      return;
    }
    onSave(parsed);
    onClose();
  };

  return (
    <Dialog labelledBy='cell-metadata-title' className='max-w-lg'>
      <h2 id='cell-metadata-title' className='mb-3 text-sm font-semibold text-neutral-100'>
        Edit cell metadata
      </h2>
      <textarea
        aria-label='Cell metadata JSON'
        value={draft}
        onChange={(event) => setDraft(event.target.value)}
        spellCheck={false}
        className='h-56 w-full rounded border border-neutral-700 bg-neutral-950 p-2 font-mono text-xs text-neutral-200 outline-none focus:border-neutral-500'
      />
      {error ? <p className='mt-2 text-xs text-red-400'>{error}</p> : null}
      <div className='mt-4 flex justify-end gap-2'>
        <Button variant='ghost' onClick={onClose}>
          Cancel
        </Button>
        <Button onClick={save}>Save metadata</Button>
      </div>
    </Dialog>
  );
};

CellMetadataDialog.displayName = 'CellMetadataDialog';
export default CellMetadataDialog;
