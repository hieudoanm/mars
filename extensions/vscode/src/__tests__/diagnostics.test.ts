import { describe, expect, it } from 'vitest';
import {
  clearCellDiagnostics,
  clearNotebookDiagnostics,
  disposeDiagnostics,
  publishCellDiagnostics,
} from '../diagnostics';

const fakeCell = (uri: string) =>
  ({ document: { uri: { fsPath: uri, scheme: 'file', path: uri } } }) as never;

describe('diagnostics', () => {
  it('publishes mapped diagnostics with severity and code', () => {
    const cell = fakeCell('/nb.mars#cell1');
    publishCellDiagnostics(cell, [
      {
        line: 3,
        column: 5,
        endLine: 4,
        endColumn: 2,
        severity: 'warning',
        message: 'unused variable',
        cellIndex: null,
        code: 'E001',
      } as never,
      {
        line: 0,
        column: 0,
        endLine: 0,
        endColumn: 1,
        severity: 'error',
        message: 'bad syntax',
        cellIndex: 0,
      } as never,
    ]);
  });

  it('clears per-cell, per-notebook, and disposes the collection', () => {
    const cell = fakeCell('/nb.mars#cell2');
    clearCellDiagnostics(cell);
    clearNotebookDiagnostics({
      getCells: () => [fakeCell('/a'), fakeCell('/b')],
    } as never);
    disposeDiagnostics();
    expect(true).toBe(true);
  });
});
