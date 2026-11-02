import { vi } from "vitest";

vi.mock("vscode", () => {
  class NotebookCellOutputItem {
    constructor(public data: Uint8Array, public mime?: string) {}
  }
  class NotebookCellOutput {
    constructor(public items: NotebookCellOutputItem[]) {}
  }
  return {
    window: {
      showErrorMessage: vi.fn(),
      showInformationMessage: vi.fn(),
      createOutputChannel: vi.fn(() => ({ appendLine: vi.fn(), show: vi.fn() })),
    },
    workspace: {
      workspaceFolders: [],
      getConfiguration: vi.fn(() => ({ get: vi.fn() })),
    },
    commands: {
      registerCommand: vi.fn(),
    },
    Uri: {
      file: (p: string) => ({ fsPath: p, path: p }),
    },
    extensions: {
      getExtension: vi.fn(),
    },
    NotebookCellOutputItem,
    NotebookCellOutput,
  };
});
