import { describe, expect, it } from 'vitest';
import { cellPreview } from '../cellPreview';
import type { NotebookCell } from '../../types/types';

const cell = (source: string): NotebookCell => ({
  id: 'c',
  cell_type: 'code',
  source,
  execution_count: null,
  outputs: [],
  metadata: {},
});

describe('cellPreview', () => {
  it('returns the first non-empty trimmed line', () => {
    expect(cellPreview(cell('\n\n  let x = 1;\nlet y = 2;'))).toBe('let x = 1;');
  });

  it('returns empty string for empty or whitespace-only cells', () => {
    expect(cellPreview(cell(''))).toBe('');
    expect(cellPreview(cell('  \n\t\n'))).toBe('');
  });
});
