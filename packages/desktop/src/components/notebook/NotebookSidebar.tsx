import { type FC, useMemo, useRef, useState } from 'react';
import { LuChevronRight, LuHistory, LuList, LuPackage, LuPencil, LuSearch } from 'react-icons/lu';
import type { KernelVariable, NotebookCell } from '../../types/types';
import { fileNameFromPath } from '../../lib/filePath';
import { cellPreview } from '../../lib/cellPreview';
import { extractRustSymbols } from '../../lib/rustSymbols';
import { Badge, IconButton, NavButton, SectionHeading, TextInput } from '../shared';
import { SidebarSection } from './SidebarSection';
import { NotebookProblems } from './NotebookProblems';
import { NotebookVariables } from './NotebookVariables';
import marsFileLogo from '../../assets/svg/mars-logo-file.svg';

export type NotebookProblem = {
  cellId: string;
  cellIndex: number;
  line: number;
  severity: string;
  message: string;
};

type NotebookSidebarProps = {
  cells: NotebookCell[];
  selectedId: string;
  filePath: string | null;
  isDirty: boolean;
  open?: boolean;
  onSelect: (id: string) => void;
  onRevealHeading?: (cellId: string, line: number) => void;
  onRenameFile: (fileName: string) => Promise<boolean>;
  onOpenSearch?: () => void;
  onOpenDependencies?: () => void;
  onOpenCheckpoints?: () => void;
  liveVariables?: KernelVariable[];
  onRefreshVariables?: () => void;
  problems?: NotebookProblem[];
  onSelectProblem?: (problem: NotebookProblem) => void;
};

const cellTypeTone: Record<NotebookCell['cell_type'], string> = {
  code: 'text-sky-400',
  markdown: 'text-emerald-400',
  raw: 'text-amber-400',
};

const NotebookSidebar: FC<NotebookSidebarProps> = ({
  cells,
  selectedId,
  filePath,
  isDirty,
  open = true,
  onSelect,
  onRevealHeading,
  onRenameFile,
  onOpenSearch,
  onOpenDependencies,
  onOpenCheckpoints,
  liveVariables = [],
  onRefreshVariables,
  problems = [],
  onSelectProblem,
}) => {
  const fileName = filePath ? fileNameFromPath(filePath) : 'Untitled.mars';
  const [isRenamingFile, setIsRenamingFile] = useState(false);
  const [fileDraft, setFileDraft] = useState(fileName);
  const skipFileBlurRef = useRef(false);

  const symbols = useMemo(
    () =>
      extractRustSymbols(
        cells.filter((cell) => cell.cell_type === 'code').map((cell) => cell.source),
      ),
    [cells],
  );

  const toc = useMemo(
    () =>
      cells.flatMap((cell, cellIndex) => {
        if (cell.cell_type !== 'markdown') return [];
        return cell.source
          .split('\n')
          .map((line, lineIndex) => ({
            match: /^(#{1,6})\s+(.+?)\s*#*\s*$/.exec(line),
            lineIndex,
          }))
          .filter(
            (entry): entry is { match: RegExpExecArray; lineIndex: number } => entry.match !== null,
          )
          .map((entry) => ({
            cellId: cell.id,
            cellIndex,
            level: entry.match[1].length,
            text: entry.match[2],
            line: entry.lineIndex,
          }));
      }),
    [cells],
  );

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
      aria-hidden={!open}
      className={`shrink-0 overflow-hidden bg-neutral-900 transition-[width] duration-300 ease-in-out ${open ? 'w-56 border-l border-neutral-800' : 'w-0'}`}>
      <div className='h-full w-56 overflow-y-auto px-3 py-4'>
        <SectionHeading>Notebook</SectionHeading>
        <div className='mb-4 flex items-center gap-2 rounded-lg border border-neutral-700 bg-neutral-800 p-2.5'>
          <img className='h-7 w-7 shrink-0 rounded-md object-cover' src={marsFileLogo} alt='' />
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
        <div className='mb-4 flex items-center gap-1'>
          {onOpenSearch ? (
            <IconButton
              label='Search notebook'
              icon={<LuSearch aria-hidden='true' className='h-3.5 w-3.5' />}
              onClick={onOpenSearch}
            />
          ) : null}
          {onOpenDependencies ? (
            <IconButton
              label='Edit dependencies'
              icon={<LuPackage aria-hidden='true' className='h-3.5 w-3.5' />}
              onClick={onOpenDependencies}
            />
          ) : null}
          {onOpenCheckpoints ? (
            <IconButton
              label='Checkpoints / version history'
              icon={<LuHistory aria-hidden='true' className='h-3.5 w-3.5' />}
              onClick={onOpenCheckpoints}
            />
          ) : null}
        </div>
        <SidebarSection id='cells' title={<span className='flex items-center gap-1'>Cells</span>}>
          <nav aria-label='Cell navigation'>
            {cells.map((cell, index) => {
              const preview = cellPreview(cell);
              return (
                <NavButton
                  key={cell.id}
                  active={cell.id === selectedId}
                  index={String(index + 1).padStart(2, '0')}
                  title={`${index + 1} · ${cell.cell_type}${preview ? `: ${preview}` : ''}`}
                  onClick={() => onSelect(cell.id)}>
                  <span className='flex min-w-0 flex-1 items-baseline gap-1.5'>
                    <span
                      className={`shrink-0 text-[10px] uppercase ${cellTypeTone[cell.cell_type]}`}>
                      {cell.cell_type.slice(0, 4)}
                    </span>
                    <span className='truncate text-xs text-neutral-400'>
                      {preview || 'Empty cell'}
                    </span>
                  </span>
                </NavButton>
              );
            })}
          </nav>
        </SidebarSection>
        {toc.length > 0 ? (
          <SidebarSection
            id='contents'
            title={
              <span className='flex items-center gap-1'>
                <LuList aria-hidden='true' className='h-3 w-3' />
                Contents
              </span>
            }>
            <ul aria-label='Notebook contents'>
              {toc.map((entry, index) => (
                <li key={`${entry.cellId}-${index}`}>
                  <button
                    type='button'
                    className='flex w-full items-center gap-1.5 rounded-md px-2 py-1 text-left text-xs text-neutral-400 hover:bg-neutral-800 hover:text-neutral-200'
                    style={{ paddingLeft: `${0.5 + Math.min(entry.level - 1, 4) * 0.75}rem` }}
                    title={`Cell ${entry.cellIndex + 1}: ${entry.text}`}
                    onClick={() => {
                      onSelect(entry.cellId);
                      onRevealHeading?.(entry.cellId, entry.line);
                    }}>
                    <LuChevronRight
                      aria-hidden='true'
                      className='h-2.5 w-2.5 shrink-0 text-neutral-600'
                    />
                    <span className='truncate'>{entry.text}</span>
                  </button>
                </li>
              ))}
            </ul>
          </SidebarSection>
        ) : null}
        {problems.length > 0 ? (
          <NotebookProblems
            problems={problems}
            onSelectProblem={onSelectProblem}
            onSelectCell={onSelect}
          />
        ) : null}
        <NotebookVariables
          liveVariables={liveVariables}
          symbols={symbols}
          onRefreshVariables={onRefreshVariables}
        />
      </div>
    </aside>
  );
};
NotebookSidebar.displayName = 'NotebookSidebar';
export default NotebookSidebar;
