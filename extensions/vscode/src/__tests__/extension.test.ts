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
      'Project Mars Kernel',
    );
    expect(vscode.commands.registerCommand).toHaveBeenCalledTimes(14);
    expect(context.subscriptions).toHaveLength(20);
    expect(MarsNotebookController.instance).toBeInstanceOf(MarsNotebookController);
  });

  it('deactivates without requiring extension state', () => {
    expect(deactivate()).toBeUndefined();
  });
});

describe('command callbacks', () => {
  it('restart, interrupt, and refresh callbacks run without a kernel', async () => {
    const context = { subscriptions: [] as { dispose?: () => void }[] } as vscode.ExtensionContext;
    activate(context);
    const registered = vi.mocked(vscode.commands.registerCommand).mock.calls;
    const callbackFor = (id: string) =>
      registered.find(([commandId]) => commandId === id)?.[1] as () => Promise<unknown>;

    await callbackFor('mars.restartKernel')();
    expect(vscode.window.showInformationMessage).toHaveBeenCalledWith(
      'Project Mars kernel restarted',
    );
    await callbackFor('mars.interruptExecution')();
    // refreshVariables invalidates completions and reloads variables; the
    // kernel is absent so reload records an error internally.
    await callbackFor('mars.refreshVariables')();
  });
});
