import { useEffect, useRef, useState } from 'react';
import type { CellType, Notebook } from '../types/types';
import {
  clearCellOutputs,
  convertCell,
  createCell,
  deleteCell,
  deleteCells,
  duplicateCell,
  insertCellAt,
  pasteCellAt,
  mergeCellWithNext,
  moveCell,
  moveCellToIndex,
  removeCellOutput,
  serializeNotebook,
  splitCellAt,
  updateCellSource,
  updateCellAttachments,
  updateCellLanguage,
  updateCellComments,
  updateCellMetadata as updateNotebookCellMetadata,
  updateCellTags,
  updateNotebookDependencies,
  type PastedCellContent,
} from '../lib/notebookOperations';
import { getCellAttachments } from '../lib/cellAttachments';

const MAX_HISTORY_ENTRIES = 100;
const MAX_HISTORY_BYTES = 2_000_000;

const trimHistory = (snapshots: string[]) => {
  let size = snapshots.reduce((total, snapshot) => total + snapshot.length, 0);
  while (
    snapshots.length > 1 &&
    (snapshots.length > MAX_HISTORY_ENTRIES || size > MAX_HISTORY_BYTES)
  ) {
    const removed = snapshots.shift();
    size -= removed?.length ?? 0;
  }
};

export const useNotebookEditor = (initialNotebook: Notebook) => {
  const [notebook, setNotebook] = useState(initialNotebook);
  const [selectedId, setSelectedId] = useState(
    initialNotebook.cells[1]?.id ?? initialNotebook.cells[0]?.id ?? '',
  );
  const [mode, setMode] = useState<'edit' | 'command'>('edit');
  const [historyTick, setHistoryTick] = useState(0);
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
      trimHistory(history.past);
      setHistoryTick((tick) => tick + 1);
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

  const pasteCell = (index: number, content: PastedCellContent) => {
    const result = pasteCellAt(notebook, index, content);
    setNotebook(result.notebook);
    setSelectedId(result.cell.id);
    setMode('command');
  };

  const duplicateCellById = (id: string) => {
    const index = notebook.cells.findIndex((cell) => cell.id === id);
    if (index < 0) return;
    const result = duplicateCell(notebook, index);
    setNotebook(result.notebook);
    setSelectedId(result.cell.id);
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

  const deleteCellsByIds = (ids: ReadonlySet<string>) => {
    setNotebook((current) => deleteCells(current, ids));
    if (ids.has(selectedId)) {
      const remaining = notebook.cells.filter((cell) => !ids.has(cell.id));
      setSelectedId(remaining[0]?.id ?? '');
    }
  };

  const splitCellById = (id: string, offset: number) => {
    setNotebook((current) => splitCellAt(current, id, offset));
  };

  const moveSelected = (offset: number) => {
    if (selectedIndex < 0) return;
    const nextIndex = selectedIndex + offset;
    if (nextIndex < 0 || nextIndex >= notebook.cells.length) return;
    setNotebook((current) => moveCell(current, selectedIndex, offset));
  };

  const moveCellTo = (fromId: string, toId: string) => {
    const fromIndex = notebook.cells.findIndex((cell) => cell.id === fromId);
    const toIndex = notebook.cells.findIndex((cell) => cell.id === toId);
    if (fromIndex < 0 || toIndex < 0 || fromIndex === toIndex) return;
    setNotebook((current) => moveCellToIndex(current, fromIndex, toIndex));
    setSelectedId(fromId);
  };

  const convertSelected = (cellType: CellType) => {
    if (selectedIndex < 0) return;
    setNotebook((current) => convertCell(current, selectedId, cellType));
  };

  const convertCellType = (id: string, cellType: CellType) => {
    setNotebook((current) => convertCell(current, id, cellType));
  };

  const moveCellById = (id: string, offset: number) => {
    setNotebook((current) => {
      const index = current.cells.findIndex((cell) => cell.id === id);
      if (index < 0) return current;
      return moveCell(current, index, offset);
    });
  };

  const clearOutputs = (cellId?: string) => {
    setNotebook((current) => clearCellOutputs(current, cellId));
  };

  const removeOutput = (cellId: string, outputIndex: number) => {
    setNotebook((current) => removeCellOutput(current, cellId, outputIndex));
  };

  const mergeCellById = (id: string) => {
    setNotebook((current) => mergeCellWithNext(current, id));
  };

  const mergeSelectedWithNext = () => {
    if (selectedIndex < 0) return;
    setNotebook((current) => mergeCellWithNext(current, selectedId));
  };

  const splitSelectedAt = (offset: number) => {
    if (selectedIndex < 0) return;
    setNotebook((current) => splitCellAt(current, selectedId, offset));
  };

  const updateCellMetadata = (cellId: string, metadata: Record<string, unknown>) => {
    setNotebook((current) => updateNotebookCellMetadata(current, cellId, metadata));
  };

  const updateTags = (cellId: string, tags: string[]) => {
    setNotebook((current) => updateCellTags(current, cellId, tags));
  };

  const updateComments = (cellId: string, comments: unknown[]) => {
    setNotebook((current) => updateCellComments(current, cellId, comments));
  };

  const setCellLanguage = (cellId: string, language: string) => {
    setNotebook((current) => updateCellLanguage(current, cellId, language));
  };

  const updateAttachments = (cellId: string, patch: Record<string, string>) => {
    setNotebook((current) => {
      const cell = current.cells.find((entry) => entry.id === cellId);
      const existing = cell ? getCellAttachments(cell.metadata) : {};
      return updateCellAttachments(current, cellId, { ...existing, ...patch });
    });
  };

  const setDependencies = (dependencies: Record<string, string>) => {
    setNotebook((current) => updateNotebookDependencies(current, dependencies));
  };

  const undo = () => {
    const history = historyRef.current;
    if (history.past.length <= 1) return;
    const current = history.past.pop();
    if (current) history.future.push(current);
    const previous = history.past.at(-1);
    if (previous) restoreSnapshot(previous);
    setHistoryTick((tick) => tick + 1);
  };

  const redo = () => {
    const history = historyRef.current;
    const next = history.future.pop();
    if (!next) return;
    history.past.push(next);
    restoreSnapshot(next);
    setHistoryTick((tick) => tick + 1);
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
    pasteCell,
    duplicateSelected,
    duplicateCellById,
    removeCell,
    deleteSelected,
    deleteCellsByIds,
    splitCellById,
    moveSelected,
    moveCellTo,
    convertSelected,
    convertCellType,
    moveCellById,
    clearOutputs,
    removeOutput,
    mergeCellById,
    mergeSelectedWithNext,
    splitSelectedAt,
    updateCellMetadata,
    updateTags,
    updateComments,
    setCellLanguage,
    updateAttachments,
    setDependencies,
    undo,
    redo,
    canUndo: historyRef.current.past.length > 1,
    canRedo: historyRef.current.future.length > 0,
    historyTick,
    resetHistory,
  };
};
