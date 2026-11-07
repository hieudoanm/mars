import { useCallback, useEffect, useRef, useState, type FC } from 'react';
import type { Notebook } from './types/types';
import type { EditorPreferences } from './hooks/useEditorPreferences';
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts';
import { useKernelExecution } from './hooks/useKernelExecution';
import { useNotebookEditor } from './hooks/useNotebookEditor';
import { useNotebookPersistence } from './hooks/useNotebookPersistence';
import { useNotebookExecutionCommands } from './hooks/useNotebookExecutionCommands';
import { useNotebookUiState } from './hooks/useNotebookUiState';
import { useSessionRestore } from './hooks/useSessionRestore';
import { useNotebookInspection } from './hooks/useNotebookInspection';
import NotebookCellList from './components/notebook/NotebookCellList';
import NotebookSidebar from './components/notebook/NotebookSidebar';
import NotebookStatusBar from './components/notebook/NotebookStatusBar';
import NotebookToolbar from './components/notebook/NotebookToolbar';
import NotebookHeader from './components/header/NotebookHeader';
import NotebookSessionOverlays from './components/notebook/NotebookSessionOverlays';
import { ToastViewport } from './components/shared';
import type { PaletteCommand } from './components/palette/CommandPalette';
import { openInTerminal } from './lib/kernelService';
import { writeUntitledNotebook } from './lib/workspace';
import { useLspIntegration } from './hooks/useLspIntegration';
import { useCellInteractions } from './hooks/useCellInteractions';
import { cellSourceHash } from './lib/cellHash';
import { isRunnableCodeCell } from './lib/cellLanguage';

const createInitialNotebook = (): Notebook => ({
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
});

export type SessionPreferences = EditorPreferences & {
  fontSize: number;
  updateFontSize: (value: number) => void;
  colorScheme: 'dark' | 'light';
};

export type NotebookSessionProps = {
  sessionId: string;
  initialPath?: string | null;
  untitledId?: string | null;
  initialNotebook?: Notebook;
  isActive: boolean;
  tabCloseRequested?: boolean;
  preferences: SessionPreferences;
  onPathChange: (sessionId: string, path: string | null) => void;
  onDirtyChange: (sessionId: string, isDirty: boolean) => void;
  onTabClosed: (sessionId: string) => void;
  onTabCloseCancel: (sessionId: string) => void;
  registerCommands?: (sessionId: string, commands: PaletteCommand[]) => void;
  onOpenSettings?: () => void;
};

const NotebookSession: FC<NotebookSessionProps> = ({
  sessionId,
  initialPath = null,
  untitledId = null,
  initialNotebook: initialNotebookProp,
  isActive,
  tabCloseRequested = false,
  preferences,
  onPathChange,
  onDirtyChange,
  onTabClosed,
  onTabCloseCancel,
  registerCommands,
  onOpenSettings,
}) => {
  const [initialNotebook] = useState(() => initialNotebookProp ?? createInitialNotebook());
  const editor = useNotebookEditor(initialNotebook);
  const ui = useNotebookUiState();
  const handlePathChange = useCallback(
    (path: string | null) => onPathChange(sessionId, path),
    [onPathChange, sessionId],
  );
  const handleDirtyChange = useCallback(
    (isDirty: boolean) => onDirtyChange(sessionId, isDirty),
    [onDirtyChange, sessionId],
  );
  const persistence = useNotebookPersistence({
    sessionId,
    notebook: editor.notebook,
    setNotebook: editor.setNotebook,
    setStatus: ui.setStatus,
    notify: ui.notify,
    resetHistory: editor.resetHistory,
    initialPath,
    onPathChange: handlePathChange,
  });
  const execution = useKernelExecution({
    sessionId,
    setNotebook: editor.setNotebook,
    timeout: ui.executionTimeout,
    setStatus: ui.setStatus,
    onCellDirectives: (_cell, dependencies) => {
      editor.setDependencies({
        ...(editor.notebook.metadata.dependencies ?? {}),
        ...Object.fromEntries(dependencies),
      });
      ui.setStatus(`Added ${dependencies.length} cell directive(s) to dependencies`);
    },
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
  const {
    rangeIds,
    selectCell,
    shiftSelectCell,
    deleteCellOrRange,
    runSelectedCells,
    dropTargetId,
    handleDragCellStart,
    handleDragCellEnd,
    handleCellDragOver,
    handleCellDrop,
  } = useCellInteractions({ editor, executeCells: execution.executeCells });
  const { collapsedCellIds, setCollapsedCellIds, dialogs, setDialogOpen } = ui;
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const scrollRef = useRef<HTMLDivElement>(null);
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
    enabled: isActive,
  });

  useEffect(() => {
    handleDirtyChange(persistence.isDirty);
  }, [persistence.isDirty, handleDirtyChange]);

  useEffect(() => {
    if (!untitledId || persistence.filePath) return;
    const timer = window.setTimeout(() => {
      writeUntitledNotebook(untitledId, editor.notebook);
    }, 800);
    return () => window.clearTimeout(timer);
  }, [untitledId, persistence.filePath, editor.notebook]);

  const { liveVariables, refreshVariables, preDiagnostics, lspDiagnosticsByCell, problems } =
    useNotebookInspection({
      sessionId,
      cells: editor.cells,
      selectedIndex: editor.selectedIndex,
      executionCount: execution.kernelInfo?.executionCount ?? 0,
    });

  const revealLspCell = useCallback(
    (cellId: string) => {
      editor.setSelectedId(cellId);
      editor.setMode('command');
      window.requestAnimationFrame(() => {
        document.getElementById(`cell-${cellId}`)?.scrollIntoView({ block: 'center' });
      });
    },
    [editor],
  );
  const {
    handleHover: handleLspHover,
    handlers: lspHandlers,
    pendingReveal,
    reveal: revealLspTarget,
  } = useLspIntegration({
    sessionId,
    cells: editor.cells,
    dependencies: editor.notebook.metadata.dependencies,
    onRevealCell: revealLspCell,
  });

  const navigateToSourceLocation = useCallback(
    (cellHash: number, line: number) => {
      const target = editor.cells.find(
        (cell) => isRunnableCodeCell(cell) && cellSourceHash(cell.source) === cellHash,
      );
      if (!target) return;
      revealLspTarget({ cellId: target.id, line: Math.max(0, line - 1), character: 0 });
    },
    [editor.cells, revealLspTarget],
  );

  const paletteActionsRef = useRef<Record<string, () => void>>({});
  paletteActionsRef.current = {
    'run-cell': () => void execution.executeCell(editor.cells[editor.selectedIndex]),
    'run-all': () => void commands.executeAll(),
    restart: () => void execution.restart(),
    'restart-run-all': () => void restartAndRunAll(),
    'run-selected': () => runSelectedCells(),
    'clear-outputs': () => clearAllOutputs(),
    save: () => void persistence.saveNotebook(),
    search: () => setDialogOpen('search', true),
    dependencies: () => setDialogOpen('dependencies', true),
    checkpoints: () => setDialogOpen('checkpoints', true),
    rename: () => setDialogOpen('rename', true),
    settings: () => onOpenSettings?.(),
  };

  useEffect(() => {
    if (!registerCommands) return;
    const definitions: [string, string, string?][] = [
      ['run-cell', 'Run selected cell'],
      ['run-selected', 'Run selected cells'],
      ['run-all', 'Run all cells'],
      ['restart', 'Restart runtime'],
      ['restart-run-all', 'Restart & run all'],
      ['clear-outputs', 'Clear all outputs'],
      ['save', 'Save notebook', 'Cmd+S'],
      ['search', 'Search notebook', 'Cmd+Shift+F'],
      ['dependencies', 'Edit dependencies'],
      ['checkpoints', 'Checkpoints / version history'],
      ['rename', 'Rename notebook'],
      ['settings', 'Open settings'],
    ];
    registerCommands(
      sessionId,
      definitions.map(([id, title, hint]) => ({
        id,
        title,
        hint,
        run: () => paletteActionsRef.current[id]?.(),
      })),
    );
    return () => registerCommands(sessionId, []);
  }, [registerCommands, sessionId]);

  const fileName = persistence.filePath?.split('/').pop() ?? 'Untitled.mars';
  const clearAllOutputs = () => {
    editor.clearOutputs();
    ui.setStatus('Outputs cleared');
  };
  const restartAndRunAll = async () => {
    await execution.restart();
    await commands.executeAll();
  };

  const showTabCloseDialog = tabCloseRequested && persistence.isDirty;

  const filePath = persistence.filePath;
  useSessionRestore({
    filePath,
    scrollRef,
    setSelectedId: editor.setSelectedId,
    setMode: editor.setMode,
    collapsedCellIds,
    setCollapsedCellIds,
    selectedId: editor.selectedId,
  });

  return (
    <div data-component='NotebookSession' className='flex h-full min-h-0 flex-col'>
      <NotebookHeader
        showBrand={false}
        status={ui.status}
        mode={editor.mode}
        onOpen={() => void persistence.openNotebook()}
        onSave={() => void persistence.saveNotebook()}
        onExport={() => void persistence.exportMarkdown()}
        onExportHtml={() => void persistence.exportHtml()}
        onExportPdf={() => void persistence.exportPdf()}
        onImportIpynb={() => void persistence.importIpynb()}
        onExportIpynb={() => void persistence.exportIpynb()}
        onRestart={() => void execution.restart()}
        onRestartRunAll={() => void restartAndRunAll()}
        onOpenTerminal={
          persistence.filePath
            ? () => {
                void openInTerminal(persistence.filePath as string).catch((error: unknown) => {
                  const message =
                    error instanceof Error ? error.message : 'Could not open terminal';
                  ui.setStatus(message);
                  ui.notify(message, 'error');
                });
              }
            : undefined
        }
        isExecuting={execution.isExecuting}
        sidebarOpen={sidebarOpen}
        onToggleSidebar={() => setSidebarOpen((current) => !current)}
      />
      <section className='flex min-h-0 flex-1 overflow-hidden'>
        <div className='flex min-h-0 min-w-0 flex-1 flex-col'>
          <div className='mx-auto w-full max-w-4xl px-6 pt-8'>
            <NotebookToolbar
              fileName={fileName}
              isDirty={persistence.isDirty}
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
            />
          </div>
          <div ref={scrollRef} className='min-h-0 flex-1 overflow-y-auto'>
            <div className='mx-auto w-full max-w-4xl px-6 pb-16'>
              <NotebookCellList
                cells={editor.cells}
                editor={editor}
                selectedId={editor.selectedId}
                rangeIds={rangeIds}
                activeCellId={execution.activeCellId}
                stream={execution.stream}
                runInfo={execution.runInfo}
                fontSize={preferences.fontSize}
                autocompleteEnabled={preferences.autocomplete}
                foldingEnabled={preferences.folding}
                colorScheme={preferences.colorScheme}
                canDelete={editor.cells.length > 1}
                collapsedCellIds={collapsedCellIds}
                toggleCellCollapse={toggleCellCollapse}
                preDiagnostics={preDiagnostics}
                lspDiagnosticsByCell={lspDiagnosticsByCell}
                lsp={lspHandlers}
                pendingReveal={pendingReveal}
                onLspHover={handleLspHover}
                onNavigateSource={navigateToSourceLocation}
                onRunAllCells={() => void commands.executeAll()}
                executeCell={(cell) => void execution.executeCell(cell)}
                setStatus={ui.setStatus}
                selectCell={selectCell}
                shiftSelectCell={shiftSelectCell}
                deleteCellOrRange={deleteCellOrRange}
                dropTargetId={dropTargetId}
                onDragCellStart={handleDragCellStart}
                onDragCellEnd={handleDragCellEnd}
                onCellDragOver={handleCellDragOver}
                onCellDrop={handleCellDrop}
              />
            </div>
          </div>
        </div>
        <NotebookSidebar
          open={sidebarOpen}
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
          onOpenSearch={() => setDialogOpen('search', true)}
          onOpenDependencies={() => setDialogOpen('dependencies', true)}
          onOpenCheckpoints={() => setDialogOpen('checkpoints', true)}
          liveVariables={liveVariables}
          onRefreshVariables={refreshVariables}
          problems={problems}
        />
      </section>
      <NotebookStatusBar
        cellCount={editor.cells.length}
        isDirty={persistence.isDirty}
        kernelInfo={execution.kernelInfo}
        isExecuting={execution.isExecuting}
        sessionId={sessionId}
        queueLength={execution.queueLength}
      />
      <NotebookSessionOverlays
        persistence={persistence}
        editor={editor}
        pendingPrompt={execution.pendingPrompt}
        respondToPrompt={execution.respondToPrompt}
        showTabCloseDialog={showTabCloseDialog}
        onTabClosed={() => onTabClosed(sessionId)}
        onTabCloseCancel={() => onTabCloseCancel(sessionId)}
        showSearch={dialogs.search}
        setShowSearch={(open) => setDialogOpen('search', open)}
        showDependencies={dialogs.dependencies}
        setShowDependencies={(open) => setDialogOpen('dependencies', open)}
        showCheckpoints={dialogs.checkpoints}
        setShowCheckpoints={(open) => setDialogOpen('checkpoints', open)}
        showRename={dialogs.rename}
        setShowRename={(open) => setDialogOpen('rename', open)}
        setStatus={ui.setStatus}
        onRevealMatch={(cellId, line, character) => revealLspTarget({ cellId, line, character })}
      />
      <ToastViewport toasts={ui.toasts} onDismiss={ui.dismissToast} />
    </div>
  );
};
NotebookSession.displayName = 'NotebookSession';
export default NotebookSession;
