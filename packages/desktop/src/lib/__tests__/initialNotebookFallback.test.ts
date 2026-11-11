import { describe, expect, it, vi } from 'vitest';

vi.mock('../../../../../examples/mars/rust/01-getting-started/getting-started.mars?raw', () => ({
  default: 'not valid json',
}));

import { createInitialNotebook } from '../initialNotebook';

describe('createInitialNotebook fallback', () => {
  it('returns the fallback notebook when the bundled source is invalid', () => {
    const notebook = createInitialNotebook();
    expect(notebook.metadata.title).toBe('Untitled notebook');
    expect(notebook.cells.length).toBeGreaterThan(0);
    for (const cell of notebook.cells) {
      expect(cell.outputs).toEqual([]);
      expect(cell.execution_count).toBeNull();
    }
  });
});
