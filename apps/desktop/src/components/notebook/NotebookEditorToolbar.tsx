import { type FC } from 'react';
import NotebookToolbar from './NotebookToolbar';
import type { useNotebookEditor } from '../../hooks/useNotebookEditor';
import type { useNotebookUiState } from '../../hooks/useNotebookUiState';
import type { useKernelExecution } from '../../hooks/useKernelExecution';
import type { useNotebookExecutionCommands } from '../../hooks/useNotebookExecutionCommands';

type Editor = ReturnType<typeof useNotebookEditor>;
type Ui = ReturnType<typeof useNotebookUiState>;
type Execution = ReturnType<typeof useKernelExecution>;
type Commands = ReturnType<typeof useNotebookExecutionCommands>;

type NotebookEditorToolbarProps = {
  editor: Editor;
  ui: Ui;
  execution: Execution;
  commands: Commands;
  preferences: { fontSize: number; updateFontSize: (value: number) => void };
  fileName: string;
  isDirty: boolean;
  clearAllOutputs: () => void;
  collapseAllCells: () => void;
  expandAllCells: () => void;
  cellClipboard?: {
    onCut: () => void;
    onCopy: () => void;
    onPasteBelow: () => void;
    onPasteAbove: () => void;
  };
};

const NotebookEditorToolbar: FC<NotebookEditorToolbarProps> = ({
  editor,
  ui,
  execution,
  commands,
  preferences,
  fileName,
  isDirty,
  clearAllOutputs,
  collapseAllCells,
  expandAllCells,
  cellClipboard,
}) => (
  <NotebookToolbar
    fileName={fileName}
    isDirty={isDirty}
    isExecuting={execution.isExecuting}
    executionTimeout={ui.executionTimeout}
    fontSize={preferences.fontSize}
    onTimeoutChange={ui.setExecutionTimeout}
    onFontSizeChange={preferences.updateFontSize}
    onCancel={() => void execution.cancelExecution()}
    onRunCurrent={commands.executeCurrent}
    onRunAll={commands.executeAll}
    onRunAbove={commands.executeAbove}
    onRunBelow={commands.executeBelow}
    onClearOutputs={clearAllOutputs}
    canDeleteCell={editor.cells.length > 1}
    onDeleteCell={editor.deleteSelected}
    onAddMarkdown={() => editor.addCell('markdown')}
    onAddCode={() => editor.addCell('code')}
    onAddRaw={() => editor.addCell('raw')}
    onCollapseAllCells={collapseAllCells}
    onExpandAllCells={expandAllCells}
    canUndo={editor.canUndo}
    canRedo={editor.canRedo}
    onUndo={editor.undo}
    onRedo={editor.redo}
    cellClipboard={cellClipboard}
  />
);

NotebookEditorToolbar.displayName = 'NotebookEditorToolbar';
export default NotebookEditorToolbar;
