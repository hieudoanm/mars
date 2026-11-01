import type { CellType, Notebook, NotebookCell } from '../types/types';

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
