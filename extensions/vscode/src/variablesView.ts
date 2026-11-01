import * as vscode from 'vscode';
import type { MarsNotebookController } from './notebookController';
import type { MarsVariableInfo } from './kernelProtocol';

class VariableItem extends vscode.TreeItem {
  constructor(readonly variable: MarsVariableInfo) {
    super(variable.name, vscode.TreeItemCollapsibleState.None);
    this.description = variable.typeName ?? undefined;
    this.tooltip = variable.value ?? variable.typeName ?? variable.name;
    this.contextValue = 'marsVariable';
    this.iconPath = new vscode.ThemeIcon('symbol-variable');
  }
}

export class MarsVariablesProvider implements vscode.TreeDataProvider<vscode.TreeItem> {
  private readonly emitter = new vscode.EventEmitter<void>();
  readonly onDidChangeTreeData = this.emitter.event;
  private variables: MarsVariableInfo[] = [];
  private error: string | null = null;

  constructor(private readonly controller: MarsNotebookController) {}

  dispose(): void {
    this.emitter.dispose();
  }

  refresh(): void {
    this.emitter.fire();
  }

  async reload(): Promise<void> {
    try {
      this.variables = await this.controller.inspectVariables();
      this.error = null;
    } catch (error) {
      this.variables = [];
      this.error = error instanceof Error ? error.message : String(error);
    }
    this.refresh();
  }

  getTreeItem(element: vscode.TreeItem): vscode.TreeItem {
    return element;
  }

  getChildren(element?: vscode.TreeItem): vscode.TreeItem[] {
    if (element) return [];
    if (this.error) {
      const item = new vscode.TreeItem(`Kernel error: ${this.error}`);
      item.iconPath = new vscode.ThemeIcon('error');
      return [item];
    }
    if (this.variables.length === 0) {
      const item = new vscode.TreeItem('No variables in session');
      item.iconPath = new vscode.ThemeIcon('info');
      return [item];
    }
    return this.variables.map((variable) => new VariableItem(variable));
  }
}
