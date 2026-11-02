import type { Notebook } from './types';
import type { Output, NotebookCell, CellType } from './types';
import type { KernelResponse } from './types';

describe('types', () => {
  it('has notebook type', () => {
    const notebook: Notebook = {
      format_version: 2,
      metadata: { language: 'rust' },
      cells: [],
    };
    expect(notebook.format_version).toBe(2);
    expect(notebook.cells).toEqual([]);
    expect(notebook.metadata.language).toBe('rust');
  });

  it('has cell type with allowed values', () => {
    const markdownCell: CellType = 'markdown';
    const codeCell: CellType = 'code';
    const rawCell: CellType = 'raw';
    expect(markdownCell).toBe('markdown');
    expect(codeCell).toBe('code');
    expect(rawCell).toBe('raw');
  });

  it('has cell shape', () => {
    const cell: NotebookCell = {
      id: 'cell-1',
      cell_type: 'code',
      source: 'let x = 1;',
      execution_count: 1,
      outputs: [],
      metadata: {},
    };
    expect(cell.id).toBe('cell-1');
    expect(cell.cell_type).toBe('code');
    expect(cell.source).toBe('let x = 1;');
    expect(cell.execution_count).toBe(1);
    expect(Array.isArray(cell.outputs)).toBe(true);
    expect(typeof cell.metadata).toBe('object');
  });

  it('has output shape for text', () => {
    const output: Output = { type: 'text', text: 'hello' };
    expect(output.type).toBe('text');
    expect(output.text).toBe('hello');
  });

  it('has output shape for error', () => {
    const output: Output = {
      type: 'error',
      name: 'RuntimeError',
      message: 'Something went wrong',
      traceback: ['line1', 'line2'],
    };
    expect(output.type).toBe('error');
    expect(output.name).toBe('RuntimeError');
    expect(output.message).toBe('Something went wrong');
    expect(output.traceback).toEqual(['line1', 'line2']);
  });

  it('has output shape for html', () => {
    const output: Output = { type: 'html', html: '<p>test</p>' };
    expect(output.type).toBe('html');
    expect(output.html).toBe('<p>test</p>');
  });

  it('has output shape for image', () => {
    const output: Output = { type: 'image', mime_type: 'image/png', data: 'abc123' };
    expect(output.type).toBe('image');
    expect(output.mime_type).toBe('image/png');
    expect(output.data).toBe('abc123');
  });

  it('has output shape for table', () => {
    const output: Output = {
      type: 'table',
      columns: ['col1', 'col2'],
      rows: [
        ['a', 'b'],
        ['c', 'd'],
      ],
    };
    expect(output.type).toBe('table');
    expect(output.columns).toEqual(['col1', 'col2']);
    expect(output.rows).toEqual([
      ['a', 'b'],
      ['c', 'd'],
    ]);
  });

  it('has output shape for latex', () => {
    const output: Output = { type: 'latex', latex: '$x^2$' };
    expect(output.type).toBe('latex');
    expect(output.latex).toBe('$x^2$');
  });

  it('has output shape for chart', () => {
    const output: Output = { type: 'chart', spec: { mark: 'bar' } };
    expect(output.type).toBe('chart');
    expect(output.spec).toEqual({ mark: 'bar' });
  });

  it('has output shape for mime bundle', () => {
    const output: Output = { type: 'mime_bundle', data: { 'text/plain': 'test' } };
    expect(output.type).toBe('mime_bundle');
    expect(output.data).toEqual({ 'text/plain': 'test' });
  });

  it('has kernel response execution type', () => {
    const response: KernelResponse = {
      type: 'execution',
      outputs: [{ type: 'text', text: 'result' }],
      execution_count: 1,
      duration_millis: 123,
    };
    expect(response.type).toBe('execution');
    expect(response.outputs).toHaveLength(1);
    expect(response.execution_count).toBe(1);
    expect(response.duration_millis).toBe(123);
  });

  it('has kernel response reset type', () => {
    const response: KernelResponse = { type: 'reset', status: 'ok' };
    expect(response.type).toBe('reset');
    expect(response.status).toBe('ok');
  });

  it('has kernel response status type', () => {
    const response: KernelResponse = { type: 'status', status: 'idle', execution_count: 0 };
    expect(response.type).toBe('status');
    expect(response.status).toBe('idle');
    expect(response.execution_count).toBe(0);
  });

  it('has kernel response error type', () => {
    const response: KernelResponse = { type: 'error', message: 'Compile error' };
    expect(response.type).toBe('error');
    expect(response.message).toBe('Compile error');
  });
});
