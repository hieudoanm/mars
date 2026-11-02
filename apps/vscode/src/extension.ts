import * as vscode from 'vscode';
import { createNotebook, exportHtml, exportMarkdown, openCell, runNotebook } from './commands';
import { MarsNotebookController } from './notebookController';
import { MarsNotebookSerializer } from './notebookSerializer';

export function activate(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.workspace.registerNotebookSerializer('mars-notebook', new MarsNotebookSerializer()),
  );
  const controller = new MarsNotebookController();
  context.subscriptions.push(controller);
  context.subscriptions.push(
    vscode.commands.registerCommand('mars.exportHtml', exportHtml),
    vscode.commands.registerCommand('mars.exportMarkdown', exportMarkdown),
    vscode.commands.registerCommand('mars.preview', exportHtml),
    vscode.commands.registerCommand('mars.runNotebook', runNotebook),
    vscode.commands.registerCommand('mars.openCell', openCell),
    vscode.commands.registerCommand('mars.createNotebook', createNotebook),
    vscode.commands.registerCommand('mars.restartKernel', async () => {
      await controller.restartKernel();
      void vscode.window.showInformationMessage('Mars kernel restarted');
    }),
  );
}

export function deactivate(): void {}
