import { type FC, useRef, useState } from 'react';
import NotebookHeaderBrand from './NotebookHeaderBrand';
import { IconButton } from '../shared';
import { LuFilePlus, LuHistory, LuPanelLeft, LuSettings, LuX } from 'react-icons/lu';
import { fileNameFromPath } from '../../lib/filePath';

export type WorkspaceTab = {
  id: string;
  label: string;
  isDirty: boolean;
};

type WorkspaceHeaderProps = {
  tabs: WorkspaceTab[];
  activeId: string;
  explorerOpen: boolean;
  status?: string;
  recents?: string[];
  onSelectTab: (id: string) => void;
  onCloseTab: (id: string) => void;
  onNewTab: () => void;
  onToggleExplorer: () => void;
  onOpenSettings: () => void;
  onOpenRecent?: (path: string) => void;
  onReorderTabs?: (fromId: string, toId: string) => void;
};

const iconClassName = 'h-4 w-4';

const RecentsMenu: FC<{ recents: string[]; onOpenRecent: (path: string) => void }> = ({
  recents,
  onOpenRecent,
}) => {
  const [open, setOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);
  if (recents.length === 0) return null;
  return (
    <div ref={rootRef} className='relative'>
      <IconButton
        label='Open recent notebook'
        icon={<LuHistory aria-hidden='true' className={iconClassName} />}
        onClick={() => setOpen((current) => !current)}
      />
      {open ? (
        <>
          <button
            type='button'
            aria-label='Close recent notebooks'
            className='fixed inset-0 z-30 cursor-default'
            onClick={() => setOpen(false)}
          />
          <div className='absolute top-full right-0 z-40 mt-1 w-80 overflow-hidden rounded-lg border border-neutral-700 bg-neutral-900 shadow-2xl'>
            <p className='border-b border-neutral-800 px-3 py-1.5 text-[10px] font-semibold tracking-wider text-neutral-500 uppercase'>
              Recent notebooks
            </p>
            <ul className='max-h-64 overflow-y-auto py-1'>
              {recents.map((path) => (
                <li key={path}>
                  <button
                    type='button'
                    className='flex w-full flex-col px-3 py-1.5 text-left hover:bg-neutral-800'
                    onClick={() => {
                      setOpen(false);
                      onOpenRecent(path);
                    }}>
                    <span className='truncate text-sm text-neutral-200'>
                      {fileNameFromPath(path)}
                    </span>
                    <span className='truncate text-[10px] text-neutral-500'>{path}</span>
                  </button>
                </li>
              ))}
            </ul>
          </div>
        </>
      ) : null}
    </div>
  );
};
RecentsMenu.displayName = 'RecentsMenu';

const WorkspaceHeader: FC<WorkspaceHeaderProps> = ({
  tabs,
  activeId,
  explorerOpen,
  status = '',
  recents = [],
  onSelectTab,
  onCloseTab,
  onNewTab,
  onToggleExplorer,
  onOpenSettings,
  onOpenRecent,
  onReorderTabs,
}) => {
  const dragTabId = useRef<string | null>(null);
  const [dropTargetId, setDropTargetId] = useState<string | null>(null);
  return (
    <header
      data-component='WorkspaceHeader'
      className='flex min-h-14 items-center gap-4 border-b border-neutral-800 bg-neutral-900 px-4 py-2'>
      <NotebookHeaderBrand compact />
      <nav
        className='mars-tab-strip flex min-w-0 flex-1 items-center gap-1 overflow-x-auto overflow-y-hidden'
        aria-label='Open notebooks'>
        {tabs.map((tab) => {
          const active = tab.id === activeId;
          return (
            <div
              key={tab.id}
              draggable={Boolean(onReorderTabs)}
              onDragStart={(event) => {
                dragTabId.current = tab.id;
                event.dataTransfer.effectAllowed = 'move';
              }}
              onDragOver={(event) => {
                if (!dragTabId.current || dragTabId.current === tab.id) return;
                event.preventDefault();
                event.dataTransfer.dropEffect = 'move';
                setDropTargetId(tab.id);
              }}
              onDragLeave={() =>
                setDropTargetId((current) => (current === tab.id ? null : current))
              }
              onDrop={(event) => {
                event.preventDefault();
                const fromId = dragTabId.current;
                dragTabId.current = null;
                setDropTargetId(null);
                if (fromId && fromId !== tab.id) onReorderTabs?.(fromId, tab.id);
              }}
              onDragEnd={() => {
                dragTabId.current = null;
                setDropTargetId(null);
              }}
              className={`group flex shrink-0 items-center rounded-lg border text-sm ${active ? 'border-orange-900 bg-neutral-800 text-neutral-100' : 'border-transparent text-neutral-400 hover:bg-neutral-800/60 hover:text-neutral-200'} ${dropTargetId === tab.id ? 'ring-1 ring-orange-600' : ''}`}>
              <button
                type='button'
                className='flex max-w-48 items-center gap-1.5 truncate px-3 py-1.5'
                aria-current={active ? 'page' : undefined}
                onClick={() => onSelectTab(tab.id)}
                onAuxClick={(event) => {
                  if (event.button === 1) onCloseTab(tab.id);
                }}>
                <span className='truncate'>{tab.label}</span>
                {tab.isDirty ? (
                  <span
                    aria-label='Unsaved changes'
                    className='h-1.5 w-1.5 shrink-0 rounded-full bg-amber-400'
                  />
                ) : null}
              </button>
              <button
                type='button'
                aria-label={`Close ${tab.label}`}
                className='mr-1 rounded p-0.5 text-neutral-500 opacity-0 transition-opacity group-hover:opacity-100 hover:bg-neutral-700 hover:text-neutral-200 focus:opacity-100'
                onClick={() => onCloseTab(tab.id)}>
                <LuX aria-hidden='true' className='h-3 w-3' />
              </button>
            </div>
          );
        })}
        <IconButton
          label='New notebook tab'
          icon={<LuFilePlus aria-hidden='true' className={iconClassName} />}
          onClick={onNewTab}
        />
      </nav>
      {status ? <span className='max-w-56 truncate text-xs text-neutral-500'>{status}</span> : null}
      {onOpenRecent ? <RecentsMenu recents={recents} onOpenRecent={onOpenRecent} /> : null}
      <IconButton
        label={explorerOpen ? 'Hide file explorer' : 'Show file explorer'}
        icon={<LuPanelLeft aria-hidden='true' className={iconClassName} />}
        onClick={onToggleExplorer}
      />
      <IconButton
        label='Settings'
        icon={<LuSettings aria-hidden='true' className={iconClassName} />}
        onClick={onOpenSettings}
      />
    </header>
  );
};
WorkspaceHeader.displayName = 'WorkspaceHeader';
export default WorkspaceHeader;
