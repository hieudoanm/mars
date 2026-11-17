import * as vscode from 'vscode';

const { execFileAsync } = vi.hoisted(() => ({ execFileAsync: vi.fn() }));
vi.mock('../cli', () => ({ execFileAsync, commandExists: vi.fn() }));

import * as fs from 'node:fs/promises';
import * as os from 'node:os';
import * as path from 'node:path';
import {
  activeMarsFilePath,
  checkCell,
  checkNotebook,
  createCheckpoint,
  createNotebook,
  exportHtml,
  exportMarkdown,
  openCell,
  restoreCheckpoint,
  runNotebook,
  sendKernelInput,
} from '../commands';
import { MarsNotebookController } from '../notebookController';

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
    expect(execFileAsync).toHaveBeenCalledWith('mars', ['notebook', 'run', '/work/book.mars']);
    expect(output.appendLine).toHaveBeenCalledWith('$ mars notebook run /work/book.mars');
    expect(output.append).toHaveBeenCalledWith('complete');
    expect(output.append).toHaveBeenCalledWith('warning');
    expect(vscode.window.showInformationMessage).toHaveBeenCalledWith(
      'Project Mars notebook completed.',
    );
  });

  it('exports HTML or Markdown and handles CLI failures', async () => {
    setWindow('activeNotebookEditor', {
      notebook: { uri: { scheme: 'file', fsPath: '/work/Book.mars' } },
    });
    execFileAsync.mockResolvedValueOnce({ stdout: '', stderr: '' });
    await exportHtml();
    expect(execFileAsync).toHaveBeenCalledWith('mars', [
      'notebook',
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
      'notebook',
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

  it('checks cells and notebooks through the kernel diagnose path', async () => {
    const fake = {
      kernelExecutionCount: vi.fn(async () => 3),
      diagnoseSource: vi.fn(async () => [
        {
          line: 1,
          column: 1,
          endLine: 1,
          endColumn: 4,
          severity: 'error',
          message: 'bad',
          cellIndex: 3,
        },
        {
          line: 2,
          column: 1,
          endLine: 2,
          endColumn: 4,
          severity: 'warning',
          message: 'stale',
          cellIndex: 0,
        },
      ]),
      sendInput: vi.fn(async () => {}),
    };
    MarsNotebookController.instance = fake as never;
    const codeCell = {
      kind: vscode.NotebookCellKind.Code,
      document: { getText: () => 'let x = 1;' },
    };
    const mdCell = { kind: vscode.NotebookCellKind.Markup, document: { getText: () => '#' } };
    const notebook = {
      uri: { scheme: 'file', fsPath: '/work/book.mars' },
      getCells: () => [codeCell, mdCell],
    };
    setWindow('activeNotebookEditor', {
      notebook,
      selection: { start: 0, end: 0 },
    });
    await checkCell();
    expect(fake.diagnoseSource).toHaveBeenCalledWith('let x = 1;', '/work/book.mars');

    await checkNotebook();
    expect(fake.kernelExecutionCount).toHaveBeenCalled();

    fake.diagnoseSource.mockRejectedValueOnce(new Error('diag failed'));
    await checkCell();
    expect(vscode.window.showErrorMessage).toHaveBeenCalledWith('diag failed');

    setWindow('activeNotebookEditor', undefined);
    await checkCell();
    await checkNotebook();
    setWindow('activeNotebookEditor', {
      notebook: { ...notebook, getCells: () => [mdCell] },
      selection: { start: 0, end: 0 },
    });
    await checkCell();
    await checkNotebook();
    MarsNotebookController.instance = null;
  });

  it('sends kernel input and reports failures', async () => {
    const sendInput = vi.fn(async () => {});
    MarsNotebookController.instance = { sendInput } as never;
    vi.mocked(vscode.window.showInputBox).mockResolvedValueOnce(undefined);
    await sendKernelInput();
    expect(sendInput).not.toHaveBeenCalled();

    vi.mocked(vscode.window.showInputBox).mockResolvedValueOnce('42');
    await sendKernelInput();
    expect(sendInput).toHaveBeenCalledWith('42');

    sendInput.mockRejectedValueOnce(new Error('no kernel'));
    vi.mocked(vscode.window.showInputBox).mockResolvedValueOnce('x');
    await sendKernelInput();
    expect(vscode.window.showErrorMessage).toHaveBeenCalledWith('no kernel');
    MarsNotebookController.instance = null;
  });

  it('creates and restores checkpoints end to end', async () => {
    const dir = await fs.mkdtemp(path.join(os.tmpdir(), 'mars-cmd-cp-'));
    const notebookPath = path.join(dir, 'book.mars');
    await fs.writeFile(notebookPath, '{"cells":[{"cell_type":"code"}]}');
    setWindow('activeTextEditor', { document: { fileName: notebookPath } });

    await createCheckpoint();
    expect(vscode.window.showInformationMessage).toHaveBeenCalledWith(
      expect.stringContaining('Checkpoint saved:'),
    );

    const other = path.join(dir, 'other.mars');
    await fs.writeFile(other, '{}');
    setWindow('activeTextEditor', { document: { fileName: other } });
    await restoreCheckpoint();
    expect(vscode.window.showInformationMessage).toHaveBeenCalledWith(
      'No checkpoints found for this notebook.',
    );

    setWindow('activeTextEditor', { document: { fileName: notebookPath } });
    vi.mocked(vscode.window.showQuickPick).mockResolvedValueOnce(undefined);
    await restoreCheckpoint();
    const checkpointDir = path.join(dir, 'book.mars.checkpoints');
    const checkpointFile = (await fs.readdir(checkpointDir))[0];
    const pick = { checkpoint: path.join(checkpointDir, checkpointFile) } as never;
    vi.mocked(vscode.window.showQuickPick).mockResolvedValue(pick);
    vi.mocked(vscode.window.showWarningMessage).mockResolvedValueOnce('Cancel' as never);
    await restoreCheckpoint();
    vi.mocked(vscode.window.showWarningMessage).mockResolvedValueOnce('Restore' as never);
    await restoreCheckpoint();
    expect(vscode.commands.executeCommand).toHaveBeenCalledWith(
      'vscode.open',
      expect.objectContaining({ fsPath: notebookPath }),
    );

    setWindow('activeTextEditor', undefined);
    setWindow('activeNotebookEditor', undefined);
    await createCheckpoint();
    await restoreCheckpoint();
    expect(vscode.window.showErrorMessage).toHaveBeenCalledWith('Open a .mars notebook first.');

    setWindow('activeTextEditor', { document: { fileName: '/nonexistent-dir-x/book.mars' } });
    await createCheckpoint();
    expect(vscode.window.showErrorMessage).toHaveBeenCalled();
  });
});
