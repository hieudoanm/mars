import { describe, expect, it } from 'vitest';
import { clearCellClipboard, copyCellToClipboard, pasteCellFromClipboard } from '../cellClipboard';
import type { Notebook } from '../../types/types';
import { pasteCellAt } from '../notebookOperations';
import type { NotebookCell } from '../../types/types';

const cell = (overrides: Partial<NotebookCell> = {}): NotebookCell => ({
  id: 'c1',
  cell_type: 'code',
  source: 'let x = 1;',
  execution_count: 2,
  outputs: [],
  metadata: {},
  ...overrides,
});

const notebook = (cells: NotebookCell[]): Notebook => ({
  format_version: 1,
  metadata: { language: 'rust' },
  cells,
});

describe('cellClipboard', () => {
  it('stores and returns the last copied cell', () => {
    clearCellClipboard();
    copyCellToClipboard(cell({ source: 'let y = 2;', metadata: { language: 'rust' } }));
    const entry = pasteCellFromClipboard();
    expect(entry).toMatchObject({
      source: 'let y = 2;',
      cellType: 'code',
      language: 'rust',
    });
    clearCellClipboard();
    expect(pasteCellFromClipboard()).toBeNull();
  });
});

describe('pasteCellAt', () => {
  it('inserts a cell with the clipboard source, type, and language', () => {
    const doc = notebook([cell({ id: 'a' }), cell({ id: 'b', cell_type: 'markdown' })]);
    const result = pasteCellAt(doc, 1, {
      source: 'fn main() {}',
      cellType: 'code',
      language: 'rust',
    });
    expect(result.notebook.cells).toHaveLength(3);
    const pasted = result.notebook.cells[1];
    expect(pasted.source).toBe('fn main() {}');
    expect(pasted.cell_type).toBe('code');
    expect(pasted.metadata.language).toBe('rust');
    expect(result.cell.id).toBe(pasted.id);
  });

  it('inserts a markdown cell without a language', () => {
    const doc = notebook([cell({ id: 'a' })]);
    const result = pasteCellAt(doc, 0, { source: '# Title', cellType: 'markdown' });
    expect(result.notebook.cells[0].cell_type).toBe('markdown');
    expect(result.notebook.cells[0].metadata.language).toBeUndefined();
  });
});
