import { describe, it, expect } from 'vitest';
import {
  diagnosticsForCell,
  liveDiagnosticsFor,
  lspDiagnosticsFor,
  parseExecutionTimeout,
} from '../../lib/diagnostics';
import type { CellDiagnostic, LspCellDiagnostic, NotebookCell, Output } from '../../types/types';

const cell = (source: string, outputs: Output[]): NotebookCell => {
  return {
    id: 'test-cell',
    cell_type: 'code',
    source,
    execution_count: null,
    outputs,
    metadata: {},
  };
};

describe('diagnostics', () => {
  it('diagnosticsForCell empty', () => {
    expect(diagnosticsForCell(cell('x', [{ type: 'text', text: 'ok' }]))).toEqual([]);
  });

  it('arrow format', () => {
    const d = diagnosticsForCell(
      cell('line1\nline2', [
        { type: 'error', name: 'CompileError', message: '  --> file:1:2  msg', traceback: [] },
      ]),
    );
    expect(d).toHaveLength(1);
    expect(d[0]).toMatchObject({ from: 1, to: 2, message: 'msg', severity: 'error' });
  });

  it('location fallback', () => {
    const d = diagnosticsForCell(
      cell('abc', [{ type: 'error', name: 'Error', message: 'err at :2:3', traceback: [] }]),
    );
    expect(d).toHaveLength(1);
    expect(d[0]).toMatchObject({ from: 2, to: 3 });
  });

  it('no location', () => {
    const d = diagnosticsForCell(
      cell('abc', [{ type: 'error', name: 'Error', message: 'oops', traceback: [] }]),
    );
    expect(d[0]).toMatchObject({ from: 0, to: 3, severity: 'error', message: 'oops' });
  });

  it('warning severity', () => {
    const d = diagnosticsForCell(
      cell('abc', [
        {
          type: 'error',
          name: 'Warning',
          message: 'warning:\n --> file:1:1 unused',
          traceback: [],
        },
      ]),
    );
    expect(d[0].severity).toBe('warning');
  });

  it('parseExecutionTimeout', () => {
    expect(parseExecutionTimeout('')).toEqual({ value: null, error: null });
    expect(parseExecutionTimeout('5')).toEqual({ value: 5, error: null });
    expect(parseExecutionTimeout('0')).toEqual({
      value: null,
      error: 'Execution limit must be a positive whole number',
    });
    expect(parseExecutionTimeout('abc')).toEqual({
      value: null,
      error: 'Execution limit must be a positive whole number',
    });
  });

  it('lspDiagnosticsFor maps LSP items to source offsets', () => {
    const source = 'fn main() {\n  let x = 1;\n}\n';
    const items: LspCellDiagnostic[] = [
      { cellId: 'c', line: 1, column: 2, severity: 'warning', message: 'unused' },
      { cellId: 'c', line: 0, column: 0, severity: 'error', message: 'bad' },
    ];
    const result = lspDiagnosticsFor(source, items);
    expect(result).toHaveLength(2);
    expect(result[0].severity).toBe('warning');
    expect(result[0].message).toBe('unused');
    expect(result[1].severity).toBe('error');
    // second line, column 2 (0-based) → offset into line 2
    expect(result[0].from).toBeGreaterThan(0);
    expect(result[0].to).toBe(result[0].from + 1);
    expect(result[1].from).toBeLessThan(result[0].from);
  });

  it('liveDiagnosticsFor maps cell diagnostics with optional codes', () => {
    const source = 'let a = 1;\nlet b = 2;\n';
    const report: CellDiagnostic[] = [
      {
        cellIndex: 0,
        line: 2,
        column: 5,
        endLine: 2,
        endColumn: 6,
        severity: 'error',
        code: 'E0308',
        message: 'mismatched types',
      },
      {
        cellIndex: 0,
        line: 1,
        column: 1,
        endLine: 1,
        endColumn: 2,
        severity: 'warning',
        code: null,
        message: 'unused variable',
      },
    ];
    const result = liveDiagnosticsFor(source, report);
    expect(result).toHaveLength(2);
    expect(result[0].message).toBe('E0308: mismatched types');
    expect(result[0].severity).toBe('error');
    expect(result[1].message).toBe('unused variable');
    expect(result[1].severity).toBe('warning');
    expect(result[0].from).toBeGreaterThanOrEqual(0);
    expect(result[0].to).toBeGreaterThan(result[0].from);
  });
});
