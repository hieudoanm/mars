import * as vscode from 'vscode';
import {
  appendOutput,
  marsOutputToNotebookOutput,
  notebookOutputToMarsOutput,
  type MarsOutput,
} from '../output';

function itemText(item: vscode.NotebookCellOutputItem): string {
  return Buffer.from(item.data).toString('utf-8');
}

function outputItem(bytes: string, mime: string) {
  return new vscode.NotebookCellOutputItem(Buffer.from(bytes), mime);
}

describe('Project Mars and VS Code notebook outputs', () => {
  it('encodes text, HTML, LaTeX, charts, tables, and structured errors', () => {
    const outputs: MarsOutput[] = [
      { type: 'text', text: 'hello' },
      { type: 'html', html: '<b>hello</b>' },
      { type: 'latex', latex: 'x^2' },
      { type: 'chart', spec: { mark: 'bar' } },
      { type: 'table', columns: ['A&B', '<Name>'], rows: [['"Project Mars"', "it's"]] },
      {
        type: 'error',
        name: 'CompileError',
        message: 'invalid',
        traceback: ['frame 1', 'frame 2'],
      },
    ];
    const encoded = outputs.map(marsOutputToNotebookOutput);
    expect(encoded[0].items[0].mime).toBe('text/plain');
    expect(itemText(encoded[0].items[0])).toBe('hello');
    expect(encoded[1].items[0].mime).toBe('text/html');
    expect(itemText(encoded[2].items[0])).toBe('x^2');
    expect(encoded[3].items[0].mime).toBe('application/vnd.vega-lite+json');
    expect(itemText(encoded[4].items[0])).toContain('<th>A&amp;B</th><th>&lt;Name&gt;</th>');
    expect(itemText(encoded[4].items[0])).toContain(
      '<td>&quot;Project Mars&quot;</td><td>it&#39;s</td>',
    );
    expect(JSON.parse(itemText(encoded[5].items[0]))).toEqual({
      name: 'CompileError',
      message: 'invalid',
      stack: 'frame 1\nframe 2',
    });
  });

  it('encodes data URIs and raw/base64 SVG and raster images', () => {
    const cases: Array<[string, string, string]> = [
      ['image/svg+xml', '<svg/>', '<svg/>'],
      ['image/svg+xml', Buffer.from('<svg/>').toString('base64'), '<svg/>'],
      ['image/svg+xml', 'data:image/svg+xml,%3Csvg%2F%3E', '<svg/>'],
      ['image/svg+xml', 'data:image/svg+xml;base64,PHN2Zy8+', '<svg/>'],
    ];
    for (const [mime_type, data, expected] of cases) {
      expect(
        itemText(marsOutputToNotebookOutput({ type: 'image', mime_type, data }).items[0]),
      ).toBe(expected);
    }
    const raster = marsOutputToNotebookOutput({
      type: 'image',
      mime_type: 'image/png',
      data: 'data:image/png;base64,aGVsbG8=',
    }).items[0];
    expect(Buffer.from(raster.data).toString()).toBe('hello');
  });

  it('sorts MIME bundle items and falls back when values cannot be rendered', () => {
    const bundle = marsOutputToNotebookOutput({
      type: 'mime_bundle',
      data: {
        'text/plain': ['first', ' second'],
        'text/html': '<p>html</p>',
        'image/png': ['aGVs', 'bG8='],
        'image/jpeg': [1, 2],
      },
    });
    expect(bundle.items.map((item) => item.mime)).toEqual(['image/png', 'text/html', 'text/plain']);
    expect(itemText(bundle.items[2])).toBe('first second');
    const fallback = marsOutputToNotebookOutput({ type: 'mime_bundle', data: { custom: {} } });
    expect(fallback.items[0].mime).toBe('custom');
    expect(
      itemText(
        marsOutputToNotebookOutput({ type: 'mime_bundle', data: { 'image/png': 7 } }).items[0],
      ),
    ).toContain('image/png');
  });

  it('decodes all recognized output types and preserves unknown MIME data', () => {
    expect(notebookOutputToMarsOutput(new vscode.NotebookCellOutput([]))).toEqual({
      type: 'text',
      text: '',
    });
    expect(
      notebookOutputToMarsOutput(new vscode.NotebookCellOutput([outputItem('hi', 'text/plain')])),
    ).toEqual({
      type: 'text',
      text: 'hi',
    });
    expect(
      notebookOutputToMarsOutput(
        new vscode.NotebookCellOutput([outputItem('<b>x</b>', 'text/html')]),
      ),
    ).toEqual({
      type: 'html',
      html: '<b>x</b>',
    });
    expect(
      notebookOutputToMarsOutput(new vscode.NotebookCellOutput([outputItem('x', 'text/latex')])),
    ).toEqual({
      type: 'latex',
      latex: 'x',
    });
    expect(
      notebookOutputToMarsOutput(
        new vscode.NotebookCellOutput([outputItem('not-json', 'application/vnd.vega.v5+json')]),
      ),
    ).toEqual({
      type: 'chart',
      spec: {},
    });
    expect(
      notebookOutputToMarsOutput(
        new vscode.NotebookCellOutput([outputItem('value', 'application/x-custom')]),
      ),
    ).toEqual({
      type: 'mime_bundle',
      data: { 'application/x-custom': 'value' },
      metadata: {},
    });
    expect(
      notebookOutputToMarsOutput(
        new vscode.NotebookCellOutput([
          outputItem('plain', 'text/plain'),
          outputItem('<b>x</b>', 'text/html'),
        ]),
      ),
    ).toEqual({
      type: 'mime_bundle',
      data: { 'text/plain': 'plain', 'text/html': '<b>x</b>' },
      metadata: {},
    });
  });

  it('decodes error payloads and returns the original output when appending', () => {
    const errorMime = 'application/vnd.code.notebook.error';
    expect(
      notebookOutputToMarsOutput(
        new vscode.NotebookCellOutput([
          outputItem(JSON.stringify({ name: 'Oops', message: 'bad', stack: 'a\nb' }), errorMime),
        ]),
      ),
    ).toEqual({ type: 'error', name: 'Oops', message: 'bad', traceback: ['a', 'b'] });
    expect(
      notebookOutputToMarsOutput(new vscode.NotebookCellOutput([outputItem('broken', errorMime)])),
    ).toEqual({
      type: 'error',
      name: 'Error',
      message: 'broken',
      traceback: [],
    });
    expect(appendOutput(new vscode.NotebookCellOutput([]))).toBeInstanceOf(
      vscode.NotebookCellOutput,
    );
  });
});
