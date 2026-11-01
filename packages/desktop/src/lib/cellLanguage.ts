import type { NotebookCell } from '../types/types';

export const SUPPORTED_LANGUAGES = ['rust', 'python', 'r', 'julia'] as const;

/// Resolves a cell's language: explicit `metadata.language` wins, otherwise
/// the notebook's declared language (`fallback`, default `"rust"`).
export const cellLanguage = (cell: NotebookCell, fallback = 'rust'): string =>
  typeof cell.metadata?.language === 'string' ? cell.metadata.language : fallback;

export const isRunnableCodeCell = (cell: NotebookCell, fallback = 'rust'): boolean =>
  cell.cell_type === 'code' &&
  (SUPPORTED_LANGUAGES as readonly string[]).includes(cellLanguage(cell, fallback));
