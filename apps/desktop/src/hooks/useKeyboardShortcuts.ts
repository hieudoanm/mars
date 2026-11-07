import { useEffect } from 'react';
import type { CellType, NotebookCell } from '../types/types';
import { comboMatches, getShortcut } from '../lib/shortcuts';

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
  insertCell: (index: number, cellType: CellType) => void;
  deleteSelected: () => void;
  convertSelected: (cellType: CellType) => void;
  moveSelected: (offset: number) => void;
  duplicateSelected: () => void;
  enabled?: boolean;
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
  enabled = true,
}: KeyboardShortcutsOptions): void => {
  useEffect(() => {
    if (!enabled) return;
    const onKeyDown = (event: KeyboardEvent) => {
      const modifier = event.metaKey || event.ctrlKey;
      const target = event.target;
      const isCodeEditor = target instanceof HTMLElement && target.closest('.cm-editor') !== null;
      const isPlainTextInput =
        target instanceof HTMLElement &&
        !isCodeEditor &&
        (target.matches('input, textarea, select') ||
          target.isContentEditable ||
          target.closest('[role="textbox"]') !== null);
      if (isPlainTextInput) return;

      const matches = (id: string) => comboMatches(getShortcut(id), event);

      if (matches('save')) {
        event.preventDefault();
        void saveNotebook();
        return;
      }
      if (matches('undo')) {
        event.preventDefault();
        undo();
        return;
      }
      if (matches('redo') || matches('redo-alt')) {
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
      if (matches('run-cell')) {
        event.preventDefault();
        void executeCell(selectedCell);
        const nextCell = cells[selectedIndex + 1];
        if (nextCell) setSelectedId(nextCell.id);
        return;
      }
      if (matches('run-stay')) {
        event.preventDefault();
        void executeCell(selectedCell);
        return;
      }
      if (matches('run-insert')) {
        event.preventDefault();
        void executeCell(selectedCell);
        insertCell(selectedIndex + 1, 'code');
        return;
      }
      if (matches('run-all')) {
        event.preventDefault();
        executeAll();
        return;
      }
      if (mode !== 'command' || modifier || isCodeEditor) return;

      const commandActions: [string, () => void][] = [
        ['insert-above', () => insertCell(selectedIndex, 'code')],
        ['insert-below', () => insertCell(selectedIndex + 1, 'code')],
        ['to-markdown', () => convertSelected('markdown')],
        ['to-code', () => convertSelected('code')],
        ['to-raw', () => convertSelected('raw')],
        ['move-down', () => moveSelected(1)],
        ['move-up', () => moveSelected(-1)],
        ['duplicate', () => duplicateSelected()],
      ];
      for (const [id, action] of commandActions) {
        if (matches(id)) {
          event.preventDefault();
          action();
          setLastCommand('');
          return;
        }
      }
      if (matches('delete-cell')) {
        event.preventDefault();
        if (lastCommand === getShortcut('delete-cell')) deleteSelected();
        setLastCommand(getShortcut('delete-cell'));
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
    enabled,
  ]);
};
