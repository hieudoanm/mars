import { type FC, useState } from 'react';
import { LuPackage, LuPlus, LuTrash2, LuX } from 'react-icons/lu';
import { IconButton } from '../shared';

type DependenciesDialogProps = {
  open: boolean;
  dependencies: Record<string, string>;
  onSave: (dependencies: Record<string, string>) => void;
  onClose: () => void;
};

const DependenciesDialog: FC<DependenciesDialogProps> = ({
  open,
  dependencies,
  onSave,
  onClose,
}) => {
  const [rows, setRows] = useState<[string, string][]>(() => Object.entries(dependencies));
  const [draftName, setDraftName] = useState('');
  const [draftVersion, setDraftVersion] = useState('');

  if (!open) return null;

  const addRow = () => {
    const name = draftName.trim();
    if (!name) return;
    setRows((current) => [
      ...current.filter(([n]) => n !== name),
      [name, draftVersion.trim() || '*'],
    ]);
    setDraftName('');
    setDraftVersion('');
  };

  const save = () => {
    onSave(Object.fromEntries(rows.filter(([name]) => name.trim())));
    onClose();
  };

  return (
    <div
      className='fixed inset-0 z-50 flex items-center justify-center bg-black/60'
      role='dialog'
      aria-modal='true'
      aria-label='Notebook dependencies'>
      <div className='w-full max-w-md rounded-xl border border-neutral-700 bg-neutral-900 p-4 shadow-2xl'>
        <div className='mb-3 flex items-center justify-between'>
          <h2 className='flex items-center gap-2 text-sm font-semibold text-neutral-100'>
            <LuPackage aria-hidden='true' /> Cargo dependencies
          </h2>
          <IconButton
            label='Close'
            icon={<LuX aria-hidden='true' className='h-4 w-4' />}
            onClick={onClose}
          />
        </div>
        <p className='mb-3 text-xs text-neutral-500'>
          Crates added here are written to the notebook manifest and used when the runtime compiles
          cells.
        </p>
        <ul className='mb-3 space-y-1'>
          {rows.map(([name, version], index) => (
            <li
              key={name}
              className='flex items-center gap-2 rounded-md bg-neutral-950 px-2 py-1.5 text-sm'>
              <span className='font-mono text-neutral-200'>{name}</span>
              <span className='text-neutral-500'>@</span>
              <input
                aria-label={`Version for ${name}`}
                value={version}
                className='min-w-0 flex-1 bg-transparent font-mono text-xs text-neutral-400 outline-none'
                onChange={(event) =>
                  setRows((current) =>
                    current.map((row, i) => (i === index ? [row[0], event.target.value] : row)),
                  )
                }
              />
              <IconButton
                label={`Remove ${name}`}
                variant='danger'
                icon={<LuTrash2 aria-hidden='true' className='h-3.5 w-3.5' />}
                onClick={() => setRows((current) => current.filter((_, i) => i !== index))}
              />
            </li>
          ))}
          {rows.length === 0 ? (
            <li className='text-xs text-neutral-600'>No dependencies.</li>
          ) : null}
        </ul>
        <div className='mb-3 flex gap-2'>
          <input
            aria-label='Dependency name'
            placeholder='crate-name'
            value={draftName}
            className='w-2/5 rounded-md border border-neutral-700 bg-neutral-950 px-2 py-1.5 font-mono text-xs text-neutral-100 outline-none'
            onChange={(event) => setDraftName(event.target.value)}
            onKeyDown={(event) => event.key === 'Enter' && addRow()}
          />
          <input
            aria-label='Dependency version'
            placeholder='version (e.g. 1.0)'
            value={draftVersion}
            className='min-w-0 flex-1 rounded-md border border-neutral-700 bg-neutral-950 px-2 py-1.5 font-mono text-xs text-neutral-100 outline-none'
            onChange={(event) => setDraftVersion(event.target.value)}
            onKeyDown={(event) => event.key === 'Enter' && addRow()}
          />
          <IconButton
            label='Add dependency'
            icon={<LuPlus aria-hidden='true' className='h-4 w-4' />}
            onClick={addRow}
          />
        </div>
        <div className='flex justify-end gap-2'>
          <button
            type='button'
            className='rounded-md px-3 py-1.5 text-sm text-neutral-400 hover:bg-neutral-800'
            onClick={onClose}>
            Cancel
          </button>
          <button
            type='button'
            className='rounded-md bg-orange-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-orange-500'
            onClick={save}>
            Save dependencies
          </button>
        </div>
      </div>
    </div>
  );
};

DependenciesDialog.displayName = 'DependenciesDialog';
export default DependenciesDialog;
