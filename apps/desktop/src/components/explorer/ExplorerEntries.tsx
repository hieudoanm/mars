import { type FC } from 'react';
import {
  LuChevronDown,
  LuChevronRight,
  LuFileCode2,
  LuFolderTree,
  LuNotebook,
  LuPencil,
  LuTrash2,
} from 'react-icons/lu';
import type { FileExplorerEntry } from '../../lib/kernelService';
import { IconButton } from '../shared';

export type ExplorerEntriesProps = {
  entries: FileExplorerEntry[];
  currentFilePath: string | null;
  expanded: Set<string>;
  renamingPath: string | null;
  renameDraft: string;
  onRenameDraft: (value: string) => void;
  onStartRename: (entry: FileExplorerEntry) => void;
  onCommitRename: (entry: FileExplorerEntry) => void;
  onCancelRename: () => void;
  onDelete: (entry: FileExplorerEntry) => void;
  onToggle: (path: string) => void;
  onOpenNotebook: (path: string) => void;
  onOpenFile: (path: string) => void;
};

const isNotebook = (name: string) => name.toLowerCase().endsWith('.mars');

const entryActions =
  'flex shrink-0 items-center gap-0.5 opacity-0 transition-opacity group-hover:opacity-100';

export const ExplorerEntries: FC<ExplorerEntriesProps> = ({
  entries,
  currentFilePath,
  expanded,
  renamingPath,
  renameDraft,
  onRenameDraft,
  onStartRename,
  onCommitRename,
  onCancelRename,
  onDelete,
  onToggle,
  onOpenNotebook,
  onOpenFile,
}) => (
  <ul className='space-y-0.5' aria-label='Notebook files'>
    {entries.map((entry) => {
      const isExpanded = expanded.has(entry.path);
      const renaming = renamingPath === entry.path;
      return (
        <li key={entry.path} className='group'>
          {renaming ? (
            <input
              autoFocus
              value={renameDraft}
              aria-label={`Rename ${entry.name}`}
              className='w-full rounded-md border border-orange-700 bg-neutral-950 px-2 py-1 text-sm text-neutral-100'
              onChange={(event) => onRenameDraft(event.target.value)}
              onBlur={() => onCommitRename(entry)}
              onKeyDown={(event) => {
                if (event.key === 'Enter') onCommitRename(entry);
                if (event.key === 'Escape') onCancelRename();
              }}
            />
          ) : entry.isDirectory ? (
            <div className='flex items-center'>
              <button
                type='button'
                className='flex min-w-0 flex-1 items-center gap-1.5 rounded-md px-2 py-1.5 text-left text-sm text-neutral-300 hover:bg-neutral-800 hover:text-white'
                aria-expanded={isExpanded}
                onClick={() => onToggle(entry.path)}>
                {isExpanded ? (
                  <LuChevronDown aria-hidden='true' />
                ) : (
                  <LuChevronRight aria-hidden='true' />
                )}
                <LuFolderTree aria-hidden='true' className='h-4 w-4 shrink-0 text-amber-500' />
                <span className='truncate'>{entry.name}</span>
              </button>
              <span className={entryActions}>
                <IconButton
                  label={`Rename ${entry.name}`}
                  icon={<LuPencil aria-hidden='true' className='h-3 w-3' />}
                  onClick={() => onStartRename(entry)}
                />
                <IconButton
                  label={`Delete ${entry.name}`}
                  variant='danger'
                  icon={<LuTrash2 aria-hidden='true' className='h-3 w-3' />}
                  onClick={() => onDelete(entry)}
                />
              </span>
            </div>
          ) : (
            <div className='flex items-center'>
              <button
                type='button'
                className={`flex min-w-0 flex-1 items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm ${currentFilePath === entry.path ? 'bg-orange-950 text-orange-100' : 'text-neutral-400 hover:bg-neutral-800 hover:text-white'}`}
                aria-current={currentFilePath === entry.path ? 'page' : undefined}
                onClick={() =>
                  isNotebook(entry.name) ? onOpenNotebook(entry.path) : onOpenFile(entry.path)
                }>
                {isNotebook(entry.name) ? (
                  <LuNotebook aria-hidden='true' className='h-4 w-4 shrink-0 text-orange-500' />
                ) : (
                  <LuFileCode2 aria-hidden='true' className='h-4 w-4 shrink-0 text-sky-400' />
                )}
                <span className='truncate'>{entry.name}</span>
              </button>
              <span className={entryActions}>
                <IconButton
                  label={`Rename ${entry.name}`}
                  icon={<LuPencil aria-hidden='true' className='h-3 w-3' />}
                  onClick={() => onStartRename(entry)}
                />
                <IconButton
                  label={`Delete ${entry.name}`}
                  variant='danger'
                  icon={<LuTrash2 aria-hidden='true' className='h-3 w-3' />}
                  onClick={() => onDelete(entry)}
                />
              </span>
            </div>
          )}
          {entry.isDirectory && isExpanded && !renaming ? (
            <div className='ml-3 border-l border-neutral-800 pl-2'>
              <ExplorerEntries
                entries={entry.children}
                currentFilePath={currentFilePath}
                expanded={expanded}
                renamingPath={renamingPath}
                renameDraft={renameDraft}
                onRenameDraft={onRenameDraft}
                onStartRename={onStartRename}
                onCommitRename={onCommitRename}
                onCancelRename={onCancelRename}
                onDelete={onDelete}
                onToggle={onToggle}
                onOpenNotebook={onOpenNotebook}
                onOpenFile={onOpenFile}
              />
            </div>
          ) : null}
        </li>
      );
    })}
  </ul>
);

ExplorerEntries.displayName = 'ExplorerEntries';
