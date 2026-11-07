import { useEffect, useRef } from 'react';
import type { PaletteCommand } from '../components/palette/CommandPalette';

const DEFINITIONS: [string, string, string?][] = [
  ['run-cell', 'Run selected cell'],
  ['run-all', 'Run all cells'],
  ['restart', 'Restart runtime'],
  ['restart-run-all', 'Restart & run all'],
  ['run-selected', 'Run selected cells'],
  ['clear-outputs', 'Clear all outputs'],
  ['save', 'Save notebook', 'Cmd+S'],
  ['search', 'Search notebook', 'Cmd+Shift+F'],
  ['dependencies', 'Edit dependencies'],
  ['checkpoints', 'Checkpoints / version history'],
  ['rename', 'Rename notebook'],
  ['edit-metadata', 'Edit selected cell metadata'],
  ['toggle-sidebar', 'Toggle notebook sidebar'],
  ['settings', 'Open settings'],
  ['settings', 'Keyboard shortcuts', '?'],
];

export const useNotebookCommands = (
  sessionId: string,
  registerCommands: ((sessionId: string, commands: PaletteCommand[]) => void) | undefined,
  actions: Record<string, () => void>,
) => {
  const actionsRef = useRef(actions);
  actionsRef.current = actions;

  useEffect(() => {
    if (!registerCommands) return;
    registerCommands(
      sessionId,
      DEFINITIONS.map(([id, title, hint]) => ({
        id,
        title,
        hint,
        run: () => actionsRef.current[id]?.(),
      })),
    );
    return () => registerCommands(sessionId, []);
  }, [registerCommands, sessionId]);
};
