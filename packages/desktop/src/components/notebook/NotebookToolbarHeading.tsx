import { type FC } from 'react';
import { LuNotebookText } from 'react-icons/lu';

export type NotebookToolbarHeadingProps = {
  fileName: string;
  isDirty: boolean;
};

const NotebookToolbarHeading: FC<NotebookToolbarHeadingProps> = ({ fileName, isDirty }) => (
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
  </header>
);

NotebookToolbarHeading.displayName = 'NotebookToolbarHeading';

export default NotebookToolbarHeading;
