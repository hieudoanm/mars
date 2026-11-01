import { describe, it, expect } from 'vitest';
import { diagnosticsForCell, parseExecutionTimeout } from '../../lib/diagnostics';
import type { NotebookCell, Output } from '../types';

function cell(source: string, outputs: Output[]): NotebookCell {
  return {
    id: 'test-cell', cell_type: 'code', source, execution_count: null, outputs, metadata: {},
  };
}

describe('diagnostics', () => {
  it('diagnosticsForCell empty', () => {
    expect(diagnosticsForCell(cell('x', [{ type: 'text', text: 'ok' }]))).toEqual([]);
  });

  it('arrow format', () => {
    const d = diagnosticsForCell(cell('line1\nline2', [
      { type: 'error', name: 'CompileError', message: '  --> file:1:2  msg', traceback: [] },
    ]));
    expect(d).toHaveLength(1);
    expect(d[0]).toMatchObject({ from: 1, to: 2, message: 'msg', severity: 'error' });
  });

  it('location fallback', () => {
    const d = diagnosticsForCell(cell('abc', [
      { type: 'error', name: 'Error', message: 'err at :2:3', traceback: [] },
    ]));
    expect(d).toHaveLength(1);
    expect(d[0]).toMatchObject({ from: 2, to: 3 });
  });

  it('no location', () => {
    const d = diagnosticsForCell(cell('abc', [
      { type: 'error', name: 'Error', message: 'oops', traceback: [] },
    ]));
    expect(d[0]).toMatchObject({ from: 0, to: 3, severity: 'error', message: 'oops' });
  });

  it('warning severity', () => {
    const d = diagnosticsForCell(cell('abc', [
      { type: 'error', name: 'Warning', message: 'warning:\n --> file:1:1 unused', traceback: [] },
    ]));
    expect(d[0].severity).toBe('warning');
  });

  it('parseExecutionTimeout', () => {
    expect(parseExecutionTimeout('')).toEqual({ value: null, error: null });
    expect(parseExecutionTimeout('5')).toEqual({ value: 5, error: null });
    expect(parseExecutionTimeout('0')).toEqual({ value: null, error: 'Execution limit must be a positive whole number' });
    expect(parseExecutionTimeout('abc')).toEqual({ value: null, error: 'Execution limit must be a positive whole number' });
  });
});
