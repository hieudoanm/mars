import * as vscode from 'vscode';
import { commandExists } from './cli';
import { clearCellDiagnostics } from './diagnostics';
import { MarsKernelService } from './kernelService';
import type { MarsCellDiagnostic, MarsVariableInfo } from './kernelProtocol';
import { marsOutputToNotebookOutput } from './output';

const SUPPORTED_LANGUAGES = new Set(['rust', 'python', 'r', 'julia', 'cpp']);

export class MarsNotebookController implements vscode.Disposable {
  private readonly controller: vscode.NotebookController;
  private executionOrder = 0;
  private kernelService: MarsKernelService | null = null;
  static instance: MarsNotebookController | null = null;

  constructor() {
    MarsNotebookController.instance = this;
    this.controller = vscode.notebooks.createNotebookController(
      'mars-kernel',
      'mars-notebook',
      'Mars Kernel',
    );
    this.controller.supportedLanguages = ['rust', 'python', 'r', 'julia', 'cpp'];
    this.controller.supportsExecutionOrder = true;
    this.controller.executeHandler = this.executeAll.bind(this);
  }

  dispose(): void {
    this.controller.dispose();
    this.kernelService?.stop();
    MarsNotebookController.instance = null;
  }

  async restartKernel(): Promise<void> {
    if (this.kernelService) await this.kernelService.reset();
  }

  private interruptRequested = false;

  /** Sends a cancel request to the running kernel without resetting state. */
  async interruptExecution(): Promise<void> {
    this.interruptRequested = true;
    await this.kernelService?.cancel();
  }

  async inspectVariables(notebookPath: string | null = null): Promise<MarsVariableInfo[]> {
    return this.getKernelService().variables(notebookPath);
  }

  async diagnoseSource(
    source: string,
    notebookPath: string | null = null,
  ): Promise<MarsCellDiagnostic[]> {
    return this.getKernelService().diagnose(source, notebookPath);
  }

  /** True once a kernel process has been started for this controller. */
  kernelRunning(): boolean {
    return this.kernelService?.isRunning() ?? false;
  }

  /** Committed cell count, used to map diagnose() candidate indices. */
  async kernelExecutionCount(notebookPath: string | null = null): Promise<number> {
    return (await this.getKernelService().status(notebookPath)).execution_count;
  }

  async sendInput(text: string): Promise<void> {
    await this.kernelService?.input(text);
  }

  private getKernelService(): MarsKernelService {
    if (!this.kernelService) {
      const cliPath = vscode.workspace.getConfiguration('mars').get<string>('cliPath', 'mars');
      this.kernelService = new MarsKernelService(cliPath);
    }
    return this.kernelService;
  }

  private async executeAll(
    cells: vscode.NotebookCell[],
    notebook: vscode.NotebookDocument,
    controller: vscode.NotebookController,
  ): Promise<void> {
    void controller;
    this.interruptRequested = false;
    for (const cell of cells) {
      if (this.interruptRequested) break;
      await this.executeCell(cell, notebook);
    }
  }

  private async executeCell(
    cell: vscode.NotebookCell,
    notebook?: vscode.NotebookDocument,
  ): Promise<void> {
    const execution = this.controller.createNotebookCellExecution(cell);
    this.executionOrder += 1;
    execution.executionOrder = this.executionOrder;
    execution.start(Date.now());
    let success = false;
    try {
      const code = cell.document.getText();
      const notebookLanguage =
        typeof notebook?.metadata?.language === 'string' && notebook.metadata.language
          ? notebook.metadata.language
          : 'rust';
      const cellLanguage = cell.document.languageId;
      const language = SUPPORTED_LANGUAGES.has(cellLanguage) ? cellLanguage : notebookLanguage;
      const cliPath = vscode.workspace.getConfiguration('mars').get<string>('cliPath', 'mars');
      if (!(await commandExists(cliPath))) {
        await execution.appendOutput(
          new vscode.NotebookCellOutput([
            new vscode.NotebookCellOutputItem(
              Buffer.from(
                `Mars CLI not found at '${cliPath}'.\n\nInstall Mars CLI to enable actual code execution.\n\nFor now, showing placeholder output:\n\n${code}`,
                'utf-8',
              ),
              'text/plain',
            ),
          ]),
        );
        success = true;
      } else {
        const kernel = this.getKernelService();
        const timeoutMs = vscode.workspace
          .getConfiguration('mars')
          .get<number>('executionTimeout', 30000);
        await execution.appendOutput(
          new vscode.NotebookCellOutput([
            new vscode.NotebookCellOutputItem(
              Buffer.from(
                `Compiling and executing ${language} code (this may take a while)...\n`,
                'utf-8',
              ),
              'text/plain',
            ),
          ]),
        );
        const result = await Promise.race([
          kernel.execute(
            code,
            timeoutMs,
            notebook?.uri.scheme === 'file' ? notebook.uri.fsPath : null,
            language,
          ),
          new Promise<never>((_, reject) =>
            setTimeout(
              () =>
                reject(
                  new Error(
                    `Execution timeout after ${timeoutMs}ms. Try increasing mars.executionTimeout in settings.`,
                  ),
                ),
              timeoutMs,
            ),
          ),
        ]);
        await execution.clearOutput();
        for (const output of result.outputs) {
          await execution.appendOutput(marsOutputToNotebookOutput(output));
        }
        success = result.status === 'success';
        if (success) {
          clearCellDiagnostics(cell);
          void vscode.commands.executeCommand('mars.refreshVariables');
        }
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      await execution.clearOutput();
      await execution.appendOutput(
        new vscode.NotebookCellOutput([
          new vscode.NotebookCellOutputItem(
            Buffer.from(
              `Error: ${message}\n\nMake sure:\n1. Mars CLI is installed and on PATH\n2. 'mars kernel serve' command is available\n3. Rust toolchain is installed`,
              'utf-8',
            ),
            'application/vnd.code.notebook.error',
          ),
        ]),
      );
    } finally {
      execution.end(success, Date.now());
    }
  }
}
