import { type FC } from 'react';
import type { Notebook } from './types/types';
import { useEditorPreferences } from './hooks/useEditorPreferences';
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts';
import { useKernelExecution } from './hooks/useKernelExecution';
import { useNotebookEditor } from './hooks/useNotebookEditor';
import { useNotebookPersistence } from './hooks/useNotebookPersistence';
import { useNotebookExecutionCommands } from './hooks/useNotebookExecutionCommands';
import { useNotebookUiState } from './hooks/useNotebookUiState';
import NotebookCellView from './components/notebook/NotebookCellView';
import NotebookSidebar from './components/notebook/NotebookSidebar';
import NotebookStatusBar from './components/notebook/NotebookStatusBar';
import NotebookToolbar from './components/notebook/NotebookToolbar';
import UnsavedChangesDialog from './components/dialogs/UnsavedChangesDialog';
import Header from './components/header/Header';
import 'katex/dist/katex.min.css';

const initialNotebook: Notebook = {
  format_version: 2,
  metadata: { language: 'rust', title: 'Untitled notebook' },
  cells: [
    {
      id: crypto.randomUUID(),
      cell_type: 'markdown',
      source: '# Welcome to Mars\n\nExplore ideas. Execute code. Discover possibilities.',
      execution_count: null,
      outputs: [],
      metadata: {},
    },
    {
      id: crypto.randomUUID(),
      cell_type: 'code',
      source: 'println!("Hello from Mars!");',
      execution_count: null,
      outputs: [],
      metadata: {},
    },
  ],
};

const App: FC = () => {
  const editor = useNotebookEditor(initialNotebook);
  const ui = useNotebookUiState();
  const preferences = useEditorPreferences();
  const persistence = useNotebookPersistence({
    notebook: editor.notebook,
    setNotebook: editor.setNotebook,
    setStatus: ui.setStatus,
    resetHistory: editor.resetHistory,
  });
  const execution = useKernelExecution({
    setNotebook: editor.setNotebook,
    timeout: ui.executionTimeout,
    setStatus: ui.setStatus,
  });
  const commands = useNotebookExecutionCommands({
    cells: editor.cells,
    selectedIndex: editor.selectedIndex,
    executeCells: execution.executeCells,
  });

  useKeyboardShortcuts({
    cells: editor.cells,
    selectedIndex: editor.selectedIndex,
    mode: editor.mode,
    lastCommand: ui.lastCommand,
    setMode: editor.setMode,
    setSelectedId: editor.setSelectedId,
    setLastCommand: ui.setLastCommand,
    saveNotebook: persistence.saveNotebook,
    undo: editor.undo,
    redo: editor.redo,
    executeCell: execution.executeCell,
    executeAll: commands.executeAll,
    insertCell: editor.insertCell,
    deleteSelected: editor.deleteSelected,
    convertSelected: editor.convertSelected,
    moveSelected: editor.moveSelected,
    duplicateSelected: editor.duplicateSelected,
  });

  const fileName = persistence.filePath?.split('/').pop() ?? 'Untitled.mars';
  const clearAllOutputs = () => {
    editor.clearOutputs();
    ui.setStatus('Outputs cleared');
  };

  return (
    <main data-component='App' className='flex min-h-screen flex-col'>
      <Header
        status={ui.status}
        mode={editor.mode}
        onOpen={() => void persistence.openNotebook()}
        onSave={() => void persistence.saveNotebook()}
        onExport={() => void persistence.exportMarkdown()}
        onRestart={() => void execution.restart()}
      />
      <section className='flex flex-1'>
        <NotebookSidebar
          cells={editor.cells}
          selectedId={editor.selectedId}
          filePath={persistence.filePath}
          isDirty={persistence.isDirty}
          onSelect={(id) => {
            editor.setSelectedId(id);
            editor.setMode('command');
          }}
          onRenameFile={persistence.renameFile}
        />
        <section className='mx-auto w-full max-w-5xl flex-1 px-14 py-10 pb-20'>
          <NotebookToolbar
            fileName={fileName}
            title={editor.notebook.metadata.title ?? 'Untitled notebook'}
            isDirty={persistence.isDirty}
            onRename={editor.renameNotebook}
            isExecuting={execution.isExecuting}
            executionTimeout={ui.executionTimeout}
            fontSize={preferences.fontSize}
            preferences={preferences.preferences}
            onTimeoutChange={ui.setExecutionTimeout}
            onFontSizeChange={preferences.updateFontSize}
            onPreferencesChange={preferences.setPreferences}
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
          />
          <div className='flex flex-col gap-5'>
            {editor.cells.map((cell) => (
              <NotebookCellView
                key={cell.id}
                cell={cell}
                selected={cell.id === editor.selectedId}
                fontSize={preferences.fontSize}
                autocompleteEnabled={preferences.preferences.autocomplete}
                foldingEnabled={preferences.preferences.folding}
                canDelete={editor.cells.length > 1}
                onSelect={() => {
                  editor.setSelectedId(cell.id);
                  editor.setMode('command');
                }}
                onEdit={(source) => editor.updateCell(cell.id, source)}
                onFocus={() => {
                  editor.setSelectedId(cell.id);
                  editor.setMode('edit');
                }}
                onExecute={() => void execution.executeCell(cell)}
                onClear={() => editor.clearOutputs(cell.id)}
                onDelete={() => editor.removeCell(cell.id)}
                onStatus={ui.setStatus}
              />
            ))}
          </div>
        </section>
      </section>
      <NotebookStatusBar cellCount={editor.cells.length} isDirty={persistence.isDirty} />
      {(persistence.pendingOpenPath || persistence.closeRequested) && (
        <UnsavedChangesDialog
          closeRequested={persistence.closeRequested}
          onCancel={() => {
            persistence.setPendingOpenPath(null);
            persistence.setCloseRequested(false);
          }}
          onDiscard={() =>
            persistence.closeRequested
              ? void persistence.discardAndClose()
              : persistence.discardAndOpen()
          }
          onSave={() =>
            persistence.closeRequested
              ? void persistence.saveAndClose()
              : void persistence.saveAndOpen()
          }
        />
      )}
    </main>
  );
};
App.displayName = 'App';
export default App;
