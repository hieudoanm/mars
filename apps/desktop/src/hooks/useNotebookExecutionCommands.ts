import type { NotebookCell } from '../types/types';

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
  const codeCells = (items: NotebookCell[]) => items.filter((cell) => cell.cell_type === 'code');
  const executeAll = () => void executeCells(codeCells(cells));
  const executeCurrent = () => {
    const cell = cells[selectedIndex];
    if (cell?.cell_type === 'code') void executeCells([cell]);
  };
  const executeAbove = () => {
    if (selectedIndex >= 0) void executeCells(codeCells(cells.slice(0, selectedIndex)));
  };
  const executeBelow = () => {
    if (selectedIndex >= 0) void executeCells(codeCells(cells.slice(selectedIndex + 1)));
  };

  return { executeAll, executeCurrent, executeAbove, executeBelow };
};
