import { lazy, Suspense, type FC } from 'react';
import { LuEraser, LuPlay, LuTrash2 } from 'react-icons/lu';
import type { NotebookCell } from '../../types/types';
import CodeEditor from '../editor/CodeEditor';
import { diagnosticsForCell } from '../../lib/diagnostics';
import { IconButton } from '../shared';

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
  onSelect: () => void;
  onEdit: (source: string) => void;
  onFocus: () => void;
  onExecute: () => void;
  onClear: () => void;
  onDelete: () => void;
  onStatus: (status: string) => void;
};

const iconClassName = 'h-4 w-4';

const NotebookCellView: FC<NotebookCellViewProps> = ({
  cell,
  selected,
  isExecuting,
  fontSize,
  autocompleteEnabled,
  foldingEnabled,
  canDelete,
  onSelect,
  onEdit,
  onFocus,
  onExecute,
  onClear,
  onDelete,
  onStatus,
}) => {
  return (
    <article
      data-component='NotebookCellView'
      id={`cell-${cell.id}`}
      aria-busy={isExecuting || undefined}
      className={`flex scroll-mt-6 rounded-xl border [contain-intrinsic-size:auto_14rem] [content-visibility:auto] ${isExecuting ? 'border-amber-400 ring-2 ring-amber-500/40' : selected ? 'border-orange-800 ring-1 ring-orange-950' : 'border-transparent'} bg-neutral-900`}
      onClick={onSelect}>
      <div className='w-24 p-2 pt-5 text-right text-xs text-neutral-500'>
        <span>{cell.cell_type === 'code' ? `In [${cell.execution_count ?? ' '}]` : ' '}</span>
      </div>
      <div className='min-w-0 flex-1 py-4 pr-5'>
        <div className='mb-2 flex items-center justify-between gap-3'>
          <span className='text-xs tracking-widest text-neutral-400 uppercase'>
            {cell.cell_type}
          </span>
          <div className='flex items-center gap-1.5'>
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
        />
        {cell.cell_type === 'raw' ? (
          <p className='mt-2 text-xs text-neutral-500'>
            Raw cells are preserved and never executed.
          </p>
        ) : null}
        {cell.cell_type === 'markdown' ? (
          <Suspense fallback={null}>
            <MarkdownPreview source={cell.source} />
          </Suspense>
        ) : null}
        {cell.outputs.map((output, outputIndex) => (
          <Suspense key={`${cell.id}-${outputIndex}`} fallback={null}>
            <OutputView output={output} />
          </Suspense>
        ))}
      </div>
    </article>
  );
};
NotebookCellView.displayName = 'NotebookCellView';
export default NotebookCellView;
