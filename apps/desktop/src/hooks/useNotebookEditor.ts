import { useEffect, useRef, useState } from 'react';
import type { Notebook } from '../types/types';
import {
  clearCellOutputs,
  convertCell,
  createCell,
  deleteCell,
  duplicateCell,
  insertCellAt,
  moveCell,
  serializeNotebook,
  updateCellSource,
} from '../lib/notebookOperations';

type CellType = 'markdown' | 'code';

export const useNotebookEditor = (initialNotebook: Notebook) => {
  const [notebook, setNotebook] = useState(initialNotebook);
  const [selectedId, setSelectedId] = useState(
    initialNotebook.cells[1]?.id ?? initialNotebook.cells[0]?.id ?? '',
  );
  const [mode, setMode] = useState<'edit' | 'command'>('edit');
  const historyRef = useRef<{ past: string[]; future: string[] }>({
    past: [serializeNotebook(initialNotebook)],
    future: [],
  });
  const applyingHistoryRef = useRef(false);
  const selectedIndex = notebook.cells.findIndex((cell) => cell.id === selectedId);

  useEffect(() => {
    if (applyingHistoryRef.current) {
      applyingHistoryRef.current = false;
      return;
    }
    const history = historyRef.current;
    const snapshot = serializeNotebook(notebook);
    if (history.past.at(-1) !== snapshot) {
      history.past.push(snapshot);
      history.future = [];
      if (history.past.length > 100) history.past.shift();
    }
  }, [notebook]);

  const resetHistory = (loaded: Notebook) => {
    historyRef.current = { past: [serializeNotebook(loaded)], future: [] };
    applyingHistoryRef.current = true;
  };

  const restoreSnapshot = (snapshot: string) => {
    const restored = JSON.parse(snapshot) as Notebook;
    applyingHistoryRef.current = true;
    setNotebook(restored);
    setSelectedId(restored.cells[0]?.id ?? '');
  };

  const updateCell = (id: string, source: string) => {
    setNotebook((current) => updateCellSource(current, id, source));
  };

  const renameNotebook = (title: string) => {
    setNotebook((current) => ({ ...current, metadata: { ...current.metadata, title } }));
  };

  const addCell = (cellType: CellType) => {
    const cell = createCell(cellType);
    setNotebook((current) => ({ ...current, cells: [...current.cells, cell] }));
    setSelectedId(cell.id);
    setMode('edit');
  };

  const insertCell = (index: number, cellType: CellType) => {
    const result = insertCellAt(notebook, index, cellType);
    setNotebook(result.notebook);
    setSelectedId(result.cell.id);
    setMode('edit');
  };

  const duplicateSelected = () => {
    if (selectedIndex < 0) return;
    const result = duplicateCell(notebook, selectedIndex);
    setNotebook(result.notebook);
    setSelectedId(result.cell.id);
  };

  const removeCell = (id: string) => {
    if (notebook.cells.length <= 1) return;
    const result = deleteCell(notebook, id);
    if (!result) return;
    setNotebook(result.notebook);
    if (id === selectedId) setSelectedId(result.nextId);
  };

  const deleteSelected = () => removeCell(selectedId);

  const moveSelected = (offset: number) => {
    if (selectedIndex < 0) return;
    const nextIndex = selectedIndex + offset;
    if (nextIndex < 0 || nextIndex >= notebook.cells.length) return;
    setNotebook((current) => moveCell(current, selectedIndex, offset));
  };

  const convertSelected = (cellType: CellType) => {
    if (selectedIndex < 0) return;
    setNotebook((current) => convertCell(current, selectedId, cellType));
  };

  const clearOutputs = (cellId?: string) => {
    setNotebook((current) => clearCellOutputs(current, cellId));
  };

  const undo = () => {
    const history = historyRef.current;
    if (history.past.length <= 1) return;
    const current = history.past.pop();
    if (current) history.future.push(current);
    const previous = history.past.at(-1);
    if (previous) restoreSnapshot(previous);
  };

  const redo = () => {
    const history = historyRef.current;
    const next = history.future.pop();
    if (!next) return;
    history.past.push(next);
    restoreSnapshot(next);
  };

  return {
    notebook,
    setNotebook,
    cells: notebook.cells,
    selectedId,
    setSelectedId,
    selectedIndex,
    mode,
    setMode,
    updateCell,
    renameNotebook,
    addCell,
    insertCell,
    duplicateSelected,
    removeCell,
    deleteSelected,
    moveSelected,
    convertSelected,
    clearOutputs,
    undo,
    redo,
    resetHistory,
  };
};
