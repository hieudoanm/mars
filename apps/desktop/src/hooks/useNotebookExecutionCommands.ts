import type { NotebookCell } from '../lib/types';

type ExecutionCommandsOptions = {
  cells: NotebookCell[];
  selectedIndex: number;
  executeCells: (cells: NotebookCell[]) => Promise<void>;
};

export function useNotebookExecutionCommands({
  cells,
  selectedIndex,
  executeCells,
}: ExecutionCommandsOptions) {
  const executeAll = () => void executeCells(cells);
  const executeAbove = () => {
    if (selectedIndex >= 0) void executeCells(cells.slice(0, selectedIndex));
  };
  const executeBelow = () => {
    if (selectedIndex >= 0) void executeCells(cells.slice(selectedIndex + 1));
  };

  return { executeAll, executeAbove, executeBelow };
}
