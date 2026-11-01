import * as vscode from 'vscode';
import { spawn, type ChildProcessWithoutNullStreams } from 'node:child_process';
import { commandExists } from './cli';
import { sendKernelRequest, type KernelRequest } from './kernelProtocol';
import type { MarsOutput } from './output';

export class MarsKernelService {
  private process: ChildProcessWithoutNullStreams | null = null;
  private requestId = 0;
  private readonly cliPath: string;
  private notebookPath: string | null = null;

  constructor(cliPath: string) {
    this.cliPath = cliPath;
  }

  async start(notebookPath: string | null = null): Promise<void> {
    if (this.process && this.notebookPath === notebookPath) return;
    if (this.process) this.stop();
    if (!(await commandExists(this.cliPath))) {
      throw new Error(
        `Mars CLI not found at '${this.cliPath}'. Please install Mars CLI and ensure it's on your PATH.`,
      );
    }

    const runtime = vscode.workspace.getConfiguration('mars').get<string>('runtime', 'replay');
    const args = ['kernel', 'serve'];
    if (runtime === 'evcxr') args.push('--runtime', 'evcxr');
    if (notebookPath) args.push('--notebook', notebookPath);

    console.log(`Starting Mars kernel: ${this.cliPath} ${args.join(' ')}`);
    this.process = spawn(this.cliPath, args, { stdio: ['pipe', 'pipe', 'pipe'] });
    this.notebookPath = notebookPath;
    this.process.on('error', (error) => {
      console.error('Mars kernel process error:', error);
      this.process = null;
    });
    this.process.on('exit', (code) => {
      console.error(`Mars kernel process exited with code ${code}`);
      this.process = null;
    });
    await new Promise((resolve) => setTimeout(resolve, 500));
  }

  async execute(
    code: string,
    timeoutMs?: number,
    notebookPath: string | null = null,
  ): Promise<{ status: 'success' | 'error'; execution_count: number; outputs: MarsOutput[] }> {
    await this.start(notebookPath);
    const response = await this.request({
      id: (++this.requestId).toString(),
      request: { type: 'execute', source: code, timeout_millis: timeoutMs },
    });
    if (response.response.type !== 'execution') {
      throw new Error(`Unexpected kernel response: ${response.response.type}`);
    }
    const execution = response.response;
    const failed = execution.outputs.some((output) => output.type === 'error');
    return {
      status: failed ? 'error' : 'success',
      execution_count: execution.execution_count,
      outputs: execution.outputs,
    };
  }

  async reset(): Promise<void> {
    if (!this.process) await this.start(this.notebookPath);
    const response = await this.request({
      id: (++this.requestId).toString(),
      request: { type: 'reset' },
    });
    if (response.response.type === 'reset' || response.response.type === 'execution') return;
    throw new Error(`Unexpected kernel response: ${response.response.type}`);
  }

  stop(): void {
    if (!this.process) return;
    this.process.kill();
    this.process = null;
  }

  private request(request: KernelRequest) {
    if (!this.process) return Promise.reject(new Error('Kernel process not available'));
    return sendKernelRequest(this.process, request);
  }
}
