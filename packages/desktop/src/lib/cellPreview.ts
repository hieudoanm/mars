import type { NotebookCell } from '../types/types';

export const cellPreview = (cell: NotebookCell): string => {
  const line = cell.source
    .split('\n')
    .map((entry) => entry.trim())
    .find((entry) => entry.length > 0);
  return line ?? '';
};
