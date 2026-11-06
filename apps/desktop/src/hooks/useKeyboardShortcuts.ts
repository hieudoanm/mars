import { useEffect } from 'react';
import type { NotebookCell } from '../types/types';

type KeyboardShortcutsOptions = {
  cells: NotebookCell[];
  selectedIndex: number;
  mode: 'edit' | 'command';
  lastCommand: string;
  setMode: (mode: 'edit' | 'command') => void;
  setSelectedId: (id: string) => void;
  setLastCommand: (command: string) => void;
  saveNotebook: () => Promise<boolean>;
  undo: () => void;
  redo: () => void;
  executeCell: (cell: NotebookCell) => Promise<boolean>;
  executeAll: () => void;
  insertCell: (index: number, cellType: 'markdown' | 'code') => void;
  deleteSelected: () => void;
  convertSelected: (cellType: 'markdown' | 'code') => void;
  moveSelected: (offset: number) => void;
  duplicateSelected: () => void;
};

export const useKeyboardShortcuts = ({
  cells,
  selectedIndex,
  mode,
  lastCommand,
  setMode,
  setSelectedId,
  setLastCommand,
  saveNotebook,
  undo,
  redo,
  executeCell,
  executeAll,
  insertCell,
  deleteSelected,
  convertSelected,
  moveSelected,
  duplicateSelected,
}: KeyboardShortcutsOptions): void => {
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      const modifier = event.metaKey || event.ctrlKey;
      if (modifier && event.key.toLowerCase() === 's') {
        event.preventDefault();
        void saveNotebook();
        return;
      }
      if (modifier && event.key.toLowerCase() === 'z') {
        event.preventDefault();
        if (event.shiftKey) redo();
        else undo();
        return;
      }
      if (modifier && event.key.toLowerCase() === 'y') {
        event.preventDefault();
        redo();
        return;
      }
      if (event.key === 'Escape') {
        setMode('command');
        return;
      }
      if (selectedIndex < 0) return;
      const selectedCell = cells[selectedIndex];
      if (event.shiftKey && event.key === 'Enter') {
        event.preventDefault();
        void executeCell(selectedCell);
        const nextCell = cells[selectedIndex + 1];
        if (nextCell) setSelectedId(nextCell.id);
        return;
      }
      if (modifier && event.key === 'Enter') {
        event.preventDefault();
        void executeCell(selectedCell);
        return;
      }
      if (event.altKey && event.key === 'Enter') {
        event.preventDefault();
        void executeCell(selectedCell);
        insertCell(selectedIndex + 1, 'code');
        return;
      }
      if (event.key === 'F6') {
        event.preventDefault();
        executeAll();
        return;
      }
      const target = event.target;
      const isEditorEvent = target instanceof HTMLElement && target.closest('.cm-editor') !== null;
      if (mode !== 'command' || modifier || isEditorEvent) return;
      switch (event.key.toLowerCase()) {
        case 'a':
          event.preventDefault();
          insertCell(selectedIndex, 'code');
          break;
        case 'b':
          event.preventDefault();
          insertCell(selectedIndex + 1, 'code');
          break;
        case 'd':
          event.preventDefault();
          if (lastCommand === 'd') deleteSelected();
          setLastCommand('d');
          break;
        case 'm':
          event.preventDefault();
          convertSelected('markdown');
          break;
        case 'y':
          event.preventDefault();
          convertSelected('code');
          break;
        case 'j':
          event.preventDefault();
          moveSelected(1);
          break;
        case 'k':
          event.preventDefault();
          moveSelected(-1);
          break;
        case 'c':
          event.preventDefault();
          duplicateSelected();
          break;
        default:
          setLastCommand('');
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [
    cells,
    selectedIndex,
    mode,
    lastCommand,
    setMode,
    setSelectedId,
    setLastCommand,
    saveNotebook,
    undo,
    redo,
    executeCell,
    executeAll,
    insertCell,
    deleteSelected,
    convertSelected,
    moveSelected,
    duplicateSelected,
  ]);
};
