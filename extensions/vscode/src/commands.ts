import * as vscode from 'vscode';
import { execFileAsync } from './cli';

export async function createNotebook(): Promise<void> {
  const notebookUri = vscode.Uri.parse('untitled:Untitled.mars');
  await vscode.commands.executeCommand('vscode.openWith', notebookUri, 'mars-notebook');
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
  const output = vscode.window.createOutputChannel('Mars');
  output.show(true);
  output.appendLine(`$ ${cliPath} run ${document.fileName} ${runArguments.join(' ')}`.trim());
  try {
    const result = await execFileAsync(cliPath, ['run', document.fileName, ...runArguments]);
    if (result.stdout) output.append(result.stdout);
    if (result.stderr) output.append(result.stderr);
    void vscode.window.showInformationMessage('Mars notebook completed.');
  } catch (error) {
    const failure = error as { stdout?: string; stderr?: string; message?: string };
    if (failure.stdout) output.append(failure.stdout);
    if (failure.stderr) output.append(failure.stderr);
    void vscode.window.showErrorMessage(failure.message ?? 'Mars notebook execution failed');
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
      void vscode.window.showInformationMessage(`Exported Mars notebook to ${outputPath}`);
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : 'Mars export failed';
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
