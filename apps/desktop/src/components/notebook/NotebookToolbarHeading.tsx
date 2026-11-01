import { type FC, useRef, useState } from 'react';
import { LuNotebookText, LuPencil } from 'react-icons/lu';
import { IconButton, TextInput } from '../shared';

export type NotebookToolbarHeadingProps = {
  fileName: string;
  title: string;
  isDirty: boolean;
  onRename: (title: string) => void;
};

const NotebookToolbarHeading: FC<NotebookToolbarHeadingProps> = ({
  fileName,
  title,
  isDirty,
  onRename,
}) => {
  const [isEditing, setIsEditing] = useState(false);
  const [draft, setDraft] = useState(title);
  const skipBlurRef = useRef(false);

  const startEditing = () => {
    skipBlurRef.current = false;
    setDraft(title);
    setIsEditing(true);
  };

  const commit = () => {
    const nextTitle = draft.trim();
    if (nextTitle && nextTitle !== title) onRename(nextTitle);
    setIsEditing(false);
  };

  const cancel = () => {
    skipBlurRef.current = true;
    setDraft(title);
    setIsEditing(false);
  };

  return (
    <header data-component='NotebookToolbarHeading' className='min-w-0'>
      <div className='flex min-w-0 items-center gap-1.5 text-xs tracking-wide text-neutral-400'>
        <LuNotebookText aria-hidden='true' className='h-3.5 w-3.5 shrink-0 text-neutral-500' />
        <span className='shrink-0'>Workspace</span>
        <span aria-hidden='true' className='text-neutral-600'>
          /
        </span>
        <span className='truncate text-neutral-300' title={fileName}>
          {fileName}
        </span>
        {isDirty ? (
          <span
            aria-hidden='true'
            title='Unsaved changes'
            data-component='NotebookDirtyIndicator'
            className='h-1.5 w-1.5 shrink-0 rounded-full bg-orange-400'
          />
        ) : null}
      </div>
      {isEditing ? (
        <div className='mt-1 flex min-w-0 items-center gap-2'>
          <TextInput
            autoFocus
            aria-label='Notebook title'
            value={draft}
            onChange={(event) => setDraft(event.target.value)}
            onBlur={() => {
              if (skipBlurRef.current) {
                skipBlurRef.current = false;
                return;
              }
              commit();
            }}
            onKeyDown={(event) => {
              if (event.key === 'Enter') {
                event.preventDefault();
                event.currentTarget.blur();
              }
              if (event.key === 'Escape') {
                event.preventDefault();
                cancel();
              }
            }}
            className='w-full text-lg font-medium tracking-tight'
          />
        </div>
      ) : (
        <div className='mt-1 flex min-w-0 items-center gap-1'>
          <h2
            className='min-w-0 flex-1 truncate text-lg font-medium tracking-tight text-neutral-100'
            title={title}>
            {title}
          </h2>
          <IconButton
            label='Rename notebook'
            variant='ghost'
            className='shrink-0'
            icon={<LuPencil aria-hidden='true' className='h-4 w-4' />}
            onClick={startEditing}
          />
        </div>
      )}
    </header>
  );
};

NotebookToolbarHeading.displayName = 'NotebookToolbarHeading';

export default NotebookToolbarHeading;
