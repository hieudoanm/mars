import * as fs from 'node:fs/promises';
import * as vscode from 'vscode';
import {
  createCheckpoint as snapshotCheckpoint,
  listCheckpoints,
  readCheckpoint,
} from './checkpoints';
import { execFileAsync } from './cli';
import { clearNotebookDiagnostics, publishCellDiagnostics } from './diagnostics';
import { MarsNotebookController } from './notebookController';
import { NOTEBOOK_TEMPLATES } from './notebookTemplates';

export async function createNotebook(): Promise<void> {
  const pick = await vscode.window.showQuickPick(
    NOTEBOOK_TEMPLATES.map((template) => ({
      label: template.name,
      description: template.description,
      template,
    })),
    { placeHolder: 'Choose a notebook template' },
  );
  const template = pick?.template ?? NOTEBOOK_TEMPLATES[0];
  const document = await vscode.workspace.openTextDocument({
    content: template.content,
    language: 'mars',
  });
  await vscode.commands.executeCommand('vscode.openWith', document.uri, 'mars-notebook');
}

export async function runNotebook(): Promise<void> {
  const document = vscode.window.activeTextEditor?.document;
  if (!document || !document.fileName.endsWith('.mars')) {
    void vscode.window.showErrorMessage('Open a .mars notebook first.');
    return;
  }
  const config = vscode.workspace.getConfiguration('mars');
  const cliPath = config.get<string>('cliPath', 'mars');
  const runArguments = config.get<string[]>('runArguments', []);
  const output = vscode.window.createOutputChannel('Project Mars');
  output.show(true);
  output.appendLine(`$ ${cliPath} run ${document.fileName} ${runArguments.join(' ')}`.trim());
  try {
    const result = await execFileAsync(cliPath, ['run', document.fileName, ...runArguments]);
    if (result.stdout) output.append(result.stdout);
    if (result.stderr) output.append(result.stderr);
    void vscode.window.showInformationMessage('Project Mars notebook completed.');
  } catch (error) {
    const failure = error as { stdout?: string; stderr?: string; message?: string };
    if (failure.stdout) output.append(failure.stdout);
    if (failure.stderr) output.append(failure.stderr);
    void vscode.window.showErrorMessage(failure.message ?? 'Project Mars notebook execution failed');
  }
}

export function activeMarsFilePath(): string | undefined {
  const notebook = vscode.window.activeNotebookEditor?.notebook;
  if (notebook?.uri.scheme === 'file' && notebook.uri.fsPath.endsWith('.mars')) {
    return notebook.uri.fsPath;
  }
  const document = vscode.window.activeTextEditor?.document;
  if (document?.fileName.endsWith('.mars')) return document.fileName;
  return undefined;
}

export async function exportNotebook(format: 'html' | 'md', extension: string): Promise<void> {
  const filePath = activeMarsFilePath();
  if (!filePath) {
    void vscode.window.showErrorMessage('Open a .mars notebook first.');
    return;
  }
  const outputPath = filePath.replace(/\.mars$/i, `.${extension}`);
  try {
    const config = vscode.workspace.getConfiguration('mars');
    const cliPath = config.get<string>('cliPath', 'mars');
    await execFileAsync(cliPath, ['export', filePath, format, outputPath]);
    if (config.get<boolean>('previewAfterExport', true)) {
      await vscode.commands.executeCommand('vscode.open', vscode.Uri.file(outputPath));
    } else {
      void vscode.window.showInformationMessage(`Exported Project Mars notebook to ${outputPath}`);
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : 'Project Mars export failed';
    void vscode.window.showErrorMessage(message);
  }
}

export async function exportHtml(): Promise<void> {
  await exportNotebook('html', 'html');
}

export async function exportMarkdown(): Promise<void> {
  await exportNotebook('md', 'md');
}

export async function openCell(): Promise<void> {
  const editor = vscode.window.activeTextEditor;
  if (!editor) return;
  const input = await vscode.window.showInputBox({ prompt: 'Cell number' });
  const cellNumber = Number(input);
  if (!Number.isInteger(cellNumber) || cellNumber < 1) return;
  const text = editor.document.getText();
  const matches = [...text.matchAll(/"cell_type"\s*:/g)];
  const match = matches[cellNumber - 1];
  if (!match || match.index === undefined) {
    void vscode.window.showWarningMessage(`Cell ${cellNumber} was not found.`);
    return;
  }
  const position = editor.document.positionAt(match.index);
  editor.selection = new vscode.Selection(position, position);
  editor.revealRange(new vscode.Range(position, position), vscode.TextEditorRevealType.InCenter);
}

function controller(): MarsNotebookController | null {
  return MarsNotebookController.instance;
}

function notebookPathOf(notebook: vscode.NotebookDocument): string | null {
  return notebook.uri.scheme === 'file' ? notebook.uri.fsPath : null;
}

/**
 * Runs the kernel's compile check for the given code cells and publishes the
 * results to the Problems panel. `diagnose` treats each source as a candidate
 * next cell appended after the committed cells, so diagnostics whose
 * `cellIndex` points at an earlier committed cell are filtered out.
 */
async function diagnoseCells(
  cells: vscode.NotebookCell[],
  notebookPath: string | null,
): Promise<void> {
  const kernel = controller();
  if (!kernel || cells.length === 0) return;
  const committed = await kernel.kernelExecutionCount(notebookPath);
  for (const cell of cells) {
    const diagnostics = await kernel.diagnoseSource(cell.document.getText(), notebookPath);
    publishCellDiagnostics(
      cell,
      diagnostics.filter(
        (diagnostic) => diagnostic.cellIndex === null || diagnostic.cellIndex === committed,
      ),
    );
  }
}

export async function checkCell(): Promise<void> {
  const editor = vscode.window.activeNotebookEditor;
  if (!editor) {
    void vscode.window.showWarningMessage('Open a Project Mars notebook first.');
    return;
  }
  const cells = editor.notebook
    .getCells()
    .slice(editor.selection.start, editor.selection.end + 1)
    .filter((cell) => cell.kind === vscode.NotebookCellKind.Code);
  if (cells.length === 0) {
    void vscode.window.showWarningMessage('Select a Project Mars code cell first.');
    return;
  }
  try {
    await diagnoseCells(cells, notebookPathOf(editor.notebook));
  } catch (error) {
    void vscode.window.showErrorMessage(
      error instanceof Error ? error.message : 'Project Mars diagnostics failed',
    );
  }
}

export async function checkNotebook(): Promise<void> {
  const notebook = vscode.window.activeNotebookEditor?.notebook;
  if (!notebook) {
    void vscode.window.showWarningMessage('Open a Project Mars notebook first.');
    return;
  }
  const cells = notebook.getCells().filter((cell) => cell.kind === vscode.NotebookCellKind.Code);
  if (cells.length === 0) {
    void vscode.window.showInformationMessage('Notebook has no code cells.');
    return;
  }
  clearNotebookDiagnostics(notebook);
  try {
    await vscode.window.withProgress(
      {
        location: vscode.ProgressLocation.Notification,
        title: 'Checking Project Mars notebook',
        cancellable: false,
      },
      () => diagnoseCells(cells, notebookPathOf(notebook)),
    );
  } catch (error) {
    void vscode.window.showErrorMessage(
      error instanceof Error ? error.message : 'Project Mars diagnostics failed',
    );
  }
}

export async function sendKernelInput(): Promise<void> {
  const text = await vscode.window.showInputBox({
    prompt: 'Send a line to the running Project Mars kernel',
    ignoreFocusOut: true,
  });
  if (text === undefined) return;
  try {
    await controller()?.sendInput(text);
  } catch (error) {
    void vscode.window.showErrorMessage(
      error instanceof Error ? error.message : 'Failed to send kernel input',
    );
  }
}

export async function createCheckpoint(): Promise<void> {
  const filePath = activeMarsFilePath();
  if (!filePath) {
    void vscode.window.showErrorMessage('Open a .mars notebook first.');
    return;
  }
  try {
    const checkpoint = await snapshotCheckpoint(filePath);
    void vscode.window.showInformationMessage(`Checkpoint saved: ${checkpoint}`);
  } catch (error) {
    void vscode.window.showErrorMessage(
      error instanceof Error ? error.message : 'Failed to create checkpoint',
    );
  }
}

const formatCheckpointName = (checkpointPath: string): string => {
  const name = checkpointPath.split(/[\\/]/).pop() ?? checkpointPath;
  const match = name.match(/checkpoint-(\d+)\.mars$/);
  if (!match) return name;
  return new Date(Number(match[1])).toLocaleString();
};

export async function restoreCheckpoint(): Promise<void> {
  const filePath = activeMarsFilePath();
  if (!filePath) {
    void vscode.window.showErrorMessage('Open a .mars notebook first.');
    return;
  }
  const checkpoints = await listCheckpoints(filePath);
  if (checkpoints.length === 0) {
    void vscode.window.showInformationMessage('No checkpoints found for this notebook.');
    return;
  }
  const pick = await vscode.window.showQuickPick(
    checkpoints.map((checkpoint) => ({
      label: formatCheckpointName(checkpoint),
      description: checkpoint,
      checkpoint,
    })),
    { placeHolder: 'Restore notebook from checkpoint' },
  );
  if (!pick) return;
  const confirmed = await vscode.window.showWarningMessage(
    `Restore ${filePath} from ${formatCheckpointName(pick.checkpoint)}? Current contents will be overwritten.`,
    { modal: true },
    'Restore',
  );
  if (confirmed !== 'Restore') return;
  try {
    const content = await readCheckpoint(pick.checkpoint);
    await fs.writeFile(filePath, content, 'utf-8');
    await vscode.commands.executeCommand('vscode.open', vscode.Uri.file(filePath));
    void vscode.window.showInformationMessage('Notebook restored from checkpoint.');
  } catch (error) {
    void vscode.window.showErrorMessage(
      error instanceof Error ? error.message : 'Failed to restore checkpoint',
    );
  }
}
