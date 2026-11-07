import { lazy, Suspense, useRef, useState, type FC } from 'react';
import type { EditorView } from '@codemirror/view';
import {
  LuChevronDown,
  LuChevronRight,
  LuEraser,
  LuEye,
  LuEyeOff,
  LuGripVertical,
  LuPlay,
  LuSearch,
  LuTrash2,
} from 'react-icons/lu';
import type { NotebookCell } from '../../types/types';
import type { KernelStream } from '../../hooks/useKernelExecution';
import { diagnosticsForCell } from '../../lib/diagnostics';
import { IconButton } from '../shared';

const CodeEditor = lazy(() => import('../editor/CodeEditor'));
const OutputView = lazy(() => import('../output/OutputView'));
const MarkdownPreview = lazy(() => import('../output/MarkdownPreview'));

export type NotebookCellViewProps = {
  cell: NotebookCell;
  selected: boolean;
  isExecuting: boolean;
  fontSize: number;
  autocompleteEnabled: boolean;
  foldingEnabled: boolean;
  canDelete: boolean;
  collapsed: boolean;
  stream?: KernelStream | null;
  isDropTarget?: boolean;
  onSelect: () => void;
  onEdit: (source: string) => void;
  onFocus: () => void;
  onExecute: () => void;
  onClear: () => void;
  onDelete: () => void;
  onToggleCollapse: () => void;
  onStatus: (status: string) => void;
  onDragCellStart?: (id: string) => void;
  onDragCellEnd?: () => void;
  onCellDragOver?: (id: string) => void;
  onCellDrop?: (id: string) => void;
};

const iconClassName = 'h-3.5 w-3.5';

const editorFallback = (
  <div className='min-h-[44px] rounded-md border border-neutral-800 bg-neutral-950' />
);

const NotebookCellView: FC<NotebookCellViewProps> = ({
  cell,
  selected,
  isExecuting,
  fontSize,
  autocompleteEnabled,
  foldingEnabled,
  canDelete,
  collapsed,
  stream = null,
  isDropTarget = false,
  onSelect,
  onEdit,
  onFocus,
  onExecute,
  onClear,
  onDelete,
  onToggleCollapse,
  onStatus,
  onDragCellStart,
  onDragCellEnd,
  onCellDragOver,
  onCellDrop,
}) => {
  const editorViewRef = useRef<EditorView | null>(null);
  const [showMarkdownPreview, setShowMarkdownPreview] = useState(true);
  const firstSourceLine =
    cell.source
      .split('\n')
      .find((line) => line.trim().length > 0)
      ?.trim() ?? '';
  const openSearch = () => {
    const view = editorViewRef.current;
    if (!view) return;
    view.focus();
    void import('@codemirror/search').then((module) => module.openSearchPanel(view));
  };
  return (
    <article
      data-component='NotebookCellView'
      id={`cell-${cell.id}`}
      aria-busy={isExecuting || undefined}
      className={`flex scroll-mt-6 rounded-xl border [contain-intrinsic-size:auto_14rem] [content-visibility:auto] ${isExecuting ? 'border-amber-400 ring-2 ring-amber-500/40' : isDropTarget ? 'border-orange-500 ring-1 ring-orange-500/60' : selected ? 'border-orange-800 ring-1 ring-orange-950' : 'border-transparent'} bg-neutral-900`}
      onClick={onSelect}
      onDragOver={(event) => {
        if (!onCellDragOver) return;
        event.preventDefault();
        onCellDragOver(cell.id);
      }}
      onDrop={(event) => {
        event.preventDefault();
        onCellDrop?.(cell.id);
      }}>
      <div className='w-12 p-1.5 pt-4 text-right text-xs text-neutral-500'>
        <span>{cell.cell_type === 'code' ? `In [${cell.execution_count ?? ' '}]` : ' '}</span>
      </div>
      <div className='min-w-0 flex-1 py-3 pr-4'>
        <div className='mb-1.5 flex items-center justify-between gap-3'>
          <div className='flex items-center gap-2'>
            <span
              draggable
              role='button'
              aria-label='Drag to reorder cell'
              title='Drag to reorder'
              className='cursor-grab text-neutral-600 hover:text-neutral-300 active:cursor-grabbing'
              onDragStart={(event) => {
                event.dataTransfer.effectAllowed = 'move';
                event.dataTransfer.setData('text/plain', cell.id);
                onDragCellStart?.(cell.id);
              }}
              onDragEnd={onDragCellEnd}>
              <LuGripVertical aria-hidden='true' className={iconClassName} />
            </span>
            <span className='text-xs tracking-widest text-neutral-400 uppercase'>
              {cell.cell_type}
            </span>
            {isExecuting ? (
              <span
                data-component='CellRunningBadge'
                className='animate-pulse text-xs font-medium text-amber-400'>
                Running…
              </span>
            ) : null}
          </div>
          <div className='flex items-center gap-1.5'>
            <IconButton
              label={collapsed ? 'Expand cell' : 'Collapse cell'}
              icon={
                collapsed ? (
                  <LuChevronRight aria-hidden='true' className={iconClassName} />
                ) : (
                  <LuChevronDown aria-hidden='true' className={iconClassName} />
                )
              }
              onClick={(event) => {
                event.stopPropagation();
                onToggleCollapse();
              }}
            />
            <IconButton
              label='Search (Cmd+F)'
              icon={<LuSearch aria-hidden='true' className={iconClassName} />}
              onClick={(event) => {
                event.stopPropagation();
                openSearch();
              }}
            />
            {cell.cell_type === 'markdown' && (
              <IconButton
                label={showMarkdownPreview ? 'Hide Markdown preview' : 'Show Markdown preview'}
                icon={
                  showMarkdownPreview ? (
                    <LuEyeOff aria-hidden='true' className={iconClassName} />
                  ) : (
                    <LuEye aria-hidden='true' className={iconClassName} />
                  )
                }
                onClick={(event) => {
                  event.stopPropagation();
                  setShowMarkdownPreview((current) => !current);
                }}
              />
            )}
            {cell.cell_type === 'code' && (
              <>
                <IconButton
                  label='Run cell (Shift+Enter)'
                  icon={<LuPlay aria-hidden='true' className={iconClassName} />}
                  onClick={(event) => {
                    event.stopPropagation();
                    onExecute();
                  }}
                />
                {cell.outputs.length > 0 && (
                  <IconButton
                    label='Clear outputs'
                    icon={<LuEraser aria-hidden='true' className={iconClassName} />}
                    onClick={(event) => {
                      event.stopPropagation();
                      onClear();
                      onStatus('Outputs cleared');
                    }}
                  />
                )}
              </>
            )}
            <IconButton
              label='Delete cell (D, D)'
              variant='danger'
              disabled={!canDelete}
              icon={<LuTrash2 aria-hidden='true' className={iconClassName} />}
              onClick={(event) => {
                event.stopPropagation();
                onDelete();
              }}
            />
          </div>
        </div>
        {collapsed ? (
          <button
            type='button'
            data-component='CollapsedCellSummary'
            className='w-full truncate rounded-md px-2 py-1.5 text-left font-mono text-xs text-neutral-500 hover:bg-neutral-800/60 hover:text-neutral-300'
            onClick={(event) => {
              event.stopPropagation();
              onToggleCollapse();
            }}>
            {firstSourceLine || 'Empty cell'}
          </button>
        ) : null}
        <div className={collapsed ? 'hidden' : undefined}>
          <Suspense fallback={editorFallback}>
            <CodeEditor
              value={cell.source}
              onChange={onEdit}
              onFocus={onFocus}
              placeholder={
                cell.cell_type === 'markdown'
                  ? 'Write Markdown...'
                  : cell.cell_type === 'raw'
                    ? 'Write raw content...'
                    : 'Write Rust code...'
              }
              cellType={cell.cell_type}
              fontSize={fontSize}
              autocompleteEnabled={autocompleteEnabled}
              foldingEnabled={foldingEnabled}
              diagnostics={diagnosticsForCell(cell)}
              onViewReady={(view) => {
                editorViewRef.current = view;
              }}
            />
          </Suspense>
          {cell.cell_type === 'raw' ? (
            <p className='mt-2 text-xs text-neutral-500'>
              Raw cells are preserved and never executed.
            </p>
          ) : null}
          {cell.cell_type === 'markdown' && showMarkdownPreview ? (
            <Suspense fallback={null}>
              <MarkdownPreview source={cell.source} />
            </Suspense>
          ) : null}
          {stream && (stream.stdout || stream.stderr) ? (
            <div
              data-component='CellStreamOutput'
              className='mt-2 rounded-md border border-neutral-800 bg-neutral-950 p-3 font-mono text-xs'>
              {stream.stdout ? (
                <pre className='whitespace-pre-wrap text-neutral-300'>{stream.stdout}</pre>
              ) : null}
              {stream.stderr ? (
                <pre className='whitespace-pre-wrap text-red-400'>{stream.stderr}</pre>
              ) : null}
            </div>
          ) : null}
          {cell.cell_type === 'code'
            ? cell.outputs.map((output, outputIndex) => (
                <Suspense key={`${cell.id}-${outputIndex}`} fallback={null}>
                  <OutputView output={output} />
                </Suspense>
              ))
            : null}
        </div>
      </div>
    </article>
  );
};
NotebookCellView.displayName = 'NotebookCellView';
export default NotebookCellView;
