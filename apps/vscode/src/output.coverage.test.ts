import * as vscode from 'vscode';
import { appendOutput, marsOutputToNotebookOutput, notebookOutputToMarsOutput } from './output';

describe('output coverage', () => {
  it('appendOutput returns output', () => {
    const out = new vscode.NotebookCellOutput([
      new vscode.NotebookCellOutputItem(Buffer.from('hi'), 'text/plain'),
    ]);
    expect(appendOutput(out)).toBe(out);
  });

  it('marsOutputToNotebookOutput text', () => {
    const out = marsOutputToNotebookOutput({ type: 'text', text: 'hello' });
    expect(out.items[0].mime).toBe('text/plain');
  });

  it('marsOutputToNotebookOutput error', () => {
    const out = marsOutputToNotebookOutput({
      type: 'error',
      name: 'E',
      message: 'bad',
      traceback: ['a', 'b'],
    });
    expect(out.items[0].mime).toBe('application/vnd.code.notebook.error');
  });

  it('marsOutputToNotebookOutput html/latex/image/table/chart', () => {
    expect(marsOutputToNotebookOutput({ type: 'html', html: '<b>x</b>' }).items[0].mime).toBe(
      'text/html',
    );
    expect(marsOutputToNotebookOutput({ type: 'latex', latex: 'x^2' }).items[0].mime).toBe(
      'text/latex',
    );
    expect(
      marsOutputToNotebookOutput({ type: 'image', mime_type: 'image/png', data: 'aGVsbG8=' }).items
        .length,
    ).toBe(1);
    expect(
      marsOutputToNotebookOutput({ type: 'image', mime_type: 'image/svg+xml', data: '<svg></svg>' })
        .items[0].mime,
    ).toBe('image/svg+xml');
    expect(
      marsOutputToNotebookOutput({
        type: 'image',
        mime_type: 'image/svg+xml',
        data: 'data:image/svg+xml;base64,PHN2Zz48L3N2Zz4=',
      }).items[0].mime,
    ).toBe('image/svg+xml');
    expect(
      marsOutputToNotebookOutput({
        type: 'image',
        mime_type: 'image/png',
        data: 'data:image/png;base64,aGVsbG8=',
      }).items.length,
    ).toBe(1);
    expect(
      marsOutputToNotebookOutput({ type: 'table', columns: ['a'], rows: [['b']] }).items[0].mime,
    ).toBe('text/html');
    expect(marsOutputToNotebookOutput({ type: 'chart', spec: { mark: 'bar' } }).items[0].mime).toBe(
      'application/vnd.vega-lite+json',
    );
  });

  it('marsOutputToNotebookOutput mime_bundle', () => {
    const out = marsOutputToNotebookOutput({
      type: 'mime_bundle',
      data: {
        'text/html': '<div>hi</div>',
        'text/plain': 'hi',
        'image/png': 'aGVsbG8=',
        'image/svg+xml': '<svg/>',
      },
    });
    expect(out.items.length).toBeGreaterThan(0);
  });

  it('marsOutputToNotebookOutput mime_bundle with array values', () => {
    const out = marsOutputToNotebookOutput({
      type: 'mime_bundle',
      data: {
        'text/plain': ['hello', ' ', 'world'],
      },
    });
    expect(out.items.length).toBe(1);
  });

  it('marsOutputToNotebookOutput mime_bundle empty fallback', () => {
    const out = marsOutputToNotebookOutput({
      type: 'mime_bundle',
      data: {
        'application/unknown': {},
      },
    });
    expect(out.items.length).toBe(1);
  });

  it('notebookOutputToMarsOutput', () => {
    const out1 = new vscode.NotebookCellOutput([
      new vscode.NotebookCellOutputItem(Buffer.from('hi'), 'text/plain'),
    ]);
    expect(notebookOutputToMarsOutput(out1).type).toBe('text');
    const out2 = new vscode.NotebookCellOutput([
      new vscode.NotebookCellOutputItem(Buffer.from(''), 'text/plain'),
    ]);
    expect(notebookOutputToMarsOutput(out2).type).toBe('text');
    const out3 = new vscode.NotebookCellOutput([]);
    expect(notebookOutputToMarsOutput(out3).type).toBe('text');
  });

  it('notebookOutputToMarsOutput error and others', () => {
    const err = { name: 'E', message: 'bad', stack: 'a\nb' };
    const outErr = new vscode.NotebookCellOutput([
      new vscode.NotebookCellOutputItem(
        Buffer.from(JSON.stringify(err)),
        'application/vnd.code.notebook.error',
      ),
    ]);
    const res = notebookOutputToMarsOutput(outErr);
    expect(res.type).toBe('error');
  });

  it('notebookOutputToMarsOutput image/html/latex/chart', () => {
    expect(
      notebookOutputToMarsOutput(
        new vscode.NotebookCellOutput([
          new vscode.NotebookCellOutputItem(Buffer.from('abc'), 'image/png'),
        ]),
      ).type,
    ).toBe('image');
    expect(
      notebookOutputToMarsOutput(
        new vscode.NotebookCellOutput([
          new vscode.NotebookCellOutputItem(Buffer.from('<p>x</p>'), 'text/html'),
        ]),
      ).type,
    ).toBe('html');
    expect(
      notebookOutputToMarsOutput(
        new vscode.NotebookCellOutput([
          new vscode.NotebookCellOutputItem(Buffer.from('x^2'), 'text/latex'),
        ]),
      ).type,
    ).toBe('latex');
    expect(
      notebookOutputToMarsOutput(
        new vscode.NotebookCellOutput([
          new vscode.NotebookCellOutputItem(
            Buffer.from(JSON.stringify({ mark: 'bar' })),
            'application/vnd.vega-lite+json',
          ),
        ]),
      ).type,
    ).toBe('chart');
    expect(
      notebookOutputToMarsOutput(
        new vscode.NotebookCellOutput([
          new vscode.NotebookCellOutputItem(Buffer.from('hi'), 'text/markdown'),
        ]),
      ).type,
    ).toBe('text');
    expect(
      notebookOutputToMarsOutput(
        new vscode.NotebookCellOutput([
          new vscode.NotebookCellOutputItem(Buffer.from('x'), 'application/json'),
        ]),
      ).type,
    ).toBe('mime_bundle');
  });

  it('notebookOutputToMarsOutput multiple items', () => {
    const out = new vscode.NotebookCellOutput([
      new vscode.NotebookCellOutputItem(Buffer.from('hi'), 'text/plain'),
      new vscode.NotebookCellOutputItem(Buffer.from('<b>hi</b>'), 'text/html'),
    ]);
    const res = notebookOutputToMarsOutput(out);
    expect(res.type).toBe('mime_bundle');
  });

  it('parse error fallbacks', () => {
    const outErr = new vscode.NotebookCellOutput([
      new vscode.NotebookCellOutputItem(
        Buffer.from('notjson'),
        'application/vnd.code.notebook.error',
      ),
    ]);
    const res = notebookOutputToMarsOutput(outErr);
    expect(res.type).toBe('error');
  });

  it('parse chart fallback', () => {
    const out = notebookOutputToMarsOutput(
      new vscode.NotebookCellOutput([
        new vscode.NotebookCellOutputItem(Buffer.from('notjson'), 'application/vnd.vega.v5+json'),
      ]),
    );
    expect(out.type).toBe('chart');
  });
});
