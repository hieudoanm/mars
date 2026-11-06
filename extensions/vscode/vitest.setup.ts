import { vi } from 'vitest';

vi.mock('vscode', () => {
  class NotebookCellOutputItem {
    constructor(
      public data: Uint8Array,
      public mime?: string,
    ) {}
  }
  class NotebookCellOutput {
    constructor(public items: NotebookCellOutputItem[]) {}
  }
  class NotebookCellData {
    metadata: Record<string, unknown> = {};
    outputs: NotebookCellOutput[] = [];
    constructor(
      public kind: number,
      public value: string,
      public languageId: string,
    ) {}
  }
  class NotebookData {
    metadata: Record<string, unknown> = {};
    constructor(public cells: NotebookCellData[]) {}
  }
  class Selection {
    constructor(
      public anchor: unknown,
      public active: unknown,
    ) {}
  }
  class Range {
    constructor(
      public start: unknown,
      public end: unknown,
    ) {}
  }
  const controller = {
    supportedLanguages: [],
    supportsExecutionOrder: false,
    executeHandler: undefined,
    dispose: vi.fn(),
    createNotebookCellExecution: vi.fn(() => ({
      start: vi.fn(),
      appendOutput: vi.fn(),
      clearOutput: vi.fn(),
      end: vi.fn(),
    })),
  };
  const workspace = {
    workspaceFolders: [],
    getConfiguration: vi.fn(() => ({ get: vi.fn((_key: string, fallback: unknown) => fallback) })),
    registerNotebookSerializer: vi.fn((_type: string, serializer: unknown) => ({ serializer })),
  };
  const commands = {
    registerCommand: vi.fn((id: string, callback: (...args: unknown[]) => unknown) => ({
      id,
      callback,
    })),
    executeCommand: vi.fn(),
  };
  const window = {
    showErrorMessage: vi.fn(),
    showWarningMessage: vi.fn(),
    showInformationMessage: vi.fn(),
    showInputBox: vi.fn(),
    createOutputChannel: vi.fn(() => ({
      appendLine: vi.fn(),
      append: vi.fn(),
      show: vi.fn(),
    })),
    activeNotebookEditor: undefined,
    activeTextEditor: undefined,
  };
  return {
    window,
    workspace,
    commands,
    notebooks: { createNotebookController: vi.fn(() => controller) },
    Uri: {
      file: (p: string) => ({ scheme: 'file', fsPath: p, path: p }),
      parse: (uri: string) => ({ scheme: uri.split(':')[0], fsPath: uri, path: uri }),
    },
    Selection,
    Range,
    TextEditorRevealType: { InCenter: 1 },
    NotebookCellKind: { Markup: 1, Code: 2 },
    NotebookCellData,
    NotebookData,
    NotebookCellOutputItem,
    NotebookCellOutput,
  };
});
