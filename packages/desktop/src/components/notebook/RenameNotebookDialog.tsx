import { type FC, useState } from 'react';
import { LuNotebookText, LuX } from 'react-icons/lu';
import { Button, IconButton, TextInput } from '../shared';

type RenameNotebookDialogProps = {
  open: boolean;
  title: string;
  onRename: (title: string) => void;
  onClose: () => void;
};

const RenameNotebookDialog: FC<RenameNotebookDialogProps> = ({
  open,
  title,
  onRename,
  onClose,
}) => {
  const [draft, setDraft] = useState(title);

  if (!open) return null;

  const commit = () => {
    const nextTitle = draft.trim();
    if (nextTitle && nextTitle !== title) onRename(nextTitle);
    onClose();
  };

  return (
    <div
      className='fixed inset-0 z-50 flex items-center justify-center bg-black/60'
      role='dialog'
      aria-modal='true'
      aria-label='Rename notebook'>
      <div className='w-full max-w-sm rounded-xl border border-neutral-700 bg-neutral-900 p-4 shadow-2xl'>
        <div className='mb-3 flex items-center justify-between'>
          <h2 className='flex items-center gap-2 text-sm font-semibold text-neutral-100'>
            <LuNotebookText aria-hidden='true' /> Rename notebook
          </h2>
          <IconButton
            label='Close'
            icon={<LuX aria-hidden='true' className='h-4 w-4' />}
            onClick={onClose}
          />
        </div>
        <TextInput
          autoFocus
          aria-label='Notebook title'
          value={draft}
          onChange={(event) => setDraft(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === 'Enter') {
              event.preventDefault();
              commit();
            }
            if (event.key === 'Escape') {
              event.preventDefault();
              onClose();
            }
          }}
          className='w-full'
        />
        <div className='mt-4 flex justify-end gap-2'>
          <Button variant='ghost' size='sm' onClick={onClose}>
            Cancel
          </Button>
          <Button variant='primary' size='sm' onClick={commit}>
            Rename
          </Button>
        </div>
      </div>
    </div>
  );
};

RenameNotebookDialog.displayName = 'RenameNotebookDialog';

export default RenameNotebookDialog;
