import { type FC, useMemo, useRef, useState } from 'react';
import {
  LuChevronRight,
  LuCopy,
  LuHistory,
  LuPackage,
  LuPencil,
  LuRefreshCw,
  LuSearch,
  LuList,
  LuTriangleAlert,
  LuVariable,
} from 'react-icons/lu';
import type { KernelVariable, NotebookCell } from '../../types/types';
import { fileNameFromPath } from '../../lib/filePath';
import { extractRustSymbols } from '../../lib/rustSymbols';
import { Badge, IconButton, NavButton, SectionHeading, TextInput } from '../shared';
import marsFileLogo from '../../assets/svg/mars-logo-file.svg';

export type NotebookProblem = {
  cellId: string;
  cellIndex: number;
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
  onRenameFile: (fileName: string) => Promise<boolean>;
  onOpenSearch?: () => void;
  onOpenDependencies?: () => void;
  onOpenCheckpoints?: () => void;
  liveVariables?: KernelVariable[];
  onRefreshVariables?: () => void;
  problems?: NotebookProblem[];
};

const NotebookSidebar: FC<NotebookSidebarProps> = ({
  cells,
  selectedId,
  filePath,
  isDirty,
  open = true,
  onSelect,
  onRenameFile,
  onOpenSearch,
  onOpenDependencies,
  onOpenCheckpoints,
  liveVariables = [],
  onRefreshVariables,
  problems = [],
}) => {
  const fileName = filePath ? fileNameFromPath(filePath) : 'Untitled.mars';
  const [isRenamingFile, setIsRenamingFile] = useState(false);
  const [expandedVariables, setExpandedVariables] = useState<ReadonlySet<string>>(new Set());
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
          .map((line) => /^(#{1,6})\s+(.+?)\s*#*\s*$/.exec(line))
          .filter((match): match is RegExpExecArray => match !== null)
          .map((match) => ({
            cellId: cell.id,
            cellIndex,
            level: match[1].length,
            text: match[2],
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
      <div className='flex h-full w-56 flex-col px-3 py-4'>
        <SectionHeading>Notebook</SectionHeading>
        <div className='mb-6 flex items-center gap-2 rounded-lg border border-neutral-700 bg-neutral-800 p-2.5'>
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
        <div className='mb-2 flex items-center justify-between'>
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
        <SectionHeading>Cell navigation</SectionHeading>
        <nav className='min-h-0 flex-1 overflow-y-auto' aria-label='Cell navigation'>
          {cells.map((cell, index) => (
            <NavButton
              key={cell.id}
              active={cell.id === selectedId}
              index={String(index + 1).padStart(2, '0')}
              onClick={() => onSelect(cell.id)}>
              {cell.cell_type}
            </NavButton>
          ))}
        </nav>
        {toc.length > 0 ? (
          <>
            <SectionHeading>
              <span className='flex items-center gap-1'>
                <LuList aria-hidden='true' className='h-3 w-3' />
                Contents
              </span>
            </SectionHeading>
            <ul className='mb-2 max-h-40 overflow-y-auto' aria-label='Notebook contents'>
              {toc.map((entry, index) => (
                <li key={`${entry.cellId}-${index}`}>
                  <button
                    type='button'
                    className='flex w-full items-center gap-1.5 px-2 py-1 text-left text-xs text-neutral-400 hover:bg-neutral-800 hover:text-neutral-200'
                    style={{ paddingLeft: `${0.5 + Math.min(entry.level - 1, 4) * 0.75}rem` }}
                    title={`Cell ${entry.cellIndex + 1}: ${entry.text}`}
                    onClick={() => onSelect(entry.cellId)}>
                    <LuChevronRight
                      aria-hidden='true'
                      className='h-2.5 w-2.5 shrink-0 text-neutral-600'
                    />
                    <span className='truncate'>{entry.text}</span>
                  </button>
                </li>
              ))}
            </ul>
          </>
        ) : null}
        {problems.length > 0 ? (
          <>
            <SectionHeading>
              <span className='flex items-center gap-1'>
                <LuTriangleAlert aria-hidden='true' className='h-3 w-3 text-amber-400' />
                Problems
                <Badge tone='warning'>{problems.length}</Badge>
              </span>
            </SectionHeading>
            <ul className='mb-2 max-h-40 overflow-y-auto' aria-label='Notebook problems'>
              {problems.slice(0, 50).map((problem, index) => (
                <li key={`${problem.cellId}-${index}`}>
                  <button
                    type='button'
                    className='flex w-full items-baseline gap-1.5 px-2 py-1 text-left text-xs hover:bg-neutral-800'
                    onClick={() => onSelect(problem.cellId)}>
                    <span
                      aria-hidden='true'
                      className={`mt-0.5 h-1.5 w-1.5 shrink-0 rounded-full ${problem.severity === 'warning' ? 'bg-amber-400' : 'bg-red-400'}`}
                    />
                    <span className='shrink-0 text-neutral-500'>
                      Cell {problem.cellIndex + 1}
                    </span>
                    <span className='truncate text-neutral-400'>{problem.message}</span>
                  </button>
                </li>
              ))}
            </ul>
          </>
        ) : null}
        {liveVariables.length > 0 || symbols.length > 0 ? (
          <>
            <SectionHeading>
              <span className='flex items-center gap-1'>
                <LuVariable aria-hidden='true' className='h-3 w-3' />{' '}
                {liveVariables.length > 0 ? 'Live variables' : 'Variables & symbols'}
                {onRefreshVariables ? (
                  <IconButton
                    label='Refresh variables'
                    icon={<LuRefreshCw aria-hidden='true' className='h-3 w-3' />}
                    onClick={onRefreshVariables}
                  />
                ) : null}
              </span>
            </SectionHeading>
            {liveVariables.length > 0 ? (
              <ul className='max-h-48 overflow-y-auto' aria-label='Live kernel variables'>
                {liveVariables.map((variable) => (
                  <li key={variable.name} className='px-2 py-1 text-xs'>
                    <div className='flex items-center justify-between gap-2'>
                      <span className='truncate font-mono text-neutral-300'>{variable.name}</span>
                      <span className='flex shrink-0 items-center gap-1'>
                        <span className='truncate text-neutral-600'>
                          {variable.typeName ?? variable.value ?? ''}
                        </span>
                        <IconButton
                          label={`Copy ${variable.name}`}
                          variant='ghost'
                          className='p-0.5'
                          icon={<LuCopy aria-hidden='true' className='h-3 w-3' />}
                          onClick={() => {
                            void navigator.clipboard
                              ?.writeText(variable.value ?? variable.name)
                              .catch(() => {});
                          }}
                        />
                      </span>
                    </div>
                    {variable.value !== null ? (
                      <button
                        type='button'
                        className='block w-full text-left font-mono text-[10px] text-neutral-500 hover:text-neutral-300'
                        title={expandedVariables.has(variable.name) ? 'Collapse value' : 'Expand value'}
                        onClick={() =>
                          setExpandedVariables((current) => {
                            const next = new Set(current);
                            if (next.has(variable.name)) next.delete(variable.name);
                            else next.add(variable.name);
                            return next;
                          })
                        }>
                        <span
                          className={
                            expandedVariables.has(variable.name)
                              ? 'block whitespace-pre-wrap break-all'
                              : 'block truncate'
                          }>
                          {variable.value}
                        </span>
                      </button>
                    ) : null}
                  </li>
                ))}
              </ul>
            ) : (
              <ul className='max-h-40 overflow-y-auto' aria-label='Notebook symbols'>
                {symbols.map((symbol) => (
                  <li
                    key={symbol.name}
                    className='flex items-center justify-between gap-2 truncate px-2 py-1 text-xs'>
                    <span className='truncate font-mono text-neutral-300'>{symbol.name}</span>
                    <span className='shrink-0 text-neutral-600'>{symbol.kind}</span>
                  </li>
                ))}
              </ul>
            )}
          </>
        ) : null}
      </div>
    </aside>
  );
};
NotebookSidebar.displayName = 'NotebookSidebar';
export default NotebookSidebar;
