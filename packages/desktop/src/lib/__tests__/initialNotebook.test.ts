import { describe, expect, it } from 'vitest';
import { createInitialNotebook } from '../initialNotebook';

describe('createInitialNotebook', () => {
  it('loads the getting-started tour as the default notebook', () => {
    const notebook = createInitialNotebook();
    expect(notebook.metadata.title).toBe('Getting started with Mars');
    expect(notebook.cells.length).toBeGreaterThan(10);
    expect(notebook.cells.some((cell) => cell.cell_type === 'code')).toBe(true);
    expect(notebook.cells.some((cell) => cell.cell_type === 'markdown')).toBe(true);
  });

  it('starts fresh: no stored outputs or execution counts', () => {
    const notebook = createInitialNotebook();
    for (const cell of notebook.cells) {
      expect(cell.outputs).toEqual([]);
      expect(cell.execution_count).toBeNull();
    }
  });
});
