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
  notebookLanguage?: string;
  editor: Editor;
  selection: {
    selectedId: string;
    rangeIds: ReadonlySet<string>;
    activeCellId: string | null;
    queuedCellIds: ReadonlySet<string>;
    canDelete: boolean;
    collapsedCellIds: ReadonlySet<string>;
    toggleCellCollapse: (id: string) => void;
    mode: 'edit' | 'command';
    hiddenOutputIds: ReadonlySet<string>;
    toggleCellOutputs: (id: string) => void;
  };
  runState: {
    stream: KernelStream | null;
    runInfo: Record<string, CellRunInfo>;
  };
  editorPreferences: {
    fontSize: number;
    autocompleteEnabled: boolean;
    foldingEnabled: boolean;
    colorScheme: 'dark' | 'light';
    lineNumbersEnabled: boolean;
  };
  lspState: {
    inlayHintsByCell: ReadonlyMap<string, import('../../lib/kernelService').LspInlayHint[]>;
    semanticTokensByCell: ReadonlyMap<string, import('../../lib/kernelService').LspSemanticToken[]>;
    preDiagnostics: Record<string, Diagnostic[]>;
    lspDiagnosticsByCell: Record<string, Diagnostic[]>;
    lsp: LspHandlers;
    pendingReveal: LspDefinitionTarget | null;
    onLspHover: (cellId: string, line: number, character: number) => Promise<string | null>;
    onNavigateSource?: (cellHash: number, line: number) => void;
  };
  actions: {
    executeCell: (cell: NotebookCell) => void;
    setStatus: (status: string) => void;
    selectCell: (cell: NotebookCell) => void;
    shiftSelectCell: (cell: NotebookCell) => void;
    deleteCellOrRange: (cell: NotebookCell) => void;
    onRunAllCells?: () => void;
    onRunFromCell?: (index: number) => void;
    onEditMetadata?: (cell: NotebookCell) => void;
    onCellClipboard?: (
      cell: NotebookCell,
      action: 'cut' | 'copy' | 'paste-below' | 'paste-above',
    ) => void;
  };
  drag: {
    dropTargetId: string | null;
    onDragCellStart: (id: string) => void;
    onDragCellEnd: () => void;
    onCellDragOver: (id: string) => void;
    onCellDrop: (id: string) => void;
  };
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
  notebookLanguage = 'rust',
  editor,
  selection,
  runState,
  editorPreferences,
  lspState,
  actions,
  drag,
}) => {
  const {
    selectedId,
    rangeIds,
    activeCellId,
    queuedCellIds,
    canDelete,
    collapsedCellIds,
    toggleCellCollapse,
    mode,
    hiddenOutputIds,
    toggleCellOutputs,
  } = selection;
  const { stream, runInfo } = runState;
  const { fontSize, autocompleteEnabled, foldingEnabled, colorScheme, lineNumbersEnabled } =
    editorPreferences;
  const {
    inlayHintsByCell,
    semanticTokensByCell,
    preDiagnostics,
    lspDiagnosticsByCell,
    lsp,
    pendingReveal,
    onLspHover,
    onNavigateSource,
  } = lspState;
  const {
    executeCell,
    setStatus,
    selectCell,
    shiftSelectCell,
    deleteCellOrRange,
    onRunAllCells,
    onRunFromCell,
    onEditMetadata,
    onCellClipboard,
  } = actions;
  const { dropTargetId, onDragCellStart, onDragCellEnd, onCellDragOver, onCellDrop } = drag;
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
      {cells.map((cell, cellIndex) => (
        <NotebookCellView
          key={cell.id}
          cell={cell}
          notebookLanguage={notebookLanguage}
          selected={cell.id === selectedId || rangeIds.has(cell.id)}
          mode={mode}
          outputsHidden={hiddenOutputIds.has(cell.id)}
          isExecuting={cell.id === activeCellId}
          isQueued={queuedCellIds.has(cell.id)}
          inlayHints={inlayHintsByCell.get(cell.id) ?? []}
          semanticTokens={semanticTokensByCell.get(cell.id) ?? []}
          runInfo={runInfo[cell.id] ?? null}
          fontSize={fontSize}
          autocompleteEnabled={autocompleteEnabled}
          foldingEnabled={foldingEnabled}
          colorScheme={colorScheme}
          lineNumbersEnabled={lineNumbersEnabled}
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
          onUpdateComments={(comments) => editor.updateComments(cell.id, comments)}
          onEditMetadata={onEditMetadata ? () => onEditMetadata(cell) : undefined}
          onChangeLanguage={(language) => editor.setCellLanguage(cell.id, language)}
          onToggleCollapse={() => toggleCellCollapse(cell.id)}
          onToggleOutputs={() => toggleCellOutputs(cell.id)}
          onCellClipboard={onCellClipboard ? (action) => onCellClipboard(cell, action) : undefined}
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
          onRunFromHere={
            onRunFromCell ? () => onRunFromCell(cellIndex) : undefined
          }
          onStatus={setStatus}
        />
      ))}
    </div>
  );
};

NotebookCellList.displayName = 'NotebookCellList';
export default NotebookCellList;
