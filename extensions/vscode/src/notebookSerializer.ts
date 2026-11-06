import * as vscode from 'vscode';
import { marsOutputToNotebookOutput, notebookOutputToMarsOutput, type MarsOutput } from './output';

interface MarsCell {
  id: string;
  cell_type: 'code' | 'markdown' | 'raw';
  source: string;
  metadata: Record<string, unknown>;
  execution_count?: number | null;
  outputs?: MarsOutput[];
}

interface MarsNotebook {
  format_version: number;
  metadata: {
    title?: string;
    language: string;
    dependencies?: Record<string, string>;
    cargo_lock?: string | null;
    [key: string]: unknown;
  };
  cells: MarsCell[];
}

export class MarsNotebookSerializer implements vscode.NotebookSerializer {
  async deserializeNotebook(
    content: Uint8Array,
    token: vscode.CancellationToken,
  ): Promise<vscode.NotebookData> {
    void token;
    const text = new TextDecoder().decode(content);
    let notebook: MarsNotebook;
    try {
      notebook = JSON.parse(text) as MarsNotebook;
    } catch {
      notebook = { format_version: 2, metadata: { language: 'rust' }, cells: [] };
    }

    const cells = notebook.cells.map((marsCell) => {
      const kind =
        marsCell.cell_type === 'code'
          ? vscode.NotebookCellKind.Code
          : vscode.NotebookCellKind.Markup;
      const cell = new vscode.NotebookCellData(
        kind,
        marsCell.source,
        marsCell.cell_type === 'code' ? 'rust' : 'markdown',
      );
      cell.metadata = { ...marsCell.metadata, marsId: marsCell.id };
      if (marsCell.outputs?.length) {
        cell.outputs = marsCell.outputs.map(marsOutputToNotebookOutput);
      }
      return cell;
    });
    const data = new vscode.NotebookData(cells);
    data.metadata = {
      ...notebook.metadata,
      marsFormatVersion: notebook.format_version,
    } as Record<string, unknown>;
    return data;
  }

  async serializeNotebook(
    data: vscode.NotebookData,
    token: vscode.CancellationToken,
  ): Promise<Uint8Array> {
    void token;
    const cells: MarsCell[] = data.cells.map((cell) => {
      const metadata = { ...(cell.metadata as Record<string, unknown>) };
      const id =
        typeof metadata.marsId === 'string' && metadata.marsId
          ? metadata.marsId
          : this.generateId();
      delete metadata.marsId;
      const marsCell: MarsCell = {
        id,
        cell_type: cell.kind === vscode.NotebookCellKind.Code ? 'code' : 'markdown',
        source: cell.value,
        metadata,
        execution_count: null,
      };
      if (cell.outputs?.length) {
        marsCell.outputs = cell.outputs.map(notebookOutputToMarsOutput);
      }
      if (marsCell.cell_type !== 'code') {
        delete marsCell.execution_count;
        delete marsCell.outputs;
      }
      return marsCell;
    });
    const metadata = { ...(data.metadata as Record<string, unknown>) };
    const version =
      typeof metadata.marsFormatVersion === 'number' && metadata.marsFormatVersion
        ? metadata.marsFormatVersion
        : 2;
    const language =
      typeof metadata.language === 'string' && metadata.language ? metadata.language : 'rust';
    delete metadata.marsFormatVersion;
    const notebook: MarsNotebook = {
      format_version: version,
      metadata: { ...metadata, language } as MarsNotebook['metadata'],
      cells,
    };
    return new TextEncoder().encode(JSON.stringify(notebook, null, 2));
  }

  private generateId(): string {
    return `cell-${Date.now()}-${Math.random().toString(36).slice(2, 11)}`;
  }
}
