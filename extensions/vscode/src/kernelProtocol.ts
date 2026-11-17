import type { ChildProcessWithoutNullStreams } from 'node:child_process';
import type { MarsOutput } from './output';

export interface MarsVariableInfo {
  name: string;
  typeName: string | null;
  value: string | null;
}

export interface MarsCellDiagnostic {
  cellIndex: number | null;
  line: number;
  column: number;
  endLine: number;
  endColumn: number;
  severity: string;
  code: string | null;
  message: string;
}

export type KernelRequest =
  | {
      id: string;
      request: {
        type: 'execute';
        source: string;
        timeout_millis?: number;
        language?: string;
      };
    }
  | { id: string; request: { type: 'reset' } }
  | { id: string; request: { type: 'status' } }
  | { id: string; request: { type: 'cancel' } }
  | { id: string; request: { type: 'variables' } }
  | { id: string; request: { type: 'diagnose'; source: string } }
  | { id: string; request: { type: 'input'; text: string } };

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
  | { id: string; response: { type: 'cancelled' } }
  | { id: string; response: { type: 'variables'; variables: MarsVariableInfo[] } }
  | { id: string; response: { type: 'diagnostics'; diagnostics: MarsCellDiagnostic[] } }
  | { id: string; response: { type: 'input' } }
  | { id: string; response: { type: 'error'; message: string } };

interface WireEnvelope {
  id?: unknown;
  response?: { type?: unknown };
  event?: unknown;
}

export const sendKernelRequest = async (
  process: ChildProcessWithoutNullStreams,
  request: KernelRequest,
): Promise<KernelResponse> => {
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
        let envelope: WireEnvelope;
        try {
          envelope = JSON.parse(line) as WireEnvelope;
        } catch {
          continue;
        }
        const body = envelope.response;
        // Stream event envelopes share the request id but carry `event`,
        // not `response`; they must not satisfy the pending request.
        // Status envelopes are noise unless this request asked for status.
        if (
          envelope.id !== request.id ||
          typeof body !== 'object' ||
          body === null ||
          (body.type === 'status' && request.request.type !== 'status')
        ) {
          continue;
        }
        cleanup();
        const response = envelope as unknown as KernelResponse;
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
};
