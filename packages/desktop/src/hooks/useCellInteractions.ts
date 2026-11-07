import { useRef, useState } from 'react';
import type { NotebookCell } from '../types/types';
import type { useNotebookEditor } from './useNotebookEditor';

type Editor = ReturnType<typeof useNotebookEditor>;

type UseCellInteractionsOptions = {
  editor: Editor;
  executeCells: (cells: NotebookCell[]) => Promise<void>;
};

export const useCellInteractions = ({ editor, executeCells }: UseCellInteractionsOptions) => {
  const draggedCellId = useRef<string | null>(null);
  const [dropTargetId, setDropTargetId] = useState<string | null>(null);
  const [rangeIds, setRangeIds] = useState<ReadonlySet<string>>(new Set());

  const selectCell = (cell: NotebookCell) => {
    setRangeIds(new Set());
    editor.setSelectedId(cell.id);
    editor.setMode('command');
  };

  const shiftSelectCell = (cell: NotebookCell) => {
    const anchorIndex = editor.cells.findIndex((item) => item.id === editor.selectedId);
    const targetIndex = editor.cells.findIndex((item) => item.id === cell.id);
    if (anchorIndex < 0 || targetIndex < 0) {
      selectCell(cell);
      return;
    }
    const [start, end] =
      anchorIndex < targetIndex ? [anchorIndex, targetIndex] : [targetIndex, anchorIndex];
    setRangeIds(new Set(editor.cells.slice(start, end + 1).map((item) => item.id)));
    editor.setSelectedId(cell.id);
    editor.setMode('command');
  };

  const deleteCellOrRange = (cell: NotebookCell) => {
    if (rangeIds.size > 1 && rangeIds.has(cell.id)) {
      editor.deleteCellsByIds(rangeIds);
      setRangeIds(new Set());
    } else {
      editor.removeCell(cell.id);
    }
  };

  const runSelectedCells = () => {
    const ids = rangeIds.size > 1 ? rangeIds : new Set([editor.selectedId]);
    void executeCells(editor.cells.filter((cell) => ids.has(cell.id)));
  };

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

  return {
    rangeIds,
    setRangeIds,
    selectCell,
    shiftSelectCell,
    deleteCellOrRange,
    runSelectedCells,
    dropTargetId,
    handleDragCellStart,
    handleDragCellEnd,
    handleCellDragOver,
    handleCellDrop,
  };
};
