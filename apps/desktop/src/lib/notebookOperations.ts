import type { CellType, Notebook, NotebookCell } from '../types/types';
import { createSearchPattern } from './textSearch';
import type { SearchOptions } from './textSearch';

export const serializeNotebook = (notebook: Notebook): string => {
  return JSON.stringify(notebook);
};

const CELL_TYPES = new Set(['markdown', 'code', 'raw']);

const isRecord = (value: unknown): value is Record<string, unknown> => {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
};

export const parseNotebookJson = (content: string): Notebook => {
  let value: unknown;
  try {
    value = JSON.parse(content);
  } catch {
    throw new Error('Notebook file is not valid JSON');
  }
  if (!isRecord(value)) throw new Error('Invalid notebook: expected a JSON object');
  if (typeof value.format_version !== 'number') {
    throw new Error('Invalid notebook: missing numeric format_version');
  }
  if (!isRecord(value.metadata)) {
    throw new Error('Invalid notebook: missing metadata object');
  }
  if (!Array.isArray(value.cells)) {
    throw new Error('Invalid notebook: missing cells array');
  }
  for (const [index, cell] of value.cells.entries()) {
    if (!isRecord(cell)) throw new Error(`Invalid notebook: cell ${index} is not an object`);
    if (typeof cell.id !== 'string' || cell.id.length === 0) {
      throw new Error(`Invalid notebook: cell ${index} is missing an id`);
    }
    if (typeof cell.cell_type !== 'string' || !CELL_TYPES.has(cell.cell_type)) {
      throw new Error(`Invalid notebook: cell ${index} has an unknown cell_type`);
    }
    if (typeof cell.source !== 'string') {
      throw new Error(`Invalid notebook: cell ${index} is missing source`);
    }
    if (!Array.isArray(cell.outputs)) {
      throw new Error(`Invalid notebook: cell ${index} is missing outputs`);
    }
    for (const output of cell.outputs) {
      if (!isRecord(output) || typeof output.type !== 'string') {
        throw new Error(`Invalid notebook: cell ${index} has a malformed output`);
      }
    }
  }
  return value as unknown as Notebook;
};

export const createCell = (cellType: CellType): NotebookCell => {
  return {
    id: crypto.randomUUID(),
    cell_type: cellType,
    source: '',
    execution_count: null,
    outputs: [],
    metadata: {},
  };
};

export type PastedCellContent = {
  source: string;
  cellType: CellType;
  language?: string;
};

export const pasteCellAt = (
  notebook: Notebook,
  index: number,
  content: PastedCellContent,
): { notebook: Notebook; cell: NotebookCell } => {
  const { notebook: inserted, cell } = insertCellAt(notebook, index, content.cellType);
  const withSource = updateCellSource(inserted, cell.id, content.source);
  const withLanguage = content.language
    ? withSource.cells.map((entry) =>
        entry.id === cell.id
          ? { ...entry, metadata: { ...entry.metadata, language: content.language } }
          : entry,
      )
    : withSource.cells;
  return {
    cell,
    notebook: { ...withSource, cells: withLanguage },
  };
};

export const updateCellSource = (notebook: Notebook, id: string, source: string): Notebook => {
  return {
    ...notebook,
    cells: notebook.cells.map((cell) => (cell.id === id ? { ...cell, source } : cell)),
  };
};

export const insertCellAt = (
  notebook: Notebook,
  index: number,
  cellType: CellType,
): {
  notebook: Notebook;
  cell: NotebookCell;
} => {
  const cell = createCell(cellType);
  return {
    cell,
    notebook: {
      ...notebook,
      cells: [...notebook.cells.slice(0, index), cell, ...notebook.cells.slice(index)],
    },
  };
};

export const duplicateCell = (
  notebook: Notebook,
  index: number,
): {
  notebook: Notebook;
  cell: NotebookCell;
} => {
  const original = notebook.cells[index];
  const cell = {
    ...original,
    id: crypto.randomUUID(),
    execution_count: null,
    outputs: [],
  };
  return {
    cell,
    notebook: {
      ...notebook,
      cells: [...notebook.cells.slice(0, index + 1), cell, ...notebook.cells.slice(index + 1)],
    },
  };
};

export const deleteCell = (
  notebook: Notebook,
  id: string,
): {
  notebook: Notebook;
  nextId: string;
} | null => {
  const index = notebook.cells.findIndex((cell) => cell.id === id);
  if (index < 0 || notebook.cells.length <= 1) return null;
  const next = notebook.cells[index + 1] ?? notebook.cells[index - 1];
  return {
    nextId: next.id,
    notebook: { ...notebook, cells: notebook.cells.filter((cell) => cell.id !== id) },
  };
};

export const moveCell = (notebook: Notebook, index: number, offset: number): Notebook => {
  const nextIndex = index + offset;
  if (index < 0 || nextIndex < 0 || nextIndex >= notebook.cells.length) return notebook;
  const cells = [...notebook.cells];
  const [cell] = cells.splice(index, 1);
  cells.splice(nextIndex, 0, cell);
  return { ...notebook, cells };
};

export const moveCellToIndex = (
  notebook: Notebook,
  fromIndex: number,
  toIndex: number,
): Notebook => {
  const length = notebook.cells.length;
  if (
    fromIndex < 0 ||
    toIndex < 0 ||
    fromIndex >= length ||
    toIndex >= length ||
    fromIndex === toIndex
  ) {
    return notebook;
  }
  const cells = [...notebook.cells];
  const [cell] = cells.splice(fromIndex, 1);
  cells.splice(toIndex, 0, cell);
  return { ...notebook, cells };
};

export const convertCell = (notebook: Notebook, id: string, cellType: CellType): Notebook => {
  return {
    ...notebook,
    cells: notebook.cells.map((cell) =>
      cell.id === id
        ? {
            ...cell,
            cell_type: cellType,
            execution_count: cellType === 'code' ? cell.execution_count : null,
            outputs: cellType === 'code' ? cell.outputs : [],
          }
        : cell,
    ),
  };
};

export const clearCellOutputs = (notebook: Notebook, id?: string): Notebook => {
  return {
    ...notebook,
    cells: notebook.cells.map((cell) =>
      id === undefined || cell.id === id ? { ...cell, execution_count: null, outputs: [] } : cell,
    ),
  };
};

export const resetNotebookOutputs = (notebook: Notebook): Notebook => {
  return clearCellOutputs(notebook);
};

export const removeCellOutput = (
  notebook: Notebook,
  cellId: string,
  outputIndex: number,
): Notebook => ({
  ...notebook,
  cells: notebook.cells.map((cell) =>
    cell.id === cellId
      ? { ...cell, outputs: cell.outputs.filter((_, index) => index !== outputIndex) }
      : cell,
  ),
});

export const mergeCellWithNext = (notebook: Notebook, id: string): Notebook => {
  const index = notebook.cells.findIndex((cell) => cell.id === id);
  const next = notebook.cells[index + 1];
  if (index < 0 || !next) return notebook;
  const separator = notebook.cells[index].source.endsWith('\n') || next.source === '' ? '' : '\n';
  const cells = notebook.cells
    .slice(0, index)
    .concat({
      ...notebook.cells[index],
      source: notebook.cells[index].source + separator + next.source,
    })
    .concat(notebook.cells.slice(index + 2));
  return { ...notebook, cells };
};

export const splitCellAt = (notebook: Notebook, id: string, offset: number): Notebook => {
  const index = notebook.cells.findIndex((cell) => cell.id === id);
  const cell = notebook.cells[index];
  if (index < 0 || offset <= 0 || offset >= cell.source.length) return notebook;
  const head = cell.source.slice(0, offset);
  const tail = cell.source.slice(offset);
  const tailCell = {
    ...createCell(cell.cell_type),
    source: tail,
    metadata: { ...cell.metadata },
  };
  const cells = notebook.cells
    .slice(0, index)
    .concat({ ...cell, source: head })
    .concat(tailCell)
    .concat(notebook.cells.slice(index + 1));
  return { ...notebook, cells };
};

export const deleteCells = (notebook: Notebook, ids: ReadonlySet<string>): Notebook => {
  if (ids.size === 0) return notebook;
  const cells = notebook.cells.filter((cell) => !ids.has(cell.id));
  if (cells.length === notebook.cells.length || cells.length === 0) return notebook;
  return { ...notebook, cells };
};

export const updateCellLanguage = (notebook: Notebook, id: string, language: string): Notebook => ({
  ...notebook,
  cells: notebook.cells.map((cell) =>
    cell.id === id ? { ...cell, metadata: { ...cell.metadata, language } } : cell,
  ),
});

export const updateCellTags = (notebook: Notebook, id: string, tags: string[]): Notebook => ({
  ...notebook,
  cells: notebook.cells.map((cell) =>
    cell.id === id ? { ...cell, metadata: { ...cell.metadata, tags } } : cell,
  ),
});

export const updateCellMetadata = (
  notebook: Notebook,
  id: string,
  metadata: Record<string, unknown>,
): Notebook => ({
  ...notebook,
  cells: notebook.cells.map((cell) =>
    cell.id === id ? { ...cell, metadata: { ...metadata } } : cell,
  ),
});

export const updateCellAttachments = (
  notebook: Notebook,
  id: string,
  attachments: Record<string, string>,
): Notebook => ({
  ...notebook,
  cells: notebook.cells.map((cell) =>
    cell.id === id ? { ...cell, metadata: { ...cell.metadata, attachments } } : cell,
  ),
});

export const updateNotebookDependencies = (
  notebook: Notebook,
  dependencies: Record<string, string>,
): Notebook => ({
  ...notebook,
  metadata: { ...notebook.metadata, dependencies },
});

export const replaceInNotebookCells = (
  notebook: Notebook,
  find: string,
  replace: string,
  options: SearchOptions,
): { notebook: Notebook; count: number } => {
  const pattern = createSearchPattern(find, options);
  if (!pattern) return { notebook, count: 0 };
  let count = 0;
  const cells = notebook.cells.map((cell) => {
    pattern.lastIndex = 0;
    const next = cell.source.replace(pattern, () => {
      count += 1;
      return replace;
    });
    return next === cell.source ? cell : { ...cell, source: next };
  });
  return { notebook: { ...notebook, cells }, count };
};
