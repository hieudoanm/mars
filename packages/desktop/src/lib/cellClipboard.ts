import type { CellType, NotebookCell } from '../types/types';

export type CellClipboardEntry = {
  source: string;
  cellType: CellType;
  language?: string;
};

let clipboard: CellClipboardEntry | null = null;

export const copyCellToClipboard = (cell: NotebookCell): void => {
  clipboard = {
    source: cell.source,
    cellType: cell.cell_type,
    language:
      typeof cell.metadata?.language === 'string' ? (cell.metadata.language as string) : undefined,
  };
};

export const pasteCellFromClipboard = (): CellClipboardEntry | null => clipboard;

export const clearCellClipboard = (): void => {
  clipboard = null;
};
