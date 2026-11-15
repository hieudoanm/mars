import * as vscode from 'vscode';
import {
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
} from './commands';
import { disposeDiagnostics } from './diagnostics';
import { MarsNotebookController } from './notebookController';
import { MarsNotebookSerializer } from './notebookSerializer';
import { MarsVariablesProvider } from './variablesView';
import { MarsCompletionProvider } from './completions';

export function activate(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.workspace.registerNotebookSerializer('mars-notebook', new MarsNotebookSerializer()),
  );
  const controller = new MarsNotebookController();
  context.subscriptions.push(controller);

  const variables = new MarsVariablesProvider(controller);
  context.subscriptions.push(variables);

  const completions = new MarsCompletionProvider(() => MarsNotebookController.instance);
  context.subscriptions.push(vscode.languages.registerCompletionItemProvider('rust', completions));

  context.subscriptions.push(
    vscode.window.registerTreeDataProvider('mars.variables', variables),
    vscode.commands.registerCommand('mars.refreshVariables', () => {
      completions.invalidate();
      return variables.reload();
    }),
  );
  context.subscriptions.push({ dispose: () => disposeDiagnostics() });

  context.subscriptions.push(
    vscode.commands.registerCommand('mars.exportHtml', exportHtml),
    vscode.commands.registerCommand('mars.exportMarkdown', exportMarkdown),
    vscode.commands.registerCommand('mars.preview', exportHtml),
    vscode.commands.registerCommand('mars.runNotebook', runNotebook),
    vscode.commands.registerCommand('mars.openCell', openCell),
    vscode.commands.registerCommand('mars.createNotebook', createNotebook),
    vscode.commands.registerCommand('mars.restartKernel', async () => {
      await controller.restartKernel();
      void vscode.window.showInformationMessage('Project Mars kernel restarted');
    }),
    vscode.commands.registerCommand('mars.interruptExecution', async () => {
      await controller.interruptExecution();
    }),
    vscode.commands.registerCommand('mars.checkCell', checkCell),
    vscode.commands.registerCommand('mars.checkNotebook', checkNotebook),
    vscode.commands.registerCommand('mars.sendKernelInput', sendKernelInput),
    vscode.commands.registerCommand('mars.createCheckpoint', createCheckpoint),
    vscode.commands.registerCommand('mars.restoreCheckpoint', restoreCheckpoint),
  );
}

export function deactivate(): void {}
