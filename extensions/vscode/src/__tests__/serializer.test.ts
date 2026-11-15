import * as vscode from 'vscode';
import { MarsNotebookSerializer } from '../notebookSerializer';

const serializer = new MarsNotebookSerializer();
const token = {} as vscode.CancellationToken;

function decode(bytes: Uint8Array) {
  return JSON.parse(new TextDecoder().decode(bytes)) as Record<string, unknown>;
}

describe('MarsNotebookSerializer', () => {
  it('deserializes cell types, metadata, IDs, and outputs', async () => {
    const input = {
      format_version: 3,
      metadata: { language: 'rust', title: 'Notebook', dependencies: { rand: '0.8' } },
      cells: [
        {
          id: 'code-1',
          cell_type: 'code',
          source: '1 + 1',
          metadata: { tag: true },
          outputs: [{ type: 'text', text: '2' }],
        },
        { id: 'md-1', cell_type: 'markdown', source: '# Title', metadata: {} },
        { id: 'raw-1', cell_type: 'raw', source: 'raw', metadata: {} },
      ],
    };
    const notebook = await serializer.deserializeNotebook(
      new TextEncoder().encode(JSON.stringify(input)),
      token,
    );
    expect(notebook.cells).toHaveLength(3);
    expect(notebook.cells[0]).toMatchObject({
      kind: vscode.NotebookCellKind.Code,
      languageId: 'rust',
      value: '1 + 1',
      metadata: { tag: true, marsId: 'code-1' },
    });
    expect(notebook.cells[0].outputs).toHaveLength(1);
    expect(notebook.cells[1]).toMatchObject({
      kind: vscode.NotebookCellKind.Markup,
      languageId: 'markdown',
    });
    expect(notebook.cells[2].metadata).toHaveProperty('marsId', 'raw-1');
    expect(notebook.metadata).toMatchObject({ title: 'Notebook', marsFormatVersion: 3 });
  });

  it('returns an empty current-format notebook for invalid JSON', async () => {
    const notebook = await serializer.deserializeNotebook(new TextEncoder().encode('{'), token);
    expect(notebook.cells).toEqual([]);
    expect(notebook.metadata).toEqual({ language: 'rust', marsFormatVersion: 2 });
  });

  it('serializes notebook metadata, stable IDs, and code outputs', async () => {
    const input = {
      cells: [
        {
          kind: vscode.NotebookCellKind.Code,
          value: 'println!("Project Mars");',
          languageId: 'rust',
          metadata: { marsId: 'stable-id', custom: 'kept' },
          outputs: [
            new vscode.NotebookCellOutput([
              new vscode.NotebookCellOutputItem(Buffer.from('ok'), 'text/plain'),
            ]),
          ],
        },
        {
          kind: vscode.NotebookCellKind.Markup,
          value: '# Hello',
          languageId: 'markdown',
          metadata: { marsId: 'markdown-id' },
          outputs: [
            new vscode.NotebookCellOutput([
              new vscode.NotebookCellOutputItem(Buffer.from('ignored'), 'text/plain'),
            ]),
          ],
        },
      ],
      metadata: { language: 'rust', title: 'Test', marsFormatVersion: 4 },
    } as unknown as vscode.NotebookData;
    const result = decode(await serializer.serializeNotebook(input, token));
    expect(result).toMatchObject({
      format_version: 4,
      metadata: { language: 'rust', title: 'Test' },
      cells: [
        {
          id: 'stable-id',
          cell_type: 'code',
          source: 'println!("Project Mars");',
          metadata: { custom: 'kept' },
          outputs: [{ type: 'text', text: 'ok' }],
        },
        { id: 'markdown-id', cell_type: 'markdown', source: '# Hello', metadata: {} },
      ],
    });
    expect((result.cells as Array<Record<string, unknown>>)[1]).not.toHaveProperty('outputs');
    expect(result.metadata as Record<string, unknown>).not.toHaveProperty('marsFormatVersion');
  });

  it('round-trips raw cell types and execution counts', async () => {
    const input = {
      format_version: 2,
      metadata: { language: 'rust' },
      cells: [
        {
          id: 'code-1',
          cell_type: 'code',
          source: '21 * 2',
          metadata: {},
          execution_count: 3,
          outputs: [{ type: 'text', text: '42' }],
        },
        { id: 'raw-1', cell_type: 'raw', source: 'plain text', metadata: {} },
      ],
    };
    const notebook = await serializer.deserializeNotebook(
      new TextEncoder().encode(JSON.stringify(input)),
      token,
    );
    const result = decode(await serializer.serializeNotebook(notebook, token));
    const cells = result.cells as Array<Record<string, unknown>>;
    expect(cells[0]).toMatchObject({ id: 'code-1', cell_type: 'code', execution_count: 3 });
    expect(cells[1]).toMatchObject({ id: 'raw-1', cell_type: 'raw' });
    expect(cells[0].metadata as Record<string, unknown>).not.toHaveProperty('marsCellType');
    expect(cells[0].metadata as Record<string, unknown>).not.toHaveProperty('marsExecutionCount');
  });

  it('fills in missing notebook version and language defaults and generates cell IDs', async () => {
    const input = {
      cells: [{ kind: vscode.NotebookCellKind.Code, value: '1', metadata: {} }],
      metadata: {},
    } as unknown as vscode.NotebookData;
    const result = decode(await serializer.serializeNotebook(input, token));
    expect(result).toMatchObject({ format_version: 2, metadata: { language: 'rust' } });
    expect((result.cells as Array<{ id: string }>)[0].id).toMatch(/^cell-\d+-[a-z0-9]+$/);
  });
});
