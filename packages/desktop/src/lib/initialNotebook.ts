import type { Notebook, NotebookCell } from '../types/types';
import gettingStartedRaw from '../../../../examples/mars/rust/01-getting-started/getting-started.mars?raw';

const FALLBACK_NOTEBOOK: Notebook = {
  format_version: 2,
  metadata: { language: 'rust', title: 'Untitled notebook' },
  cells: [
    {
      id: crypto.randomUUID(),
      cell_type: 'markdown',
      source: '# Welcome to Mars\n\nExplore ideas. Execute code. Discover possibilities.',
      execution_count: null,
      outputs: [],
      metadata: {},
    },
    {
      id: crypto.randomUUID(),
      cell_type: 'code',
      source: 'println!("Hello from Mars!");',
      execution_count: null,
      outputs: [],
      metadata: {},
    },
  ],
};

const isNotebookCell = (value: unknown): value is NotebookCell => {
  if (typeof value !== 'object' || value === null) return false;
  const cell = value as Partial<NotebookCell>;
  return (
    typeof cell.id === 'string' &&
    (cell.cell_type === 'markdown' || cell.cell_type === 'code' || cell.cell_type === 'raw') &&
    typeof cell.source === 'string'
  );
};

const freshCell = (cell: NotebookCell): NotebookCell => ({
  ...cell,
  execution_count: null,
  outputs: [],
});

export const createInitialNotebook = (): Notebook => {
  try {
    const parsed: unknown = JSON.parse(gettingStartedRaw);
    if (typeof parsed !== 'object' || parsed === null) return FALLBACK_NOTEBOOK;
    const candidate = parsed as Partial<Notebook>;
    if (!Array.isArray(candidate.cells) || !candidate.cells.every(isNotebookCell)) {
      return FALLBACK_NOTEBOOK;
    }
    return {
      format_version: candidate.format_version ?? 2,
      metadata: {
        language: 'rust',
        ...(typeof candidate.metadata === 'object' && candidate.metadata !== null
          ? candidate.metadata
          : {}),
      },
      cells: candidate.cells.map(freshCell),
    };
  } catch {
    return FALLBACK_NOTEBOOK;
  }
};
