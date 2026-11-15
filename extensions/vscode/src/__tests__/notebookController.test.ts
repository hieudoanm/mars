import * as vscode from 'vscode';

const { commandExists, kernel, KernelService } = vi.hoisted(() => {
  const kernel = { execute: vi.fn(), reset: vi.fn(), stop: vi.fn() };
  const KernelService = vi.fn(function KernelServiceMock() {
    return kernel;
  });
  return { commandExists: vi.fn(), kernel, KernelService };
});
vi.mock('../cli', () => ({ commandExists, execFileAsync: vi.fn() }));
vi.mock('../kernelService', () => ({ MarsKernelService: KernelService }));

import { MarsNotebookController } from '../notebookController';

function configure(values: Record<string, unknown> = {}) {
  vi.mocked(vscode.workspace.getConfiguration).mockReturnValue({
    get: (key: string, fallback: unknown) => values[key] ?? fallback,
  } as never);
}

function createCell(source: string) {
  return {
    document: { getText: () => source },
  } as unknown as vscode.NotebookCell;
}

function executionHandler() {
  const controller = new MarsNotebookController();
  const mockController = vi
    .mocked(vscode.notebooks.createNotebookController)
    .mock.results.at(-1)?.value;
  return {
    controller,
    handler: mockController.executeHandler as NonNullable<typeof mockController.executeHandler>,
  };
}

describe('Project Mars notebook execution controller', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.clearAllMocks();
    commandExists.mockResolvedValue(false);
    kernel.execute.mockResolvedValue({
      status: 'success',
      execution_count: 1,
      outputs: [{ type: 'text', text: 'ok' }],
    });
    kernel.reset.mockResolvedValue(undefined);
    configure();
  });
  afterEach(() => vi.useRealTimers());

  it('configures the notebook controller and executes missing-CLI fallback output', async () => {
    const { controller, handler } = executionHandler();
    const cell = createCell('println!("hi");');
    await handler([cell], {} as vscode.NotebookDocument, {} as vscode.NotebookController);
    const api = vi.mocked(vscode.notebooks.createNotebookController).mock.results.at(-1)?.value;
    expect(api.supportedLanguages).toEqual(['rust', 'python', 'r', 'julia', 'cpp']);
    expect(api.supportsExecutionOrder).toBe(true);
    const execution = api.createNotebookCellExecution.mock.results[0].value;
    expect(execution.appendOutput).toHaveBeenCalledOnce();
    expect(Buffer.from(execution.appendOutput.mock.calls[0][0].items[0].data).toString()).toContain(
      'CLI not found',
    );
    expect(execution.end).toHaveBeenCalledWith(true, expect.any(Number));
    controller.dispose();
    expect(api.dispose).toHaveBeenCalledOnce();
  });

  it('runs cells through the kernel and supports restart and sequential execution', async () => {
    commandExists.mockResolvedValue(true);
    configure({ cliPath: '/bin/mars', executionTimeout: 5000 });
    const { controller, handler } = executionHandler();
    const notebook = { uri: { scheme: 'file', fsPath: '/book.mars' } } as vscode.NotebookDocument;
    await handler(
      [createCell('first'), createCell('second')],
      notebook,
      {} as vscode.NotebookController,
    );
    expect(KernelService).toHaveBeenCalledWith('/bin/mars');
    expect(kernel.execute).toHaveBeenNthCalledWith(1, 'first', 5000, '/book.mars', 'rust');
    expect(kernel.execute).toHaveBeenNthCalledWith(2, 'second', 5000, '/book.mars', 'rust');
    const api = vi.mocked(vscode.notebooks.createNotebookController).mock.results.at(-1)?.value;
    expect(api.createNotebookCellExecution.mock.results[1].value.end).toHaveBeenCalledWith(
      true,
      expect.any(Number),
    );
    await controller.restartKernel();
    expect(kernel.reset).toHaveBeenCalledOnce();
    controller.dispose();
    expect(kernel.stop).toHaveBeenCalledOnce();
  });

  it('renders thrown runtime errors as notebook error output and always ends execution', async () => {
    commandExists.mockRejectedValue(new Error('lookup failed'));
    const { handler } = executionHandler();
    await handler(
      [createCell('broken')],
      {} as vscode.NotebookDocument,
      {} as vscode.NotebookController,
    );
    const api = vi.mocked(vscode.notebooks.createNotebookController).mock.results.at(-1)?.value;
    const execution = api.createNotebookCellExecution.mock.results[0].value;
    expect(execution.clearOutput).toHaveBeenCalledOnce();
    const errorItem = execution.appendOutput.mock.calls[0][0].items[0];
    expect(errorItem.mime).toBe('application/vnd.code.notebook.error');
    expect(Buffer.from(errorItem.data).toString()).toContain('lookup failed');
    expect(execution.end).toHaveBeenCalledWith(false, expect.any(Number));
  });
});
