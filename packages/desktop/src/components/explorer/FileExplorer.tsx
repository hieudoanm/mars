import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { type FC, useEffect, useState } from 'react';
import { LuFilePlus, LuFolderOpen, LuFolderPlus } from 'react-icons/lu';
import {
  createDirectory,
  openPathExternal,
  createNotebookFile,
  deletePath,
  listNotebookFiles,
  renameNotebookFile,
  type FileExplorerEntry,
} from '../../lib/kernelService';
import { IconButton, SectionHeading } from '../shared';
import { ExplorerEntries } from './ExplorerEntries';

type FileExplorerProps = {
  currentFilePath: string | null;
  open?: boolean;
  requestedRootPath?: string | null;
  onOpenNotebook: (path: string) => void;
  onStatus: (status: string) => void;
};

const joinPath = (parent: string, name: string) => `${parent.replace(/\/+$/, '')}/${name}`;

const uniqueName = (entries: FileExplorerEntry[], base: string) => {
  const names = new Set(entries.map((entry) => entry.name));
  if (!names.has(base)) return base;
  let index = 2;
  while (names.has(`${base}-${index}`)) index += 1;
  return `${base}-${index}`;
};

const FileExplorer: FC<FileExplorerProps> = ({
  currentFilePath,
  open = true,
  requestedRootPath = null,
  onOpenNotebook,
  onStatus,
}) => {
  const [rootPath, setRootPath] = useState<string | null>(null);
  const [entries, setEntries] = useState<FileExplorerEntry[]>([]);
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  const [renamingPath, setRenamingPath] = useState<string | null>(null);
  const [renameDraft, setRenameDraft] = useState('');

  const refresh = async (root: string) => {
    const nextEntries = await listNotebookFiles(root);
    setEntries(nextEntries);
    setExpanded(
      new Set(nextEntries.filter((entry) => entry.isDirectory).map((entry) => entry.path)),
    );
  };

  useEffect(() => {
    if (!requestedRootPath || requestedRootPath === rootPath) return;
    setRootPath(requestedRootPath);
    void refresh(requestedRootPath)
      .then(() => onStatus('Folder opened'))
      .catch((error: unknown) =>
        onStatus(error instanceof Error ? error.message : 'Unable to open folder'),
      );
  }, [requestedRootPath]);

  const openFolder = async () => {
    try {
      const path = await openDialog({
        directory: true,
        multiple: false,
        title: 'Open notebook folder',
      });
      if (!path || Array.isArray(path)) return;
      setRootPath(path);
      await refresh(path);
      onStatus('Folder opened');
    } catch (error) {
      onStatus(error instanceof Error ? error.message : 'Unable to open folder');
    }
  };

  const createEntry = async (directory: boolean) => {
    if (!rootPath) {
      onStatus('Open a folder first');
      return;
    }
    const name = uniqueName(entries, directory ? 'new-folder' : 'untitled.mars');
    const path = joinPath(rootPath, name);
    try {
      if (directory) await createDirectory(path);
      else await createNotebookFile(path);
      await refresh(rootPath);
      onStatus(directory ? 'Folder created' : 'Notebook created');
      if (!directory) onOpenNotebook(path);
    } catch (error) {
      onStatus(error instanceof Error ? error.message : 'Create failed');
    }
  };

  const commitRename = async (entry: FileExplorerEntry) => {
    const name = renameDraft.trim();
    setRenamingPath(null);
    if (!name || name === entry.name || !rootPath) return;
    const nextPath = joinPath(entry.path.slice(0, entry.path.length - entry.name.length), name);
    try {
      await renameNotebookFile(entry.path, nextPath);
      await refresh(rootPath);
      onStatus('Renamed');
    } catch (error) {
      onStatus(error instanceof Error ? error.message : 'Rename failed');
    }
  };

  const removeEntry = async (entry: FileExplorerEntry) => {
    if (!rootPath) return;
    if (!window.confirm(`Delete ${entry.name}? This cannot be undone.`)) return;
    try {
      await deletePath(entry.path);
      await refresh(rootPath);
      onStatus('Deleted');
    } catch (error) {
      onStatus(error instanceof Error ? error.message : 'Delete failed');
    }
  };

  return (
    <aside
      data-component='FileExplorer'
      aria-hidden={!open}
      className={`shrink-0 overflow-hidden bg-neutral-900 transition-[width] duration-300 ease-in-out ${open ? 'w-56 border-r border-neutral-800' : 'w-0'}`}>
      <div className='flex h-full w-56 flex-col px-2 py-4'>
        <div className='mb-3 flex items-center justify-between gap-2'>
          <SectionHeading>Explorer</SectionHeading>
          <div className='flex items-center gap-1'>
            <IconButton
              label='New notebook'
              icon={<LuFilePlus aria-hidden='true' className='h-4 w-4' />}
              onClick={() => void createEntry(false)}
            />
            <IconButton
              label='New folder'
              icon={<LuFolderPlus aria-hidden='true' className='h-4 w-4' />}
              onClick={() => void createEntry(true)}
            />
            <IconButton
              label='Open folder'
              icon={<LuFolderOpen aria-hidden='true' className='h-4 w-4' />}
              onClick={() => void openFolder()}
            />
          </div>
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
              renamingPath={renamingPath}
              renameDraft={renameDraft}
              onRenameDraft={setRenameDraft}
              onStartRename={(entry) => {
                setRenamingPath(entry.path);
                setRenameDraft(entry.name);
              }}
              onCommitRename={(entry) => void commitRename(entry)}
              onCancelRename={() => setRenamingPath(null)}
              onDelete={(entry) => void removeEntry(entry)}
              onToggle={(path) =>
                setExpanded((current) => {
                  const next = new Set(current);
                  if (next.has(path)) next.delete(path);
                  else next.add(path);
                  return next;
                })
              }
              onOpenNotebook={onOpenNotebook}
              onOpenFile={(path) => {
                void openPathExternal(path)
                  .then(() => onStatus(`Opened ${path.split('/').pop() ?? path}`))
                  .catch((error: unknown) =>
                    onStatus(error instanceof Error ? error.message : 'Open failed'),
                  );
              }}
            />
          ) : (
            <p className='px-2 text-sm leading-6 text-neutral-500'>
              Open a folder to browse notebooks and Rust project files.
            </p>
          )}
        </div>
      </div>
    </aside>
  );
};

FileExplorer.displayName = 'FileExplorer';
export default FileExplorer;
