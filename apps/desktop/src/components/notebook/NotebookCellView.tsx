import { type FC } from 'react';
import { LuEraser, LuPlay, LuTrash2 } from 'react-icons/lu';
import type { NotebookCell } from '../../types/types';
import CodeEditor from '../editor/CodeEditor';
import OutputView from '../output/OutputView';
import { diagnosticsForCell } from '../../lib/diagnostics';
import { IconButton } from '../shared';

export type NotebookCellViewProps = {
  cell: NotebookCell;
  selected: boolean;
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
      className={`flex rounded-xl border ${selected ? 'border-orange-800 ring-1 ring-orange-950' : 'border-transparent'} bg-neutral-900`}
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
          placeholder={cell.cell_type === 'markdown' ? 'Write Markdown...' : 'Write Rust code...'}
          cellType={cell.cell_type}
          fontSize={fontSize}
          autocompleteEnabled={autocompleteEnabled}
          foldingEnabled={foldingEnabled}
          diagnostics={diagnosticsForCell(cell)}
        />
        {cell.outputs.map((output, outputIndex) => (
          <OutputView key={`${cell.id}-${outputIndex}`} output={output} />
        ))}
      </div>
    </article>
  );
};
NotebookCellView.displayName = 'NotebookCellView';
export default NotebookCellView;
