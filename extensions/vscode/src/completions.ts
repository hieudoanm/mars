import * as vscode from 'vscode';
import type { MarsNotebookController } from './notebookController';
import type { MarsVariableInfo } from './kernelProtocol';

/**
 * Completion provider for Rust code cells. Notebook cell documents use the
 * `vscode-notebook-cell` URI scheme, which rust-analyzer does not index, so
 * this provider supplies Mars-aware suggestions: live kernel session
 * variables plus a small set of Rust keywords and std prelude items.
 */
export class MarsCompletionProvider implements vscode.CompletionItemProvider {
  private variables: MarsVariableInfo[] = [];
  private variablesFetchedAt = 0;
  private fetching: Promise<void> | null = null;

  constructor(private readonly controller: () => MarsNotebookController | null) {}

  /** Called after executions and manual refreshes to pick up new bindings. */
  invalidate(): void {
    this.variablesFetchedAt = 0;
  }

  async provideCompletionItems(document: vscode.TextDocument): Promise<vscode.CompletionItem[]> {
    if (document.uri.scheme !== 'vscode-notebook-cell') return [];
    if (document.languageId !== 'rust') return [];
    await this.refreshVariables();
    const items: vscode.CompletionItem[] = [];
    for (const variable of this.variables) {
      const item = new vscode.CompletionItem(variable.name, vscode.CompletionItemKind.Variable);
      item.detail = variable.typeName ?? 'session variable';
      item.documentation = variable.value
        ? new vscode.MarkdownString('```\n' + variable.value + '\n```')
        : undefined;
      item.sortText = `0-${variable.name}`;
      items.push(item);
    }
    for (const keyword of RUST_KEYWORDS) {
      const item = new vscode.CompletionItem(keyword, vscode.CompletionItemKind.Keyword);
      item.sortText = `1-${keyword}`;
      items.push(item);
    }
    return items;
  }

  private async refreshVariables(): Promise<void> {
    // Cache briefly: completions are requested on every keystroke.
    if (Date.now() - this.variablesFetchedAt < 2_000) return;
    this.fetching ??= this.fetchVariables().finally(() => {
      this.fetching = null;
    });
    await this.fetching;
  }

  private async fetchVariables(): Promise<void> {
    const kernel = this.controller();
    // Never spawn a kernel just for completions; only query a live session.
    if (!kernel?.kernelRunning()) return;
    try {
      this.variables = await kernel.inspectVariables();
    } catch {
      // Kernel unavailable (CLI missing, not started); keep last snapshot.
    } finally {
      // Back off after failures too so dead kernels aren't queried per keystroke.
      this.variablesFetchedAt = Date.now();
    }
  }
}

const RUST_KEYWORDS = [
  'as',
  'async',
  'await',
  'break',
  'const',
  'continue',
  'crate',
  'dyn',
  'else',
  'enum',
  'extern',
  'false',
  'fn',
  'for',
  'if',
  'impl',
  'in',
  'let',
  'loop',
  'match',
  'mod',
  'move',
  'mut',
  'pub',
  'ref',
  'return',
  'self',
  'static',
  'struct',
  'super',
  'trait',
  'true',
  'type',
  'unsafe',
  'use',
  'where',
  'while',
];
