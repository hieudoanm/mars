import type { ChildProcessWithoutNullStreams } from 'node:child_process';
import type { MarsOutput } from './output';

export type KernelRequest =
  | { id: string; request: { type: 'execute'; source: string; timeout_millis?: number } }
  | { id: string; request: { type: 'reset' } };

export type KernelResponse =
  | {
      id: string;
      response: {
        type: 'execution';
        outputs: MarsOutput[];
        execution_count: number;
        duration_millis: number;
      };
    }
  | { id: string; response: { type: 'reset'; status: string } }
  | { id: string; response: { type: 'status'; status: string; execution_count: number } }
  | { id: string; response: { type: 'error'; message: string } };

export async function sendKernelRequest(
  process: ChildProcessWithoutNullStreams,
  request: KernelRequest,
): Promise<KernelResponse> {
  return new Promise((resolve, reject) => {
    let buffer = '';
    const cleanup = () => {
      process.stdout.removeListener('data', onData);
      process.stdout.removeListener('error', onError);
    };
    const onData = (chunk: Buffer) => {
      buffer += chunk.toString();
      const lines = buffer.split('\n');
      buffer = lines.pop() ?? '';
      for (const line of lines) {
        if (!line.trim()) continue;
        let response: KernelResponse;
        try {
          response = JSON.parse(line) as KernelResponse;
        } catch {
          continue;
        }
        if (response.id !== request.id || response.response.type === 'status') continue;
        cleanup();
        if (response.response.type === 'error') reject(new Error(response.response.message));
        else resolve(response);
        return;
      }
    };
    const onError = (error: Error) => {
      cleanup();
      reject(error);
    };

    process.stdout.on('data', onData);
    process.stdout.on('error', onError);
    process.stdin.write(`${JSON.stringify(request)}\n`);
  });
}
