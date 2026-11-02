import {
  clearCellOutputs,
  convertCell,
  createCell,
  deleteCell,
  duplicateCell,
  insertCellAt,
  moveCell,
  resetNotebookOutputs,
  serializeNotebook,
  updateCellSource,
} from './notebookOperations';
import type { Notebook } from './types';

const baseNotebook: Notebook = {
  format_version: 2,
  metadata: { language: 'rust' },
  cells: [createCell('markdown')],
};

describe('notebookOperations', () => {
  it('creates markdown cell', () => {
    const cell = createCell('markdown');
    expect(cell.cell_type).toBe('markdown');
    expect(cell.source).toBe('');
  });

  it('creates code cell', () => {
    const cell = createCell('code');
    expect(cell.cell_type).toBe('code');
    expect(cell.outputs).toEqual([]);
  });

  it('duplicates cell', () => {
    const result = duplicateCell(baseNotebook, 0);
    expect(result.cell.id).not.toBe(baseNotebook.cells[0].id);
    expect(result.notebook.cells).toHaveLength(2);
  });

  it('moves cell up', () => {
    const nb: Notebook = {
      format_version: 2,
      metadata: { language: 'rust' },
      cells: [createCell('markdown'), createCell('code')],
    };
    const moved = moveCell(nb, 0, -1);
    expect(moved).toEqual(nb);
  });

  it('moves cell down', () => {
    const nb: Notebook = {
      format_version: 2,
      metadata: { language: 'rust' },
      cells: [createCell('markdown'), createCell('code')],
    };
    const moved = moveCell(nb, 0, 1);
    expect(moved.cells[1].id).toBe(nb.cells[0].id);
  });

  it('inserts cell', () => {
    const result = insertCellAt(baseNotebook, 1, 'code');
    expect(result.notebook.cells).toHaveLength(2);
  });

  it('serializes notebook to JSON', () => {
    const notebook: Notebook = {
      format_version: 2,
      metadata: { language: 'rust', title: 'Test' },
      cells: [createCell('markdown')],
    };
    const json = serializeNotebook(notebook);
    expect(JSON.parse(json)).toEqual(notebook);
  });

  it('updates cell source', () => {
    const notebook: Notebook = {
      format_version: 2,
      metadata: { language: 'rust' },
      cells: [createCell('markdown')],
    };
    const updated = updateCellSource(notebook, notebook.cells[0].id, 'new source');
    expect(updated.cells[0].source).toBe('new source');
    expect(updated.cells).toHaveLength(1);
  });

  it('deletes cell when not last', () => {
    const notebook: Notebook = {
      format_version: 2,
      metadata: { language: 'rust' },
      cells: [createCell('markdown'), createCell('code')],
    };
    const result = deleteCell(notebook, notebook.cells[0].id);
    expect(result).not.toBeNull();
    expect(result!.notebook.cells).toHaveLength(1);
    expect(result!.notebook.cells[0].cell_type).toBe('code');
    expect(result!.nextId).toBe(notebook.cells[1].id);
  });

  it("deletes cell when it's the only cell returns null", () => {
    const notebook: Notebook = {
      format_version: 2,
      metadata: { language: 'rust' },
      cells: [createCell('markdown')],
    };
    const result = deleteCell(notebook, notebook.cells[0].id);
    expect(result).toBeNull();
  });

  it('converts markdown to code cell', () => {
    const notebook: Notebook = {
      format_version: 2,
      metadata: { language: 'rust' },
      cells: [createCell('markdown')],
    };
    const converted = convertCell(notebook, notebook.cells[0].id, 'code');
    expect(converted.cells[0].cell_type).toBe('code');
    expect(converted.cells[0].execution_count).toBeNull();
    expect(converted.cells[0].outputs).toEqual([]);
  });

  it('converts code to markdown cell', () => {
    const notebook: Notebook = {
      format_version: 2,
      metadata: { language: 'rust' },
      cells: [createCell('code')],
    };
    const converted = convertCell(notebook, notebook.cells[0].id, 'markdown');
    expect(converted.cells[0].cell_type).toBe('markdown');
    expect(converted.cells[0].execution_count).toBeNull();
    expect(converted.cells[0].outputs).toEqual([]);
  });

  it('clears outputs for specific cell', () => {
    const notebook: Notebook = {
      format_version: 2,
      metadata: { language: 'rust' },
      cells: [
        { ...createCell('code'), id: 'id1', outputs: [{ type: 'text', text: 'hello' }] },
        { ...createCell('code'), id: 'id2', outputs: [{ type: 'text', text: 'world' }] },
      ],
    };
    const cleared = clearCellOutputs(notebook, 'id1');
    expect(cleared.cells[0].outputs).toEqual([]);
    expect(cleared.cells[1].outputs).toEqual([{ type: 'text', text: 'world' }]);
  });

  it('clears outputs for all cells when id omitted', () => {
    const notebook: Notebook = {
      format_version: 2,
      metadata: { language: 'rust' },
      cells: [
        { ...createCell('code'), id: 'id1', outputs: [{ type: 'text', text: 'hello' }] },
        { ...createCell('code'), id: 'id2', outputs: [{ type: 'text', text: 'world' }] },
      ],
    };
    const cleared = clearCellOutputs(notebook);
    expect(cleared.cells[0].outputs).toEqual([]);
    expect(cleared.cells[1].outputs).toEqual([]);
  });

  it('resets all outputs', () => {
    const notebook: Notebook = {
      format_version: 2,
      metadata: { language: 'rust' },
      cells: [
        {
          ...createCell('code'),
          id: 'id1',
          execution_count: 5,
          outputs: [{ type: 'text', text: 'hello' }],
        },
      ],
    };
    const reset = resetNotebookOutputs(notebook);
    expect(reset.cells[0].outputs).toEqual([]);
    expect(reset.cells[0].execution_count).toBeNull();
  });
});
