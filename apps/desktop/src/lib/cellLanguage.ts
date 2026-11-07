import type { NotebookCell } from '../types/types';

export const cellLanguage = (cell: NotebookCell): string =>
  typeof cell.metadata?.language === 'string' ? cell.metadata.language : 'rust';

export const isRunnableCodeCell = (cell: NotebookCell): boolean =>
  cell.cell_type === 'code' && cellLanguage(cell) === 'rust';
