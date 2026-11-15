import * as vscode from 'vscode';

const { execFileAsync } = vi.hoisted(() => ({ execFileAsync: vi.fn() }));
vi.mock('../cli', () => ({ execFileAsync, commandExists: vi.fn() }));

import {
  activeMarsFilePath,
  createNotebook,
  exportHtml,
  exportMarkdown,
  openCell,
  runNotebook,
} from '../commands';

const windowApi = vscode.window as unknown as Record<string, unknown>;
const setWindow = (name: string, value: unknown) => {
  Object.defineProperty(windowApi, name, { configurable: true, value });
};

describe('VS Code commands', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    setWindow('activeNotebookEditor', undefined);
    setWindow('activeTextEditor', undefined);
    vi.mocked(vscode.window.showInputBox).mockResolvedValue(undefined);
    vi.mocked(vscode.commands.executeCommand).mockResolvedValue(undefined);
  });

  it('creates a new Project Mars notebook in the notebook editor', async () => {
    vi.mocked(vscode.workspace.openTextDocument).mockResolvedValue({
      uri: { scheme: 'untitled' },
    } as never);
    await createNotebook();
    expect(vscode.commands.executeCommand).toHaveBeenCalledWith(
      'vscode.openWith',
      expect.objectContaining({ scheme: 'untitled' }),
      'mars-notebook',
    );
    expect(vscode.workspace.openTextDocument).toHaveBeenCalledWith(
      expect.objectContaining({ language: 'mars' }),
    );
  });

  it('resolves active notebook and text document paths and rejects unrelated files', () => {
    setWindow('activeNotebookEditor', { notebook: { uri: { scheme: 'file', fsPath: '/a.mars' } } });
    expect(activeMarsFilePath()).toBe('/a.mars');
    setWindow('activeNotebookEditor', {
      notebook: { uri: { scheme: 'untitled', fsPath: '/b.mars' } },
    });
    setWindow('activeTextEditor', { document: { fileName: '/b.mars' } });
    expect(activeMarsFilePath()).toBe('/b.mars');
    setWindow('activeTextEditor', { document: { fileName: '/readme.md' } });
    expect(activeMarsFilePath()).toBeUndefined();
  });

  it('runs only Project Mars documents and shows CLI results', async () => {
    await runNotebook();
    expect(vscode.window.showErrorMessage).toHaveBeenCalledWith('Open a .mars notebook first.');
    const output = { appendLine: vi.fn(), append: vi.fn(), show: vi.fn() };
    vi.mocked(vscode.window.createOutputChannel).mockReturnValue(output as never);
    setWindow('activeTextEditor', {
      document: { fileName: '/work/book.mars' },
    });
    execFileAsync.mockResolvedValueOnce({ stdout: 'complete', stderr: 'warning' });
    await runNotebook();
    expect(execFileAsync).toHaveBeenCalledWith('mars', ['run', '/work/book.mars']);
    expect(output.appendLine).toHaveBeenCalledWith('$ mars run /work/book.mars');
    expect(output.append).toHaveBeenCalledWith('complete');
    expect(output.append).toHaveBeenCalledWith('warning');
    expect(vscode.window.showInformationMessage).toHaveBeenCalledWith('Project Mars notebook completed.');
  });

  it('exports HTML or Markdown and handles CLI failures', async () => {
    setWindow('activeNotebookEditor', {
      notebook: { uri: { scheme: 'file', fsPath: '/work/Book.mars' } },
    });
    execFileAsync.mockResolvedValueOnce({ stdout: '', stderr: '' });
    await exportHtml();
    expect(execFileAsync).toHaveBeenCalledWith('mars', [
      'export',
      '/work/Book.mars',
      'html',
      '/work/Book.html',
    ]);
    expect(vscode.commands.executeCommand).toHaveBeenCalledWith(
      'vscode.open',
      expect.objectContaining({ fsPath: '/work/Book.html' }),
    );

    execFileAsync.mockRejectedValueOnce(new Error('export failed'));
    await exportMarkdown();
    expect(execFileAsync).toHaveBeenLastCalledWith('mars', [
      'export',
      '/work/Book.mars',
      'md',
      '/work/Book.md',
    ]);
    expect(vscode.window.showErrorMessage).toHaveBeenCalledWith('export failed');
  });

  it('moves the editor to a numbered cell and handles invalid or missing cells', async () => {
    const source = '{"cell_type":"markdown"}\n{"cell_type":"code"}';
    const editor = {
      document: {
        getText: () => source,
        positionAt: vi.fn((offset: number) => ({ offset })),
      },
      revealRange: vi.fn(),
    };
    setWindow('activeTextEditor', editor);
    vi.mocked(vscode.window.showInputBox)
      .mockResolvedValueOnce('0')
      .mockResolvedValueOnce('3')
      .mockResolvedValueOnce('2');
    await openCell();
    await openCell();
    expect(vscode.window.showWarningMessage).toHaveBeenCalledWith('Cell 3 was not found.');
    await openCell();
    expect(editor.document.positionAt).toHaveBeenCalledWith(source.lastIndexOf('"cell_type"'));
    expect(editor.revealRange).toHaveBeenCalled();
  });
});
