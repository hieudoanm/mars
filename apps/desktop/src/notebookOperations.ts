import type { CellType, Notebook, NotebookCell } from "./types";

export function serializeNotebook(notebook: Notebook): string {
  return JSON.stringify(notebook);
}

export function createCell(cellType: CellType): NotebookCell {
  return {
    id: crypto.randomUUID(),
    cell_type: cellType,
    source: "",
    execution_count: null,
    outputs: [],
    metadata: {},
  };
}

export function updateCellSource(notebook: Notebook, id: string, source: string): Notebook {
  return {
    ...notebook,
    cells: notebook.cells.map((cell) =>
      cell.id === id ? { ...cell, source } : cell,
    ),
  };
}

export function insertCellAt(notebook: Notebook, index: number, cellType: CellType): {
  notebook: Notebook;
  cell: NotebookCell;
} {
  const cell = createCell(cellType);
  return {
    cell,
    notebook: {
      ...notebook,
      cells: [
        ...notebook.cells.slice(0, index),
        cell,
        ...notebook.cells.slice(index),
      ],
    },
  };
}

export function duplicateCell(notebook: Notebook, index: number): {
  notebook: Notebook;
  cell: NotebookCell;
} {
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
      cells: [
        ...notebook.cells.slice(0, index + 1),
        cell,
        ...notebook.cells.slice(index + 1),
      ],
    },
  };
}

export function deleteCell(notebook: Notebook, id: string): {
  notebook: Notebook;
  nextId: string;
} | null {
  const index = notebook.cells.findIndex((cell) => cell.id === id);
  if (index < 0 || notebook.cells.length <= 1) return null;
  const next = notebook.cells[index + 1] ?? notebook.cells[index - 1];
  return {
    nextId: next.id,
    notebook: { ...notebook, cells: notebook.cells.filter((cell) => cell.id !== id) },
  };
}

export function moveCell(notebook: Notebook, index: number, offset: number): Notebook {
  const nextIndex = index + offset;
  if (index < 0 || nextIndex < 0 || nextIndex >= notebook.cells.length) return notebook;
  const cells = [...notebook.cells];
  const [cell] = cells.splice(index, 1);
  cells.splice(nextIndex, 0, cell);
  return { ...notebook, cells };
}

export function convertCell(notebook: Notebook, id: string, cellType: CellType): Notebook {
  return {
    ...notebook,
    cells: notebook.cells.map((cell) =>
      cell.id === id
        ? {
            ...cell,
            cell_type: cellType,
            execution_count: cellType === "code" ? cell.execution_count : null,
            outputs: cellType === "code" ? cell.outputs : [],
          }
        : cell,
    ),
  };
}

export function clearCellOutputs(notebook: Notebook, id?: string): Notebook {
  return {
    ...notebook,
    cells: notebook.cells.map((cell) =>
      id === undefined || cell.id === id
        ? { ...cell, execution_count: null, outputs: [] }
        : cell,
    ),
  };
}

export function resetNotebookOutputs(notebook: Notebook): Notebook {
  return clearCellOutputs(notebook);
}
