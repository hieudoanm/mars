import * as vscode from 'vscode';
import type { MarsCellDiagnostic } from './kernelProtocol';

const collection = vscode.languages.createDiagnosticCollection('mars-kernel');

const toSeverity = (severity: string): vscode.DiagnosticSeverity =>
  severity === 'warning' ? vscode.DiagnosticSeverity.Warning : vscode.DiagnosticSeverity.Error;

/**
 * Publishes kernel compile diagnostics for a single notebook cell.
 * `cellIndex` is the index of the checked source inside the kernel's
 * candidate cell list; diagnostics pointing at other (already committed)
 * cells are filtered out by the caller.
 */
export function publishCellDiagnostics(
  cell: vscode.NotebookCell,
  diagnostics: MarsCellDiagnostic[],
): void {
  const items = diagnostics.map((diagnostic) => {
    const line = Math.max(0, diagnostic.line - 1);
    const endLine = Math.max(line, diagnostic.endLine - 1);
    const range = new vscode.Range(
      line,
      Math.max(0, diagnostic.column - 1),
      endLine,
      Math.max(0, diagnostic.endColumn - 1),
    );
    const item = new vscode.Diagnostic(range, diagnostic.message, toSeverity(diagnostic.severity));
    item.source = 'mars';
    if (diagnostic.code) item.code = diagnostic.code;
    return item;
  });
  collection.set(cell.document.uri, items);
}

export function clearCellDiagnostics(cell: vscode.NotebookCell): void {
  collection.delete(cell.document.uri);
}

export function clearNotebookDiagnostics(notebook: vscode.NotebookDocument): void {
  for (const cell of notebook.getCells()) collection.delete(cell.document.uri);
}

export function disposeDiagnostics(): void {
  collection.dispose();
}
