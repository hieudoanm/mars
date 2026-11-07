import type { NotebookCell } from '../types/types';
import { isRunnableCodeCell } from '../lib/cellLanguage';

type ExecutionCommandsOptions = {
  cells: NotebookCell[];
  selectedIndex: number;
  executeCells: (cells: NotebookCell[]) => Promise<void>;
};

export const useNotebookExecutionCommands = ({
  cells,
  selectedIndex,
  executeCells,
}: ExecutionCommandsOptions) => {
  const codeCells = (items: NotebookCell[]) => items.filter(isRunnableCodeCell);
  const executeAll = () => executeCells(codeCells(cells));
  const executeCurrent = () => {
    const cell = cells[selectedIndex];
    if (cell && isRunnableCodeCell(cell)) return executeCells([cell]);
    return Promise.resolve();
  };
  const executeAbove = () => {
    if (selectedIndex < 0) return Promise.resolve();
    return executeCells(codeCells(cells.slice(0, selectedIndex)));
  };
  const executeBelow = () => {
    if (selectedIndex < 0) return Promise.resolve();
    return executeCells(codeCells(cells.slice(selectedIndex + 1)));
  };

  return { executeAll, executeCurrent, executeAbove, executeBelow };
};
