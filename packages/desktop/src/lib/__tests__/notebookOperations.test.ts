import {
  clearCellOutputs,
  convertCell,
  createCell,
  deleteCell,
  deleteCells,
  duplicateCell,
  insertCellAt,
  mergeCellWithNext,
  moveCell,
  moveCellToIndex,
  parseNotebookJson,
  removeCellOutput,
  replaceInNotebookCells,
  resetNotebookOutputs,
  serializeNotebook,
  splitCellAt,
  updateCellAttachments,
  updateCellLanguage,
  updateCellMetadata,
  updateCellSource,
  updateCellTags,
  updateNotebookDependencies,
} from '../notebookOperations';
import { DEFAULT_SEARCH_OPTIONS } from '../textSearch';
import type { Notebook } from '../../types/types';

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

  it('deletes multiple cells by id', () => {
    const notebook: Notebook = {
      format_version: 2,
      metadata: { language: 'rust' },
      cells: [createCell('markdown'), createCell('code'), createCell('code')],
    };
    const ids = new Set([notebook.cells[0].id, notebook.cells[2].id]);
    const result = deleteCells(notebook, ids);
    expect(result.cells).toHaveLength(1);
    expect(result.cells[0].id).toBe(notebook.cells[1].id);
  });

  it('deleteCells is a no-op for empty or full selections', () => {
    const notebook: Notebook = {
      format_version: 2,
      metadata: { language: 'rust' },
      cells: [createCell('code'), createCell('code')],
    };
    expect(deleteCells(notebook, new Set())).toBe(notebook);
    expect(deleteCells(notebook, new Set(notebook.cells.map((c) => c.id)))).toBe(notebook);
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

  it('moves a cell to an arbitrary index', () => {
    const notebook: Notebook = {
      format_version: 2,
      metadata: { language: 'rust' },
      cells: [
        { ...createCell('markdown'), id: 'a' },
        { ...createCell('code'), id: 'b' },
        { ...createCell('raw'), id: 'c' },
      ],
    };
    const moved = moveCellToIndex(notebook, 0, 2);
    expect(moved.cells.map((cell) => cell.id)).toEqual(['b', 'c', 'a']);
    expect(moveCellToIndex(notebook, 0, 0)).toBe(notebook);
    expect(moveCellToIndex(notebook, 0, 9)).toBe(notebook);
  });

  it('parses a valid notebook document', () => {
    const notebook: Notebook = {
      format_version: 2,
      metadata: { language: 'rust', title: 'Demo' },
      cells: [
        { ...createCell('markdown'), id: 'm', source: '# Hi' },
        {
          ...createCell('code'),
          id: 'c',
          source: '1 + 1',
          outputs: [{ type: 'text', text: '2' }],
        },
      ],
    };
    expect(parseNotebookJson(serializeNotebook(notebook))).toEqual(notebook);
  });

  it('rejects malformed notebook documents', () => {
    expect(() => parseNotebookJson('{bad')).toThrow('not valid JSON');
    expect(() => parseNotebookJson('[]')).toThrow('expected a JSON object');
    expect(() => parseNotebookJson('{"format_version":2}')).toThrow('metadata');
    expect(() => parseNotebookJson('{"format_version":2,"metadata":{},"cells":"nope"}')).toThrow(
      'cells',
    );
    expect(() =>
      parseNotebookJson(
        '{"format_version":2,"metadata":{},"cells":[{"id":"x","cell_type":"bogus","source":"","outputs":[]}]}',
      ),
    ).toThrow('cell_type');
    expect(() =>
      parseNotebookJson(
        '{"format_version":2,"metadata":{},"cells":[{"id":"x","cell_type":"code","source":"","outputs":[{}]}]}',
      ),
    ).toThrow('malformed output');
  });

  it('removes a single output from a cell', () => {
    const cell = {
      ...createCell('code'),
      outputs: [
        { type: 'text' as const, text: 'a' },
        { type: 'text' as const, text: 'b' },
      ],
    };
    const nb: Notebook = { format_version: 2, metadata: { language: 'rust' }, cells: [cell] };
    const result = removeCellOutput(nb, cell.id, 0);
    expect(result.cells[0].outputs).toEqual([{ type: 'text', text: 'b' }]);
  });

  it('merges a cell with the next one', () => {
    const a = { ...createCell('code'), source: 'let a = 1;' };
    const b = { ...createCell('code'), source: 'let b = 2;' };
    const nb: Notebook = { format_version: 2, metadata: { language: 'rust' }, cells: [a, b] };
    const result = mergeCellWithNext(nb, a.id);
    expect(result.cells).toHaveLength(1);
    expect(result.cells[0].source).toBe('let a = 1;\nlet b = 2;');
  });

  it('does not add a separator when the first cell ends with a newline', () => {
    const a = { ...createCell('code'), source: 'let a = 1;\n' };
    const b = { ...createCell('code'), source: 'let b = 2;' };
    const nb: Notebook = { format_version: 2, metadata: { language: 'rust' }, cells: [a, b] };
    expect(mergeCellWithNext(nb, a.id).cells[0].source).toBe('let a = 1;\nlet b = 2;');
  });

  it('mergeCellWithNext is a no-op for the last or missing cell', () => {
    const nb: Notebook = {
      format_version: 2,
      metadata: { language: 'rust' },
      cells: [createCell('code'), createCell('code')],
    };
    expect(mergeCellWithNext(nb, nb.cells[1].id)).toBe(nb);
    expect(mergeCellWithNext(nb, 'missing')).toBe(nb);
  });

  it('splits a cell at an offset', () => {
    const a = { ...createCell('code'), source: 'hello world' };
    const nb: Notebook = { format_version: 2, metadata: { language: 'rust' }, cells: [a] };
    const result = splitCellAt(nb, a.id, 5);
    expect(result.cells).toHaveLength(2);
    expect(result.cells[0].source).toBe('hello');
    expect(result.cells[1].source).toBe(' world');
    expect(result.cells[1].id).not.toBe(a.id);
  });

  it('splitCellAt rejects invalid offsets', () => {
    const a = { ...createCell('code'), source: 'abc' };
    const nb: Notebook = { format_version: 2, metadata: { language: 'rust' }, cells: [a] };
    expect(splitCellAt(nb, a.id, 0)).toBe(nb);
    expect(splitCellAt(nb, a.id, 3)).toBe(nb);
    expect(splitCellAt(nb, 'missing', 1)).toBe(nb);
  });

  it('updates cell language, tags, metadata, and attachments', () => {
    const cell = { ...createCell('code'), metadata: { language: 'rust', keep: true } };
    const nb: Notebook = { format_version: 2, metadata: { language: 'rust' }, cells: [cell] };
    expect(updateCellLanguage(nb, cell.id, 'python').cells[0].metadata.language).toBe('python');
    expect(updateCellTags(nb, cell.id, ['ml']).cells[0].metadata.tags).toEqual(['ml']);
    expect(updateCellMetadata(nb, cell.id, { fresh: 1 }).cells[0].metadata).toEqual({ fresh: 1 });
    expect(
      updateCellAttachments(nb, cell.id, { 'a.png': 'data:image/png;base64,AA==' }).cells[0]
        .metadata.attachments,
    ).toEqual({ 'a.png': 'data:image/png;base64,AA==' });
  });

  it('updates notebook dependencies', () => {
    const nb: Notebook = { format_version: 2, metadata: { language: 'rust' }, cells: [] };
    const result = updateNotebookDependencies(nb, { serde: '1.0' });
    expect(result.metadata.dependencies).toEqual({ serde: '1.0' });
    expect(nb.metadata.dependencies).toBeUndefined();
  });

  it('replaces text across cells and counts matches', () => {
    const nb: Notebook = {
      format_version: 2,
      metadata: { language: 'rust' },
      cells: [
        { ...createCell('code'), source: 'foo bar foo' },
        { ...createCell('markdown'), source: 'no match' },
        { ...createCell('code'), source: 'foo' },
      ],
    };
    const { notebook, count } = replaceInNotebookCells(nb, 'foo', 'baz', DEFAULT_SEARCH_OPTIONS);
    expect(count).toBe(3);
    expect(notebook.cells[0].source).toBe('baz bar baz');
    expect(notebook.cells[1].source).toBe('no match');
    expect(notebook.cells[2].source).toBe('baz');
  });

  it('replaceInNotebookCells returns the notebook unchanged for empty queries', () => {
    const nb: Notebook = {
      format_version: 2,
      metadata: { language: 'rust' },
      cells: [createCell('code')],
    };
    const { notebook, count } = replaceInNotebookCells(nb, '  ', 'x', DEFAULT_SEARCH_OPTIONS);
    expect(count).toBe(0);
    expect(notebook).toBe(nb);
  });
});
