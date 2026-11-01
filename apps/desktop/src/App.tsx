import { useRef, useState, type FC } from 'react';
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
import FileExplorer from './components/explorer/FileExplorer';

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
    onExecutionHalted: (cell) => {
      editor.setSelectedId(cell.id);
      editor.setMode('command');
      window.requestAnimationFrame(() => {
        document.getElementById(`cell-${cell.id}`)?.scrollIntoView({ block: 'center' });
      });
    },
  });
  const commands = useNotebookExecutionCommands({
    cells: editor.cells,
    selectedIndex: editor.selectedIndex,
    executeCells: execution.executeCells,
  });
  const draggedCellId = useRef<string | null>(null);
  const [dropTargetId, setDropTargetId] = useState<string | null>(null);
  const [collapsedCellIds, setCollapsedCellIds] = useState<ReadonlySet<string>>(new Set());
  const toggleCellCollapse = (id: string) => {
    setCollapsedCellIds((current) => {
      const next = new Set(current);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };
  const collapseAllCells = () => setCollapsedCellIds(new Set(editor.cells.map((cell) => cell.id)));
  const expandAllCells = () => setCollapsedCellIds(new Set());
  const handleDragCellStart = (id: string) => {
    draggedCellId.current = id;
  };
  const handleDragCellEnd = () => {
    draggedCellId.current = null;
    setDropTargetId(null);
  };
  const handleCellDragOver = (id: string) => {
    if (draggedCellId.current && draggedCellId.current !== id) setDropTargetId(id);
  };
  const handleCellDrop = (id: string) => {
    const fromId = draggedCellId.current;
    draggedCellId.current = null;
    setDropTargetId(null);
    if (fromId && fromId !== id) editor.moveCellTo(fromId, id);
  };

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
    <main data-component='App' className='flex h-screen flex-col overflow-hidden'>
      <Header
        status={ui.status}
        mode={editor.mode}
        onOpen={() => void persistence.openNotebook()}
        onSave={() => void persistence.saveNotebook()}
        onExport={() => void persistence.exportMarkdown()}
        onImportIpynb={() => void persistence.importIpynb()}
        onExportIpynb={() => void persistence.exportIpynb()}
        onRestart={() => void execution.restart()}
        isExecuting={execution.isExecuting}
      />
      <section className='flex min-h-0 flex-1 overflow-hidden'>
        <FileExplorer
          currentFilePath={persistence.filePath}
          onOpenNotebook={persistence.openNotebookAtPath}
          onStatus={ui.setStatus}
        />
        <section className='min-w-0 flex-1 overflow-y-auto'>
          <div className='mx-auto w-full max-w-5xl px-8 py-10 pb-20'>
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
              onAddRaw={() => editor.addCell('raw')}
              onCollapseAllCells={collapseAllCells}
              onExpandAllCells={expandAllCells}
            />
            <div className='flex flex-col gap-5'>
              {editor.cells.map((cell) => (
                <NotebookCellView
                  key={cell.id}
                  cell={cell}
                  selected={cell.id === editor.selectedId}
                  isExecuting={cell.id === execution.activeCellId}
                  fontSize={preferences.fontSize}
                  autocompleteEnabled={preferences.preferences.autocomplete}
                  foldingEnabled={preferences.preferences.folding}
                  canDelete={editor.cells.length > 1}
                  collapsed={collapsedCellIds.has(cell.id)}
                  stream={execution.stream?.cellId === cell.id ? execution.stream : null}
                  isDropTarget={dropTargetId === cell.id}
                  onDragCellStart={handleDragCellStart}
                  onDragCellEnd={handleDragCellEnd}
                  onCellDragOver={handleCellDragOver}
                  onCellDrop={handleCellDrop}
                  onSelect={() => {
                    editor.setSelectedId(cell.id);
                    editor.setMode('command');
                  }}
                  onEdit={(source) => editor.updateCell(cell.id, source)}
                  onFocus={() => {
                    editor.setSelectedId(cell.id);
                    editor.setMode('edit');
                  }}
                  onExecute={() => {
                    editor.setSelectedId(cell.id);
                    void execution.executeCell(cell);
                  }}
                  onClear={() => editor.clearOutputs(cell.id)}
                  onDelete={() => editor.removeCell(cell.id)}
                  onToggleCollapse={() => toggleCellCollapse(cell.id)}
                  onStatus={ui.setStatus}
                />
              ))}
            </div>
          </div>
        </section>
        <NotebookSidebar
          cells={editor.cells}
          selectedId={editor.selectedId}
          filePath={persistence.filePath}
          isDirty={persistence.isDirty}
          onSelect={(id) => {
            editor.setSelectedId(id);
            editor.setMode('command');
            window.requestAnimationFrame(() => {
              document.getElementById(`cell-${id}`)?.scrollIntoView({ block: 'center' });
            });
          }}
          onRenameFile={persistence.renameFile}
        />
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
