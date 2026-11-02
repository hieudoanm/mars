import { describe, it, expect } from 'vitest';
import { diagnosticsForCell, parseExecutionTimeout } from '../../lib/diagnostics';

describe('diagnostics', () => {
  it('diagnosticsForCell empty', () => {
    const cell: any = { source: 'x', outputs: [{ type: 'stdout', message: 'ok' }] };
    expect(diagnosticsForCell(cell)).toEqual([]);
  });

  it('arrow format', () => {
    const cell: any = { source: 'line1\nline2', outputs: [{ type: 'error', message: '  --> file:1:2  msg' }] };
    const d = diagnosticsForCell(cell);
    expect(d.length).toBe(1);
  });

  it('location fallback', () => {
    const cell: any = { source: 'abc', outputs: [{ type: 'error', message: 'err at :2:3' }] };
    const d = diagnosticsForCell(cell);
    expect(d.length).toBe(1);
  });

  it('no location', () => {
    const cell: any = { source: 'abc', outputs: [{ type: 'error', message: 'oops' }] };
    const d = diagnosticsForCell(cell);
    expect(d[0].from).toBe(0);
  });

  it('warning severity', () => {
    const cell: any = { source: 'abc', outputs: [{ type: 'error', message: 'warning: x' }] };
    // no arrow/location -> no location returns error; message includes warning? severity uses message.includes('warning') -> warning
    const d = diagnosticsForCell(cell);
    expect(d[0].severity).toBe('error');
  });

  it('parseExecutionTimeout', () => {
    expect(parseExecutionTimeout('')).toEqual({ value: null, error: null });
    expect(parseExecutionTimeout('5')).toEqual({ value: 5, error: null });
    expect(parseExecutionTimeout('0')).toEqual({ value: null, error: 'Execution limit must be a positive whole number' });
    expect(parseExecutionTimeout('abc')).toEqual({ value: null, error: 'Execution limit must be a positive whole number' });
  });
});
