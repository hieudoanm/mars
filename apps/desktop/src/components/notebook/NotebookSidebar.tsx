import { type FC, useRef, useState } from 'react';
import { LuPencil } from 'react-icons/lu';
import type { NotebookCell } from '../../types/types';
import { fileNameFromPath } from '../../lib/filePath';
import { Badge, IconButton, NavButton, SectionHeading, TextInput } from '../shared';

type NotebookSidebarProps = {
  cells: NotebookCell[];
  selectedId: string;
  filePath: string | null;
  isDirty: boolean;
  onSelect: (id: string) => void;
  onRenameFile: (fileName: string) => Promise<boolean>;
};

const NotebookSidebar: FC<NotebookSidebarProps> = ({
  cells,
  selectedId,
  filePath,
  isDirty,
  onSelect,
  onRenameFile,
}) => {
  const fileName = filePath ? fileNameFromPath(filePath) : 'Untitled.mars';
  const [isRenamingFile, setIsRenamingFile] = useState(false);
  const [fileDraft, setFileDraft] = useState(fileName);
  const skipFileBlurRef = useRef(false);

  const startRenamingFile = () => {
    skipFileBlurRef.current = false;
    setFileDraft(fileName);
    setIsRenamingFile(true);
  };

  const cancelRenamingFile = () => {
    skipFileBlurRef.current = true;
    setFileDraft(fileName);
    setIsRenamingFile(false);
  };

  const commitFileRename = async () => {
    if (skipFileBlurRef.current) {
      skipFileBlurRef.current = false;
      return;
    }
    const nextName = fileDraft.trim();
    setIsRenamingFile(false);
    if (!nextName || nextName === fileName) return;
    await onRenameFile(nextName);
  };

  return (
    <aside
      data-component='NotebookSidebar'
      className='w-56 border-r border-neutral-800 bg-neutral-900 px-3 py-6'>
      <SectionHeading>Notebook</SectionHeading>
      <div className='mb-8 flex items-center gap-2 rounded-lg border border-neutral-700 bg-neutral-800 p-3'>
        <img
          className='h-7 w-7 shrink-0 rounded-md object-cover'
          src='/branding/svg/mars-file-logo.svg'
          alt=''
        />
        <div className='min-w-0 flex-1'>
          {isRenamingFile ? (
            <TextInput
              autoFocus
              aria-label='File name'
              value={fileDraft}
              onChange={(event) => setFileDraft(event.target.value)}
              onBlur={() => void commitFileRename()}
              onKeyDown={(event) => {
                if (event.key === 'Enter') {
                  event.preventDefault();
                  event.currentTarget.blur();
                }
                if (event.key === 'Escape') {
                  event.preventDefault();
                  cancelRenamingFile();
                }
              }}
              className='w-full text-sm'
            />
          ) : (
            <div className='flex min-w-0 items-center gap-1'>
              <strong className='block min-w-0 flex-1 truncate text-sm' title={fileName}>
                {fileName}
                {isDirty ? ' •' : ''}
              </strong>
              {filePath ? (
                <IconButton
                  label='Rename file'
                  variant='ghost'
                  className='shrink-0 p-1'
                  icon={<LuPencil aria-hidden='true' className='h-3.5 w-3.5' />}
                  onClick={startRenamingFile}
                />
              ) : null}
            </div>
          )}
          <Badge className='mt-0.5' tone={isDirty ? 'warning' : 'success'}>
            {isDirty ? 'Unsaved changes' : 'Saved notebook'}
          </Badge>
        </div>
      </div>
      <SectionHeading>Cells</SectionHeading>
      {cells.map((cell, index) => (
        <NavButton
          key={cell.id}
          active={cell.id === selectedId}
          index={String(index + 1).padStart(2, '0')}
          onClick={() => onSelect(cell.id)}>
          {cell.cell_type}
        </NavButton>
      ))}
    </aside>
  );
};
NotebookSidebar.displayName = 'NotebookSidebar';
export default NotebookSidebar;
