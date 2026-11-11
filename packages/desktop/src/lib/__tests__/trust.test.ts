import { beforeEach, describe, expect, it } from 'vitest';
import { isNotebookTrusted, notebookHasCode, trustNotebook } from '../trust';

describe('trust', () => {
  beforeEach(() => window.localStorage.clear());

  it('trusts and reports notebooks by path', () => {
    expect(isNotebookTrusted('/tmp/a.mars')).toBe(false);
    trustNotebook('/tmp/a.mars');
    expect(isNotebookTrusted('/tmp/a.mars')).toBe(true);
    expect(isNotebookTrusted('/tmp/b.mars')).toBe(false);
  });

  it('accumulates multiple trusted paths', () => {
    trustNotebook('/tmp/a.mars');
    trustNotebook('/tmp/b.mars');
    expect(isNotebookTrusted('/tmp/a.mars')).toBe(true);
    expect(isNotebookTrusted('/tmp/b.mars')).toBe(true);
  });

  it('ignores corrupt stored state', () => {
    window.localStorage.setItem('mars.trusted.notebooks', '{not json');
    expect(isNotebookTrusted('/tmp/a.mars')).toBe(false);
    trustNotebook('/tmp/a.mars');
    expect(isNotebookTrusted('/tmp/a.mars')).toBe(true);
  });

  it('filters non-string entries from stored state', () => {
    window.localStorage.setItem('mars.trusted.notebooks', '["/tmp/a.mars", 42, null]');
    expect(isNotebookTrusted('/tmp/a.mars')).toBe(true);
    expect(isNotebookTrusted('42')).toBe(false);
  });

  it('detects notebooks containing code cells', () => {
    expect(
      notebookHasCode(
        JSON.stringify({
          cells: [{ cell_type: 'markdown' }, { cell_type: 'code', source: 'fn main() {}' }],
        }),
      ),
    ).toBe(true);
    expect(notebookHasCode(JSON.stringify({ cells: [{ cell_type: 'markdown' }] }))).toBe(false);
    expect(notebookHasCode(JSON.stringify({ cells: [] }))).toBe(false);
    expect(notebookHasCode('not json')).toBe(false);
    expect(notebookHasCode('null')).toBe(false);
    expect(notebookHasCode('"string"')).toBe(false);
  });
});
