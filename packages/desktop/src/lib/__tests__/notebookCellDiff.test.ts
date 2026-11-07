import { describe, expect, it } from 'vitest';
import { computeCellDiffs } from '../notebookCellDiff';

const notebook = (cells: { id: string; cell_type?: string; source: string }[]): string =>
  JSON.stringify({
    format_version: 2,
    metadata: { language: 'rust' },
    cells: cells.map((cell) => ({
      id: cell.id,
      cell_type: cell.cell_type ?? 'code',
      source: cell.source,
      execution_count: null,
      outputs: [],
      metadata: {},
    })),
  });

describe('computeCellDiffs', () => {
  it('returns an empty list for identical notebooks', () => {
    const content = notebook([{ id: 'a', source: 'let x = 1;' }]);
    expect(computeCellDiffs(content, content)).toEqual([]);
  });

  it('reports changed lines per cell', () => {
    const before = notebook([
      { id: 'a', source: 'let x = 1;\nlet y = 2;' },
      { id: 'b', cell_type: 'markdown', source: '# Title' },
    ]);
    const after = notebook([
      { id: 'a', source: 'let x = 1;\nlet y = 3;' },
      { id: 'b', cell_type: 'markdown', source: '# Title' },
    ]);
    const diffs = computeCellDiffs(before, after);
    expect(diffs).toHaveLength(1);
    expect(diffs[0]).toMatchObject({ cellId: 'a', index: 0, added: 1, removed: 1 });
  });

  it('treats missing cells as fully removed and new cells as fully added', () => {
    const before = notebook([{ id: 'a', source: 'old' }]);
    const after = notebook([{ id: 'b', source: 'new' }]);
    const diffs = computeCellDiffs(before, after);
    expect(diffs).toHaveLength(2);
    expect(diffs.find((diff) => diff.cellId === 'a')?.removed).toBe(1);
    expect(diffs.find((diff) => diff.cellId === 'b')?.added).toBe(1);
  });

  it('returns an empty list for invalid JSON', () => {
    expect(computeCellDiffs('not json', '{}')).toEqual([]);
  });
});
