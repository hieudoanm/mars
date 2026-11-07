import type { NotebookCell } from '../types/types';
import { diffLines, type DiffRow } from './textDiff';

export type CellDiff = {
  cellId: string;
  index: number;
  cellType: string;
  added: number;
  removed: number;
  rows: DiffRow[];
};

const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === 'object' && value !== null;

const parseCells = (content: string): NotebookCell[] | null => {
  try {
    const parsed: unknown = JSON.parse(content);
    if (!isRecord(parsed) || !Array.isArray(parsed.cells)) return null;
    return parsed.cells.filter(
      (cell): cell is NotebookCell =>
        isRecord(cell) &&
        typeof cell.id === 'string' &&
        typeof cell.source === 'string' &&
        typeof cell.cell_type === 'string',
    );
  } catch {
    return null;
  }
};

export const computeCellDiffs = (beforeContent: string, afterContent: string): CellDiff[] => {
  const before = parseCells(beforeContent);
  const after = parseCells(afterContent);
  if (!before || !after) return [];

  const afterById = new Map(after.map((cell) => [cell.id, cell]));
  const beforeIds = new Set(before.map((cell) => cell.id));
  const diffs: CellDiff[] = [];

  before.forEach((cell, index) => {
    const current = afterById.get(cell.id);
    const rows = diffLines(cell.source, current?.source ?? '');
    const added = rows.filter((row) => row.type === 'add').length;
    const removed = rows.filter((row) => row.type === 'del').length;
    if (added + removed > 0) {
      diffs.push({
        cellId: cell.id,
        index,
        cellType: cell.cell_type,
        added,
        removed,
        rows,
      });
    }
  });

  after.forEach((cell, index) => {
    if (beforeIds.has(cell.id)) return;
    const rows = diffLines('', cell.source);
    diffs.push({
      cellId: cell.id,
      index,
      cellType: cell.cell_type,
      added: rows.filter((row) => row.type === 'add').length,
      removed: 0,
      rows,
    });
  });

  return diffs.sort((a, b) => a.index - b.index);
};
