import type { NotebookCell } from '../lib/types';
import CodeEditor from './CodeEditor';
import OutputView from './OutputView';
import { diagnosticsForCell } from '../lib/diagnostics';

type NotebookCellViewProps = {
  cell: NotebookCell;
  selected: boolean;
  fontSize: number;
  autocompleteEnabled: boolean;
  foldingEnabled: boolean;
  onSelect: () => void;
  onEdit: (source: string) => void;
  onFocus: () => void;
  onExecute: () => void;
  onClear: () => void;
  onStatus: (status: string) => void;
};

export default function NotebookCellView({
  cell,
  selected,
  fontSize,
  autocompleteEnabled,
  foldingEnabled,
  onSelect,
  onEdit,
  onFocus,
  onExecute,
  onClear,
  onStatus,
}: NotebookCellViewProps) {
  return (
    <article className={`notebook-cell ${selected ? 'active' : ''}`} onClick={onSelect}>
      <div className='cell-gutter'>
        <span>{cell.cell_type === 'code' ? `In [${cell.execution_count ?? ' '}]` : ' '}</span>
      </div>
      <div className='cell-content'>
        <div className='cell-heading'>
          <span className='cell-type'>{cell.cell_type}</span>
          {cell.cell_type === 'code' && (
            <div className='cell-actions'>
              <button
                onClick={(event) => {
                  event.stopPropagation();
                  onExecute();
                }}>
                Run cell <kbd>⇧↵</kbd>
              </button>
              {cell.outputs.length > 0 && (
                <button
                  onClick={(event) => {
                    event.stopPropagation();
                    onClear();
                    onStatus('Outputs cleared');
                  }}>
                  Clear
                </button>
              )}
            </div>
          )}
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
}
