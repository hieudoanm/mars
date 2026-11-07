import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { type FC, useState } from 'react';
import {
  LuChevronDown,
  LuChevronRight,
  LuFolderOpen,
  LuFolderTree,
  LuNotebook,
} from 'react-icons/lu';
import { listNotebookFiles, type FileExplorerEntry } from '../../lib/kernelService';
import { IconButton, SectionHeading } from '../shared';

type FileExplorerProps = {
  currentFilePath: string | null;
  open?: boolean;
  onOpenNotebook: (path: string) => void;
  onStatus: (status: string) => void;
};

const FileExplorer: FC<FileExplorerProps> = ({
  currentFilePath,
  open = true,
  onOpenNotebook,
  onStatus,
}) => {
  const [rootPath, setRootPath] = useState<string | null>(null);
  const [entries, setEntries] = useState<FileExplorerEntry[]>([]);
  const [expanded, setExpanded] = useState<Set<string>>(new Set());

  const openFolder = async () => {
    try {
      const path = await openDialog({
        directory: true,
        multiple: false,
        title: 'Open notebook folder',
      });
      if (!path || Array.isArray(path)) return;
      const nextEntries = await listNotebookFiles(path);
      setRootPath(path);
      setEntries(nextEntries);
      setExpanded(
        new Set(nextEntries.filter((entry) => entry.isDirectory).map((entry) => entry.path)),
      );
      onStatus('Folder opened');
    } catch (error) {
      onStatus(error instanceof Error ? error.message : 'Unable to open folder');
    }
  };

  return (
    <aside
      aria-hidden={!open}
      className={`shrink-0 overflow-hidden bg-neutral-900 transition-[width] duration-300 ease-in-out ${open ? 'w-56 border-r border-neutral-800' : 'w-0'}`}>
      <div className='flex h-full w-56 flex-col px-2 py-4'>
        <div className='mb-3 flex items-center justify-between gap-2'>
          <SectionHeading>Explorer</SectionHeading>
          <IconButton
            label='Open folder'
            icon={<LuFolderOpen aria-hidden='true' className='h-4 w-4' />}
            onClick={() => void openFolder()}
          />
        </div>
        {rootPath ? (
          <p className='mb-3 truncate text-xs text-neutral-500' title={rootPath}>
            {rootPath}
          </p>
        ) : null}
        <div className='min-h-0 flex-1 overflow-y-auto pr-1'>
          {rootPath ? (
            <ExplorerEntries
              entries={entries}
              currentFilePath={currentFilePath}
              expanded={expanded}
              onToggle={(path) =>
                setExpanded((current) => {
                  const next = new Set(current);
                  if (next.has(path)) next.delete(path);
                  else next.add(path);
                  return next;
                })
              }
              onOpenNotebook={onOpenNotebook}
            />
          ) : (
            <p className='px-2 text-sm leading-6 text-neutral-500'>
              Open a folder to browse Mars notebooks.
            </p>
          )}
        </div>
      </div>
    </aside>
  );
};

type ExplorerEntriesProps = {
  entries: FileExplorerEntry[];
  currentFilePath: string | null;
  expanded: Set<string>;
  onToggle: (path: string) => void;
  onOpenNotebook: (path: string) => void;
};

const ExplorerEntries: FC<ExplorerEntriesProps> = ({
  entries,
  currentFilePath,
  expanded,
  onToggle,
  onOpenNotebook,
}) => (
  <ul className='space-y-0.5' aria-label='Notebook files'>
    {entries.map((entry) => {
      const isExpanded = expanded.has(entry.path);
      if (entry.isDirectory) {
        return (
          <li key={entry.path}>
            <button
              type='button'
              className='flex w-full items-center gap-1.5 rounded-md px-2 py-1.5 text-left text-sm text-neutral-300 hover:bg-neutral-800 hover:text-white'
              aria-expanded={isExpanded}
              onClick={() => onToggle(entry.path)}>
              {isExpanded ? (
                <LuChevronDown aria-hidden='true' />
              ) : (
                <LuChevronRight aria-hidden='true' />
              )}
              <LuFolderTree aria-hidden='true' className='h-4 w-4 text-amber-500' />
              <span className='truncate'>{entry.name}</span>
            </button>
            {isExpanded ? (
              <div className='ml-3 border-l border-neutral-800 pl-2'>
                <ExplorerEntries
                  entries={entry.children}
                  currentFilePath={currentFilePath}
                  expanded={expanded}
                  onToggle={onToggle}
                  onOpenNotebook={onOpenNotebook}
                />
              </div>
            ) : null}
          </li>
        );
      }
      return (
        <li key={entry.path}>
          <button
            type='button'
            className={`flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm ${currentFilePath === entry.path ? 'bg-orange-950 text-orange-100' : 'text-neutral-400 hover:bg-neutral-800 hover:text-white'}`}
            aria-current={currentFilePath === entry.path ? 'page' : undefined}
            onClick={() => onOpenNotebook(entry.path)}>
            <LuNotebook aria-hidden='true' className='h-4 w-4 shrink-0 text-orange-500' />
            <span className='truncate'>{entry.name}</span>
          </button>
        </li>
      );
    })}
  </ul>
);

FileExplorer.displayName = 'FileExplorer';
export default FileExplorer;
