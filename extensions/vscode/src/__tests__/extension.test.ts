import * as vscode from 'vscode';
import { activate, deactivate } from '../extension';
import { MarsNotebookController } from '../notebookController';

describe('extension activation', () => {
  beforeEach(() => {
    vi.mocked(vscode.workspace.registerNotebookSerializer).mockClear();
    vi.mocked(vscode.commands.registerCommand).mockClear();
    vi.mocked(vscode.notebooks.createNotebookController).mockClear();
  });

  it('registers serializer, controller, and all public commands in subscriptions', () => {
    const context = { subscriptions: [] as { dispose?: () => void }[] } as vscode.ExtensionContext;
    activate(context);
    expect(vscode.workspace.registerNotebookSerializer).toHaveBeenCalledWith(
      'mars-notebook',
      expect.any(Object),
    );
    expect(vscode.notebooks.createNotebookController).toHaveBeenCalledWith(
      'mars-kernel',
      'mars-notebook',
      'Mars Kernel',
    );
    expect(vscode.commands.registerCommand).toHaveBeenCalledTimes(14);
    expect(context.subscriptions).toHaveLength(20);
    expect(MarsNotebookController.instance).toBeInstanceOf(MarsNotebookController);
  });

  it('deactivates without requiring extension state', () => {
    expect(deactivate()).toBeUndefined();
  });
});
