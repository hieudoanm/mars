import CodeEditor from "./CodeEditor";
import type { Notebook } from "./types";
import OutputView from "./OutputView";
import { diagnosticsForCell } from "./diagnostics";
import { useEditorPreferences } from "./useEditorPreferences";
import { useKeyboardShortcuts } from "./useKeyboardShortcuts";
import { useKernelExecution } from "./useKernelExecution";
import { useNotebookEditor } from "./useNotebookEditor";
import { useNotebookPersistence } from "./useNotebookPersistence";
import { useNotebookExecutionCommands } from "./useNotebookExecutionCommands";
import { useNotebookUiState } from "./useNotebookUiState";
import "katex/dist/katex.min.css";

const initialNotebook: Notebook = {
  format_version: 2,
  metadata: { language: "rust", title: "Untitled notebook" },
  cells: [
    {
      id: crypto.randomUUID(),
      cell_type: "markdown",
      source: "# Welcome to Mars\n\nExplore ideas. Execute code. Discover possibilities.",
      execution_count: null,
      outputs: [],
      metadata: {},
    },
    {
      id: crypto.randomUUID(),
      cell_type: "code",
      source: 'println!("Hello from Mars!");',
      execution_count: null,
      outputs: [],
      metadata: {},
    },
  ],
};

export default function App() {
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
  const {
    filePath, isDirty, pendingOpenPath, closeRequested,
    setPendingOpenPath, setCloseRequested, openNotebook, saveNotebook,
    discardAndOpen, saveAndOpen, discardAndClose, saveAndClose,
  } = persistence;
  const { isExecuting, executeCell, restart, cancelExecution } = execution;

  useKeyboardShortcuts({
    cells: editor.cells,
    selectedIndex: editor.selectedIndex,
    mode: editor.mode,
    lastCommand: ui.lastCommand,
    setMode: editor.setMode,
    setSelectedId: editor.setSelectedId,
    setLastCommand: ui.setLastCommand,
    saveNotebook,
    undo: editor.undo,
    redo: editor.redo,
    executeCell,
    executeAll: commands.executeAll,
    insertCell: editor.insertCell,
    deleteSelected: editor.deleteSelected,
    convertSelected: editor.convertSelected,
    moveSelected: editor.moveSelected,
    duplicateSelected: editor.duplicateSelected,
  });

  return (
    <main className="app-shell">
      <header className="titlebar">
        <div>
          <img className="brand-mark" src="/branding/mars-logo.svg" alt="Mars" />
          <span className="eyebrow">MARS NOTEBOOK</span>
          <h1>Explore ideas.</h1>
        </div>
        <div className="titlebar-actions">
          <span className="status-dot" />
          <span>{ui.status} · {editor.mode === "command" ? "Command mode" : "Edit mode"}</span>
          <button onClick={() => void openNotebook()}>Open</button>
          <button onClick={() => void saveNotebook()}>Save</button>
          <button onClick={() => void restart()}>Restart runtime</button>
        </div>
      </header>
      <section className="workspace">
        <aside className="sidebar">
          <div className="sidebar-heading">Notebook</div>
          <div className="file-card">
            <img className="file-icon" src="/branding/mars-file-logo.svg" alt="" />
            <div>
              <strong>{filePath?.split("/").pop() ?? "Untitled.mars"}{isDirty ? " •" : ""}</strong>
              <small>{isDirty ? "Unsaved changes" : "Saved notebook"}</small>
            </div>
          </div>
          <div className="sidebar-heading">Cells</div>
          {editor.cells.map((cell, index) => (
            <button
              className={`cell-nav ${cell.id === editor.selectedId ? "selected" : ""}`}
              key={cell.id}
              onClick={() => {
                editor.setSelectedId(cell.id);
                editor.setMode("command");
              }}
            >
              <span>{String(index + 1).padStart(2, "0")}</span>
              {cell.cell_type}
            </button>
          ))}
        </aside>
        <section className="notebook">
          <div className="notebook-toolbar">
            <div>
              <span className="breadcrumb">Workspace / {filePath?.split("/").pop() ?? "Untitled.mars"}{isDirty ? " • unsaved" : ""}</span>
              <h2>{editor.notebook.metadata.title ?? "Untitled notebook"}</h2>
            </div>
            <div className="toolbar-actions">
              <label className="timeout-control">
                <span>Limit (ms)</span>
                <input type="number" min="1" placeholder="Unlimited" value={ui.executionTimeout} onChange={(event) => ui.setExecutionTimeout(event.target.value)} aria-label="Execution timeout in milliseconds" />
              </label>
              {isExecuting && <button className="danger" onClick={() => void cancelExecution()}>Stop</button>}
              <button onClick={commands.executeAll}>Run all <kbd>F6</kbd></button>
              <button onClick={commands.executeAbove}>Run above</button>
              <button onClick={commands.executeBelow}>Run below</button>
              <button onClick={() => {
                editor.clearOutputs();
                ui.setStatus("Outputs cleared");
              }}>Clear outputs</button>
              <button onClick={() => editor.addCell("markdown")}>+ Markdown</button>
              <button className="primary" onClick={() => editor.addCell("code")}>+ Code</button>
              <label className="font-size-control">
                <span>Font</span>
                <input type="number" min="10" max="28" value={preferences.fontSize} aria-label="Editor font size" onChange={(event) => preferences.updateFontSize(Number(event.target.value))} />
              </label>
              <label className="editor-preference">
                <input type="checkbox" checked={preferences.preferences.autocomplete} onChange={(event) => preferences.setPreferences((current) => ({ ...current, autocomplete: event.target.checked }))} />
                Autocomplete
              </label>
              <label className="editor-preference">
                <input type="checkbox" checked={preferences.preferences.folding} onChange={(event) => preferences.setPreferences((current) => ({ ...current, folding: event.target.checked }))} />
                Folding
              </label>
            </div>
          </div>
          <div className="cell-list">
            {editor.cells.map((cell) => (
              <article className={`notebook-cell ${cell.id === editor.selectedId ? "active" : ""}`} key={cell.id} onClick={() => {
                editor.setSelectedId(cell.id);
                editor.setMode("command");
              }}>
                <div className="cell-gutter">
                  <span>{cell.cell_type === "code" ? `In [${cell.execution_count ?? " "}]` : " "}</span>
                </div>
                <div className="cell-content">
                  <div className="cell-heading">
                    <span className="cell-type">{cell.cell_type}</span>
                    {cell.cell_type === "code" && (
                      <div className="cell-actions">
                        <button onClick={(event) => {
                          event.stopPropagation();
                          void executeCell(cell);
                        }}>Run cell <kbd>⇧↵</kbd></button>
                        {cell.outputs.length > 0 && (
                          <button onClick={(event) => {
                            event.stopPropagation();
                            editor.clearOutputs(cell.id);
                            ui.setStatus("Outputs cleared");
                          }}>Clear</button>
                        )}
                      </div>
                    )}
                  </div>
                  <CodeEditor
                    value={cell.source}
                    onChange={(source) => editor.updateCell(cell.id, source)}
                    onFocus={() => {
                      editor.setSelectedId(cell.id);
                      editor.setMode("edit");
                    }}
                    placeholder={cell.cell_type === "markdown" ? "Write Markdown..." : "Write Rust code..."}
                    cellType={cell.cell_type}
                    fontSize={preferences.fontSize}
                    autocompleteEnabled={preferences.preferences.autocomplete}
                    foldingEnabled={preferences.preferences.folding}
                    diagnostics={diagnosticsForCell(cell)}
                  />
                  {cell.outputs.map((output, outputIndex) => (
                    <OutputView key={`${cell.id}-${outputIndex}`} output={output} />
                  ))}
                </div>
              </article>
            ))}
          </div>
        </section>
      </section>
      <footer className="statusbar">
        <span>Rust 2024</span>
        <span>Kernel connected</span>
        <span>{editor.cells.length} cells</span>
        <span>{isDirty ? "Unsaved changes" : "All changes saved"}</span>
      </footer>
      {(pendingOpenPath || closeRequested) && (
        <div className="modal-backdrop" role="presentation">
          <section className="confirm-modal" role="dialog" aria-modal="true" aria-labelledby="unsaved-title">
            <span className="eyebrow">UNSAVED NOTEBOOK</span>
            <h2 id="unsaved-title">{closeRequested ? "Save changes before closing?" : "Save changes before opening?"}</h2>
            <p>Your current notebook has unsaved changes. Choose how Mars should continue.</p>
            <div className="modal-actions">
              <button onClick={() => {
                setPendingOpenPath(null);
                setCloseRequested(false);
              }}>Cancel</button>
              {closeRequested ? (
                <button onClick={() => void discardAndClose()}>Close without saving</button>
              ) : (
                <button onClick={discardAndOpen}>Open without saving</button>
              )}
              {closeRequested ? (
                <button className="primary" onClick={() => void saveAndClose()}>Save and close</button>
              ) : (
                <button className="primary" onClick={() => void saveAndOpen()}>Save and open</button>
              )}
            </div>
          </section>
        </div>
      )}
    </main>
  );
}
