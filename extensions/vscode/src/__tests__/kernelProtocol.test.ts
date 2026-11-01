import { EventEmitter } from 'node:events';
import type { ChildProcessWithoutNullStreams } from 'node:child_process';
import { sendKernelRequest, type KernelRequest } from '../kernelProtocol';

function fakeProcess(send: (write: string, stdout: EventEmitter) => void) {
  const stdout = new EventEmitter();
  const stdin = { write: vi.fn((value: string) => send(value, stdout)) };
  return { process: { stdout, stdin } as unknown as ChildProcessWithoutNullStreams, stdin, stdout };
}

describe('kernel JSON-line protocol', () => {
  const request: KernelRequest = {
    id: '7',
    request: { type: 'execute', source: '1 + 1', timeout_millis: 2000 },
  };

  it('writes a newline-delimited request and waits past status events', async () => {
    const { process, stdin, stdout } = fakeProcess((write, stream) => {
      const sent = JSON.parse(write) as KernelRequest;
      queueMicrotask(() => {
        stream.emit('data', Buffer.from('{not-json}\n'));
        stream.emit(
          'data',
          Buffer.from(
            JSON.stringify({
              id: sent.id,
              response: { type: 'status', status: 'running', execution_count: 1 },
            }) + '\n',
          ),
        );
        stream.emit(
          'data',
          Buffer.from(
            JSON.stringify({
              id: sent.id,
              response: {
                type: 'execution',
                outputs: [{ type: 'text', text: '2' }],
                execution_count: 1,
                duration_millis: 5,
              },
            }) + '\n',
          ),
        );
      });
    });
    await expect(sendKernelRequest(process, request)).resolves.toMatchObject({
      id: '7',
      response: { type: 'execution', execution_count: 1 },
    });
    expect(stdin.write).toHaveBeenCalledWith(`${JSON.stringify(request)}\n`);
    expect(stdout.listenerCount('data')).toBe(0);
  });

  it('keeps incomplete lines buffered and skips responses for another request', async () => {
    const { process } = fakeProcess((_, stream) => {
      queueMicrotask(() => {
        stream.emit(
          'data',
          Buffer.from(
            '{"id":"other","response":{"type":"reset","status":"ok"}}\n{"id":"7","response":{"type":"reset"}}',
          ),
        );
        stream.emit('data', Buffer.from('\n'));
      });
    });
    await expect(
      sendKernelRequest(process, { id: '7', request: { type: 'reset' } }),
    ).resolves.toMatchObject({
      id: '7',
      response: { type: 'reset' },
    });
  });

  it('rejects kernel errors and stream errors', async () => {
    const errorProcess = fakeProcess((write, stream) => {
      const sent = JSON.parse(write) as KernelRequest;
      queueMicrotask(() =>
        stream.emit(
          'data',
          Buffer.from(
            JSON.stringify({
              id: sent.id,
              response: { type: 'error', message: 'compile failed' },
            }) + '\n',
          ),
        ),
      );
    });
    await expect(sendKernelRequest(errorProcess.process, request)).rejects.toThrow(
      'compile failed',
    );

    const streamProcess = fakeProcess((_, stream) => {
      queueMicrotask(() => stream.emit('error', new Error('pipe closed')));
    });
    await expect(sendKernelRequest(streamProcess.process, request)).rejects.toThrow('pipe closed');
  });
});
