import { type FC } from 'react';
import type { Diagnostic } from '@codemirror/lint';
import type { NotebookCell } from '../../types/types';
import type { useNotebookEditor } from '../../hooks/useNotebookEditor';
import type { CellRunInfo, KernelStream } from '../../hooks/useKernelExecution';
import type { LspHandlers } from '../../hooks/useLspIntegration';
import type { LspCellTextEdit, LspDefinitionTarget } from '../../lib/kernelService';
import { applyTextEdits } from '../../lib/cellTextEdit';
import { addCellAttachment } from '../../lib/cellAttachments';
import NotebookCellView from './NotebookCellView';

type Editor = ReturnType<typeof useNotebookEditor>;

export type NotebookCellListProps = {
  cells: NotebookCell[];
  editor: Editor;
  selectedId: string;
  rangeIds: ReadonlySet<string>;
  activeCellId: string | null;
  stream: KernelStream | null;
  runInfo: Record<string, CellRunInfo>;
  fontSize: number;
  autocompleteEnabled: boolean;
  foldingEnabled: boolean;
  colorScheme: 'dark' | 'light';
  canDelete: boolean;
  collapsedCellIds: ReadonlySet<string>;
  toggleCellCollapse: (id: string) => void;
  preDiagnostics: Record<string, Diagnostic[]>;
  lspDiagnosticsByCell: Record<string, Diagnostic[]>;
  lsp: LspHandlers;
  pendingReveal: LspDefinitionTarget | null;
  onLspHover: (cellId: string, line: number, character: number) => Promise<string | null>;
  onNavigateSource?: (cellHash: number, line: number) => void;
  onRunAllCells?: () => void;
  executeCell: (cell: NotebookCell) => void;
  setStatus: (status: string) => void;
  selectCell: (cell: NotebookCell) => void;
  shiftSelectCell: (cell: NotebookCell) => void;
  deleteCellOrRange: (cell: NotebookCell) => void;
  dropTargetId: string | null;
  onDragCellStart: (id: string) => void;
  onDragCellEnd: () => void;
  onCellDragOver: (id: string) => void;
  onCellDrop: (id: string) => void;
};

const groupEditsByCell = (edits: LspCellTextEdit[]) => {
  const grouped = new Map<string, LspCellTextEdit[]>();
  for (const edit of edits) {
    const list = grouped.get(edit.cellId) ?? [];
    list.push(edit);
    grouped.set(edit.cellId, list);
  }
  return grouped;
};

const NotebookCellList: FC<NotebookCellListProps> = ({
  cells,
  editor,
  selectedId,
  rangeIds,
  activeCellId,
  stream,
  runInfo,
  fontSize,
  autocompleteEnabled,
  foldingEnabled,
  colorScheme,
  canDelete,
  collapsedCellIds,
  toggleCellCollapse,
  preDiagnostics,
  lspDiagnosticsByCell,
  lsp,
  pendingReveal,
  onLspHover,
  onNavigateSource,
  onRunAllCells,
  executeCell,
  setStatus,
  selectCell,
  shiftSelectCell,
  deleteCellOrRange,
  dropTargetId,
  onDragCellStart,
  onDragCellEnd,
  onCellDragOver,
  onCellDrop,
}) => {
  const lspWithActions = {
    ...lsp,
    applyEdits: (edits: LspCellTextEdit[]) => {
      const byCell = groupEditsByCell(edits);
      for (const [cellId, cellEdits] of byCell) {
        const cell = cells.find((item) => item.id === cellId);
        if (!cell) continue;
        editor.updateCell(cellId, applyTextEdits(cell.source, cellEdits));
      }
    },
  };
  return (
    <div className='flex flex-col gap-4'>
      {cells.map((cell) => (
        <NotebookCellView
          key={cell.id}
          cell={cell}
          selected={cell.id === selectedId || rangeIds.has(cell.id)}
          isExecuting={cell.id === activeCellId}
          runInfo={runInfo[cell.id] ?? null}
          fontSize={fontSize}
          autocompleteEnabled={autocompleteEnabled}
          foldingEnabled={foldingEnabled}
          colorScheme={colorScheme}
          canDelete={canDelete}
          collapsed={collapsedCellIds.has(cell.id)}
          stream={stream?.cellId === cell.id ? stream : null}
          isDropTarget={dropTargetId === cell.id}
          onDragCellStart={onDragCellStart}
          onDragCellEnd={onDragCellEnd}
          onCellDragOver={onCellDragOver}
          onCellDrop={onCellDrop}
          liveDiagnostics={[
            ...(preDiagnostics[cell.id] ?? []),
            ...(lspDiagnosticsByCell[cell.id] ?? []),
          ]}
          onLspHover={(line, character) => onLspHover(cell.id, line, character)}
          lsp={lspWithActions}
          revealRequest={pendingReveal?.cellId === cell.id ? pendingReveal : null}
          onSelect={() => selectCell(cell)}
          onShiftSelect={() => shiftSelectCell(cell)}
          onEdit={(source) => editor.updateCell(cell.id, source)}
          onFocus={() => {
            editor.setSelectedId(cell.id);
            editor.setMode('edit');
          }}
          onExecute={() => {
            editor.setSelectedId(cell.id);
            executeCell(cell);
          }}
          onClear={() => editor.clearOutputs(cell.id)}
          onDelete={() => deleteCellOrRange(cell)}
          onSplitCell={(offset) => editor.splitCellById(cell.id, offset)}
          onUpdateTags={(tags) => editor.updateTags(cell.id, tags)}
          onChangeLanguage={(language) => editor.setCellLanguage(cell.id, language)}
          onToggleCollapse={() => toggleCellCollapse(cell.id)}
          onMoveUp={() => editor.moveCellById(cell.id, -1)}
          onMoveDown={() => editor.moveCellById(cell.id, 1)}
          onMergeNext={
            cell.id !== cells.at(-1)?.id ? () => editor.mergeCellById(cell.id) : undefined
          }
          onChangeType={(cellType) => editor.convertCellType(cell.id, cellType)}
          onRemoveOutput={(outputIndex) => editor.removeOutput(cell.id, outputIndex)}
          onNavigateSource={onNavigateSource}
          onAddAttachment={(dataUrl, mimeType) =>
            addCellAttachment(cell, dataUrl, mimeType, (patch) =>
              editor.updateAttachments(cell.id, patch),
            )
          }
          onDuplicate={() => editor.duplicateCellById(cell.id)}
          onInsertAbove={() =>
            editor.insertCell(
              cells.findIndex((entry) => entry.id === cell.id),
              cell.cell_type,
            )
          }
          onInsertBelow={() =>
            editor.insertCell(cells.findIndex((entry) => entry.id === cell.id) + 1, cell.cell_type)
          }
          onRunAll={onRunAllCells}
          onStatus={setStatus}
        />
      ))}
    </div>
  );
};

NotebookCellList.displayName = 'NotebookCellList';
export default NotebookCellList;
