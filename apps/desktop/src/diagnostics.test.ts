import { diagnosticsForCell, parseExecutionTimeout } from './diagnostics';
import type { NotebookCell } from './types';

describe('diagnosticsForCell', () => {
  it('returns empty for non-error outputs', () => {
    expect(diagnosticsForCell({ outputs: [], source: '' } as unknown as NotebookCell)).toEqual([]);
  });

  it('returns empty for text outputs', () => {
    const cell: NotebookCell = {
      id: '1',
      cell_type: 'code',
      source: 'println!("hello");',
      execution_count: 1,
      outputs: [{ type: 'text', text: 'hello' }],
      metadata: {},
    };
    expect(diagnosticsForCell(cell)).toEqual([]);
  });

  it('parses error message with line:column fallback', () => {
    const cell: NotebookCell = {
      id: '1',
      cell_type: 'code',
      source: 'let x = 1;',
      execution_count: null,
      outputs: [
        {
          type: 'error',
          name: 'CompileError',
          message: 'error[E0308]: expected `;`, found `x`\n  --> main:3:9',
          traceback: [],
        },
      ],
      metadata: {},
    };
    const diagnostics = diagnosticsForCell(cell);
    expect(diagnostics.length).toBeGreaterThan(0);
    expect(diagnostics[0].severity).toBe('error');
    expect(diagnostics[0].message).toContain('expected');
  });

  it('returns single diagnostic when no location found', () => {
    const cell: NotebookCell = {
      id: '1',
      cell_type: 'code',
      source: 'let x = 1;',
      execution_count: null,
      outputs: [
        {
          type: 'error',
          name: 'RuntimeError',
          message: 'Something went wrong',
          traceback: [],
        },
      ],
      metadata: {},
    };
    const diagnostics = diagnosticsForCell(cell);
    expect(diagnostics).toHaveLength(1);
    expect(diagnostics[0].severity).toBe('error');
    expect(diagnostics[0].message).toBe('Something went wrong');
    expect(diagnostics[0].from).toBe(0);
    expect(diagnostics[0].to).toBe(Math.max(1, cell.source.length));
  });

  it('detects warning severity in message', () => {
    const cell: NotebookCell = {
      id: '1',
      cell_type: 'code',
      source: 'let x = 1;',
      execution_count: null,
      outputs: [
        {
          type: 'error',
          name: 'Warning',
          message: 'warning: unused variable\n  --> main:1:5',
          traceback: [],
        },
      ],
      metadata: {},
    };
    const diagnostics = diagnosticsForCell(cell);
    expect(diagnostics.some((d) => d.severity === 'warning')).toBe(true);
  });

  it('parses multi-line diagnostic with --> marker', () => {
    const source = 'line1\nline2\nline3';
    const cell: NotebookCell = {
      id: '1',
      cell_type: 'code',
      source,
      execution_count: null,
      outputs: [
        {
          type: 'error',
          name: 'CompileError',
          message: 'error: bad code\n  --> main:2:1-(2:5) [primary] mismatched type',
          traceback: [],
        },
      ],
      metadata: {},
    };
    const diagnostics = diagnosticsForCell(cell);
    expect(diagnostics.length).toBeGreaterThan(0);
    expect(diagnostics[0].message).toContain('mismatched type');
  });

  it('clamps offsets to source length', () => {
    const source = 'short';
    const cell: NotebookCell = {
      id: '1',
      cell_type: 'code',
      source,
      execution_count: null,
      outputs: [
        {
          type: 'error',
          name: 'CompileError',
          message: 'error: bad\n  --> main:100:100',
          traceback: [],
        },
      ],
      metadata: {},
    };
    const diagnostics = diagnosticsForCell(cell);
    expect(diagnostics[0].to).toBeLessThanOrEqual(source.length);
  });
});

describe('parseExecutionTimeout', () => {
  it('parses empty string', () => {
    expect(parseExecutionTimeout('')).toEqual({ value: null, error: null });
  });

  it('parses valid integer', () => {
    expect(parseExecutionTimeout('10')).toEqual({ value: 10, error: null });
  });

  it('parses large valid integer', () => {
    expect(parseExecutionTimeout('5000')).toEqual({ value: 5000, error: null });
  });

  it('rejects non-numeric string', () => {
    const result = parseExecutionTimeout('bad');
    expect(result.value).toBeNull();
    expect(result.error).not.toBeNull();
    expect(typeof result.error).toBe('string');
  });

  it('rejects negative number', () => {
    const result = parseExecutionTimeout('-5');
    expect(result.value).toBeNull();
    expect(result.error).not.toBeNull();
  });

  it('rejects zero', () => {
    const result = parseExecutionTimeout('0');
    expect(result.value).toBeNull();
    expect(result.error).not.toBeNull();
  });

  it('rejects decimal', () => {
    const result = parseExecutionTimeout('1.5');
    expect(result.value).toBeNull();
    expect(result.error).not.toBeNull();
  });

  it('rejects very large number beyond safe integer', () => {
    const result = parseExecutionTimeout('999999999999999999999');
    expect(result.value).toBeNull();
    expect(result.error).not.toBeNull();
  });
});
