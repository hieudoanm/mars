import type { NotebookCell } from '../types/types';
import { isRunnableLanguage } from './languages';

/// Resolves a cell's language: explicit `metadata.language` wins, otherwise
/// the notebook's declared language (`fallback`, default `"rust"`).
export const cellLanguage = (cell: NotebookCell, fallback = 'rust'): string =>
  typeof cell.metadata?.language === 'string' ? cell.metadata.language : fallback;

export const isRunnableCodeCell = (cell: NotebookCell, fallback = 'rust'): boolean =>
  cell.cell_type === 'code' && isRunnableLanguage(cellLanguage(cell, fallback));
